//! AI任务执行模块
//!
//! 负责任务的执行逻辑、Alarm处理和超时控制

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, ToolCall, TaskStatus};
use crate::durable_objects::ai_task::AITaskDO;
use crate::agent::ai_client::{AiClient, AiMessage};
use crate::durable_objects::ai_task_state::{TaskState, get_state_key, get_request_key, get_result_key, get_pending_tool_calls_key, get_conversation_history_key, get_tool_result_key};
use crate::durable_objects::ai_task_ai::DefaultAiCaller;
use crate::durable_objects::ai_task_persistence::TaskPersistence;
use crate::utils::time::current_timestamp_secs;
use worker::{Result, console_log, console_error};

#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use futures::future::{select, Either};
#[cfg(target_arch = "wasm32")]
use std::pin::Pin;

/// 任务执行上下文
pub struct TaskExecutionContext<'a> {
    pub storage: &'a worker::Storage,
    pub task_name: &'a str,
    pub env: &'a worker::Env,
}

impl<'a> TaskExecutionContext<'a> {
    pub fn new(storage: &'a worker::Storage, task_name: &'a str, env: &'a worker::Env) -> Self {
        Self { storage, task_name, env }
    }

    pub fn get_current_timestamp(&self) -> u64 {
        current_timestamp_secs()
    }

    pub fn get_current_timestamp_millis(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::Date;
            Date::now() as u64
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        }
    }
}

/// 任务执行trait
pub trait TaskExecutor {
    /// 处理任务初始化和启动（合并操作）
    async fn handle_init_and_start(&self, req: worker::Request) -> Result<worker::Response>;

    /// 处理任务开始执行
    async fn handle_start(&self) -> Result<worker::Response>;

    /// 执行AI任务
    async fn execute_task(&self) -> Result<()>;

    /// 简化版任务执行
    async fn execute_task_simplified(&self) -> Result<()>;

    /// 执行任务（带超时保护）
    async fn execute_task_with_timeout(&self) -> Result<()>;

    /// 加载状态（带超时保护）
    async fn load_state_with_timeout(&self) -> Result<TaskState>;
}

/// 任务执行器
pub struct TaskExecutorImpl<'a> {
    pub ctx: TaskExecutionContext<'a>,
    pub ai_caller: DefaultAiCaller,
}

impl<'a> TaskExecutorImpl<'a> {
    pub fn new(ctx: TaskExecutionContext<'a>) -> Self {
        Self {
            ctx,
            ai_caller: DefaultAiCaller,
        }
    }

    /// 加载状态
    async fn load_state(&self) -> Result<TaskState> {
        console_log!("[TASK-EXECUTOR] load_state called");

        let storage = self.ctx.storage;
        let task_name = self.ctx.task_name;
        let state_key = get_state_key(task_name);
        let state_data = storage.get::<serde_json::Value>(&state_key).await?;

        let status = state_data.get("status")
            .and_then(|s| s.as_str())
            .map(|s| match s {
                "queued" => TaskStatus::Queued,
                "running" => TaskStatus::Running,
                "processing" => TaskStatus::Processing,
                "completed" => TaskStatus::Completed,
                "failed" => TaskStatus::Failed,
                _ => TaskStatus::Queued,
            })
            .unwrap_or(TaskStatus::Queued);

        let progress = state_data.get("progress")
            .and_then(|p| p.as_f64())
            .map(|p| p as f32)
            .unwrap_or(0.0);

        let current_step = state_data.get("current_step")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "初始化".to_string());

        let created_at = state_data.get("created_at")
            .and_then(|c| c.as_u64())
            .unwrap_or_else(current_timestamp_secs);

        let updated_at = state_data.get("updated_at")
            .and_then(|u| u.as_u64())
            .unwrap_or(created_at);

        let error = state_data.get("error")
            .and_then(|e| e.as_str())
            .map(|e| e.to_string());

