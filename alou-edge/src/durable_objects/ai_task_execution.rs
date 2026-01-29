use crate::agent::ai_client::{AiClient, AiMessage};
use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, TaskStatus, ToolCall};
use crate::durable_objects::ai_task_state::TaskState;
use crate::durable_objects::ai_task_ai::DefaultAiCaller;
use crate::durable_objects::ai_task_persistence::TaskPersistence;
use crate::durable_objects::ai_task_state::{get_state_key, get_conversation_history_key, get_tool_result_key};
use worker::{Result, Error, console_log, console_error, Storage};

/// TaskExecutor trait 定义任务执行器接口
pub trait TaskExecutor {
    /// 执行完整任务
    async fn execute_full_task(&self) -> Result<()>;
    
    /// 基于工具结果继续对话
    async fn continue_with_tool_results(&self) -> Result<()>;
}

/// 任务执行器实现
pub struct TaskExecutorImpl<'a> {
    pub ctx: TaskExecutionContext<'a>,
    pub ai_caller: DefaultAiCaller,
}

/// 任务执行上下文
pub struct TaskExecutionContext<'a> {
    pub storage: &'a Storage,
    pub task_name: &'a str,
    pub env: &'a worker::Env,
}

impl<'a> TaskExecutionContext<'a> {
    /// 创建新的任务执行上下文
    pub fn new(storage: &'a Storage, task_name: &'a str, env: &'a worker::Env) -> Self {
        Self { storage, task_name, env }
    }

    /// 获取当前时间戳（秒）
    pub fn get_current_timestamp(&self) -> u64 {
        worker::Date::now().as_millis() / 1000
    }

    /// 获取当前时间戳（毫秒）
    pub fn get_current_timestamp_millis(&self) -> u64 {
        worker::Date::now().as_millis()
    }
}

impl<'a> TaskExecutorImpl<'a> {
    /// 创建新的任务执行器
    pub fn new(ctx: TaskExecutionContext<'a>) -> Self {
        Self {
            ai_caller: DefaultAiCaller::new(),
            ctx,
        }
    }

