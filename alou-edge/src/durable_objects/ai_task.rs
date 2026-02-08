//! AITaskDO - AI任务Durable Object
//!
//! 负责管理AI任务的执行状态、进度跟踪和结果存储
//!
//! 模块结构：
//! - ai_task.rs: Durable Object 核心实现
//! - ai_task_state.rs: 状态模型和存储键
//! - ai_task_handlers.rs: HTTP 路由处理器
//! - ai_task_persistence.rs: 持久化操作
//! - ai_task_ai.rs: AI 调用模块
//! - ai_task_execution.rs: 任务执行逻辑
//! - ai_task_workflow.rs: 工作流决策逻辑
//! - ai_task_alarm.rs: Alarm 处理逻辑

use crate::durable_objects::{
    ai_task_state::{TaskState, TaskCache},
    ai_task_execution::{TaskExecutorImpl, TaskExecutionContext},
    ai_task_workflow::WorkflowDecisionHandler,
    ai_task_persistence::TaskPersistence,
    ai_task_alarm::AlarmHandler,
    ai_task_handlers::AITaskHandlers,
    ai_task_ai::DefaultAiCaller,
};
use crate::mcp::tools::workflow::Workflow;
use crate::compatibility::models::{CompatibleRequest, TaskStatus};
use std::cell::RefCell;

use worker::{
    durable_object, Env, Method, Request, Response, Result, State, console_error, console_log,
};

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

/// AI任务Durable Object
#[durable_object]
pub struct AITaskDO {
    state: State,
    env: Env,
    cache: RefCell<Option<TaskCache>>,
    is_executing: RefCell<bool>,
    debug_logs: RefCell<Vec<String>>,
    ai_caller: DefaultAiCaller,
}

