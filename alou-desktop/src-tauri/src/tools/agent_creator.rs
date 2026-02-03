//! 智能体创建工具
//!
//! 支持通过 Tauri 命令创建新智能体，并在前端显示

use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 智能体创建工具
pub struct AgentCreatorTool {
    metadata: ToolMetadata,
}

/// 智能体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// 智能体ID
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 描述
    pub description: String,
    /// 人设/角色定义
    pub persona: String,
    /// 能力列表
    pub capabilities: Vec<String>,
    /// 约束条件
    pub constraints: Vec<String>,
    /// 使用的模型
    pub model: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 头像/图标
    pub avatar: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 关联的 Skills
    pub skills: Vec<String>,
    /// 记忆配置
    pub memory_config: MemoryConfig,
    /// 工具权限
    pub tool_permissions: Vec<String>,
    /// 创建时间
    pub created_at: i64,
}

/// 记忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 启用长期记忆
    pub enable_long_term: bool,
    /// 启用工作记忆
    pub enable_working_memory: bool,
    /// 记忆容量限制
    pub memory_limit: usize,
    /// 关联的 IPNS 名称（用于分布式记忆）
    pub ipns_name: Option<String>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enable_long_term: true,
            enable_working_memory: true,
            memory_limit: 1000,
            ipns_name: None,
        }
    }
}

/// 创建智能体结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentResult {
    /// 智能体ID
    pub agent_id: String,
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 智能体配置
    pub config: Option<AgentConfig>,
    /// 前端显示数据
    pub frontend_data: FrontendAgentData,
}

/// 前端显示数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendAgentData {
    /// 名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 头像
    pub avatar: String,
    /// 状态
    pub status: String,
    /// 能力标签
    pub capability_tags: Vec<String>,
    /// 创建时间
    pub created_at: String,
}

impl AgentCreatorTool {
    /// 创建新的智能体创建工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "agent_creator".to_string(),
                name: "Agent Creator Tool".to_string(),
                description: "创建新的AI智能体，支持在前端显示".to_string(),
                category: ToolCategory::Automation,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
            },
        }
    }

    /// 生成智能体ID
    fn generate_agent_id(&self, name: &str) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let timestamp = chrono::Utc::now().timestamp();
        let input = format!("{}_{}_{}", name, timestamp, uuid::Uuid::new_v4());
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let hash = hasher.finish();
        
        format!("agent_{:x}", hash)
    }

    /// 构建系统提示词
    fn build_system_prompt(&self, persona: &str, capabilities: &[String], constraints: &[String]) -> String {
        let capabilities_text = if capabilities.is_empty() {
            "无特定能力".to_string()
        } else {
            capabilities.join("\n- ")
        };

        let constraints_text = if constraints.is_empty() {
            "无特定约束".to_string()
        } else {
            constraints.join("\n- ")
        };

        format!(
            r#"{}

## 你的能力
- {}

## 约束条件
- {}

## 执行要求
1. 始终以专业、友好的态度回应
2. 充分利用你的能力帮助用户
3. 遵守约束条件
4. 如果不确定，诚实说明"#,
            persona,
            capabilities_text,
            constraints_text
        )
    }

    /// 创建智能体
    async fn create_agent(&self, config: AgentConfig) -> Result<CreateAgentResult, String> {
        // 保存智能体配置到存储
        self.save_agent_config(&config).await?;

        // 准备前端显示数据
        let frontend_data = FrontendAgentData {
            name: config.display_name.clone(),
            description: config.description.clone(),
            avatar: config.avatar.clone().unwrap_or_else(|| "🤖".to_string()),
            status: "ready".to_string(),
            capability_tags: config.capabilities.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // 触发前端事件通知
        self.notify_frontend_agent_created(&config, &frontend_data).await?;

        Ok(CreateAgentResult {
            agent_id: config.id.clone(),
            success: true,
            message: format!("Agent '{}' created successfully", config.display_name),
            config: Some(config),
            frontend_data,
        })
    }

    /// 保存智能体配置
    async fn save_agent_config(&self, config: &AgentConfig) -> Result<(), String> {
        // 使用 KV 存储保存智能体配置
        // 实际实现应该调用 kv_commands
        println!("[AgentCreator] Saving agent config: {}", config.id);
        Ok(())
    }

    /// 通知前端智能体已创建
    async fn notify_frontend_agent_created(
        &self,
        config: &AgentConfig,
        frontend_data: &FrontendAgentData,
    ) -> Result<(), String> {
        // 这里应该通过 Tauri 事件系统通知前端
        // 实际实现需要访问 AppHandle
        println!("[AgentCreator] Notifying frontend: agent {} created", config.id);
        Ok(())
    }

    /// 获取智能体列表
    async fn list_agents(&self) -> Result<Vec<AgentConfig>, String> {
        // 从存储中获取所有智能体配置
        Ok(vec![])
    }

    /// 获取智能体详情
    async fn get_agent(&self, agent_id: &str) -> Result<Option<AgentConfig>, String> {
        // 从存储中获取智能体配置
        Ok(None)
    }

    /// 删除智能体
    async fn delete_agent(&self, agent_id: &str) -> Result<bool, String> {
        // 从存储中删除智能体配置
        println!("[AgentCreator] Deleting agent: {}", agent_id);
        Ok(true)
    }

    /// 更新智能体
    async fn update_agent(&self, agent_id: &str, updates: serde_json::Value) -> Result<AgentConfig, String> {
        // 更新智能体配置
        Err("Not implemented".to_string())
    }
}

