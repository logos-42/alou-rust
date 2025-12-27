use crate::agent::batch_create::{
    AgentCreationResult, AgentCreationSpec, AgentStatus, BatchCreateTask, GenerationConfig, TaskStatus
};
use crate::agent::content_generator::ContentGenerator;
use crate::agent::session::SessionManager;
use crate::storage::kv::KvStore;
use crate::utils::error::Result;
use serde_json::json;
use worker::*;

/// 批量处理器，用于处理批量创建智能体的任务
pub struct BatchProcessor {
    session_manager: SessionManager,
    content_generator: ContentGenerator,
    kv_store: KvStore,
}

impl BatchProcessor {
    pub fn new(
        session_manager: SessionManager,
        content_generator: ContentGenerator,
        kv_store: KvStore,
    ) -> Self {
        Self {
            session_manager,
            content_generator,
            kv_store,
        }
    }

    /// 获取批量任务中所有已创建的智能体会话ID
    /// 
    /// 这个方法用于前端查询批量创建的智能体，以便自动添加到频道
    pub async fn get_batch_agent_sessions(&self, task_id: &str, _env: &Env) -> Result<Vec<String>> {
        match self.get_task(task_id, _env).await? {
            Some(task) => {
                let session_ids: Vec<String> = task
                    .agents
                    .iter()
                    .filter_map(|agent| agent.session_id.clone())
                    .collect();
                Ok(session_ids)
            }
            None => Ok(vec![]),
        }
    }

    /// 处理批量创建任务
    pub async fn process_batch(&self, mut task: BatchCreateTask, env: &Env) -> Result<()> {
        // 更新任务状态为Processing
        task.status = TaskStatus::Processing;
        task.updated_at = crate::utils::time::now_timestamp();
        self.save_task(&task, env).await?;

        // 处理每个智能体
        // 先收集所有规格，避免借用冲突
        let specs: Vec<AgentCreationSpec> = task.agent_specs.clone();
        let generation_config = task.generation_config.clone();
        
        for (index, agent_result) in task.agents.iter_mut().enumerate() {
            agent_result.status = AgentStatus::Processing;
            
            // 获取对应的规格
            let spec = specs.get(index);
            match self
                .process_single_agent(agent_result, spec, &generation_config, env)
                .await
            {
                Ok(()) => {
                    agent_result.status = AgentStatus::Completed;
                }
                Err(e) => {
                    agent_result.status = AgentStatus::Failed;
                    agent_result.error = Some(e.to_string());
                    console_error!("Failed to process agent {}: {}", index, e);
                }
            }
        }
        
        // 更新最终状态
        let all_completed = task
            .agents
            .iter()
            .all(|a| a.status == AgentStatus::Completed);
        task.status = if all_completed {
            TaskStatus::Completed
        } else {
            TaskStatus::Failed
        };
        task.updated_at = crate::utils::time::now_timestamp();
        self.save_task(&task, env).await?;

        Ok(())
    }

    /// 处理单个智能体创建
    async fn process_single_agent(
        &self,
        agent_result: &mut AgentCreationResult,
        spec: Option<&AgentCreationSpec>,
        _generation_config: &Option<GenerationConfig>,
        _env: &Env,
    ) -> Result<()> {
        let spec = match spec {
            Some(s) => s,
            None => {
                return Err(crate::utils::error::AloudError::InvalidInput(
                    "AgentCreationSpec not found".to_string(),
                ));
            }
        };

        // 1. 生成内容（名字、prompt），如果未提供且需要生成
        let generation_config = _generation_config;
        let use_ai_for_name = generation_config
            .as_ref()
            .and_then(|c| c.use_ai_for_name)
            .unwrap_or(true);
        let use_ai_for_prompt = generation_config
            .as_ref()
            .and_then(|c| c.use_ai_for_prompt)
            .unwrap_or(true);

        let name = if let Some(ref provided_name) = spec.name {
            provided_name.clone()
        } else if use_ai_for_name {
            // 需要生成名字
            let default_name = "智能助手".to_string();
            let description = spec
                .description
                .as_ref()
                .or(spec.role_description.as_ref())
                .unwrap_or(&default_name);
            self.content_generator.generate_name(description).await?
        } else {
            // 使用默认名字
            "智能助手".to_string()
        };

        let role_description = if let Some(ref provided_desc) = spec.role_description {
            provided_desc.clone()
        } else if use_ai_for_prompt {
            // 需要生成prompt
            let default_desc = "智能助手".to_string();
            let description = spec
                .description
                .as_ref()
                .unwrap_or(&default_desc);
            self.content_generator
                .generate_prompt(description, spec.category.as_deref())
                .await?
        } else {
            // 使用默认描述
            "一个通用的智能助手，可以帮助用户完成各种任务。".to_string()
        };

        // 2. 创建会话
        let session_id = self
            .session_manager
            .create_session(
                spec.wallet_address.clone(),
                spec.chain.clone(),
            )
            .await?;

        // 3. 创建智能体元数据
        let mut agent_metadata = json!({
            "agent_type": "claude_agent_sdk",
            "display_name": name.clone(),
            "session_id": session_id.clone(),
            "role_description": role_description.clone(),
            "mode": "agent", // 设置为 agent 模式，使用自定义智能体 prompt
        });

        if let Some(avatar_cid) = &spec.avatar_cid {
            agent_metadata["avatar_cid"] = json!(avatar_cid);
        }

        // 添加 MCP 配置
        if let Some(mcp_config_cid) = &spec.mcp_config_cid {
            agent_metadata["mcp_config_cid"] = json!(mcp_config_cid);
        }

        if let Some(ref mcp_ports) = &spec.mcp_ports {
            if let Ok(value) = serde_json::to_value(mcp_ports) {
                agent_metadata["mcp_ports"] = value;
            }
        }

        // 4. 保存智能体元数据
        if let Err(e) = self
            .session_manager
            .set_agent_metadata(&session_id, agent_metadata.clone())
            .await
        {
            console_warn!(
                "Failed to store agent metadata for session {}: {}",
                session_id,
                e
            );
        }

        // 5. 更新结果
        agent_result.session_id = Some(session_id.clone());
        agent_result.name = Some(name.clone());
        agent_result.role_description = Some(role_description.clone());
        agent_result.avatar_cid = spec.avatar_cid.clone();
        agent_result.mcp_config_cid = spec.mcp_config_cid.clone();
        agent_result.mcp_ports = spec.mcp_ports.clone();

        console_log!(
            "✓ Agent created: session_id={}, name={}, mcp_config={}",
            session_id, name, spec.mcp_config_cid.as_ref().unwrap_or(&"none".to_string())
        );

        Ok(())
    }

    /// 保存任务状态到KV
    async fn save_task(&self, task: &BatchCreateTask, _env: &Env) -> Result<()> {
        let key = format!("batch_task:{}", task.task_id);
        let value = serde_json::to_string(task)?;
        self.kv_store.put(&key, &value, Some(86400 * 7)).await?; // 7 days TTL
        Ok(())
    }

    /// 获取任务状态
    pub async fn get_task(&self, task_id: &str, _env: &Env) -> Result<Option<BatchCreateTask>> {
        let key = format!("batch_task:{}", task_id);
        match self.kv_store.get::<BatchCreateTask>(&key).await? {
            Some(task) => Ok(Some(task)),
            None => Ok(None),
        }
    }
}

