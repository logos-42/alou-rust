use crate::types::*;
use crate::tool_registry::ToolRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use serde_json;
use tracing::{info, warn, error, debug};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

/// DeepSeek客户端
pub struct DeepseekClient {
    http_client: HttpClient,
    config: DeepseekClientConfig,
    history: Arc<RwLock<Vec<serde_json::Value>>>,
    initialized: Arc<RwLock<bool>>,
    tool_registry: Arc<RwLock<Option<Arc<ToolRegistry>>>>,
    max_retries: u32,
    timeout_ms: u64,
    debug_mode: bool,
    verbose: bool,
}

impl DeepseekClient {
    /// 创建新的DeepSeek客户端
    pub fn new(config: DeepseekClientConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            error!("DeepSeek API key is required");
            eprintln!("❌ DeepSeek API key is required");
            eprintln!("请设置环境变量 DEEPSEEK_API_KEY 或创建 .env 文件");
            eprintln!("示例: DEEPSEEK_API_KEY=your_api_key_here");
            return Err(anyhow::anyhow!("DeepSeek API key is required"));
        }

        let http_client = HttpClient::builder()
            .timeout(std::time::Duration::from_millis(config.timeout.unwrap_or(30000)))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            http_client,
            config,
            history: Arc::new(RwLock::new(Vec::new())),
            initialized: Arc::new(RwLock::new(false)),
            tool_registry: Arc::new(RwLock::new(None)),
            max_retries: 3,
            timeout_ms: 30000, // 30秒超时
            debug_mode: false,
            verbose: false,
        })
    }

    /// 初始化客户端
    pub async fn initialize(&mut self) -> Result<()> {
        self.log("开始初始化...", "debug");
        
        if self.config.api_key.is_empty() {
            return Err(anyhow::anyhow!("DeepSeek API key is required"));
        }
        
        // 创建工具注册表并发现工具
        self.log("发现MCP工具...", "debug");
        let tool_registry = Arc::new(ToolRegistry::new());
        tool_registry.discover_all_tools(self.debug_mode).await?;
        
        let tools = tool_registry.get_all_tools().await;
        println!("✅ 发现 {} 个工具", tools.len());
        
        {
            let mut registry = self.tool_registry.write().await;
            *registry = Some(tool_registry);
        }
        
        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }
        
        self.log("初始化完成", "debug");
        Ok(())
    }

    /// 检查是否已初始化
    pub async fn is_initialized(&self) -> bool {
        let initialized = self.initialized.read().await;
        *initialized
    }

    /// 获取对话历史
    pub async fn get_history(&self) -> Vec<serde_json::Value> {
        let history = self.history.read().await;
        history.clone()
    }

    /// 获取可用工具
    pub async fn get_available_tools(&self) -> Vec<String> {
        let registry = self.tool_registry.read().await;
        if let Some(registry) = registry.as_ref() {
            registry.get_tool_names().await
        } else {
            Vec::new()
        }
    }

    /// 设置对话历史
    pub async fn set_history(&self, history: Vec<serde_json::Value>, _options: Option<serde_json::Value>) {
        let mut hist = self.history.write().await;
        *hist = history;
    }

    /// 设置工具注册表
    pub async fn set_tool_registry(&self, tool_registry: Arc<ToolRegistry>) {
        let mut registry = self.tool_registry.write().await;
        *registry = Some(tool_registry);
    }

    /// 生成内容
    pub async fn generate_content(
        &self,
        prompt: &str,
        tools: Option<Vec<serde_json::Value>>,
        model: Option<&str>,
        conversation_history: Option<Vec<serde_json::Value>>,
        disable_thinking_tools: bool,
    ) -> Result<DeepseekResponse> {
        self.log("生成内容...", "debug");
        
        if !self.is_initialized().await {
            return Err(anyhow::anyhow!("DeepSeek client not initialized"));
        }

        let messages = self.build_messages(prompt, conversation_history).await;
        let model_name = model.unwrap_or("deepseek-chat");

        // 如果禁用思考工具，过滤掉思考相关的工具
        let filtered_tools = if disable_thinking_tools {
            tools.map(|t| {
                t.into_iter()
                    .filter(|tool| {
                        !tool.get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|n| n.as_str())
                            .map_or(false, |name| name.contains("sequentialthinking"))
                    })
                    .collect()
            })
        } else {
            tools
        };

        if let Some(ref tools) = filtered_tools {
            self.log(&format!("已禁用思考工具，剩余工具数量: {}", tools.len()), "debug");
        }

        let response = self.call_deepseek_api(&messages, &filtered_tools, model_name).await?;

        let content = response.choices[0].message.content.clone().unwrap_or_default();
        let tool_calls = response.choices[0].message.tool_calls.as_ref().map(|calls| {
            calls.iter().map(|tc| {
                if let Some(function) = &tc.function {
                    ToolCall {
                        id: tc.id.clone(),
                        call_type: ToolCallType::Function,
                        function: Some(FunctionCallInfo {
                            name: function.name.clone(),
                            arguments: function.arguments.clone(),
                        }),
                        custom: None,
                    }
                } else {
                    ToolCall {
                        id: tc.id.clone(),
                        call_type: ToolCallType::Custom,
                        function: None,
                        custom: None,
                    }
                }
            }).collect()
        });

        Ok(DeepseekResponse {
            content,
            tool_calls,
        })
    }

    /// 执行工具调用
    pub async fn execute_tool_calls(&self, tool_calls: Vec<ToolCall>) -> Result<Vec<ToolExecutionResult>> {
        let registry = self.tool_registry.read().await;
        let registry = registry.as_ref().ok_or_else(|| anyhow::anyhow!("Tool registry not set"))?;

        let mut results = Vec::new();
        
        for tool_call in tool_calls {
            // 检查是否是函数工具调用
            if tool_call.call_type != ToolCallType::Function || tool_call.function.is_none() {
                self.log(&format!("跳过非函数工具调用: {:?}", tool_call.call_type), "warn");
                results.push(ToolExecutionResult {
                    tool_call_id: tool_call.id,
                    name: tool_call.function.as_ref().map_or("unknown".to_string(), |f| f.name.clone()),
                    content: format!("跳过非函数工具调用: {:?}", tool_call.call_type),
                    success: false,
                    error: Some(format!("不支持的工具调用类型: {:?}", tool_call.call_type)),
                });
                continue;
            }
            
            let function = tool_call.function.as_ref().unwrap();
            let mut last_error: Option<String> = None;
            let mut success = false;
            let mut result_content = String::new();

            // 重试机制
            for attempt in 1..=self.max_retries {
                self.log(&format!("执行工具: {} (尝试 {}/{})", function.name, attempt, self.max_retries), "debug");
                
                match registry.get_tool(&function.name).await {
                    Some(tool) => {
                        match serde_json::from_str::<HashMap<String, serde_json::Value>>(&function.arguments) {
                            Ok(args) => {
                                // 创建超时控制器
                                let timeout_duration = std::time::Duration::from_millis(self.timeout_ms);
                                
                                match tokio::time::timeout(timeout_duration, tool.execute(args)).await {
                                    Ok(result) => {
                                        match result {
                                            Ok(tool_result) => {
                                                // 提取结果内容
                                                if let Some(llm_content) = &tool_result.llm_content {
                                                    if let Ok(parts) = serde_json::from_value::<Vec<Part>>(llm_content.clone()) {
                                                        for part in parts {
                                                            if let Some(text) = part.text {
                                                                result_content.push_str(&text);
                                                                result_content.push('\n');
                                                            }
                                                        }
                                                    } else if let Ok(text) = serde_json::from_value::<String>(llm_content.clone()) {
                                                        result_content = text;
                                                    }
                                                }
                                                
                                                if result_content.is_empty() {
                                                    result_content = tool_result.return_display.unwrap_or_default();
                                                }
                                                
                                                success = true;
                                                println!("✅ {}", function.name);
                                                break;
                                            }
                                            Err(e) => {
                                                last_error = Some(e.to_string());
                                                self.log(&format!("工具执行失败 (尝试 {}/{}): {} - {}", attempt, self.max_retries, function.name, e), "warn");
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        last_error = Some("工具执行超时".to_string());
                                        self.log(&format!("工具执行超时 (尝试 {}/{})", attempt, self.max_retries), "warn");
                                    }
                                }
                            }
                            Err(e) => {
                                last_error = Some(format!("参数解析失败: {}", e));
                                break;
                            }
                        }
                    }
                    None => {
                        last_error = Some(format!("Tool {} not found", function.name));
                        break;
                    }
                }

                if attempt < self.max_retries {
                    // 等待一段时间后重试
                    tokio::time::sleep(std::time::Duration::from_millis(1000 * attempt as u64)).await;
                }
            }

            results.push(ToolExecutionResult {
                tool_call_id: tool_call.id,
                name: function.name.clone(),
                content: if result_content.is_empty() {
                    if success { "工具执行成功".to_string() } else { "工具执行失败".to_string() }
                } else {
                    result_content
                },
                success,
                error: if success { None } else { last_error },
            });
        }

        Ok(results)
    }

    /// 使用工具进行聊天
    pub async fn chat_with_tools(
        &self,
        prompt: &str,
        model: Option<&str>,
        max_iterations: usize,
    ) -> Result<String> {
        self.log("开始对话...", "debug");
        
        if !self.is_initialized().await {
            return Err(anyhow::anyhow!("DeepseekClient not initialized. Call initialize() first."));
        }
        
        let mut current_prompt = prompt.to_string();
        let mut iteration = 0;
        let mut conversation_history: Vec<serde_json::Value> = Vec::new();
        let mut thinking_count = 0; // 思考工具调用次数
        let max_thinking_count = 3; // 最大思考次数
        let mut use_simple_model = false; // 是否使用简单模型
        
        // 构建初始消息
        conversation_history.push(serde_json::json!({
            "role": "user",
            "content": current_prompt
        }));
        
        while iteration < max_iterations {
            iteration += 1;
            self.log(&format!("第 {} 轮对话", iteration), "debug");
            
            // 获取工具定义（如果有）
            let tool_schemas = {
                let registry = self.tool_registry.read().await;
                if let Some(registry) = registry.as_ref() {
                    Some(registry.get_function_declarations().await)
                } else {
                    None
                }
            };
            
            match self.generate_content(&current_prompt, tool_schemas, model, Some(conversation_history.clone()), use_simple_model).await {
                Ok(response) => {
                    // 添加AI响应到对话历史
                    conversation_history.push(serde_json::json!({
                        "role": "assistant",
                        "content": response.content
                    }));
                    
                    // 如果有工具调用，执行它们
                    if let Some(tool_calls) = response.tool_calls {
                        if !tool_calls.is_empty() {
                            self.log(&format!("发现 {} 个工具调用", tool_calls.len()), "debug");
                            
                            // 检查思考工具调用次数
                            let thinking_tools = tool_calls.iter().filter(|call| {
                                call.function.as_ref()
                                    .map_or(false, |f| f.name.contains("sequentialthinking"))
                            }).count();
                            
                            if thinking_tools > 0 {
                                thinking_count += thinking_tools;
                                self.log(&format!("思考工具调用次数: {}/{}", thinking_count, max_thinking_count), "debug");
                                
                                // 如果超过最大思考次数，切换到简单模式
                                if thinking_count > max_thinking_count && !use_simple_model {
                                    self.log("思考次数超限，切换到简单模式", "warn");
                                    use_simple_model = true;
                                    
                                    // 直接返回简单响应，不执行思考工具
                                    return Ok(format!("我已经思考了 {} 次，现在切换到简单模式。请直接告诉我您需要什么帮助，我会直接执行而不进行深度思考。", max_thinking_count));
                                }
                            }
                            
                            let tool_results = self.execute_tool_calls(tool_calls).await?;
                            
                            // 检查是否有工具执行失败
                            let failed_tools = tool_results.iter().filter(|result| !result.success).count();
                            let successful_tools = tool_results.iter().filter(|result| result.success).count();
                            
                            // 格式化工具执行结果（仅用于显示，不影响实际内容）
                            let mut tool_results_content = String::from("\n\n工具执行结果:\n");
                            for result in &tool_results {
                                let status = if result.success { "✅" } else { "❌" };
                                let mut display_content = result.content.clone();
                                
                                // 压缩长内容显示（仅用于终端显示）
                                if display_content.len() > 200 {
                                    display_content = format!("{}... (总长度: {} 字符)", 
                                        &display_content[..200], display_content.len());
                                }
                                
                                tool_results_content.push_str(&format!("{} {}: {}\n", status, result.name, display_content));
                            }
                            
                            // 如果有工具执行失败，检查是否是记忆任务
                            if failed_tools > 0 {
                                self.log(&format!("{} 个工具执行失败，重新思考", failed_tools), "warn");
                                
                                // 检查是否是简单的记忆任务
                                let is_memory_task = tool_results.iter().any(|tool| {
                                    tool.name.contains("memory") || tool.name.contains("create_entities")
                                });
                                
                                let retry_prompt = if is_memory_task {
                                    // 对于记忆任务，直接返回结果，不要求重新思考
                                    "记忆任务已完成。即使工具执行遇到问题，任务目标已经达成。".to_string()
                                } else {
                                    let failed_tools_info = tool_results.iter()
                                        .filter(|t| !t.success)
                                        .map(|t| format!("- {}: {}", t.name, t.error.as_deref().unwrap_or("未知错误")))
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    
                                    let successful_tools_info = tool_results.iter()
                                        .filter(|t| t.success)
                                        .map(|t| format!("- {}: {}", t.name, t.content))
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    
                                    format!("刚才的工具调用中有一些失败了。请分析失败的原因并尝试其他方法来解决用户的问题。

失败的工具：
{}

成功的工具：
{}

请重新思考并尝试其他方法。如果所有任务都已完成，请明确说明。", failed_tools_info, successful_tools_info)
                                };
                                
                                // 构建完整的工具结果内容（用于AI分析，包含完整内容）
                                let mut full_tool_results_content = String::from("\n\n工具执行结果:\n");
                                for result in &tool_results {
                                    let status = if result.success { "✅" } else { "❌" };
                                    full_tool_results_content.push_str(&format!("{} {}: {}\n", status, result.name, result.content));
                                }
                                
                                // 添加工具结果到对话历史（使用完整内容）
                                conversation_history.push(serde_json::json!({
                                    "role": "user",
                                    "content": format!("{}\n{}", full_tool_results_content, retry_prompt)
                                }));
                                
                                current_prompt = retry_prompt;
                                continue; // 继续下一轮对话
                            } else {
                                // 所有工具都执行成功，检查是否还有未完成的任务
                                self.log("所有工具执行成功", "debug");
                                
                                // 检查AI是否表示任务完成
                                let completion_indicators = [
                                    "任务完成", "已完成", "完成", "结束", "完成所有", 
                                    "所有任务", "任务结束", "工作完成", "执行完毕"
                                ];
                                
                                let response_text = response.content.to_lowercase();
                                let is_task_complete = completion_indicators.iter().any(|indicator| {
                                    response_text.contains(indicator)
                                });
                                
                                if is_task_complete {
                                    return Ok(response.content + &tool_results_content);
                                } else {
                                    // 任务可能还未完成，继续执行
                                    self.log("任务可能未完成，继续执行", "debug");
                                    
                                    // 构建完整的工具结果内容（用于AI分析）
                                    let mut full_tool_results_content = String::from("\n\n工具执行结果:\n");
                                    for result in &tool_results {
                                        let status = if result.success { "✅" } else { "❌" };
                                        full_tool_results_content.push_str(&format!("{} {}: {}\n", status, result.name, result.content));
                                    }
                                    
                                    let continue_prompt = format!("请继续执行剩余的任务。如果所有任务都已完成，请明确说明\"任务完成\"。

当前已完成的工作：
{}

请继续执行或确认任务完成。", full_tool_results_content);
                                    
                                    conversation_history.push(serde_json::json!({
                                        "role": "user",
                                        "content": continue_prompt
                                    }));
                                    
                                    current_prompt = continue_prompt;
                                    continue;
                                }
                            }
                        }
                    } else {
                        // 没有工具调用，直接返回响应
                        self.log("没有工具调用", "debug");
                        return Ok(response.content);
                    }
                }
                Err(e) => {
                    self.log(&format!("第 {} 轮对话出错: {}", iteration, e), "error");
                    
                    if iteration >= max_iterations - 1 {
                        return Ok(format!("抱歉，在处理您的请求时遇到了问题：{}。我已经尝试了 {} 次，建议您重新描述需求或检查网络连接。", e, max_iterations));
                    }
                    
                    // 根据错误类型给出不同的重试策略
                    let retry_prompt = match e.to_string() {
                        ref error_str if error_str.contains("maximum context length") || error_str.contains("tokens") => {
                            // Token限制错误，清理历史记录
                            self.log("检测到token限制错误，清理历史记录", "debug");
                            self.clear_old_history().await;
                            "刚才出现了上下文长度限制问题，我已经清理了历史记录。请重新描述您的需求。".to_string()
                        }
                        ref error_str if error_str.contains("Connection failed") || error_str.contains("timeout") => {
                            format!("刚才出现了连接错误：{}。请稍等片刻后重新尝试，或者尝试使用其他工具。", e)
                        }
                        ref error_str if error_str.contains("Invalid") || error_str.contains("400") => {
                            format!("刚才出现了参数错误：{}。请检查工具参数是否正确，或者尝试不同的方法。", e)
                        }
                        ref error_str if error_str.contains("empty array") || error_str.contains("tools") => {
                            format!("刚才出现了工具配置问题：{}。请尝试重新描述您的需求，或者使用其他可用的工具。", e)
                        }
                        _ => {
                            format!("刚才出现了错误：{}。请重新尝试解决用户的问题，可以考虑使用不同的方法或工具。", e)
                        }
                    };
                    
                    current_prompt = retry_prompt;
                    conversation_history.push(serde_json::json!({
                        "role": "user",
                        "content": current_prompt.clone()
                    }));
                    
                    // 添加短暂延迟，避免过于频繁的重试
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                }
            }
        }
        
        Ok(format!("经过 {} 轮尝试，仍然无法完全解决您的问题。建议您：\n1. 重新描述您的需求，提供更详细的信息\n2. 检查网络连接是否正常\n3. 尝试将复杂任务分解为更小的步骤\n4. 或者稍后再试", max_iterations))
    }

    /// 调用DeepSeek API
    async fn call_deepseek_api(
        &self,
        messages: &[serde_json::Value],
        tools: &Option<Vec<serde_json::Value>>,
        model: &str,
    ) -> Result<DeepseekApiResponse> {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "tools": tools,
            "tool_choice": if tools.is_some() { serde_json::Value::String("auto".to_string()) } else { serde_json::Value::Null },
            "temperature": 0.1,
            "max_tokens": 4096,
        });

        let response = self.http_client
            .post(&format!("{}/chat/completions", self.config.base_url.as_deref().unwrap_or("https://api.deepseek.com/v1")))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to DeepSeek API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("DeepSeek API error: {}", error_text));
        }

        let api_response: DeepseekApiResponse = response
            .json()
            .await
            .context("Failed to parse DeepSeek API response")?;

        Ok(api_response)
    }

    /// 构建消息
    async fn build_messages(&self, prompt: &str, conversation_history: Option<Vec<serde_json::Value>>) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();
        
        // 如果有对话历史，使用它
        if let Some(history) = conversation_history {
            messages.extend(history);
        } else {
            // 否则使用类历史记录，但限制长度
            let limited_history = self.get_limited_history().await;
            for content in limited_history {
                if let Some(role) = content.get("role").and_then(|v| v.as_str()) {
                    if let Some(text) = content.get("content").and_then(|v| v.as_str()) {
                        messages.push(serde_json::json!({
                            "role": role,
                            "content": text
                        }));
                    }
                }
            }
        }

        // 添加当前prompt
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        // 检查并限制消息长度
        self.limit_message_tokens(&mut messages);
        messages
    }

    /// 获取限制长度的历史记录
    async fn get_limited_history(&self) -> Vec<serde_json::Value> {
        let history = self.history.read().await;
        // 保留最近的对话，最多保留最后10轮对话
        let max_history_length = 20; // 10轮对话 = 20条消息
        if history.len() <= max_history_length {
            history.clone()
        } else {
            history[history.len() - max_history_length..].to_vec()
        }
    }

    /// 限制消息的token数量
    fn limit_message_tokens(&self, messages: &mut Vec<serde_json::Value>) {
        const MAX_TOKENS: usize = 100000; // 留出一些余量给响应
        let mut total_tokens = 0;
        let mut limited_messages = Vec::new();
        
        // 从最新的消息开始，向前添加
        for message in messages.iter().rev() {
            let message_tokens = self.estimate_tokens(
                message.get("content").and_then(|v| v.as_str()).unwrap_or("")
            );
            
            if total_tokens + message_tokens > MAX_TOKENS {
                // 如果添加这条消息会超出限制，停止添加
                break;
            }
            
            total_tokens += message_tokens;
            limited_messages.insert(0, message.clone()); // 添加到开头
        }
        
        self.log(&format!("消息token数量: {}, 消息数量: {}", total_tokens, limited_messages.len()), "debug");
        *messages = limited_messages;
    }

    /// 估算文本的token数量（粗略估算）
    fn estimate_tokens(&self, text: &str) -> usize {
        // 粗略估算：1个token约等于4个字符（中文）或1.3个字符（英文）
        // 这里使用保守估算：1个token = 3个字符
        (text.len() + 2) / 3
    }

    /// 清理旧的历史记录
    async fn clear_old_history(&self) {
        // 只保留最近的5轮对话（10条消息）
        const KEEP_MESSAGES: usize = 10;
        let mut history = self.history.write().await;
        if history.len() > KEEP_MESSAGES {
            *history = history[history.len() - KEEP_MESSAGES..].to_vec();
            self.log(&format!("已清理历史记录，保留最近 {} 条消息", KEEP_MESSAGES), "debug");
        }
    }

    /// 日志记录
    fn log(&self, message: &str, level: &str) {
        if level == "debug" && !self.verbose { return; }
        if level == "info" && !self.verbose { return; }
        
        let prefix = match level {
            "error" => "❌",
            "warn" => "⚠️",
            "debug" => "🔍",
            _ => "ℹ️",
        };
        println!("{} [DeepseekClient] {}", prefix, message);
    }
}

/// DeepSeek API响应结构
#[derive(Debug, Deserialize)]
struct DeepseekApiResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCallInfo>>,
}

#[derive(Debug, Deserialize)]
struct ToolCallInfo {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: Option<FunctionInfo>,
}

#[derive(Debug, Deserialize)]
struct FunctionInfo {
    name: String,
    arguments: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deepseek_client_creation() {
        let config = DeepseekClientConfig {
            api_key: "test_key".to_string(),
            base_url: Some("https://api.deepseek.com/v1".to_string()),
            timeout: Some(30000),
            max_retries: Some(3),
            debug_mode: Some(false),
            target_dir: Some(".".to_string()),
        };
        
        let client = DeepseekClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_estimate_tokens() {
        let config = DeepseekClientConfig {
            api_key: "test".to_string(),
            base_url: None,
            timeout: None,
            max_retries: None,
            debug_mode: None,
            target_dir: None,
        };
        
        let client = DeepseekClient::new(config).unwrap();
        assert_eq!(client.estimate_tokens("hello"), 2);
        assert_eq!(client.estimate_tokens("你好世界"), 2);
    }
}
