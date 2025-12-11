use crate::agent::cluster_action::{AgentAssignment, Task};
use crate::agent::session::SessionManager;
use crate::router::pubsub::PubSubManager;
use crate::utils::error::{AloudError, Result};
use serde_json::{json, Value};

/// 智能体信息
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub agent_id: String,
    pub agent_name: Option<String>,
    pub agent_mode: Option<String>, // "agent" or "alou"
    pub capabilities: Vec<String>,
    pub description: Option<String>,
}

/// 智能体协调器
pub struct AgentCoordinator {
    session_manager: SessionManager,
    pubsub_manager: PubSubManager,
}

impl AgentCoordinator {
    pub fn new(session_manager: SessionManager, pubsub_manager: PubSubManager) -> Self {
        Self {
            session_manager,
            pubsub_manager,
        }
    }

    /// 根据任务需求选择合适的智能体
    pub async fn select_agents(
        &self,
        task: &Task,
        available_agent_ids: &[String],
    ) -> Result<Vec<String>> {
        // 获取所有可用智能体的信息
        let mut agent_infos = Vec::new();
        for agent_id in available_agent_ids {
            if let Ok(Some(info)) = self.get_agent_info(agent_id).await {
                agent_infos.push(info);
            }
        }

        // 匹配智能体能力
        let matched = self.match_agent_capabilities(&task.task_type, &agent_infos)?;

        // 返回匹配的智能体 ID 列表
        Ok(matched.iter().map(|a| a.agent_id.clone()).collect())
    }

    /// 匹配智能体能力
    pub fn match_agent_capabilities(
        &self,
        required_capability: &str,
        agents: &[AgentInfo],
    ) -> Result<Vec<AgentInfo>> {
        let mut matched = Vec::new();

        for agent in agents {
            // 检查智能体是否有所需能力
            if agent.capabilities.iter().any(|c| {
                c.eq_ignore_ascii_case(required_capability)
                    || c.contains(required_capability)
                    || required_capability.contains(c)
            }) {
                matched.push(agent.clone());
            }
        }

        // 如果没有精确匹配，返回所有智能体（降级处理）
        if matched.is_empty() {
            matched = agents.to_vec();
        }

        Ok(matched)
    }

    /// 获取智能体的模式（agent/alou）
    pub async fn get_agent_mode(&self, agent_id: &str) -> Result<Option<String>> {
        if let Ok(Some(metadata)) = self.session_manager.get_agent_metadata(agent_id).await {
            if let Some(mode) = metadata.get("mode").and_then(|v| v.as_str()) {
                return Ok(Some(mode.to_string()));
            }
        }
        Ok(None)
    }

    /// 将任务分配给智能体
    pub async fn assign_task_to_agent(
        &self,
        task: &mut Task,
        agent_id: &str,
    ) -> Result<()> {
        let agent_mode = self.get_agent_mode(agent_id).await?;
        let agent_name = self.get_agent_name(agent_id).await?;

        let assignment = AgentAssignment {
            agent_id: agent_id.to_string(),
            agent_name,
            agent_mode,
            task_id: task.task_id.clone(),
            assigned_at: crate::utils::time::now_timestamp(),
        };

        task.assigned_agent = Some(assignment);
        Ok(())
    }

    /// 为集群行动创建 PubSub 群聊
    pub async fn create_group_chat(
        &self,
        action_id: &str,
        agent_ids: &[String],
    ) -> Result<String> {
        let topic = format!("diap/cluster_action/{}", action_id);
        
        // 发布群聊创建消息
        let message = crate::router::pubsub::PubSubMessage {
            id: format!("msg_{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            msg_type: "system".to_string(),
            from: Some("system".to_string()),
            to: None,
            content: format!("集群行动 {} 已创建，参与智能体: {:?}", action_id, agent_ids),
            topic: topic.clone(),
            timestamp: crate::utils::time::now_timestamp(),
            metadata: json!({
                "type": "group_chat_created",
                "action_id": action_id,
                "agents": agent_ids,
            }),
        };

        self.pubsub_manager.publish(&topic, message).await?;
        
        Ok(topic)
    }

    /// 发布任务消息到群聊
    pub async fn publish_task_message(
        &self,
        topic: &str,
        task: &Task,
        message_type: &str,
    ) -> Result<()> {
        let content = match message_type {
            "task_request" => format!("任务请求: {}", task.description),
            "task_result" => {
                if let Some(ref output) = task.output {
                    format!("任务完成: {}", serde_json::to_string(output).unwrap_or_default())
                } else {
                    "任务完成".to_string()
                }
            }
            _ => format!("任务消息: {}", task.description),
        };

        let from = task.assigned_agent
            .as_ref()
            .map(|a| a.agent_id.clone())
            .unwrap_or_else(|| "system".to_string());

        let message = crate::router::pubsub::PubSubMessage {
            id: format!("msg_{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            msg_type: message_type.to_string(),
            from: Some(from),
            to: None,
            content,
            topic: topic.to_string(),
            timestamp: crate::utils::time::now_timestamp(),
            metadata: json!({
                "task_id": task.task_id,
                "task_type": task.task_type,
                "status": format!("{:?}", task.status),
            }),
        };

        self.pubsub_manager.publish(topic, message).await?;
        Ok(())
    }

    /// 获取智能体信息
    async fn get_agent_info(&self, agent_id: &str) -> Result<Option<AgentInfo>> {
        // 尝试从 session 获取智能体元数据
        if let Ok(Some(metadata)) = self.session_manager.get_agent_metadata(agent_id).await {
            let agent_name = metadata
                .get("name")
                .or_else(|| metadata.get("display_name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let agent_mode = metadata
                .get("mode")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let description = metadata
                .get("role_description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // 从元数据中提取能力（简化实现）
            let capabilities = vec!["query".to_string(), "transaction".to_string()];

            return Ok(Some(AgentInfo {
                agent_id: agent_id.to_string(),
                agent_name,
                agent_mode,
                capabilities,
                description,
            }));
        }

        // 如果没有元数据，返回默认信息
        Ok(Some(AgentInfo {
            agent_id: agent_id.to_string(),
            agent_name: None,
            agent_mode: None,
            capabilities: vec!["general".to_string()],
            description: None,
        }))
    }

    /// 获取智能体名称
    async fn get_agent_name(&self, agent_id: &str) -> Result<Option<String>> {
        if let Ok(Some(metadata)) = self.session_manager.get_agent_metadata(agent_id).await {
            if let Some(name) = metadata
                .get("name")
                .or_else(|| metadata.get("display_name"))
                .and_then(|v| v.as_str())
            {
                return Ok(Some(name.to_string()));
            }
        }
        Ok(None)
    }
}

