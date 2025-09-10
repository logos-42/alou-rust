use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

/// MCP服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
    pub url: Option<String>,
    pub http_url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub timeout: Option<u64>,
    pub trust: Option<bool>,
    pub include_tools: Option<Vec<String>>,
    pub exclude_tools: Option<Vec<String>>,
    pub oauth: Option<OAuthConfig>,
}

/// OAuth配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub enabled: bool,
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub scopes: Option<Vec<String>>,
}

/// MCP配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// 智能体上下文
pub struct AgentContext {
    pub available_tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
    pub conversation_history: Vec<ConversationMessage>,
    pub current_working_directory: String,
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
}

/// 智能体响应
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub message: String,
    pub actions: Vec<AgentAction>,
    pub content: Option<String>,
    pub memory_updates: Option<serde_json::Value>,
}

/// 智能体动作
#[derive(Debug, Clone)]
pub struct AgentAction {
    pub tool: String,
    pub params: HashMap<String, serde_json::Value>,
    pub result: Option<ToolResult>,
}

/// 工具调用确认结果
#[derive(Debug, Clone, PartialEq)]
pub enum ToolConfirmationOutcome {
    Approved,
    Rejected,
    Unknown,
    ProceedAlwaysServer,
    ProceedAlwaysTool,
}

/// 工具种类
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Other,
}

/// 工具调用确认详情
#[derive(Debug, Clone)]
pub struct ToolCallConfirmationDetails {
    pub tool_name: String,
    pub params: HashMap<String, serde_json::Value>,
}

/// MCP工具调用确认详情
#[derive(Debug, Clone)]
pub struct ToolMcpConfirmationDetails {
    pub tool_type: String,
    pub title: String,
    pub server_name: String,
    pub tool_name: String,
    pub tool_display_name: String,
}

/// 工具结果内容
#[derive(Debug, Clone)]
pub struct ToolResultContent {
    pub content: String,
    pub mime_type: Option<String>,
    pub llm_content: Option<serde_json::Value>,
    pub return_display: Option<String>,
}

/// 函数调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: HashMap<String, serde_json::Value>,
}

/// DeepSeek响应
#[derive(Debug, Clone)]
pub struct DeepseekResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub call_type: ToolCallType,
    pub function: Option<FunctionCallInfo>,
    pub custom: Option<CustomCallInfo>,
}

/// 工具调用类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolCallType {
    Function,
    Custom,
}

/// 函数调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallInfo {
    pub name: String,
    pub arguments: String,
}

/// 自定义调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCallInfo {
    pub name: String,
    pub input: String,
}

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub tool_call_id: String,
    pub name: String,
    pub content: String,
    pub success: bool,
    pub error: Option<String>,
}

/// DeepSeek客户端配置
#[derive(Debug, Clone)]
pub struct DeepseekClientConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub timeout: Option<u64>,
    pub max_retries: Option<u32>,
    pub debug_mode: Option<bool>,
    pub target_dir: Option<String>,
}

/// MCP服务器状态
#[derive(Debug, Clone, PartialEq)]
pub enum McpServerStatus {
    Disconnected,
    Connecting,
    Connected,
}

/// MCP发现状态
#[derive(Debug, Clone, PartialEq)]
pub enum McpDiscoveryState {
    NotStarted,
    InProgress,
    Completed,
}

/// 认证提供者类型
#[derive(Debug, Clone, PartialEq)]
pub enum AuthProviderType {
    GoogleCredentials,
    None,
}

/// 内容部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub text: Option<String>,
    pub inline_data: Option<InlineData>,
    pub function_response: Option<FunctionResponse>,
}

/// 内联数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

/// 函数响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

/// MCP内容块类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContentBlock {
    Text { text: String },
    Image { mime_type: String, data: String },
    Audio { mime_type: String, data: String },
    Resource { resource: ResourceData },
    ResourceLink { uri: String, title: Option<String>, name: Option<String> },
}

/// 资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub text: Option<String>,
    pub blob: Option<String>,
    pub mime_type: Option<String>,
}

/// 可调用工具trait
#[async_trait::async_trait]
pub trait CallableTool {
    async fn call_tool(&self, function_calls: Vec<FunctionCall>) -> Result<Vec<Part>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 工具trait
#[async_trait::async_trait]
pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn display_name(&self) -> &str;
    fn kind(&self) -> Kind;
    fn parameter_schema(&self) -> &serde_json::Value;
    fn is_output_markdown(&self) -> bool;
    fn can_update_output(&self) -> bool;
    
    fn as_any(&self) -> &dyn std::any::Any;
    
    async fn should_confirm_execute(&self, _abort_signal: &CancellationToken) -> Result<Option<ToolCallConfirmationDetails>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
    
    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn build_and_execute(&self, params: HashMap<String, serde_json::Value>, _abort_signal: Option<&CancellationToken>) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>> {
        self.execute(params).await
    }
}

/// 工具调用trait
#[async_trait::async_trait]
pub trait ToolInvocation {
    fn name(&self) -> &str;
    fn params(&self) -> &HashMap<String, serde_json::Value>;
    
    async fn should_confirm_execute(&self, abort_signal: &CancellationToken) -> Result<Option<ToolCallConfirmationDetails>, Box<dyn std::error::Error + Send + Sync>>;
    async fn execute(&self) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>>;
    fn get_description(&self) -> &str;
}
