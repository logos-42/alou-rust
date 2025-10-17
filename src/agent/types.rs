use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::error::Error;
use crate::workspace_context::WorkspaceContext;

/// DeepSeek API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    /// API基础URL
    pub base_url: String,
    /// API密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// 最大token数
    pub max_tokens: u32,
    /// 温度参数
    pub temperature: f32,
}

/// 智能体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// DeepSeek API配置
    pub deepseek: DeepSeekConfig,
    /// 智能体行为配置
    pub behavior: BehaviorConfig,
    /// 工作空间配置
    pub workspace: WorkspaceConfig,
}

/// 智能体行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
    /// 是否启用详细日志
    pub verbose_logging: bool,
    /// 工具调用策略
    pub tool_strategy: ToolStrategy,
}

/// 工具调用策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStrategy {
    /// 自动选择最佳工具
    Auto,
    /// 按优先级顺序调用
    Priority(Vec<String>),
    /// 并行调用多个工具
    Parallel,
}

/// 工作空间配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// 工作空间目录
    pub directories: Vec<String>,
    /// 是否启用智能检测
    pub smart_detection: bool,
    /// 排除的目录模式
    pub exclude_patterns: Vec<String>,
}

/// 智能体状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    /// 空闲状态
    Idle,
    /// 思考状态
    Thinking,
    /// 执行工具状态
    ExecutingTool(String),
    /// 等待API响应状态
    WaitingForAPI,
    /// 错误状态
    Error(String),
}

/// 智能体消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// 消息ID
    pub id: String,
    /// 消息类型
    pub message_type: MessageType,
    /// 消息内容
    pub content: String,
    /// 时间戳
    pub timestamp: u64,
    /// 相关工具调用
    pub tool_calls: Vec<ToolCall>,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 用户输入
    UserInput,
    /// 智能体响应
    AgentResponse,
    /// 工具调用
    ToolCall,
    /// 工具结果
    ToolResult,
    /// 系统消息
    System,
    /// 错误消息
    Error,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 工具名称
    pub name: String,
    /// 工具参数
    pub arguments: HashMap<String, serde_json::Value>,
    /// 调用ID
    pub call_id: String,
    /// 状态
    pub status: ToolCallStatus,
}

/// 工具调用状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCallStatus {
    /// 待执行
    Pending,
    /// 执行中
    Executing,
    /// 成功
    Success,
    /// 失败
    Failed(String),
}

/// 智能体上下文
#[derive(Debug)]
pub struct AgentContext {
    /// 当前状态
    pub state: AgentState,
    /// 消息历史
    pub message_history: Vec<AgentMessage>,
    /// 工作空间上下文
    pub workspace_context: Box<dyn WorkspaceContext + Send + Sync>,
    /// 可用工具列表
    pub available_tools: HashMap<String, ToolInfo>,
    /// 当前任务
    pub current_task: Option<String>,
}

/// 工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 工具参数schema
    pub input_schema: serde_json::Value,
    /// 所属服务器
    pub server: String,
}

/// 智能体trait
#[async_trait]
pub trait Agent: Send + Sync {
    /// 初始化智能体
    async fn initialize(&mut self) -> Result<(), Error>;
    
    /// 处理用户输入
    async fn process_input(&mut self, input: &str) -> Result<String, Error>;
    
    /// 带迭代次数限制的输入处理
    async fn process_input_with_iterations(&mut self, input: &str, max_iterations: usize) -> Result<String, Error>;
    
    /// 执行工具调用
    async fn execute_tool(&mut self, tool_call: &ToolCall) -> Result<serde_json::Value, Error>;
    
    /// 获取当前状态
    fn get_state(&self) -> &AgentState;
    
    /// 获取上下文
    fn get_context(&self) -> &AgentContext;
    
    /// 重置智能体状态
    async fn reset(&mut self) -> Result<(), Error>;
}