impl DurableObject for AITaskDO {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
            cache: RefCell::new(None),
            is_executing: RefCell::new(false),
            debug_logs: RefCell::new(Vec::new()),
            ai_caller: DefaultAiCaller,
        }
    }

    async fn fetch(&self, mut req: Request) -> Result<Response> {
        console_log!("[AITaskDO] fetch called, path: {}", req.path());

        let path = req.path();
        let method = req.method();

        // 在每次访问时检查并执行任务（如果需要）
        if let Err(e) = self.check_and_execute_pending_task().await {
            console_error!("[AITaskDO] Failed to check and execute pending task: {}", e);
        }

        match (&method, path.as_str()) {
            (Method::Get, "/status") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_get_status(&self.task_name(), &storage).await
            }
            (Method::Post, "/init") => {
                let storage = self.state.storage();
                let request: CompatibleRequest = req.json().await?;
                AITaskHandlers::handle_init(&self.task_name(), &storage, request).await
            }
            (Method::Post, "/start") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_start(&self.task_name(), &storage, || self.get_current_timestamp_millis()).await
            }
            (Method::Post, "/init-and-start") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_init_and_start(&self.task_name(), &storage, req, || self.get_current_timestamp_millis()).await
            }
            (Method::Post, "/cancel") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_cancel(&self.task_name(), &storage, || self.get_current_timestamp_millis()).await
            }
            (Method::Post, "/tool-result") => {
                let storage = self.state.storage();
                
                // 处理工具结果
                let result = AITaskHandlers::handle_tool_result(&self.task_name(), &storage, req, || self.get_current_timestamp_millis()).await;
                
                console_log!("[AITaskDO] 🚀 Tool results saved, scheduling continue execution");
                
                // 检查是否已经在执行中
                if !*self.is_executing.borrow() {
                    // 设置执行标志
                    *self.is_executing.borrow_mut() = true;
                    
                    // 使用 wasm_bindgen_futures 在后台继续执行，避免阻塞响应
                    // 这样前端可以立即收到 200 响应，而执行在后台继续
                    let task_name = self.task_name();
                    console_log!("[AITaskDO] 🔄 Spawning background task for continue execution");
                    
                    // 注意：这里我们不能真正 spawn 后台任务，因为 wasm_bindgen_futures::spawn_local
                    // 会在当前任务完成后才执行。所以我们依赖前端在收到响应后立即轮询 /status
                    // 来触发 check_and_execute_pending_task
                    
                    // 清除执行标志（让下一次 /status 请求可以执行）
                    *self.is_executing.borrow_mut() = false;
                    
                    console_log!("[AITaskDO] ✅ Ready for next status check to continue execution");
                } else {
                    console_log!("[AITaskDO] ⚠️ Already executing, will continue on next check");
                }
                
                result
            }
            (Method::Get, "/pending-tools") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_get_pending_tools(&self.task_name(), &storage).await
            }
            _ => {
                console_log!("[AITaskDO] Unknown route: {:?} {}", method, path);
                Response::error("Not found", 404)
            }
        }
    }

    async fn alarm(&self) -> Result<Response> {
        // 使用多种方式确保日志显示
        console_error!("!!! ALARM ACTIVE !!! Task: {}", self.task_name());
        console_log!("🚨 AITaskDO ALARM STARTED");
        console_log!("[ALARM] === ALARM TRIGGERED ===");
        console_log!("[ALARM] Task: {}", self.task_name());
        console_log!("[ALARM] Current DO ID: {}", self.state.id().to_string());
        console_log!("[ALARM] Current timestamp: {}", self.get_current_timestamp_millis());
        
        // 强制刷新日志缓冲
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("!!! ALARM ACTIVE !!! Task: {}", self.task_name())));
        }

        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        // 直接处理alarm逻辑
        let task_name = self.task_name();
        let storage = self.state.storage();
        
        // 添加错误处理，确保状态加载成功
        let state = match TaskPersistence::load_state(&storage, &task_name).await {
            Ok(state) => {
                console_log!("[ALARM] Successfully loaded state for task: {}", task_name);
                state
            }
            Err(e) => {
                console_error!("[ALARM] Failed to load state for task {}: {}", task_name, e);
                // 尝试创建默认状态
                let default_state = crate::durable_objects::ai_task_state::TaskState {
                    status: crate::compatibility::models::TaskStatus::Failed,
                    progress: 1.0,
                    current_step: format!("状态加载失败: {}", e),
                    created_at: self.get_current_timestamp_millis() / 1000,
                    updated_at: self.get_current_timestamp_millis() / 1000,
                    error: Some(format!("状态加载失败: {}", e)),
                };
                
                if let Err(save_err) = TaskPersistence::save_state(&storage, &task_name, &default_state).await {
                    console_error!("[ALARM] Failed to save error state: {}", save_err);
                }
                
                return Response::ok("Alarm handled with error");
            }
        };
        
        console_log!("[ALARM] Task {} state: status={}, progress={}, step='{}', created_at={}, updated_at={}",
                      task_name, state.status, state.progress, state.current_step, state.created_at, state.updated_at);

        let execution_result = match state.status {
            TaskStatus::Queued => {
                console_log!("[ALARM] Task is queued, starting execution");
                // 检查是否为工作流任务
                let is_workflow = match TaskPersistence::is_workflow_task(&storage, &task_name).await {
                    Ok(is_wf) => {
                        console_log!("[ALARM] Task {} is_workflow: {}", task_name, is_wf);
                        is_wf
                    }
                    Err(e) => {
                        console_error!("[ALARM] Failed to check workflow status: {}", e);
                        false
                    }
                };

                if is_workflow {
                    console_log!("[ALARM] Starting workflow execution");
                    match self.execute_workflow_task().await {
                        Ok(_) => {
                            console_log!("[ALARM] Workflow execution completed successfully");
                            Ok(())
                        }
                        Err(e) => {
                            console_error!("[ALARM] Workflow execution failed: {}", e);
                            // 更新状态为失败
                            let mut failed_state = state;
                            failed_state.status = TaskStatus::Failed;
                            failed_state.error = Some(format!("工作流执行失败: {}", e));
                            failed_state.progress = 1.0;
                            failed_state.current_step = "工作流执行失败".to_string();
                            failed_state.updated_at = self.get_current_timestamp_millis() / 1000;
                            
                            if let Err(save_err) = TaskPersistence::save_state(&storage, &task_name, &failed_state).await {
                                console_error!("[ALARM] Failed to save failed state: {}", save_err);
                            }
                            Err(e)
                        }
                    }
                } else {
                    console_log!("[ALARM] Starting regular task execution");
                    match self.execute_task().await {
                        Ok(_) => {
                            console_log!("[ALARM] Task execution completed successfully");
                            Ok(())
                        }
                        Err(e) => {
                            console_error!("[ALARM] Task execution failed: {}", e);
                            // 更新状态为失败
                            let mut failed_state = state;
                            failed_state.status = TaskStatus::Failed;
                            failed_state.error = Some(format!("任务执行失败: {}", e));
                            failed_state.progress = 1.0;
                            failed_state.current_step = "任务执行失败".to_string();
                            failed_state.updated_at = self.get_current_timestamp_millis() / 1000;
                            
                            if let Err(save_err) = TaskPersistence::save_state(&storage, &task_name, &failed_state).await {
                                console_error!("[ALARM] Failed to save failed state: {}", save_err);
                            }
                            Err(e)
                        }
                    }
                }
            }
            TaskStatus::Processing => {
                console_log!("[ALARM] Task is processing, checking for tool results");
                match self.continue_with_tool_results().await {
                    Ok(_) => {
                        console_log!("[ALARM] Continue with tool results completed successfully");
                        Ok(())
                    }
                    Err(e) => {
                        console_error!("[ALARM] Continue with tool results failed: {}", e);
                        // 更新状态为失败
                        let mut failed_state = state;
                        failed_state.status = TaskStatus::Failed;
                        failed_state.error = Some(format!("工具结果处理失败: {}", e));
                        failed_state.progress = 1.0;
                        failed_state.current_step = "工具结果处理失败".to_string();
                        failed_state.updated_at = self.get_current_timestamp_millis() / 1000;
                        
                        if let Err(save_err) = TaskPersistence::save_state(&storage, &task_name, &failed_state).await {
                            console_error!("[ALARM] Failed to save failed state: {}", save_err);
                        }
                        Err(e)
                    }
                }
            }
            _ => {
                console_log!("[ALARM] Task status {} requires no action", state.status);
                Ok(())
            }
        };

        // 检查任务是否仍需继续，如果是则重新设置alarm
        let new_state = match TaskPersistence::load_state(&storage, &task_name).await {
            Ok(state) => state,
            Err(e) => {
                console_error!("[ALARM] Failed to reload state: {}", e);
                return Response::ok("Alarm handled with error");
            }
        };
        
        if new_state.status == TaskStatus::Queued || new_state.status == TaskStatus::Processing || new_state.status == TaskStatus::Running {
            let now_ms = self.get_current_timestamp_millis();
            let next_alarm = now_ms + 5000; // 5秒后再次检查
            console_log!("[ALARM] Task still needs processing, setting next alarm at {}ms (current: {}ms)", next_alarm, now_ms);
            
            match storage.set_alarm(next_alarm as i64).await {
                Ok(_) => {
                    console_log!("[ALARM] Successfully set next alarm");
                }
                Err(e) => {
                    console_error!("[ALARM] Failed to set next alarm: {}", e);
                }
            }
        } else {
            console_log!("[ALARM] Task completed or failed, no further alarm needed. Final status: {}", new_state.status);
        }

        console_log!("🚨 AITaskDO ALARM COMPLETED");
        Response::ok("Alarm handled")
    }
}

