//! 兼容性数据结构 - 用于保持与claude-agent.js的兼容性

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 历史消息结构（兼容claude-agent.js）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,    // "user" | "assistant"
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

/// 工具定义（兼容claude-agent.js）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub arguments: Value,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool: String,
    pub result: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 兼容claude-agent.js的请求格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibleRequest {
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    pub prompt: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub history: Vec<HistoryMessage>,
    #[serde(rename = "agentInfo")]
    pub agent_info: Option<Value>,
    #[serde(default)]
    pub tools: Vec<Tool>,
    pub model: String,
    #[serde(rename = "maxTokens")]
    pub max_tokens: Option<i32>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    
    // 新增字段（向后兼容）
    #[serde(rename = "taskType")]
    pub task_type: Option<String>, // "sync" | "async"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(rename = "callbackUrl")]
    pub callback_url: Option<String>,
    
    // 现有后端字段（保持兼容）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(default)]
    pub context_events: Vec<Value>,
}

fn default_temperature() -> f32 {
    0.7
}

impl Default for CompatibleRequest {
    fn default() -> Self {
        Self {
            api_key: None,
            prompt: String::new(),
            system_prompt: None,
            history: Vec::new(),
            agent_info: None,
            tools: Vec::new(),
            model: "deepseek-chat".to_string(),
            max_tokens: Some(4096),
            temperature: 0.7,
            task_type: None,
            timeout: None,
            callback_url: None,
            session_id: None,
            wallet_address: None,
            chain: None,
            context_events: Vec::new(),
        }
    }
}

/// 兼容claude-agent.js的响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibleResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    #[serde(rename = "toolCalls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    
    // 新增字段（用于异步任务）
    #[serde(rename = "taskId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // "queued", "running", "completed", "failed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    #[serde(rename = "estimatedTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CompatibleResponse {
    /// 创建成功的同步响应
    pub fn success_response(response: String, tool_calls: Option<Vec<ToolCall>>) -> Self {
        Self {
            success: true,
            response: Some(response),
            tool_calls,
            metadata: Some(Value::Object(serde_json::Map::new())),
            task_id: None,
            status: None,
            progress: None,
            estimated_time: None,
            error: None,
        }
    }
    
    /// 创建异步任务响应
    pub fn async_task(task_id: String, estimated_time: u64) -> Self {
        Self {
            success: true,
            response: None,
            tool_calls: None,
            metadata: Some(Value::Object(serde_json::Map::new())),
            task_id: Some(task_id),
            status: Some("queued".to_string()),
            progress: Some(0.0),
            estimated_time: Some(estimated_time),
            error: None,
        }
    }
    
    /// 创建错误响应
    pub fn error_response(error: String) -> Self {
        Self {
            success: false,
            response: None,
            tool_calls: None,
            metadata: None,
            task_id: None,
            status: None,
            progress: None,
            estimated_time: None,
            error: Some(error),
        }
    }
}

/// 任务状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub status: String, // "queued", "running", "completed", "failed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    #[serde(rename = "currentStep")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<CompatibleResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    #[serde(rename = "updatedAt")]
    pub updated_at: u64,
}

impl TaskStatusResponse {
    pub fn new(task_id: String, status: String) -> Self {
        // 使用兼容WASM的时间函数
        let now = crate::utils::time::current_timestamp_secs();
        
        Self {
            task_id,
            status,
            progress: None,
            current_step: None,
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 任务状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Processing,  // 新增：正在处理中（如调用AI、执行工具等）
    Completed,
    Failed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Queued => write!(f, "queued"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Processing => write!(f, "processing"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
        }
    }
}

/// 判断是否应该使用异步处理
pub fn should_use_async(request: &CompatibleRequest) -> bool {
    // 如果明确指定了任务类型
    if let Some(task_type) = &request.task_type {
        return task_type == "async";
    }
    
    // 根据模型和内容判断
    let is_long_model = request.model.contains("claude-3-5-sonnet") || 
                       request.model.contains("claude-3-opus");
    
    let is_long_prompt = request.prompt.len() > 1000 || 
                        request.history.len() > 10;
    
    let has_complex_tools = !request.tools.is_empty();
    
    is_long_model || is_long_prompt || has_complex_tools
}

/// 估计任务执行时间（秒）
pub fn estimate_execution_time(request: &CompatibleRequest) -> u64 {
    let base_time = 10; // 基础时间
    
    let prompt_length_factor = (request.prompt.len() as f32 / 1000.0).ceil() as u64 * 5;
    let history_factor = request.history.len() as u64 * 2;
    let tools_factor = request.tools.len() as u64 * 3;
    
    // 模型因子
    let model_factor = if request.model.contains("claude-3-5-sonnet") {
        15
    } else if request.model.contains("claude-3-opus") {
        20
    } else {
        5
    };
    
    base_time + prompt_length_factor + history_factor + tools_factor + model_factor
}










