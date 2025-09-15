use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn};

use crate::connection_pool::{ConnectionPool, McpServerConfig};
use crate::workspace_context::WorkspaceContext;
use crate::protocol::Request;
use crate::error::Error;

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
    
    /// 执行工具调用
    async fn execute_tool(&mut self, tool_call: &ToolCall) -> Result<serde_json::Value, Error>;
    
    /// 获取当前状态
    fn get_state(&self) -> &AgentState;
    
    /// 获取上下文
    fn get_context(&self) -> &AgentContext;
    
    /// 重置智能体状态
    async fn reset(&mut self) -> Result<(), Error>;
}

/// 智能体实现
pub struct McpAgent {
    /// 配置
    #[allow(dead_code)]
    config: AgentConfig,
    /// 上下文
    context: Arc<RwLock<AgentContext>>,
    /// MCP连接池
    connection_pool: Arc<ConnectionPool>,
    /// DeepSeek客户端
    deepseek_client: Arc<DeepSeekClient>,
}

/// DeepSeek API客户端
pub struct DeepSeekClient {
    /// HTTP客户端
    client: reqwest::Client,
    /// 配置
    config: DeepSeekConfig,
}

impl DeepSeekClient {
    /// 创建新的DeepSeek客户端
    pub fn new(config: DeepSeekConfig) -> Self {
        let client = reqwest::Client::new();
        Self { client, config }
    }
    
    /// 发送聊天请求
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse, Error> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: Some(false),
        };
        
        let response = self.client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Other(format!("API请求失败: {}", error_text)));
        }
        
        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
            
        Ok(chat_response)
    }
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 角色
    pub role: String,
    /// 内容
    pub content: String,
    /// 工具调用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallMessage>>,
}

/// 工具调用消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallMessage {
    /// 工具调用ID
    pub id: Option<String>,
    /// 工具调用类型
    #[serde(rename = "type")]
    pub call_type: String,
    /// 函数调用
    pub function: FunctionCall,
}

/// 函数调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// 函数名称
    pub name: String,
    /// 函数参数
    pub arguments: String,
}

/// 聊天请求
#[derive(Debug, Serialize)]
struct ChatRequest {
    /// 模型名称
    model: String,
    /// 消息列表
    messages: Vec<ChatMessage>,
    /// 最大token数
    max_tokens: Option<u32>,
    /// 温度参数
    temperature: Option<f32>,
    /// 是否流式响应
    stream: Option<bool>,
}

/// 聊天响应
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// 选择列表
    pub choices: Vec<Choice>,
    /// 使用情况
    pub usage: Option<Usage>,
}

/// 选择
#[derive(Debug, Deserialize)]
pub struct Choice {
    /// 消息
    pub message: ChatMessage,
    /// 完成原因
    pub finish_reason: Option<String>,
}

/// 使用情况
#[derive(Debug, Deserialize)]
pub struct Usage {
    /// 提示token数
    pub prompt_tokens: u32,
    /// 完成token数
    pub completion_tokens: u32,
    /// 总token数
    pub total_tokens: u32,
}

impl McpAgent {
    /// 创建新的智能体
    pub async fn new(config: AgentConfig) -> Result<Self, Error> {
        Self::with_connection_pool(config, Arc::new(ConnectionPool::new())).await
    }
    
    /// 使用指定的连接池创建智能体
    pub async fn with_connection_pool(config: AgentConfig, connection_pool: Arc<ConnectionPool>) -> Result<Self, Error> {
        // 创建工作空间上下文
        let workspace_context = if config.workspace.smart_detection {
            crate::workspace_context::WorkspaceContextFactory::create_smart()
        } else {
            let directories: Vec<PathBuf> = config.workspace.directories
                .iter()
                .map(|s| PathBuf::from(s))
                .collect();
            crate::workspace_context::WorkspaceContextFactory::create_custom(directories)
        };
        
        // 创建DeepSeek客户端
        let deepseek_client = Arc::new(DeepSeekClient::new(config.deepseek.clone()));
        
        // 创建智能体上下文
        let context = Arc::new(RwLock::new(AgentContext {
            state: AgentState::Idle,
            message_history: Vec::new(),
            workspace_context,
            available_tools: HashMap::new(),
            current_task: None,
        }));
        
        Ok(Self {
            config,
            context,
            connection_pool,
            deepseek_client,
        })
    }
    