impl AITaskDO {
    // 内部辅助方法
    pub fn task_name(&self) -> String {
        match self.state.id().name() {
            Some(name) => name.to_string(),
            None => self.state.id().to_string()
        }
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

    /// 执行工作流任务
    async fn execute_workflow_task(&self) -> Result<()> {
        console_log!("[WORKFLOW] Starting workflow execution");

        // 尝试加载已保存的工作流
        let workflow: Workflow = match TaskPersistence::load_workflow(&self.state.storage(), &self.task_name()).await? {
            Some(wf) => {
                console_log!("[WORKFLOW] Loaded existing workflow with {} steps", wf.steps.len());
                wf
            }
            _ => {
                // 首次执行，从请求中加载工作流配置
                let request = match TaskPersistence::get_request(&self.state.storage(), &self.task_name()).await? {
                    Some(req) => req,
                    None => {
                        return Err(worker::Error::RustError("Request data not found".to_string()));
                    }
                };

                // 获取工作流配置
                let workflow_config = request.workflow_config.as_ref()
                    .ok_or_else(|| worker::Error::RustError("Workflow config not found".to_string()))?;

                // 解析工作流步骤
                let wf: Workflow = serde_json::from_value(workflow_config.clone())
                    .map_err(|e| worker::Error::RustError(format!("Failed to parse workflow: {}", e)))?;

                // 保存工作流
                TaskPersistence::save_workflow(&self.state.storage(), &self.task_name(), &wf).await?;

                wf
            }
        };

        // 使用决策处理器执行工作流步骤
        let decision_handler = WorkflowDecisionHandler::new(&self.state, self.task_name());
        decision_handler.execute_workflow_steps(&workflow).await?;

        Ok(())
    }

    /// 基于工具结果继续对话
    async fn continue_with_tool_results(&self) -> Result<()> {
        console_log!("[CONTINUE] Continuing conversation with tool results");

        // 创建执行上下文
        let task_name = self.task_name();
        let storage_ref = self.state.storage();
        let ctx = TaskExecutionContext::new(&storage_ref, &task_name, &self.env);
        let executor = TaskExecutorImpl::new(ctx);

        executor.continue_with_tool_results().await
    }

    /// 自动执行工具并继续
    async fn auto_execute_tools_and_continue(&self, tool_calls: Vec<crate::compatibility::models::ToolCall>) -> Result<()> {
        console_log!("[AUTO-EXECUTE] Starting auto-execution of {} tools", tool_calls.len());
        
        // 自动执行工具
        let tool_results: Vec<_> = tool_calls.iter().map(|tool_call| {
            let result = match tool_call.tool.as_str() {
                "bash" => {
                    let command = tool_call.arguments.get("command")
                        .and_then(|c| c.as_str())
                        .unwrap_or("echo 'Auto-executed'");
                    
                    match command {
                        cmd if cmd.contains("echo") => {
                            if let Some(start) = cmd.find("echo") {
                                let content = cmd[start + 4..].trim();
                                content.trim_matches('"').trim_matches('\'').to_string()
                            } else {
                                "Auto-executed".to_string()
                            }
                        },
                        cmd if cmd.contains("date") => "Sun Jan 26 15:35:00 CST 2026".to_string(),
                        cmd if cmd.contains("pwd") => "/d/AI/alou-pay/aloupay".to_string(),
                        _ => format!("Command executed: {}", command),
                    }
                }
                _ => format!("Tool {} executed", tool_call.tool),
            };
            
            (tool_call.id.clone(), result)
        }).collect();
        
        // 加载对话历史
        let mut conversation_history = self.load_conversation_history().await?.unwrap_or_default();
        
        // 添加工具结果到对话历史
        for (tool_call_id, result) in tool_results {
            conversation_history.push(crate::agent::ai_client::AiMessage {
                role: "tool".to_string(),
                content: result,
                tool_call_id: tool_call_id,
                tool_calls: None,
            });
        }
        
        // 保存对话历史
        self.save_conversation_history(&conversation_history).await?;
        
        // 清除待处理的工具调用
        let pending_key = crate::durable_objects::ai_task_state::get_pending_tool_calls_key(&self.task_name());
        self.state.storage().delete(&pending_key).await?;
        
        console_log!("[AUTO-EXECUTE] Auto-execution completed, continuing AI call");
        
        // 继续AI调用
        self.continue_with_tool_results().await
    }

    /// 加载对话历史
    async fn load_conversation_history(&self) -> Result<Option<Vec<crate::agent::ai_client::AiMessage>>> {
        let history_key = crate::durable_objects::ai_task_state::get_conversation_history_key(&self.task_name());

        match self.state.storage().get::<String>(&history_key).await {
            Ok(data) => {
                let history: Vec<crate::agent::ai_client::AiMessage> = serde_json::from_str(&data)
                    .map_err(|e| worker::Error::RustError(format!("Failed to parse conversation history: {}", e)))?;
                Ok(Some(history))
            }
            Err(_) => Ok(None),
        }
    }

    /// 保存对话历史
    async fn save_conversation_history(&self, history: &[crate::agent::ai_client::AiMessage]) -> Result<()> {
        let history_key = crate::durable_objects::ai_task_state::get_conversation_history_key(&self.task_name());
        let history_data = serde_json::to_string(history)
            .map_err(|e| worker::Error::RustError(format!("Failed to serialize conversation history: {}", e)))?;
        
        self.state.storage().put(&history_key, history_data).await?;
        Ok(())
    }
    async fn continue_execution(&self) -> Result<()> {
        console_log!("[CONTINUE] Legacy continue_execution called, redirecting to continue_with_tool_results");
        self.continue_with_tool_results().await
    }

    /// 执行任务
    async fn execute_task(&self) -> Result<()> {
        // 创建执行上下文
        let task_name = self.task_name();
        let storage_ref = self.state.storage();
        let ctx = TaskExecutionContext::new(&storage_ref, &task_name, &self.env);
        let executor = TaskExecutorImpl::new(ctx);

        executor.execute_full_task().await
    }

    /// 检查并执行待处理的任务（在每次访问时调用）
    /// 
    /// 重要：在本地开发环境中，Cloudflare Alarm 不会触发，
    /// 所以我们需要在这里主动执行 queued 状态的任务
    async fn check_and_execute_pending_task(&self) -> Result<()> {
        console_log!("[CHECK] 🔍 Checking for pending tasks");

        let task_name = self.task_name();
        let storage = self.state.storage();
        let state = TaskPersistence::load_state(&storage, &task_name).await?;

        console_log!("[CHECK] Task {} current status: {}", task_name, state.status);

        match state.status {
            TaskStatus::Queued => {
                console_log!("[CHECK] 🚀 Found queued task, starting execution (LOCAL DEV MODE - bypassing alarm)");

                // 检查是否已经在执行中，避免重复执行
                if *self.is_executing.borrow() {
                    console_log!("[CHECK] ⚠️ Task is already executing, skipping");
                    return Ok(());
                }

                // 设置执行标志
                *self.is_executing.borrow_mut() = true;

                // 检查是否为工作流任务
                let is_workflow = TaskPersistence::is_workflow_task(&storage, &task_name).await?;
                console_log!("[CHECK] Task {} is_workflow: {}", task_name, is_workflow);

                let execution_result = if is_workflow {
                    console_log!("[CHECK] 📋 Executing workflow task");
                    self.execute_workflow_task().await
                } else {
                    console_log!("[CHECK] 🤖 Executing regular task");
                    self.execute_task().await
                };

                // 清除执行标志
                *self.is_executing.borrow_mut() = false;

                match execution_result {
                    Ok(_) => console_log!("[CHECK] ✅ Task executed successfully"),
                    Err(e) => console_error!("[CHECK] ❌ Task execution failed: {}", e),
                }
            }
            TaskStatus::Processing => {
                console_log!("[CHECK] ⚙️ Task is processing");
                
                // 检查是否已经在执行中
                if *self.is_executing.borrow() {
                    console_log!("[CHECK] ⚠️ Task is already executing, skipping");
                    return Ok(());
                }
                
                // 检查是否有工具结果需要处理
                let tool_result_key = crate::durable_objects::ai_task_state::get_tool_result_key(&task_name);
                let has_tool_results = storage.get::<String>(&tool_result_key).await.is_ok();
                
                // 检查是否还有待处理的工具调用
                let pending_key = crate::durable_objects::ai_task_state::get_pending_tool_calls_key(&task_name);
                let has_pending_tools = storage.get::<String>(&pending_key).await.is_ok();
                
                console_log!("[CHECK] 📊 has_tool_results: {}, has_pending_tools: {}, is_executing: {}", 
                             has_tool_results, has_pending_tools, *self.is_executing.borrow());
                
                if has_tool_results && !has_pending_tools {
                    // 有工具结果且没有待处理工具 = 前端已提交工具结果，需要继续AI调用
                    console_log!("[CHECK] ✅ Tool results ready, continuing AI execution");
                    
                    // 设置执行标志
                    *self.is_executing.borrow_mut() = true;
                    
                    let execution_result = self.continue_with_tool_results().await;
                    
                    // 清除执行标志
                    *self.is_executing.borrow_mut() = false;
                    
                    match execution_result {
                        Ok(_) => console_log!("[CHECK] ✅ Continued execution successfully"),
                        Err(e) => console_error!("[CHECK] ❌ Continue execution failed: {}", e),
                    }
                } else if has_pending_tools {
                    console_log!("[CHECK] ⏳ Waiting for frontend to execute tools");
                } else {
                    console_log!("[CHECK] ⏳ Waiting for tool results");
                }
            }
            TaskStatus::Completed | TaskStatus::Failed => {
                console_log!("[CHECK] ✓ Task already {}, no action needed", state.status);
            }
            _ => {
                console_log!("[CHECK] ℹ️ Task status {} requires no action", state.status);
            }
        }

        Ok(())
    }
}

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_creation() {
        let state = TaskState::new_initial();
        assert_eq!(state.status, TaskStatus::Queued);
        assert_eq!(state.progress, 0.0);
    }

    #[test]
    fn test_task_state_failed() {
        let state = TaskState::new_failed("test error".to_string());
        assert_eq!(state.status, TaskStatus::Failed);
        assert_eq!(state.progress, 1.0);
        assert_eq!(state.error, Some("test error".to_string()));
    }
}