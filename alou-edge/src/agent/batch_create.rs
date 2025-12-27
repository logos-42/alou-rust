use serde::{Deserialize, Serialize};

/// 批量创建智能体请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateAgentRequest {
    pub agents: Vec<AgentCreationSpec>,
    pub generation_config: Option<GenerationConfig>,
}

/// 单个智能体创建规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCreationSpec {
    // 可选：如果提供则使用，否则自动生成
    pub name: Option<String>,
    pub role_description: Option<String>,
    pub avatar_cid: Option<String>,
    // 用于生成内容的提示词
    pub description: Option<String>,
    pub category: Option<String>,
    // 可选：wallet地址和链信息
    pub wallet_address: Option<String>,
    pub chain: Option<String>,
    // MCP 配置
    pub mcp_config_cid: Option<String>,
    pub mcp_ports: Option<Vec<crate::router::agent::McpPortConfig>>,
}

/// 生成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub use_ai_for_name: Option<bool>,
    pub use_ai_for_prompt: Option<bool>,
    // 图像生成配置暂不使用（当前版本不实现图像生成）
}

/// 批量创建任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateTask {
    pub task_id: String,
    pub status: TaskStatus,
    pub agents: Vec<AgentCreationResult>,
    pub generation_config: Option<GenerationConfig>,
    pub agent_specs: Vec<AgentCreationSpec>, // 保存原始规格用于处理
    pub created_at: i64,
    pub updated_at: i64,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// 单个智能体创建结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCreationResult {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub name: Option<String>,
    pub role_description: Option<String>,
    pub avatar_cid: Option<String>,
    pub mcp_config_cid: Option<String>,
    pub mcp_ports: Option<Vec<crate::router::agent::McpPortConfig>>,
    pub status: AgentStatus,
    pub error: Option<String>,
}

/// 智能体创建状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