        console_log!("[TASK-EXECUTOR] Successfully loaded state");
        Ok(TaskState {
            status,
            progress,
            current_step,
            created_at,
            updated_at,
            error,
        })
    }

    /// 保存状态
    async fn save_state(&self, state: &TaskState) -> Result<()> {
        let storage = self.ctx.storage;
        let task_name = self.ctx.task_name;

        let state_data = serde_json::json!({
            "status": state.status.to_string(),
            "progress": state.progress,
            "current_step": state.current_step,
            "created_at": state.created_at,
            "updated_at": state.updated_at,
            "error": state.error,
        });

        console_log!("[DEBUG] Saving state");
        let state_key = get_state_key(task_name);
        storage.put(&state_key, state_data).await?;
        Ok(())
    }

    /// 加载对话历史
    async fn load_conversation_history(&self) -> Result<Option<Vec<AiMessage>>> {
        let storage = self.ctx.storage;
        let task_name = self.ctx.task_name;
        let key = get_conversation_history_key(task_name);
        Ok(storage.get::<Vec<AiMessage>>(&key).await.ok())
    }

    /// 保存对话历史
    async fn save_conversation_history(&self, history: &[AiMessage]) -> Result<()> {
        let storage = self.ctx.storage;
        let task_name = self.ctx.task_name;
        let key = get_conversation_history_key(task_name);
        storage.put(&key, history).await?;
        Ok(())
    }

    /// 执行完整任务
    pub async fn execute_full_task(&self) -> Result<()> {
        console_log!("[EXECUTE] Starting task execution for: {}", self.ctx.task_name);

        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        // 1. 更新状态
        let mut state = self.load_state().await?;
        console_log!("[EXECUTE] Loaded initial state: status={}, progress={}", state.status, state.progress);
        state.progress = 0.2;
        state.current_step = "加载请求数据".to_string();
        state.updated_at = self.ctx.get_current_timestamp();
        self.save_state(&state).await?;
        console_log!("[EXECUTE] State updated to loading request data");

        // 2. 获取请求数据
        let request = match TaskPersistence::get_request(self.ctx.storage, self.ctx.task_name).await? {
            Some(req) => req,
            None => {
                state.status = TaskStatus::Failed;
                state.error = Some("请求数据不存在".to_string());
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(worker::Error::RustError("Request data not found".to_string()));
            }
        };

        // 3. 初始化AI客户端
        state.progress = 0.3;
        state.current_step = "初始化AI客户端".to_string();
        self.save_state(&state).await?;
        console_log!("[EXECUTE] Initializing AI client");

        let ai_api_key = match self.ctx.env.secret("AI_API_KEY") {
            Ok(key) => {
                console_log!("[EXECUTE] Found AI_API_KEY");
                key.to_string()
            },
            Err(_) => match self.ctx.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => {
                    console_log!("[EXECUTE] Found DEEPSEEK_API_KEY");
                    key.to_string()
                },
                Err(_) => {
                    console_error!("[EXECUTE] No AI API key configured");
                    state.status = TaskStatus::Failed;
                    state.error = Some("AI_API_KEY 或 DEEPSEEK_API_KEY 未配置".to_string());
                    state.progress = 1.0;
                    self.save_state(&state).await?;
                    return Err(worker::Error::RustError("AI key not configured".to_string()));
                }
            }
        };

        console_log!("[EXECUTE] Creating AI client with model: {}", &request.model);
        let ai_client = AiClient::new("deepseek", ai_api_key, Some(request.model.clone()))
            .map_err(|e| {
                console_error!("[EXECUTE] Failed to create AI client: {}", e);
                state.status = TaskStatus::Failed;
                state.error = Some(format!("创建AI客户端失败: {}", e));
                state.progress = 1.0;
                worker::Error::RustError(format!("Failed to create AI client: {}", e))
            })?;
        console_log!("[EXECUTE] AI client created successfully");

        // 4. 准备消息和工具
        let mut messages = self.ai_caller.convert_to_ai_messages(&request);

        // 如果有对话历史，添加到消息中
        if let Ok(Some(history)) = self.load_conversation_history().await {
            messages.extend(history);
        }

        let tools = self.ai_caller.convert_to_ai_tools(&request.tools);

        // 5. 调用AI服务
        state.progress = 0.5;
        state.current_step = "调用AI服务".to_string();
        self.save_state(&state).await?;
        console_log!("[EXECUTE] Calling AI service with {} messages and {} tools", messages.len(), tools.len());

        match self.ai_caller.call_ai_with_timeout(ai_client, messages, tools, self.ctx.task_name).await {
            Ok(ai_response) => {
                // 保存对话历史
                let mut conversation_history = self.load_conversation_history().await.unwrap_or(Some(Vec::new())).unwrap_or_default();
                conversation_history.push(AiMessage::assistant_with_tools(ai_response.content.clone(), ai_response.tool_calls.clone()));
                self.save_conversation_history(&conversation_history).await?;

                // 检查是否有工具调用
                if !ai_response.tool_calls.is_empty() {
                    console_log!("🔧 [EXECUTE] AI returned {} tool calls, waiting for execution", ai_response.tool_calls.len());

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
                    // DeepSeek 要求：assistant 消息包含 tool_calls 后必须跟随工具响应消息
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

                    console_log!("⏰ [EXECUTE] Set alarm to wait for tool results");
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

                    console_log!("✅ [EXECUTE] Task completed without tool calls");
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

                console_error!("❌ [EXECUTE] Task failed: {}", e);
                Err(e)
            }
        }
    }

    /// 处理工具结果
    async fn process_tool_results(&self, tool_results_data: &str, pending_calls: &[ToolCall]) -> Result<()> {
        console_log!("[PROCESS] Processing tool results data");
        
        let tool_results: serde_json::Value = serde_json::from_str(tool_results_data)
            .map_err(|e| worker::Error::RustError(format!("Failed to parse tool results: {}", e)))?;
        
        let results_array = tool_results.get("results")
            .and_then(|r| r.as_array())
            .ok_or_else(|| worker::Error::RustError("Invalid tool results format".to_string()))?;
        
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
        let tool_result_key = format!("tool_result_{}", self.ctx.task_name);
        self.ctx.storage.delete(&tool_result_key).await?;
        console_log!("[PROCESS] Cleared tool results data");
        
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
                    return Err(worker::Error::RustError("AI key not configured".to_string()));
                }
            }
        };

        let ai_client = AiClient::new("deepseek", ai_api_key, Some(request.model.clone()))
            .map_err(|e| {
                state.status = TaskStatus::Failed;
                state.error = Some(format!("创建AI客户端失败: {}", e));
                state.progress = 1.0;
                worker::Error::RustError(format!("Failed to create AI client: {}", e))
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
                state.progress = 1.0;
                state.current_step = "任务失败".to_string();
                state.updated_at = self.ctx.get_current_timestamp();
                self.save_state(&state).await?;

                console_error!("❌ [CONTINUE] Task failed: {}", e);
                Err(e)
            }
        }
    }

    /// 基于工具结果继续对话
    pub async fn continue_with_tool_results(&self) -> Result<()> {
        console_log!("[CONTINUE] Continuing conversation with tool results");

        // 1. 更新状态
        let mut state = self.load_state().await?;
        state.progress = 0.8;
        state.current_step = "基于工具结果继续对话".to_string();
        state.updated_at = self.ctx.get_current_timestamp();
        self.save_state(&state).await?;

        // 2. 检查并处理待处理的工具结果
        console_log!("[CONTINUE] Checking for pending tool results to process");
        let pending_tool_calls = TaskPersistence::load_pending_tool_calls(self.ctx.storage, self.ctx.task_name).await?;
        
        // 如果没有待处理的工具调用，直接跳过
        let pending_calls = match pending_tool_calls {
            Some(calls) if !calls.is_empty() => calls,
            _ => {
                console_log!("[CONTINUE] No pending tool calls found");
                self.ctx.storage.delete(&format!("pending_tool_calls_{}", self.ctx.task_name)).await?;
                // 3. 获取原始请求和对话历史
                let request = match TaskPersistence::get_request(self.ctx.storage, self.ctx.task_name).await? {
                    Some(req) => req,
                    None => {
                        let mut state = self.load_state().await?;
                        state.status = TaskStatus::Failed;
                        state.error = Some("请求数据不存在".to_string());
                        state.progress = 1.0;
                        self.save_state(&state).await?;
                        return Err(worker::Error::RustError("Request data not found".to_string()));
                    }
                };
                return self.continue_without_tool_results(request, state).await;
            }
        };
        console_log!("[CONTINUE] Found {} pending tool calls, checking for results", pending_calls.len());
        
        // 检查工具结果
        let tool_result_key = get_tool_result_key(self.ctx.task_name);
        let tool_results_data: Option<String> = match self.ctx.storage.get::<String>(&tool_result_key).await {
            Ok(data) => Some(data),
            Err(_) => {
                console_log!("[CONTINUE] No tool results found, continuing without them");
                None
            }
        };

        // 处理工具结果
        if let Some(ref tool_results_data) = tool_results_data {
            console_log!("[CONTINUE] Processing tool results data");
            self.process_tool_results(tool_results_data, &pending_calls).await?;
        }

        // 清除待处理的工具调用
        self.ctx.storage.delete(&format!("pending_tool_calls_{}", self.ctx.task_name)).await?;
        console_log!("[CONTINUE] Cleared pending tool calls");

        // 3. 获取原始请求和对话历史
        let request = match TaskPersistence::get_request(self.ctx.storage, self.ctx.task_name).await? {
            Some(req) => req,
            None => {
                state.status = TaskStatus::Failed;
                state.error = Some("请求数据不存在".to_string());
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(worker::Error::RustError("Request data not found".to_string()));
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
                    return Err(worker::Error::RustError("AI key not configured".to_string()));
                }
            }
        };

        let ai_client = AiClient::new("deepseek", ai_api_key, Some(request.model.clone()))
            .map_err(|e| {
                state.status = TaskStatus::Failed;
                state.error = Some(format!("创建AI客户端失败: {}", e));
                state.progress = 1.0;
                worker::Error::RustError(format!("Failed to create AI client: {}", e))
            })?;

        // 4. 准备消息（包含对话历史和工具结果）
        let mut messages = self.ai_caller.convert_to_ai_messages(&request);

        // 添加对话历史（包括工具调用和工具结果）
        if let Ok(Some(history)) = self.load_conversation_history().await {
            messages.extend(history);
        }

        let tools = self.ai_caller.convert_to_ai_tools(&request.tools);

        // 5. 调用AI服务进行下一轮对话
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
                state.progress = 1.0;
                state.current_step = "任务失败".to_string();
                state.updated_at = self.ctx.get_current_timestamp();
                self.save_state(&state).await?;

                console_error!("❌ [CONTINUE] Task failed: {}", e);
                Err(e)
            }
        }
    }

    /// 简化版任务执行
    pub async fn execute_task_simplified(&self) -> Result<()> {
        console_log!("=== SIMPLIFIED TASK EXECUTION START ===");

        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        // 1. 快速更新状态
        let now = self.ctx.get_current_timestamp();
        let running_state = serde_json::json!({
            "status": "running",
            "progress": 0.3,
            "current_step": "简化执行中",
            "created_at": now,
            "updated_at": now,
            "error": serde_json::Value::Null
        });

        let storage = self.ctx.storage;
        let state_key = get_state_key(self.ctx.task_name);
        storage.put(&state_key, running_state).await?;

        // 2. 获取请求数据
        let request_key = get_request_key(self.ctx.task_name);
        let request = storage.get::<CompatibleRequest>(&request_key).await?;

        // 3. 简单的AI调用
        let ai_api_key = match self.ctx.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.ctx.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    return Err(worker::Error::RustError("No API key configured".to_string()));
                }
            }
        };

        let ai_client = AiClient::new("deepseek", ai_api_key, Some(request.model.clone()))?;

        let messages = vec![
            AiMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
                tool_call_id: None,
                tool_calls: None,
            }
        ];

        match self.ai_caller.call_ai_simplified(ai_client, messages, self.ctx.task_name).await {
            Ok(ai_response) => {
                let response = CompatibleResponse {
                    success: true,
                    response: Some(ai_response.content),
                    tool_calls: None,
                    metadata: None,
                    task_id: Some(self.ctx.task_name.to_string()),
                    status: Some("completed".to_string()),
                    progress: Some(1.0),
                    estimated_time: None,
                    error: None,
                };

                let result_key = get_result_key(self.ctx.task_name);
                let _ = storage.put(&result_key, &response).await;

                let completed_state = serde_json::json!({
                    "status": "completed",
                    "progress": 1.0,
                    "current_step": "任务完成",
                    "created_at": now,
                    "updated_at": self.ctx.get_current_timestamp(),
                    "error": serde_json::Value::Null
                });

                let state_key = get_state_key(self.ctx.task_name);
                let _ = storage.put(&state_key, completed_state).await;

                console_log!("✅ [SIMPLIFIED] Task completed");
                Ok(())
            }
            Err(e) => {
                console_error!("❌ [SIMPLIFIED] Task failed: {}", e);
                Err(e)
            }
        }
    }

    /// 加载状态（带超时保护）
    pub async fn load_state_with_timeout(&self) -> Result<TaskState> {
        #[cfg(target_arch = "wasm32")]
        {
            let state_future = self.load_state();
            let timeout_future = TimeoutFuture::new(5_000);

            match select(Pin::from(Box::pin(state_future)), Pin::from(Box::pin(timeout_future))).await {
                Either::Left((Ok(state), _)) => Ok(state),
                Either::Left((Err(e), _)) => Err(e),
                Either::Right((_, _)) => {
                    Err(worker::Error::RustError("State loading timeout".to_string()))
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.load_state().await
        }
    }
}

/// 实现AITaskDO的Alarm逻辑（已移至AITaskDO内部）
pub async fn handle_alarm(_executor: &AITaskDO) -> Result<worker::Response> {
    // Alarm逻辑现在在AITaskDO.handle_alarm_logic中处理
    worker::Response::ok("Alarm handled by AITaskDO")
}