    /// 执行完整任务
    pub async fn execute_full_task(&self) -> Result<()> {
        console_log!("=== TASK EXECUTION START ===");
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        // 1. 更新状态为运行中
        let mut state = self.load_state().await?;
        state.status = TaskStatus::Running;
        state.progress = 0.1;
        state.current_step = "初始化任务执行".to_string();
        state.updated_at = self.ctx.get_current_timestamp();
        self.save_state(&state).await?;

        // 2. 获取请求数据
        let request = match TaskPersistence::get_request(self.ctx.storage, self.ctx.task_name).await? {
            Some(req) => req,
            None => {
                state.status = TaskStatus::Failed;
                state.error = Some("请求数据不存在".to_string());
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(Error::RustError("Request data not found".to_string()));
            }
        };

        // 3. 初始化AI客户端
        let ai_api_key = match self.ctx.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.ctx.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    state.status = TaskStatus::Failed;
                    state.error = Some("AI_API_KEY 或 DEEPSEEK_API_KEY 未配置".to_string());
                    state.progress = 1.0;
                    self.save_state(&state).await?;
                    return Err(Error::RustError("AI key not configured".to_string()));
                }
            }
        };

        let ai_client = AiClient::new(
            request.provider.as_deref().unwrap_or("deepseek"), 
            ai_api_key, 
            Some(request.model.clone())
        )
            .map_err(|e| {
                state.status = TaskStatus::Failed;
                state.error = Some(format!("创建AI客户端失败: {}", e));
                state.progress = 1.0;
                Error::RustError(format!("Failed to create AI client: {}", e))
            })?;

        // 4. 准备消息
        let mut messages = self.ai_caller.convert_to_ai_messages(&request);
        let tools = self.ai_caller.convert_to_ai_tools(&request.tools);

        // 5. 调用AI服务
        state.progress = 0.3;
        state.current_step = "调用AI服务".to_string();
        state.updated_at = self.ctx.get_current_timestamp();
        self.save_state(&state).await?;

        match self.ai_caller.call_ai_with_timeout(ai_client, messages, tools, self.ctx.task_name).await {
            Ok(ai_response) => {
                // 保存对话历史
                let mut conversation_history = Vec::new();
                conversation_history.push(AiMessage::assistant_with_tools(ai_response.content.clone(), ai_response.tool_calls.clone()));
                self.save_conversation_history(&conversation_history).await?;

                // 检查是否有工具调用
                if !ai_response.tool_calls.is_empty() {
                    console_log!("🔧 AI requested {} tool calls", ai_response.tool_calls.len());

                    // 保存待处理的工具调用
                    let tool_calls: Vec<ToolCall> = ai_response.tool_calls.iter().map(|tc| {
                        ToolCall {
                            tool: tc.name.clone(),
                            arguments: tc.arguments.clone(),
                            id: Some(tc.id.clone()),
                        }
                    }).collect();
                    TaskPersistence::save_pending_tool_calls(self.ctx.storage, self.ctx.task_name, &tool_calls).await?;

                    // 为 DeepSeek API 添加占位工具响应消息
                    for tc in &ai_response.tool_calls {
                        conversation_history.push(AiMessage {
                            role: "tool".to_string(),
                            content: "".to_string(), // 使用空内容作为占位，等待实际工具结果
                            tool_call_id: Some(tc.id.clone()),
                            tool_calls: None,
                        });
                    }
                    self.save_conversation_history(&conversation_history).await?;

                    // 更新状态为等待工具结果
                    state.status = TaskStatus::Processing;
                    state.progress = 0.7;
                    state.current_step = format!("等待执行 {} 个工具调用", tool_calls.len());
                    state.updated_at = self.ctx.get_current_timestamp();
                    self.save_state(&state).await?;

                    // 设置alarm等待工具结果（5秒后检查）
                    let now_ms = self.ctx.get_current_timestamp_millis();
                    self.ctx.storage.set_alarm((now_ms + 5000) as i64).await?;

                    console_log!("⏰ Set alarm to wait for tool results");
                    Ok(())
                } else {
                    // 没有工具调用，直接完成
                    let response = self.ai_caller.convert_from_ai_response(ai_response, self.ctx.task_name);
                    let _ = TaskPersistence::save_result(self.ctx.storage, self.ctx.task_name, &response).await;

                    // 更新状态为完成
                    state.status = TaskStatus::Completed;
                    state.progress = 1.0;
                    state.current_step = "任务完成".to_string();
                    state.updated_at = self.ctx.get_current_timestamp();
                    self.save_state(&state).await?;

                    console_log!("✅ Task completed without tool calls");
                    Ok(())
                }
            }
            Err(e) => {
                // 更新状态为失败
                state.status = TaskStatus::Failed;
                state.error = Some(format!("AI服务错误: {}", e));
                state.progress = 1.0;
                state.current_step = "任务失败".to_string();
                state.updated_at = self.ctx.get_current_timestamp();
                self.save_state(&state).await?;

                console_error!("❌ Task failed: {}", e);
                Err(e)
            }
        }
    }

    /// 基于 Ralph Loop 循环调用工具的对话继续
    pub async fn continue_with_tool_results(&self) -> Result<()> {
        console_log!("[RALPH-LOOP] 🔄 Starting AI-driven Ralph Loop for tool-based conversation");

        // 1. 更新状态
        let mut state = self.load_state().await?;
        state.status = TaskStatus::Processing;
        state.progress = 0.8;
        state.current_step = "Ralph Loop: 基于工具结果继续对话".to_string();
        state.updated_at = self.ctx.get_current_timestamp();
        self.save_state(&state).await?;

        // 2. 检查并处理待处理的工具结果
        console_log!("[RALPH-LOOP] Checking for pending tool results to process");
        let pending_tool_calls = TaskPersistence::load_pending_tool_calls(self.ctx.storage, self.ctx.task_name).await?;
        
        // 如果没有待处理的工具调用，直接跳过
        let pending_calls = match pending_tool_calls {
            Some(calls) if !calls.is_empty() => calls,
            _ => {
                console_log!("[RALPH-LOOP] No pending tool calls found");
                self.ctx.storage.delete(&format!("pending_tool_calls_{}", self.ctx.task_name)).await?;
                // 获取原始请求和对话历史
                let request = match TaskPersistence::get_request(self.ctx.storage, self.ctx.task_name).await? {
                    Some(req) => req,
                    None => {
                        let mut state = self.load_state().await?;
                        state.status = TaskStatus::Failed;
                        state.error = Some("请求数据不存在".to_string());
                        state.progress = 1.0;
                        self.save_state(&state).await?;
                        return Err(Error::RustError("Request data not found".to_string()));
                    }
                };
                return self.continue_without_tool_results(request, state).await;
            }
        };
        console_log!("[RALPH-LOOP] Found {} pending tool calls, checking for results", pending_calls.len());
        
        // 检查工具结果
        let tool_result_key = get_tool_result_key(self.ctx.task_name);
        let tool_results_data: Option<String> = match self.ctx.storage.get::<String>(&tool_result_key).await {
            Ok(data) => Some(data),
            Err(_) => {
                console_log!("[RALPH-LOOP] No tool results found, continuing without them");
                None
            }
        };

        // 处理工具结果
        if let Some(ref tool_results_data) = tool_results_data {
            console_log!("[RALPH-LOOP] Processing tool results data");
            self.process_tool_results(tool_results_data, &pending_calls).await?;
        }

        // 清除待处理的工具调用
        self.ctx.storage.delete(&format!("pending_tool_calls_{}", self.ctx.task_name)).await?;
        console_log!("[RALPH-LOOP] Cleared pending tool calls");

        // 3. 获取原始请求
        let request = match TaskPersistence::get_request(self.ctx.storage, self.ctx.task_name).await? {
            Some(req) => req,
            None => {
                state.status = TaskStatus::Failed;
                state.error = Some("请求数据不存在".to_string());
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(Error::RustError("Request data not found".to_string()));
            }
        };

        // 4. Ralph Loop 核心逻辑：循环进行 AI 对话直到任务完成
        self.execute_ralph_loop(request, state).await
    }

    /// Ralph Loop 核心执行逻辑 - AI 自主决定何时结束
    async fn execute_ralph_loop(&self, request: CompatibleRequest, mut state: TaskState) -> Result<()> {
        console_log!("[RALPH-LOOP] 🚀 Executing AI-driven Ralph Loop core logic");
        
        // 初始化AI客户端
        let ai_api_key = match self.ctx.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.ctx.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    state.status = TaskStatus::Failed;
                    state.error = Some("AI_API_KEY 或 DEEPSEEK_API_KEY 未配置".to_string());
                    state.progress = 1.0;
                    self.save_state(&state).await?;
                    return Err(Error::RustError("AI key not configured".to_string()));
                }
            }
        };

        // Ralph Loop: AI 自主决定何时结束，设置软性安全限制
        let soft_max_iterations = 50; // 软性限制，防止无限循环
        let mut current_iteration = 0;
        let mut consecutive_no_tool_calls = 0; // 连续无工具调用次数
        let max_consecutive_no_tool_calls = 3; // 连续3次无工具调用则认为完成

        console_log!("[RALPH-LOOP] 🚀 Starting AI-driven Ralph Loop with soft limit of {}", soft_max_iterations);

        loop {
            current_iteration += 1;
            console_log!("[RALPH-LOOP] 🔄 Iteration {} (soft limit: {})", current_iteration, soft_max_iterations);

            // 安全检查：达到软性限制时强制结束
            if current_iteration >= soft_max_iterations {
                console_error!("[RALPH-LOOP] ⚠️ Soft limit reached, forcing completion");
                break;
            }

            // 准备消息（包含完整的对话历史）
            let messages = self.ai_caller.convert_to_ai_messages(&request);

            // 添加对话历史（包括所有之前的工具调用和工具结果）
            let mut messages = messages;
            if let Ok(Some(history)) = self.load_conversation_history().await {
                messages.extend(history);
            }

            let tools = self.ai_caller.convert_to_ai_tools(&request.tools);

            // 更新状态
            state.current_step = format!("Ralph Loop 第 {} 轮对话", current_iteration);
            state.updated_at = self.ctx.get_current_timestamp();
            self.save_state(&state).await?;

            // 调用AI服务进行下一轮对话 - 每次重新创建客户端避免移动问题
            let ai_client_for_iteration = AiClient::new(
                request.provider.as_deref().unwrap_or("deepseek"), 
                ai_api_key.clone(), 
                Some(request.model.clone())
            )
                .map_err(|e| {
                    state.status = TaskStatus::Failed;
                    state.error = Some(format!("创建AI客户端失败: {}", e));
                    state.progress = 1.0;
                    Error::RustError(format!("Failed to create AI client: {}", e))
                })?;

            match self.ai_caller.call_ai_with_timeout(ai_client_for_iteration, messages, tools, self.ctx.task_name).await {
                Ok(ai_response) => {
                    // 保存对话历史
                    let mut conversation_history = self.load_conversation_history().await.unwrap_or(Some(Vec::new())).unwrap_or_default();
                    conversation_history.push(AiMessage::assistant_with_tools(ai_response.content.clone(), ai_response.tool_calls.clone()));
                    self.save_conversation_history(&conversation_history).await?;

                    // 检查是否有新的工具调用
                    if !ai_response.tool_calls.is_empty() {
                        console_log!("[RALPH-LOOP] 🔧 AI requested {} tool calls in iteration {}", ai_response.tool_calls.len(), current_iteration);
                        
                        // 重置连续无工具调用计数
                        consecutive_no_tool_calls = 0;

                        // 保存新的待处理工具调用
                        let tool_calls: Vec<ToolCall> = ai_response.tool_calls.iter().map(|tc| {
                            ToolCall {
                                tool: tc.name.clone(),
                                arguments: tc.arguments.clone(),
                                id: Some(tc.id.clone()),
                            }
                        }).collect();
                        TaskPersistence::save_pending_tool_calls(self.ctx.storage, self.ctx.task_name, &tool_calls).await?;

                        // 为 DeepSeek API 添加占位工具响应消息
                        for tc in &ai_response.tool_calls {
                            conversation_history.push(AiMessage {
                                role: "tool".to_string(),
                                content: "".to_string(), // 使用空内容作为占位，等待实际工具结果
                                tool_call_id: Some(tc.id.clone()),
                                tool_calls: None,
                            });
                        }
                        self.save_conversation_history(&conversation_history).await?;

                        // 保持Processing状态，等待工具结果
                        state.current_step = format!("Ralph Loop: 等待执行 {} 个工具调用", tool_calls.len());
                        state.updated_at = self.ctx.get_current_timestamp();
                        self.save_state(&state).await?;

                        // 设置alarm等待工具结果（较短时间，提高响应速度）
                        let now_ms = self.ctx.get_current_timestamp_millis();
                        self.ctx.storage.set_alarm((now_ms + 500) as i64).await?;

                        console_log!("[RALPH-LOOP] ⏰ Set alarm for tool results in 500ms, continuing loop");
                        return Ok(()); // 暂时退出，等待工具结果后继续循环
                    } else {
                        // 没有工具调用，增加连续无工具调用计数
                        consecutive_no_tool_calls += 1;
                        console_log!("[RALPH-LOOP] 📝 No tool calls in iteration {} (consecutive: {})", current_iteration, consecutive_no_tool_calls);
                        
                        // 检查AI是否明确表示任务完成
                        let ai_content_lower = ai_response.content.to_lowercase();
                        let completion_indicators = [
                            "任务完成", "已完成", "完成", "结束了", "没有更多",
                            "task completed", "completed", "finished", "done", "no more",
                            "不需要", "不需要了", "就这样", "可以了"
                        ];
                        
                        let is_explicit_completion = completion_indicators.iter().any(|indicator| {
                            ai_content_lower.contains(&indicator.to_lowercase())
                        });
                        
                        // 如果AI明确表示完成，或者连续多次无工具调用，则结束
                        if is_explicit_completion || consecutive_no_tool_calls >= max_consecutive_no_tool_calls {
                            console_log!("[RALPH-LOOP] ✅ Conversation completed after {} iterations", current_iteration);
                            if is_explicit_completion {
                                console_log!("[RALPH-LOOP] 🎯 AI explicitly indicated completion");
                            } else {
                                console_log!("[RALPH-LOOP] ⏹️ Ending due to {} consecutive no-tool calls", consecutive_no_tool_calls);
                            }
                            
                            let response = self.ai_caller.convert_from_ai_response(ai_response, self.ctx.task_name);
                            let _ = TaskPersistence::save_result(self.ctx.storage, self.ctx.task_name, &response).await;

                            // 更新状态为完成
                            state.status = TaskStatus::Completed;
                            state.progress = 1.0;
                            state.current_step = format!("Ralph Loop 完成 (共 {} 轮对话)", current_iteration);
                            state.updated_at = self.ctx.get_current_timestamp();
                            self.save_state(&state).await?;

                            return Ok(());
                        } else {
                            // 继续下一轮对话，让AI决定是否还需要更多信息
                            console_log!("[RALPH-LOOP] 🔄 Continuing to next iteration ({} consecutive no-tool calls)", consecutive_no_tool_calls);
                            
                            // 更新进度，但保持Processing状态
                            state.progress = 0.5 + (current_iteration as f32 / soft_max_iterations as f32) * 0.4;
                            state.current_step = format!("Ralph Loop 第 {} 轮对话 (连续 {} 次无工具调用)", current_iteration, consecutive_no_tool_calls);
                            state.updated_at = self.ctx.get_current_timestamp();
                            self.save_state(&state).await?;
                            
                            // 继续循环，不退出
                            continue;
                        }
                    }
                }
                Err(e) => {
                    // 更新状态为失败
                    state.status = TaskStatus::Failed;
                    state.error = Some(format!("Ralph Loop AI服务错误: {}", e));
                    state.progress = 1.0;
                    state.current_step = format!("Ralph Loop 失败 (第 {} 轮)", current_iteration);
                    state.updated_at = self.ctx.get_current_timestamp();
                    self.save_state(&state).await?;

                    console_error!("[RALPH-LOOP] ❌ Ralph Loop failed: {}", e);
                    return Err(e);
                }
            }
        }

        // 达到软性循环次数，强制结束
        console_error!("[RALPH-LOOP] ⚠️ Soft limit ({}) reached, forcing completion", soft_max_iterations);
        
        // 保存当前对话结果
        let conversation_history = self.load_conversation_history().await.unwrap_or(Some(Vec::new())).unwrap_or_default();
        let final_response = CompatibleResponse {
            success: true,
            response: Some(format!("Ralph Loop 达到软性循环次数 {}，已执行多轮工具调用对话。AI 自主决定何时结束，但为防止无限循环设置了安全限制。请查看对话历史获取完整信息。", soft_max_iterations)),
            tool_calls: None,
            metadata: None,
            task_id: Some(self.ctx.task_name.to_string()),
            status: Some("completed".to_string()),
            progress: Some(1.0),
            estimated_time: None,
            error: Some(format!("达到软性循环次数限制 {}", soft_max_iterations)),
        };
        
        let _ = TaskPersistence::save_result(self.ctx.storage, self.ctx.task_name, &final_response).await;

        // 更新状态
        state.status = TaskStatus::Completed;
        state.progress = 1.0;
        state.current_step = format!("Ralph Loop 达到软性循环次数 {}", soft_max_iterations);
        state.updated_at = self.ctx.get_current_timestamp();
        self.save_state(&state).await?;

        Ok(())
    }

    /// 继续对话（不处理工具结果）
    async fn continue_without_tool_results(&self, request: CompatibleRequest, mut state: TaskState) -> Result<()> {
        console_log!("[CONTINUE] Continuing without tool results");
        
        // 初始化AI客户端
        let ai_api_key = match self.ctx.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.ctx.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    state.status = TaskStatus::Failed;
                    state.error = Some("AI_API_KEY 或 DEEPSEEK_API_KEY 未配置".to_string());
                    state.progress = 1.0;
                    self.save_state(&state).await?;
                    return Err(Error::RustError("AI key not configured".to_string()));
                }
            }
        };

        let ai_client = AiClient::new(
            request.provider.as_deref().unwrap_or("deepseek"), 
            ai_api_key, 
            Some(request.model.clone())
        )
            .map_err(|e| {
                state.status = TaskStatus::Failed;
                state.error = Some(format!("创建AI客户端失败: {}", e));
                state.progress = 1.0;
                Error::RustError(format!("Failed to create AI client: {}", e))
            })?;

        // 准备消息
        let mut messages = self.ai_caller.convert_to_ai_messages(&request);
        if let Ok(Some(history)) = self.load_conversation_history().await {
            messages.extend(history);
        }

        let tools = self.ai_caller.convert_to_ai_tools(&request.tools);

        // 调用AI服务
        state.progress = 0.9;
        state.current_step = "进行下一轮AI对话".to_string();
        self.save_state(&state).await?;

        match self.ai_caller.call_ai_with_timeout(ai_client, messages, tools, self.ctx.task_name).await {
            Ok(ai_response) => {
                // 保存对话历史
                let mut conversation_history = self.load_conversation_history().await.unwrap_or(Some(Vec::new())).unwrap_or_default();
                conversation_history.push(AiMessage::assistant_with_tools(ai_response.content.clone(), ai_response.tool_calls.clone()));
                self.save_conversation_history(&conversation_history).await?;

                // 检查是否有新的工具调用
                if !ai_response.tool_calls.is_empty() {
                    console_log!("🔄 [CONTINUE] AI requested {} more tool calls", ai_response.tool_calls.len());

                    // 保存新的待处理工具调用
                    let tool_calls: Vec<ToolCall> = ai_response.tool_calls.iter().map(|tc| {
                        ToolCall {
                            tool: tc.name.clone(),
                            arguments: tc.arguments.clone(),
                            id: Some(tc.id.clone()),
                        }
                    }).collect();
                    TaskPersistence::save_pending_tool_calls(self.ctx.storage, self.ctx.task_name, &tool_calls).await?;

                    // 保持Processing状态，等待更多工具结果
                    state.current_step = format!("等待执行 {} 个新工具调用", tool_calls.len());
                    state.updated_at = self.ctx.get_current_timestamp();
                    self.save_state(&state).await?;

                    // 设置alarm等待新工具结果
                    let now_ms = self.ctx.get_current_timestamp_millis();
                    self.ctx.storage.set_alarm((now_ms + 5000) as i64).await?;

                    console_log!("⏰ [CONTINUE] Set alarm for new tool results");
                    Ok(())
                } else {
                    // 没有更多工具调用，对话完成
                    let response = self.ai_caller.convert_from_ai_response(ai_response, self.ctx.task_name);
                    let _ = TaskPersistence::save_result(self.ctx.storage, self.ctx.task_name, &response).await;

                    // 更新状态为完成
                    state.status = TaskStatus::Completed;
                    state.progress = 1.0;
                    state.current_step = "任务完成".to_string();
                    state.updated_at = self.ctx.get_current_timestamp();
                    self.save_state(&state).await?;

                    console_log!("✅ [CONTINUE] Conversation completed without further tool calls");
                    Ok(())
                }
            }
            Err(e) => {
                // 更新状态为失败
                state.status = TaskStatus::Failed;
                state.error = Some(format!("AI服务错误: {}", e));
                state.current_step = "任务失败".to_string();
                state.updated_at = self.ctx.get_current_timestamp();
                self.save_state(&state).await?;

                console_error!("❌ [CONTINUE] Task failed: {}", e);
                Err(e)
            }
        }
    }

    /// 处理工具结果
    async fn process_tool_results(&self, tool_results_data: &str, pending_calls: &[ToolCall]) -> Result<()> {
        console_log!("[PROCESS] Processing tool results data");
        
        let tool_results: serde_json::Value = serde_json::from_str(tool_results_data)
            .map_err(|e| Error::RustError(format!("Failed to parse tool results: {}", e)))?;
        
        let results_array = tool_results.get("results")
            .and_then(|r| r.as_array())
            .ok_or_else(|| Error::RustError("Invalid tool results format".to_string()))?;
        
        let mut conversation_history = self.load_conversation_history().await?.unwrap_or_default();
        
        for (result_index, tool_result) in results_array.iter().enumerate() {
            let tool_name = tool_result.get("tool")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");
            
            let success = tool_result.get("success")
                .and_then(|s| s.as_bool())
                .unwrap_or(false);
            
            let result_content = if success {
                tool_result.get("result")
                    .and_then(|r| r.as_str())
                    .unwrap_or("工具执行成功")
            } else {
                tool_result.get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("工具执行失败")
            };
            
            // 查找对应的 tool_call_id
            let tool_call_id = pending_calls.iter()
                .find(|tc| tc.tool == tool_name)
                .and_then(|tc| tc.id.clone())
                .unwrap_or_else(|| format!("{}_{}", tool_name, result_index));
            
            console_log!("[PROCESS] Adding tool response for {} (success: {}, tool_call_id: {})", tool_name, success, tool_call_id);
            
            // 添加工具响应消息到对话历史
            conversation_history.push(AiMessage {
                role: "tool".to_string(),
                content: result_content.to_string(),
                tool_call_id: Some(tool_call_id),
                tool_calls: None,
            });
        }
        
        console_log!("[PROCESS] Updated conversation history: {} messages", conversation_history.len());
        self.save_conversation_history(&conversation_history).await?;
        
        // 清除工具结果数据
        let tool_result_key = get_tool_result_key(self.ctx.task_name);
        self.ctx.storage.delete(&tool_result_key).await?;
        console_log!("[PROCESS] Cleared tool results data");
        
        Ok(())
    }

    /// 加载状态
    async fn load_state(&self) -> Result<TaskState> {
        let state_key = get_state_key(self.ctx.task_name);
        match self.ctx.storage.get::<TaskState>(&state_key).await {
            Ok(state) => Ok(state),
            Err(_) => Ok(TaskState::new_initial())
        }
    }

    /// 保存状态
    async fn save_state(&self, state: &TaskState) -> Result<()> {
        let state_key = get_state_key(self.ctx.task_name);
        self.ctx.storage.put(&state_key, state).await
    }

    /// 加载对话历史
    async fn load_conversation_history(&self) -> Result<Option<Vec<AiMessage>>> {
        let history_key = get_conversation_history_key(self.ctx.task_name);
        match self.ctx.storage.get::<Vec<AiMessage>>(&history_key).await {
            Ok(history) => Ok(Some(history)),
            Err(_) => Ok(None)
        }
    }

    /// 保存对话历史
    async fn save_conversation_history(&self, history: &[AiMessage]) -> Result<()> {
        let history_key = get_conversation_history_key(self.ctx.task_name);
        self.ctx.storage.put(&history_key, history).await
    }
}

impl<'a> TaskExecutor for TaskExecutorImpl<'a> {
    async fn execute_full_task(&self) -> Result<()> {
        self.execute_full_task().await
    }
    
    async fn continue_with_tool_results(&self) -> Result<()> {
        self.continue_with_tool_results().await
    }
}
