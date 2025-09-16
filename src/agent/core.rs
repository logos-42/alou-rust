use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn};

use crate::connection_pool::{ConnectionPool, McpServerConfig};
use crate::protocol::Request;
use crate::error::Error;

use super::types::{
    Agent, AgentConfig, AgentContext, AgentState, AgentMessage, MessageType, 
    ToolCall, ToolCallStatus, ToolInfo
};
use super::deepseek::{DeepSeekClient, ChatMessage};

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
            "{}\n\n# 当前可用工具\n{}\n\n# 重要：工具使用要求\n- 你必须使用可用的工具来完成用户的请求\n- 对于文件操作（创建文件夹、创建文件等），必须调用相应的工具\n- 不要只是描述要做什么，而是实际执行工具调用\n- 每个操作都需要使用对应的工具函数\n\n# 智能体状态\n当前状态: {:?}\n当前任务: {}",
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
    
    /// 处理用户输入
    async fn process_input(&mut self, input: &str) -> Result<String, Error> {
        self.process_input_with_iterations(input, 10).await
    }

    /// 带迭代次数限制的输入处理
    async fn process_input_with_iterations(&mut self, input: &str, max_iterations: usize) -> Result<String, Error> {
        tracing::info!("处理用户输入: {}", input);
        
        // 更新状态
        {
            let mut context = self.context.write().await;
            context.current_task = Some(input.to_string());
        }
        
        let mut current_input = input.to_string();
        let mut iteration = 0;
        
        loop {
            iteration += 1;
            if iteration > max_iterations {
                return Ok(format!("经过 {} 轮尝试，仍然无法完全解决您的问题。", max_iterations));
            }
            
            tracing::info!("对话循环第 {} 轮，当前输入: {}", iteration, current_input);
            
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
                
                // 添加用户消息到历史（仅在第一轮）
                if iteration == 1 {
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
            }
            
            // 构建消息列表
            let mut messages = vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: self.build_system_prompt().await,
                    tool_calls: None,
                    tool_call_id: None,
                }
            ];
            
            // 添加历史消息和当前输入
            {
                let context = self.context.read().await;
                
                // 添加历史消息（除了最后一轮的用户输入，因为我们要用current_input替换）
                for msg in &context.message_history {
                    match msg.message_type {
                        MessageType::UserInput => {
                            // 第一轮使用原始输入，后续轮次跳过，使用current_input
                            if iteration == 1 {
                                messages.push(ChatMessage {
                                    role: "user".to_string(),
                                    content: msg.content.clone(),
                                    tool_calls: None,
                                    tool_call_id: None,
                                });
                            }
                        }
                        MessageType::AgentResponse => {
                            // 传递所有Assistant消息，包括有工具调用的
                            messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: msg.content.clone(),
                                tool_calls: None, // 暂时不发送工具调用，避免复杂的格式问题
                                tool_call_id: None,
                            });
                        }
                        MessageType::ToolResult => {
                            // 暂时跳过工具结果消息，避免API格式错误
                            // 工具执行的结果已经体现在后续的对话中
                            continue;
                        }
                        _ => continue,
                    }
                }
                
                // 添加当前轮次的用户输入（后续轮次使用current_input）
                if iteration > 1 {
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: current_input.clone(),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }
            }
            
            // 构建工具调用schema
            let tools_schema = {
                let context = self.context.read().await;
                tracing::info!("可用工具数量: {}", context.available_tools.len());
                if context.available_tools.is_empty() {
                    tracing::warn!("没有可用的工具！");
                } else {
                    tracing::info!("可用工具: {:?}", context.available_tools.keys().collect::<Vec<_>>());
                }
                
                let schema = context.available_tools
                    .values()
                    .map(|tool| {
                        serde_json::json!({
                            "type": "function",
                            "function": {
                                "name": tool.name,
                                "description": tool.description,
                                "parameters": tool.input_schema
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                
                tracing::debug!("发送给DeepSeek的工具schema: {}", serde_json::to_string_pretty(&schema).unwrap_or_default());
                schema
            };
            
            // 调用DeepSeek API
            {
                let mut context = self.context.write().await;
                context.state = AgentState::WaitingForAPI;
            }
            
            let response = self.deepseek_client.chat_with_tools(messages, tools_schema).await?;
            
            if let Some(choice) = response.choices.first() {
                let response_content = choice.message.content.clone();
                
                // 调试：打印响应内容
                tracing::info!("=== 第{}轮 DeepSeek响应 ===", iteration);
                tracing::info!("响应内容: {}", response_content);
                tracing::info!("是否有工具调用: {:?}", choice.message.tool_calls.is_some());
                if let Some(tool_calls) = &choice.message.tool_calls {
                    tracing::info!("工具调用数量: {}", tool_calls.len());
                    for (i, tc) in tool_calls.iter().enumerate() {
                        tracing::info!("工具调用 {}: {} - {}", i+1, tc.function.name, tc.function.arguments);
                    }
                } else {
                    tracing::warn!("AI没有调用任何工具！");
                }
                tracing::debug!("完整响应: {}", serde_json::to_string_pretty(&choice.message).unwrap_or_default());
                
                // 检查是否有工具调用
                if let Some(tool_calls) = &choice.message.tool_calls {
                    if !tool_calls.is_empty() {
                        // 执行工具调用
                        let mut tool_results = Vec::new();
                        let mut executed_tool_calls = Vec::new();
                        let mut has_failed_tools = false;
                        
                        for tool_call in tool_calls {
                            // 解析工具调用
                            let name = &tool_call.function.name;
                            let arguments = &tool_call.function.arguments;
                            
                            // 解析参数
                            let args: HashMap<String, serde_json::Value> = 
                                match serde_json::from_str(arguments) {
                                    Ok(args) => args,
                                    Err(e) => {
                                        tracing::error!("工具调用参数解析失败: {}, 原始参数: {}", e, arguments);
                                        HashMap::new()
                                    }
                                };
                            
                            tracing::debug!("工具调用: {}, 参数: {:?}", name, args);
                            
                            let tool_call_info = ToolCall {
                                name: name.clone(),
                                arguments: args,
                                call_id: tool_call.id.clone().unwrap_or_default(),
                                status: ToolCallStatus::Pending,
                            };
                            
                            // 执行工具
                            match self.execute_tool(&tool_call_info).await {
                                Ok(result) => {
                                    // 将工具执行结果转换为字符串
                                    let result_str = if result.is_string() {
                                        result.as_str().unwrap_or("").to_string()
                                    } else {
                                        serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                                    };
                                    
                                    tool_results.push(format!("工具 {} 执行成功，结果：{}", name, result_str));
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
                                    has_failed_tools = true;
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
                                tool_calls: executed_tool_calls.clone(),
                            };
                            context.message_history.push(tool_result_message);
                        }
                        
                        // 构建下一轮的输入
                        if has_failed_tools {
                            // 有工具执行失败，构建重试提示
                            let failed_tools = executed_tool_calls.iter()
                                .filter(|tc| matches!(tc.status, ToolCallStatus::Failed(_)))
                                .map(|tc| {
                                    if let ToolCallStatus::Failed(error) = &tc.status {
                                        format!("- {}: {}", tc.name, error)
                                    } else {
                                        format!("- {}", tc.name)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            
                            let successful_results = tool_results.iter()
                                .filter(|result| result.contains("执行成功"))
                                .cloned()
                                .collect::<Vec<_>>()
                                .join("\n");
                            
                            current_input = format!(
                                "刚才的工具调用结果：

成功的工具执行结果：
{}

失败的工具：
{}

请分析失败的原因并尝试其他方法来解决用户的问题。如果所有任务都已完成，请明确说明。",
                                successful_results, failed_tools
                            );
                        } else {
                            // 检查任务是否完成
                            let completion_indicators = [
                                "任务完成", "已完成", "完成", "结束", "完成所有",
                                "所有任务", "任务结束", "工作完成", "执行完毕"
                            ];
                            
                            let is_task_complete = completion_indicators.iter().any(|&indicator| 
                                response_content.contains(indicator)
                            );
                            
                            if is_task_complete {
                                // 任务完成，返回最终结果
                                {
                                    let mut context = self.context.write().await;
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
                                return Ok(response_content);
                            } else {
                                // 任务未完成，继续执行，但要告知AI前面的工具执行结果
                                let all_results = tool_results.join("\n");
                                current_input = format!(
                                    "刚才的工具执行结果：
{}

请根据上述执行结果，继续执行剩余的任务。如果所有任务都已完成，请明确说明\"任务完成\"。",
                                    all_results
                                );
                            }
                        }
                        
                        // 添加AI响应到对话历史
                        {
                            let mut context = self.context.write().await;
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
                        
                        // 继续下一轮对话
                        continue;
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
                
                return Ok(response_content);
            } else {
                return Err(Error::Other("API响应中没有选择".to_string()));
            }
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