#[async_trait]
impl ToolExecutor for AgentCreatorTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "create" => {
                let display_name = args.get("display_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'display_name' field".to_string()))?
                    .to_string();

                let description = args.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let persona = args.get("persona")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let capabilities: Vec<String> = args.get("capabilities")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let constraints: Vec<String> = args.get("constraints")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let model = args.get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("claude-3-sonnet")
                    .to_string();

                let avatar = args.get("avatar").and_then(|v| v.as_str()).map(|s| s.to_string());

                let tags: Vec<String> = args.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let skills: Vec<String> = args.get("skills")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let agent_id = self.generate_agent_id(&display_name);
                let system_prompt = self.build_system_prompt(&persona, &capabilities, &constraints);

                let config = AgentConfig {
                    id: agent_id,
                    display_name,
                    description,
                    persona,
                    capabilities,
                    constraints,
                    model,
                    system_prompt,
                    avatar,
                    tags,
                    skills,
                    memory_config: MemoryConfig::default(),
                    tool_permissions: vec!["read".to_string()],
                    created_at: chrono::Utc::now().timestamp(),
                };

                let result = self.create_agent(config).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(result),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(result.message.clone()),
                    warnings: vec![],
                    context: None,
                })
            }

            "list" => {
                let agents = self.list_agents().await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({ "agents": agents, "count": agents.len() }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} agents", agents.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "get" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?;

                let agent = self.get_agent(agent_id).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?
                    .ok_or_else(|| ToolError::InvalidArguments(format!("Agent '{}' not found", agent_id)))?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(agent),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Retrieved agent '{}'", agent.display_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "delete" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?;

                let deleted = self.delete_agent(agent_id).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult {
                    success: deleted,
                    data: serde_json::json!({ "deleted": deleted }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(if deleted { format!("Deleted agent '{}'", agent_id) } else { "Agent not found".to_string() }),
                    warnings: vec![],
                    context: None,
                })
            }

            _ => Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }
        if args.get("action").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: action".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        r#"Agent Creator Tool

Create and manage AI agents that can be displayed in the frontend.

Actions:
  - create: Create a new agent
    params: {
      "display_name": "My Agent",
      "description": "An agent for code review",
      "persona": "You are a code review expert...",
      "capabilities": ["find bugs", "suggest improvements"],
      "constraints": ["be concise"],
      "model": "claude-3-sonnet",
      "avatar": "🤖",
      "tags": ["coding", "review"],
      "skills": ["skill_code_formatter"]
    }
  
  - list: List all agents
  
  - get: Get agent details
    params: { "agent_id": "agent_xxx" }
  
  - delete: Delete an agent
    params: { "agent_id": "agent_xxx" }

Examples:

Create a code review agent:
{
  "action": "create",
  "display_name": "Code Reviewer",
  "description": "Reviews code for issues",
  "persona": "You are an expert code reviewer with 20 years of experience...",
  "capabilities": ["identify bugs", "suggest optimizations", "check security"],
  "constraints": ["be specific", "provide examples"],
  "tags": ["code", "review", "quality"]
}"#.to_string()
    }
}