    /// 注册服务器配置（不立即连接）
    async fn register_server_configs(&mut self) -> Result<(), Error> {
        // 从mcp.json加载服务器配置
        if std::path::Path::new("mcp.json").exists() {
            let content = std::fs::read_to_string("mcp.json")?;
            let mcp_config: serde_json::Value = serde_json::from_str(&content)?;
            
            if let Some(servers) = mcp_config.get("mcpServers").and_then(|s| s.as_object()) {
                for (name, config) in servers {
                    let server_config = McpServerConfig {
                        command: config.get("command")
                            .and_then(|c| c.as_str())
                            .unwrap_or("")
                            .to_string(),
                        args: config.get("args")
                            .and_then(|a| a.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                            .unwrap_or_default(),
                        env: Some(config.get("env")
                            .and_then(|e| e.as_object())
                            .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect::<std::collections::HashMap<String, String>>())
                            .unwrap_or_default()),
                        directory: config.get("directory")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                    };
                    
                    self.connection_pool.register_server(name.clone(), server_config).await;
                    info!("已注册MCP服务器配置: {}", name);
                }
            }
        } else {
            warn!("未找到mcp.json文件，将使用默认配置");
        }
        
        Ok(())
    }

    /// 启动后台加载MCP服务器
    async fn start_background_loading(&self) {
        let connection_pool = self.connection_pool.clone();
        let context = self.context.clone();
        
        // 在后台任务中加载工具
        tokio::spawn(async move {
            // 检查是否已经在加载或已加载完成
            {
                let ctx = context.read().await;
                if !ctx.available_tools.is_empty() {
                    return;
                }
            }
            
            // 获取所有已注册的服务器
            let servers = connection_pool.list_registered_servers().await;
            let mut server_tool_counts = std::collections::HashMap::new();
            
            for server_name in &servers {
                if let Ok(connection) = connection_pool.get_connection(server_name).await {
                    let client = connection.lock().await;
                    
                    // 获取工具列表
                    match client.request("tools/list", None).await {
                        Ok(result) => {
                            if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                                let tool_count = tools.len();
                                server_tool_counts.insert(server_name.clone(), tool_count);
                                
                                // 将工具添加到上下文中
                                {
                                    let mut ctx = context.write().await;
                                    for tool in tools {
                                        if let (Some(name), Some(description), Some(input_schema)) = (
                                            tool.get("name").and_then(|n| n.as_str()),
                                            tool.get("description").and_then(|d| d.as_str()),
                                            tool.get("inputSchema")
                                        ) {
                                            let tool_info = ToolInfo {
                                                name: name.to_string(),
                                                description: description.to_string(),
                                                input_schema: input_schema.clone(),
                                                server: server_name.clone(),
                                            };
                                            ctx.available_tools.insert(name.to_string(), tool_info);
                                        }
                                    }
                                }
                                
                                // 静默加载，不输出日志
                            }
                        }
                        Err(_) => {
                            // 静默处理错误，不输出日志
                        }
                    }
                }
            }
            
            // 后台加载完成，静默处理-
        });
    }


    /// 静默发现可用工具（不输出任何日志）
    async fn discover_tools_silent(&mut self) -> Result<(), Error> {
        let mut context = self.context.write().await;
        context.available_tools.clear();
        
        // 从连接池获取所有已注册的服务器
        let servers = self.connection_pool.list_registered_servers().await;
        let mut server_tool_counts = std::collections::HashMap::new();
        
        for server_name in &servers {
            if let Ok(connection) = self.connection_pool.get_connection(server_name).await {
                let client = connection.lock().await;
                
                // 获取工具列表
                match client.request("tools/list", None).await {
                    Ok(result) => {
                        if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                                let tool_count = tools.len();
                            server_tool_counts.insert(server_name.clone(), tool_count);
                                
                            // 将工具添加到上下文中
                                for tool in tools {
                                if let (Some(name), Some(description), Some(input_schema)) = (
                                        tool.get("name").and_then(|n| n.as_str()),
                                    tool.get("description").and_then(|d| d.as_str()),
                                    tool.get("inputSchema")
                                    ) {
                                        let tool_info = ToolInfo {
                                            name: name.to_string(),
                                            description: description.to_string(),
                                        input_schema: input_schema.clone(),
                                        server: server_name.clone(),
                                        };
                                        context.available_tools.insert(name.to_string(), tool_info);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // 静默处理错误
                    }
                }
            }
        }
        
        // 静默完成，不输出任何信息
        Ok(())
    }
    
    /// 获取workspace上下文信息
    pub async fn get_workspace_info(&self) -> Vec<std::path::PathBuf> {
        let context = self.context.read().await;
        context.workspace_context.get_directories()
    }

    /// 构建系统提示
    async fn build_system_prompt(&self) -> String {
        let context = self.context.read().await;
        let workspace_dirs = context.workspace_context.get_directories();
        let workspace_root = workspace_dirs
            .first()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        
        // 使用现有的MCP系统提示词
        let base_prompt = crate::prompts::get_mcp_system_prompt(&workspace_root);
        
        // 添加可用工具信息
        let tools_info = context.available_tools
            .values()
            .map(|tool| format!("- {}: {}", tool.name, tool.description))
            .collect::<Vec<_>>()
            .join("\n");
        
        format!(
            "{}\n\n# 当前可用工具\n{}\n\n# 智能体状态\n当前状态: {:?}\n当前任务: {}",
            base_prompt,
            tools_info,
            context.state,
            context.current_task.as_deref().unwrap_or("无")
        )
    }
}

#[async_trait]
impl Agent for McpAgent {
    async fn initialize(&mut self) -> Result<(), Error> {
        tracing::info!("初始化智能体...");
        
        // 只注册服务器配置，不立即连接
        self.register_server_configs().await?;
        
        // 更新状态
        {
            let mut context = self.context.write().await;
            context.state = AgentState::Idle;
        }
        
        tracing::info!("智能体初始化完成");
        
        // 启动后台加载MCP服务器
        self.start_background_loading().await;
        
        Ok(())
    }
    
    async fn process_input(&mut self, input: &str) -> Result<String, Error> {
        tracing::info!("处理用户输入: {}", input);
        
        // 静默检查工具是否已加载，如果没有则等待后台加载或手动加载
        {
            let context = self.context.read().await;
            if context.available_tools.is_empty() {
                drop(context); // 释放读锁
                // 静默等待后台加载完成，最多等待5秒
                let start_time = std::time::Instant::now();
                while start_time.elapsed().as_secs() < 5 {
                    let context = self.context.read().await;
                    if !context.available_tools.is_empty() {
                        drop(context);
                        break;
                    }
                    drop(context);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                
                // 如果后台加载还没完成，静默手动加载
                {
                    let context = self.context.read().await;
                    if context.available_tools.is_empty() {
                        drop(context);
                        self.discover_tools_silent().await?;
                    }
                }
            }
        }
        
        // 更新状态
        {
            let mut context = self.context.write().await;
            context.state = AgentState::Thinking;
            context.current_task = Some(input.to_string());
            
            // 添加用户消息到历史
            let user_message = AgentMessage {
                id: Uuid::new_v4().to_string(),
                message_type: MessageType::UserInput,
                content: input.to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                tool_calls: Vec::new(),
            };
            context.message_history.push(user_message);
        }
        
        // 构建消息列表
        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: self.build_system_prompt().await,
                tool_calls: None,
            }
        ];
        
        // 添加历史消息
        {
            let context = self.context.read().await;
            for msg in &context.message_history {
                let role = match msg.message_type {
                    MessageType::UserInput => "user",
                    MessageType::AgentResponse => "assistant",
                    _ => continue,
                };
                messages.push(ChatMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                    tool_calls: None,
                });
            }
        }
        
        // 调用DeepSeek API
        {
            let mut context = self.context.write().await;
            context.state = AgentState::WaitingForAPI;
        }
        
        let response = self.deepseek_client.chat(messages).await?;
        
        if let Some(choice) = response.choices.first() {
            let response_content = choice.message.content.clone();
            
            // 调试：打印响应内容
            tracing::debug!("DeepSeek响应内容: {}", response_content);
            tracing::debug!("是否有工具调用: {:?}", choice.message.tool_calls);
            
            // 检查是否有工具调用
            if let Some(tool_calls) = &choice.message.tool_calls {
                if !tool_calls.is_empty() {
                    // 执行工具调用
                    let mut tool_results = Vec::new();
                    let mut executed_tool_calls = Vec::new();
                    
                    for tool_call in tool_calls {
                        // 解析工具调用
                        let name = &tool_call.function.name;
                        let arguments = &tool_call.function.arguments;
                        
                        // 解析参数
                        let args: HashMap<String, serde_json::Value> = 
                            serde_json::from_str(arguments).unwrap_or_default();
                        
                        let tool_call_info = ToolCall {
                            name: name.clone(),
                            arguments: args,
                            call_id: tool_call.id.clone().unwrap_or_default(),
                            status: ToolCallStatus::Pending,
                        };
                        
                        // 执行工具
                        match self.execute_tool(&tool_call_info).await {
                            Ok(result) => {
                                tool_results.push(format!("工具 {} 执行成功: {}", name, result));
                                executed_tool_calls.push(ToolCall {
                                    name: name.clone(),
                                    arguments: tool_call_info.arguments,
                                    call_id: tool_call_info.call_id,
                                    status: ToolCallStatus::Success,
                                });
                            }
                            Err(e) => {
                                tool_results.push(format!("工具 {} 执行失败: {}", name, e));
                                executed_tool_calls.push(ToolCall {
                                    name: name.clone(),
                                    arguments: tool_call_info.arguments,
                                    call_id: tool_call_info.call_id,
                                    status: ToolCallStatus::Failed(e.to_string()),
                                });
                            }
                        }
                    }
                    
                    // 将工具结果添加到消息历史
                    {
                        let mut context = self.context.write().await;
                        context.state = AgentState::Idle;
                        
                        let tool_result_message = AgentMessage {
                            id: Uuid::new_v4().to_string(),
                            message_type: MessageType::ToolResult,
                            content: tool_results.join("\n"),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            tool_calls: executed_tool_calls,
                        };
                        context.message_history.push(tool_result_message);
                    }
                    
                    // 返回工具执行结果
                    return Ok(tool_results.join("\n"));
                }
            }
            
            // 没有工具调用，直接返回响应
            {
                let mut context = self.context.write().await;
                context.state = AgentState::Idle;
                
                let agent_message = AgentMessage {
                    id: Uuid::new_v4().to_string(),
                    message_type: MessageType::AgentResponse,
                    content: response_content.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    tool_calls: Vec::new(),
                };
                context.message_history.push(agent_message);
            }
            
            Ok(response_content)
        } else {
            Err(Error::Other("API响应中没有选择".to_string()))
        }
    }
    
    async fn execute_tool(&mut self, tool_call: &ToolCall) -> Result<serde_json::Value, Error> {
        tracing::info!("执行工具: {}", tool_call.name);
        
        // 更新状态
        {
            let mut context = self.context.write().await;
            context.state = AgentState::ExecutingTool(tool_call.name.clone());
        }
        
        // 获取工具信息
        let tool_info = {
            let context = self.context.read().await;
            context.available_tools.get(&tool_call.name)
                .ok_or_else(|| Error::Other(format!("工具 {} 不存在", tool_call.name)))?
                .clone()
        };
        
        // 获取连接
        let connection = self.connection_pool.get_connection(&tool_info.server).await?;
        let client = connection.lock().await;
        
        // 构建工具调用请求
        let request = Request {
            jsonrpc: "2.0".to_string(),
            id: crate::protocol::RequestId::String(tool_call.call_id.clone()),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": tool_call.name,
                "arguments": tool_call.arguments
            })),
        };
        
        // 发送请求
        let response = client.request(&request.method, request.params).await?;
        
        // 更新状态
        {
            let mut context = self.context.write().await;
            context.state = AgentState::Idle;
        }
        
        Ok(response)
    }
    
    fn get_state(&self) -> &AgentState {
        // 这里需要返回引用，但Arc<RwLock<>> 使得这变得复杂
        // 在实际使用中，应该通过方法获取状态
        &AgentState::Idle // 临时实现
    }
    
    fn get_context(&self) -> &AgentContext {
        // 由于AgentContext包含Arc<RwLock<>>，这里需要重新设计
        // 临时实现 - 返回一个静态的空上下文
        // 在实际使用中，应该通过异步方法获取上下文
        unimplemented!("需要重新设计状态访问方式 - 使用异步方法获取上下文")
    }
    
    async fn reset(&mut self) -> Result<(), Error> {
        let mut context = self.context.write().await;
        context.state = AgentState::Idle;
        context.message_history.clear();
        context.current_task = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            deepseek: DeepSeekConfig {
                base_url: "https://api.deepseek.com".to_string(),
                api_key: "test_key".to_string(),
                model: "deepseek-chat".to_string(),
                max_tokens: 1000,
                temperature: 0.7,
            },
            behavior: BehaviorConfig {
                max_retries: 3,
                timeout_seconds: 30,
                verbose_logging: true,
                tool_strategy: ToolStrategy::Auto,
            },
            workspace: WorkspaceConfig {
                directories: vec![".".to_string()],
                smart_detection: true,
                exclude_patterns: vec!["target".to_string()],
            },
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.deepseek.model, deserialized.deepseek.model);
    }
}
