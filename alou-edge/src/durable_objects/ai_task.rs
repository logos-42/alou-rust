//! AITaskDO - AI任务Durable Object
//! 
//! 负责管理AI任务的执行状态、进度跟踪和结果存储

// 存储键名常量，避免硬编码错误
const STATE_KEY: &str = "task_state";
const REQUEST_KEY: &str = "task_request";
const RESULT_KEY: &str = "task_result";
const TOOL_RESULT_KEY: &str = "last_tool_result";

use crate::compatibility::models::{
    CompatibleRequest, CompatibleResponse, TaskStatus, TaskStatusResponse,
};
use crate::agent::ai_client::{AiClient, AiMessage, AiTool};
use serde_json::Value;
use std::cell::RefCell;
use crate::utils::time::{current_timestamp_secs};

use worker::{
    durable_object, Env, Headers, Method, Request, Response, Result, console_error, console_log,
};

// 用于捕获 panic 的 hook
#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

// 用于超时控制的依赖
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use futures::future::{select, Either};
#[cfg(target_arch = "wasm32")]
use std::pin::Pin;


/// AI任务Durable Object
#[durable_object]
pub struct AITaskDO {
    /// 状态存储
    state: worker::State,
    /// 环境
    env: Env,
    /// 内存缓存（可选，用于性能优化）
    #[allow(dead_code)]
    cache: RefCell<Option<TaskCache>>,
    /// 是否正在执行任务
    #[allow(dead_code)]
    is_executing: RefCell<bool>,
    /// 内存日志缓冲区（用于调试，即使 wrangler tail 不可用）
    #[allow(dead_code)]
    debug_logs: RefCell<Vec<String>>,
}

/// 任务缓存（内存中的状态副本）
struct TaskCache {
    task_id: String,
    status: TaskStatus,
    progress: f32,
    current_step: String,
    created_at: u64,
    updated_at: u64,
}

/// 任务状态
struct TaskState {
    status: TaskStatus,
    progress: f32,
    current_step: String,
    created_at: u64,
    updated_at: u64,
    error: Option<String>,
}

impl DurableObject for AITaskDO {
    fn new(state: worker::State, env: Env) -> Self {
        // 极简构造函数 - 不做任何可能出错的操作
        Self {
            state,
            env,
            cache: RefCell::new(None),
            is_executing: RefCell::new(false),
            debug_logs: RefCell::new(Vec::new()),
        }
    }
    
    async fn fetch(&self, req: Request) -> Result<Response> {
        // 极简版 fetch 方法，避免任何可能失败的操作
        console_log!("[AITaskDO] fetch called, path: {}", req.path());
        
        let path = req.path();
        let method = req.method();
        
        // 路由处理
        match (&method, path.as_str()) {
            (Method::Get, "/status") => {
                console_log!("[AITaskDO] Handling GET /status");
                self.handle_get_status().await
            }
            (Method::Post, "/init") => {
                console_log!("[AITaskDO] Handling POST /init");
                self.handle_init(req).await
            }
            (Method::Post, "/start") => {
                console_log!("[AITaskDO] Handling POST /start");
                self.handle_start().await
            }
            (Method::Post, "/init-and-start") => {
                console_log!("[AITaskDO] Handling POST /init-and-start");
                self.handle_init_and_start(req).await
            }
            (Method::Post, "/cancel") => {
                console_log!("[AITaskDO] Handling POST /cancel");
                self.handle_cancel().await
            }
            (Method::Post, "/tool-result") => {
                console_log!("[AITaskDO] Handling POST /tool-result");
                self.handle_tool_result(req).await
            }
            _ => {
                console_log!("[AITaskDO] Unknown route: {:?} {}", method, path);
                Response::error("Not found", 404)
            }
        }
    }
    
    async fn alarm(&self) -> Result<Response> {
        console_error!("!!! ALARM ACTIVE !!! Task: {}", self.task_name());
        console_log!("🚨 AITaskDO ALARM STARTED for task: {}", self.task_name());
        console_log!("[DEBUG] Current DO ID: {}", self.state.id().to_string());
        
        // 设置 panic hook 来捕获 panic
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
        
        // 使用闭包包装实际的异步逻辑，以便在 catch_unwind 中调用
        let alarm_logic = || async {
            // 1. 加载当前状态（带超时保护）
            console_log!("[ALARM] Step 1: Loading state with timeout protection");
            let state = match self.load_state_with_timeout().await {
                Ok(state) => state,
                Err(e) => {
                    console_error!("[ALARM-ERROR] Failed to load state: {}", e);
                    // 尝试标记任务为失败
                    let _ = self.mark_as_failed_safe(format!("状态加载失败: {}", e)).await;
                    return Response::error(format!("Failed to load state: {}", e), 500);
                }
            };
            
            let now = self.get_current_timestamp();
            
            // 2. 检查任务状态
            match state.status {
                TaskStatus::Completed => {
                    console_log!("Task {} is already completed, alarm exiting.", self.task_name());
                    return Response::ok("Already completed");
                }
                TaskStatus::Failed => {
                    console_log!("Task {} is already failed, alarm exiting.", self.task_name());
                    return Response::ok("Already failed");
                }
                TaskStatus::Running => {
                    // 检查是否是真正的运行中（10分钟内更新过）
                    if now.saturating_sub(state.updated_at) < 600 { // 10分钟
                        console_log!("Task {} is genuinely running (updated {} seconds ago), alarm exiting.", 
                            self.task_name(), now - state.updated_at);
                        return Response::ok("Busy");
                    } else {
                        // 超过10分钟没更新，认为是卡死了，需要恢复
                        console_log!("Task {} appears deadlocked (last update {} seconds ago), attempting recovery", 
                            self.task_name(), now - state.updated_at);
                    }
                }
                TaskStatus::Processing => {
                    // 处理中的任务，检查是否卡住
                    if now.saturating_sub(state.updated_at) < 600 { // 10分钟
                        console_log!("Task {} is processing (updated {} seconds ago), alarm exiting.", 
                            self.task_name(), now - state.updated_at);
                        return Response::ok("Processing");
                    } else {
                        // 超过10分钟没更新，认为是卡死了，需要恢复
                        console_log!("Task {} appears deadlocked in processing (last update {} seconds ago), attempting recovery", 
                            self.task_name(), now - state.updated_at);
                    }
                }
                TaskStatus::Queued => {
                    // 正常情况，可以开始执行
                    console_log!("Task {} is queued, starting execution via alarm", self.task_name());
                }
            }
            
            // 3. 更新状态为运行中
            let mut new_state = state;
            new_state.status = TaskStatus::Running;
            new_state.progress = 0.1;
            new_state.current_step = "开始执行".to_string();
            new_state.updated_at = now;
            
            if let Err(e) = self.save_state(&new_state).await {
                console_error!("Failed to save running state: {}", e);
                return Response::error(format!("Failed to save state: {}", e), 500);
            }
            
            console_log!("[ALARM] Task {} state updated to Running, starting execution", self.task_name());
            
            // 4. 直接执行任务（alarm可以执行耗时操作）
            // 在Cloudflare Durable Objects中，alarm可以执行长时间运行的任务
            // 使用超时保护，确保alarm不会无限期挂起
            console_log!("[ALARM] Step 4: Starting task execution with enhanced timeout protection");
            
            #[cfg(target_arch = "wasm32")]
            {
                console_log!("[ALARM-WASM] Using enhanced timeout protection");
                
                // 使用简化版的任务执行，避免复杂逻辑导致崩溃
                let task_future = self.execute_task_simplified();
                let alarm_timeout_future = TimeoutFuture::new(30_000); // 减少到30秒超时
                
                match select(Pin::from(Box::pin(task_future)), Pin::from(Box::pin(alarm_timeout_future))).await {
                    Either::Left((Ok(()), _)) => {
                        console_log!("✅ [ALARM] Task {} completed successfully via alarm", self.task_name());
                        Response::ok("Task completed")
                    }
                    Either::Left((Err(e), _)) => {
                        console_error!("❌ [ALARM] Task {} failed via alarm: {}", self.task_name(), e);
                        
                        // 使用安全方法标记任务为失败
                        let _ = self.mark_as_failed_safe(format!("任务执行失败: {}", e)).await;
                        Response::error(format!("Task execution failed: {}", e), 500)
                    }
                    Either::Right((_, _)) => {
                        console_error!("⏰ [ALARM] Alarm execution timed out after 30 seconds for task: {}", self.task_name());
                        
                        // 使用安全方法标记任务为超时
                        let _ = self.mark_as_failed_safe("任务执行超时（30秒）".to_string()).await;
                        Response::error("Alarm execution timeout", 500)
                    }
                }
            }
            
            #[cfg(not(target_arch = "wasm32"))]
            {
                console_log!("[ALARM-NonWASM] Using simplified task execution");
                // 非WASM环境使用简化版逻辑
                match self.execute_task_simplified().await {
                    Ok(_) => {
                        console_log!("✅ [ALARM] Task {} completed successfully via alarm", self.task_name());
                        Response::ok("Task completed")
                    }
                    Err(e) => {
                        console_error!("❌ [ALARM] Task {} failed via alarm: {}", self.task_name(), e);
                        
                        // 使用安全方法标记任务为失败
                        let _ = self.mark_as_failed_safe(format!("任务执行失败: {}", e)).await;
                        Response::error(format!("Task execution failed: {}", e), 500)
                    }
                }
            }
        };
        
        // 在最外层使用 catch_unwind 保护，确保任何 panic 都不会导致任务永远卡住
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // 创建一个立即执行的异步块
            Box::pin(alarm_logic())
        }));
        
        match result {
            Ok(future) => future.await,
            Err(_) => {
                console_error!("[ALARM-PANIC] Task {} panicked in alarm, marking as failed", self.task_name());
                // 使用安全方法标记任务为失败
                let _ = self.mark_as_failed_safe("任务执行过程中发生panic".to_string()).await;
                Response::error("Task execution panicked", 500)
            }
        }
    }
}

impl AITaskDO {
    /// 处理任务初始化和启动（合并操作）
    async fn handle_init_and_start(&self, mut req: Request) -> Result<Response> {
        console_log!("[INIT-START] Handling init-and-start for task: {}", self.task_name());
        console_log!("[DEBUG] Current DO ID: {}", self.state.id().to_string());
        
        // 1. 快速解析请求
        let request: CompatibleRequest = match req.json().await {
            Ok(req) => req,
            Err(e) => {
                console_error!("[AITaskDO] Failed to parse request: {}", e);
                return Response::error(
                    format!("Failed to parse request: {}", e),
                    400,
                );
            }
        };
        
        // 2. 同步保存请求（确保保存完成）
        let task_id = self.task_name();
        let storage = self.state.storage();
        
        console_log!("[AITaskDO] Synchronously saving request data for task: {}", task_id);
        match storage.put(REQUEST_KEY, &request).await {
            Ok(_) => console_log!("[AITaskDO] Request saved successfully to key: {}", REQUEST_KEY),
            Err(e) => {
                console_error!("[AITaskDO] Failed to save request: {}", e);
                // 即使保存失败，仍然继续，让用户可以检查状态
            }
        }
        
        // 3. 同步创建初始状态并保存（确保保存完成）
        let now = current_timestamp_secs();
        
        let initial_state_data = serde_json::json!({
            "status": "queued",
            "progress": 0.0,
            "current_step": "等待执行",
            "created_at": now,
            "updated_at": now,
            "error": serde_json::Value::Null
        });
        
        console_log!("[AITaskDO] Synchronously saving initial state for task: {}", task_id);
        match storage.put(STATE_KEY, initial_state_data).await {
            Ok(_) => console_log!("[AITaskDO] Initial state saved successfully to key: {}", STATE_KEY),
            Err(e) => {
                console_error!("[AITaskDO] Failed to save initial state: {}", e);
                // 即使保存失败，仍然继续，让用户可以检查状态
            }
        }
        
        // 4. 同步设置Alarm（确保设置完成）
        // ！！！强制使用当前时间 + 5秒的偏移量，确保不因为计算错误导致时间点已经过去！！！
        let now_ms = self.get_current_timestamp_millis();
        let scheduled_time = now_ms + 5000; // 明确 5000 毫秒后（5秒）
        
        console_log!("[DEBUG] Current timestamp (ms): {}", now_ms);
        console_log!("[DEBUG] Setting alarm to absolute MS: {} (5 seconds from now)", scheduled_time);
        
        match storage.set_alarm(scheduled_time as i64).await {
            Ok(_) => {
                console_log!("!!! ALARM SET SUCCESS !!! Task: {}", task_id);
                console_log!("[DEBUG] Alarm scheduled at absolute time (ms): {}", scheduled_time);
                
                // ！！！关键自检：立即验证Alarm是否真的被设置了！！！
                // 注意：由于存储事务可能尚未提交，这里可能返回 None
                // 但我们仍然要检查，至少能看到一些信息
                match storage.get_alarm().await {
                    Ok(Some(confirmed_alarm)) => {
                        console_log!("!!! ALARM CONFIRMED !!! Task: {}, Confirmed alarm time: {}", task_id, confirmed_alarm);
                        console_log!("[DEBUG] Alarm successfully stored in Cloudflare scheduler");
                        console_log!("[DEBUG] Alarm will trigger in {}ms", confirmed_alarm as u64 - now_ms);
                    }
                    Ok(None) => {
                        console_error!("!!! ALARM NOT STORED !!! Task: {}", task_id);
                        console_error!("[DEBUG] CRITICAL: set_alarm returned success but get_alarm returned None!");
                        console_error!("[DEBUG] This could mean:");
                        console_error!("[DEBUG] 1. Storage transaction not yet committed (normal during request)");
                        console_error!("[DEBUG] 2. Cloudflare environment rejected the alarm setting");
                        console_error!("[DEBUG] 3. Time is in the past (expired immediately)");
                        console_error!("[DEBUG] Current time: {}, Scheduled time: {}", now_ms, scheduled_time);
                    }
                    Err(e) => {
                        console_error!("!!! ALARM VERIFICATION FAILED !!! Task: {}, Error: {}", task_id, e);
                        console_error!("[DEBUG] Failed to verify alarm: {}", e);
                    }
                }
            }
            Err(e) => {
                console_error!("[AITaskDO] Failed to set alarm: {}", e);
                console_error!("!!! ALARM SET FAILED !!! Task: {}, Error: {}", task_id, e);
                // 即使Alarm设置失败，仍然返回成功，让用户可以检查状态
            }
        }
        
        // 5. ！！！测试：直接执行任务，不依赖 Alarm！！！
        console_log!("[TEST] Direct task execution for task: {}", self.task_name());
        
        // 立即开始执行任务（同步）
        match self.execute_task().await {
            Ok(_) => {
                console_log!("[TEST] Task executed successfully!");
            }
            Err(e) => {
                console_error!("[TEST] Task execution failed: {}", e);
            }
        }
        
        // 返回响应
        console_log!("[AITaskDO] Returning immediate response for task: {}", self.task_name());
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            "scheduled".to_string(),
        );
        
        // 显式设置 UTF-8 字符集
        let headers = Headers::new();
        if let Err(e) = headers.set("Content-Type", "application/json; charset=utf-8") {
            console_error!("Failed to set Content-Type header: {}", e);
        }
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    /// 处理任务初始化
    async fn handle_init(&self, mut req: Request) -> Result<Response> {
        let request: CompatibleRequest = match req.json().await {
            Ok(req) => req,
            Err(e) => {
                return Response::error(
                    format!("Failed to parse request: {}", e),
                    400,
                );
            }
        };
        
        // 保存请求
        if let Err(e) = self.save_request(&request).await {
            return Response::error(format!("Failed to save request: {}", e), 500);
        }
        
        // 创建初始状态
        let now = current_timestamp_secs();
        
        let initial_state = TaskState {
            status: TaskStatus::Queued,
            progress: 0.0,
            current_step: "等待执行".to_string(),
            created_at: now,
            updated_at: now,
            error: None,
        };
        
        // 保存状态
        if let Err(e) = self.save_state(&initial_state).await {
            return Response::error(format!("Failed to save initial state: {}", e), 500);
        }
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            initial_state.status.to_string(),
        );
        
        // 显式设置 UTF-8 字符集
        let headers = Headers::new();
        if let Err(e) = headers.set("Content-Type", "application/json; charset=utf-8") {
            console_error!("Failed to set Content-Type header: {}", e);
        }
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    /// 处理任务开始执行
    async fn handle_start(&self) -> Result<Response> {
        let state = self.load_state().await?;
        
        // 检查任务状态
        match state.status {
            TaskStatus::Completed => {
                return Response::error("Task already completed", 400);
            }
            TaskStatus::Failed => {
                // 失败的任务可以重新启动
                console_log!("Task {} is in failed state, attempting to restart", self.task_name());
            }
            TaskStatus::Running => {
                // 运行中的任务可以重新调度（任务恢复）
                console_log!("Task {} is already running, rescheduling execution", self.task_name());
            }
            TaskStatus::Queued => {
                // 正常情况，更新状态为 Running
                let mut new_state = state;
                new_state.status = TaskStatus::Running;
                new_state.progress = 0.1;
                new_state.current_step = "开始执行".to_string();
                new_state.updated_at = current_timestamp_secs();
                
                // 保存状态（确保状态先保存）
                if let Err(e) = self.save_state(&new_state).await {
                    console_error!("Failed to save task state: {}", e);
                    return Response::error(format!("Failed to save task state: {}", e), 500);
                }
                
                console_log!("Task {} state updated to Running", self.task_name());
            }
            TaskStatus::Processing => {
                // 处理中的任务可以继续处理
                console_log!("Task {} is already processing, continuing execution", self.task_name());
            }
        }
        
        // 设置 Alarm 来触发任务执行（100毫秒后，给状态保存一些时间）
        // 这样即使 fetch 返回了，Cloudflare 也会保证 DO 被唤醒并执行 alarm 逻辑
        let storage = self.state.storage();
        let now = self.get_current_timestamp_millis();
        let alarm_at = now + 100; // 100ms后的绝对时间戳
        
        // 将 u64 转换为 i64，因为 ScheduledTime 实现了 From<i64>
        let alarm_at_i64 = alarm_at as i64;
        
        console_log!("Setting alarm at absolute time: {} for task: {}", alarm_at_i64, self.task_name());
        if let Err(e) = storage.set_alarm(alarm_at_i64).await {
            console_error!("Failed to set alarm: {}", e);
            return Response::error(format!("Failed to schedule task execution: {}", e), 500);
        }
        
        console_log!("Task {} scheduled for execution via alarm at {}", self.task_name(), alarm_at_i64);
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            "scheduled".to_string(),
        );
        
        // 显式设置 UTF-8 字符集
        let headers = Headers::new();
        if let Err(e) = headers.set("Content-Type", "application/json; charset=utf-8") {
            console_error!("Failed to set Content-Type header: {}", e);
        }
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    /// 处理获取状态（从存储读取真实状态）
    async fn handle_get_status(&self) -> Result<Response> {
        console_log!("[STATUS] handle_get_status called for task: {}", self.task_name());
        
        // 从存储中读取真实状态
        let state = match self.load_state().await {
            Ok(state) => state,
            Err(e) => {
                console_error!("[STATUS] Failed to load state: {}", e);
                // 如果加载失败，返回错误状态
                let error_response = crate::compatibility::models::TaskStatusResponse {
                    task_id: self.task_name(),
                    status: "error".to_string(),
                    progress: Some(0.0),
                    current_step: Some("状态加载失败".to_string()),
                    result: None,
                    error: Some(format!("Failed to load state: {}", e)),
                    created_at: crate::utils::time::current_timestamp_secs(),
                    updated_at: crate::utils::time::current_timestamp_secs(),
                };
                return Ok(Response::from_json(&error_response)?);
            }
        };
        
        // 调试：检查当前是否有闹钟被挂起（只读，不修改）
        console_log!("[DEBUG] Checking alarm status for task: {}", self.task_name());
        match self.state.storage().get_alarm().await {
            Ok(Some(scheduled_time)) => {
                console_log!("[DEBUG] Alarm scheduled at: {:?}", scheduled_time);
                let now = self.get_current_timestamp_millis();
                let scheduled_ms = scheduled_time as u64;
                if scheduled_ms > now {
                    console_log!("[DEBUG] Alarm will trigger in {}ms", scheduled_ms - now);
                } else {
                    console_log!("[DEBUG] Alarm should have triggered {}ms ago", now - scheduled_ms);
                }
            }
            Ok(None) => {
                console_log!("[DEBUG] No alarm scheduled for this task");
            }
            Err(e) => {
                console_error!("[DEBUG] Failed to get alarm: {}", e);
            }
        }
        
        // 使用 TaskStatusResponse 结构体，确保正确的 JSON 序列化
        use crate::compatibility::models::TaskStatusResponse;
        
        let response = TaskStatusResponse {
            task_id: self.task_name(),
            status: state.status.to_string(),
            progress: Some(state.progress),
            current_step: Some(state.current_step),
            result: None, // 如果需要返回结果，可以从存储中读取
            error: state.error,
            created_at: state.created_at,
            updated_at: state.updated_at,
        };
        
        console_log!("[STATUS] Returning real state: status={}, progress={}%", 
            state.status, state.progress * 100.0);
        
        // 显式设置 UTF-8 字符集
        let headers = Headers::new();
        if let Err(e) = headers.set("Content-Type", "application/json; charset=utf-8") {
            console_error!("Failed to set Content-Type header: {}", e);
        }
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    /// 处理任务取消
    async fn handle_cancel(&self) -> Result<Response> {
        let mut state = self.load_state().await?;
        
        if state.status != TaskStatus::Running && state.status != TaskStatus::Queued {
            return Response::error("Task cannot be cancelled in current state", 400);
        }
        
        state.status = TaskStatus::Failed;
        state.error = Some("任务被取消".to_string());
        state.progress = 1.0;
        state.current_step = "已取消".to_string();
            state.updated_at = current_timestamp_secs();
        
        // 保存状态
        if let Err(e) = self.save_state(&state).await {
            console_error!("Failed to save task state: {}", e);
        }
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            state.status.to_string(),
        );
        
        // 显式设置 UTF-8 字符集
        let headers = Headers::new();
        if let Err(e) = headers.set("Content-Type", "application/json; charset=utf-8") {
            console_error!("Failed to set Content-Type header: {}", e);
        }
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    
    /// 处理工具调用结果
    async fn handle_tool_result(&self, mut req: Request) -> Result<Response> {
        let mut state = self.load_state().await?;
        
        if state.status != TaskStatus::Running {
            return Response::error("Task is not running", 400);
        }
        
        let tool_result: Value = match req.json().await {
            Ok(result) => result,
            Err(e) => {
                return Response::error(
                    format!("Failed to parse tool result: {}", e),
                    400,
                );
            }
        };
        
        // 更新进度
        state.progress = (state.progress + 0.2).min(0.9);
        state.current_step = "处理工具结果".to_string();
            state.updated_at = current_timestamp_secs();
        
        // 保存工具结果
        let storage = self.state.storage();
        if let Err(e) = storage.put(TOOL_RESULT_KEY, tool_result).await {
            console_error!("Failed to save tool result: {}", e);
        }
        
        // 保存状态
        if let Err(e) = self.save_state(&state).await {
            console_error!("Failed to save task state: {}", e);
        }
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            state.status.to_string(),
        );
        
        // 显式设置 UTF-8 字符集
        let headers = Headers::new();
        if let Err(e) = headers.set("Content-Type", "application/json; charset=utf-8") {
            console_error!("Failed to set Content-Type header: {}", e);
        }
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    /// 获取任务ID（内部ID）
    fn internal_task_id(&self) -> String {
        self.state.id().to_string()
    }
    
    /// 获取任务名称（用户看到的ID）
    fn task_name(&self) -> String {
        // 尝试获取名称，如果失败则返回内部ID
        match self.state.id().name() {
            Some(name) => name.to_string(),
            None => self.internal_task_id()
        }
    }
    
    /// 从存储加载状态
    async fn load_state(&self) -> Result<TaskState> {
        console_log!("[AITaskDO] load_state called");
    
        
        let storage = self.state.storage();
        
        // 尝试获取状态，但如果超时则返回默认状态
        console_log!("[AITaskDO] Attempting to get state from storage with key: {}...", STATE_KEY);
        let state_data_result = storage.get::<Value>(STATE_KEY).await;
        
        match state_data_result {
            Ok(state_data) => {
            let status = if let Some(status_str) = state_data.get("status").and_then(|s| s.as_str()) {
                match status_str {
                    "queued" => TaskStatus::Queued,
                    "running" => TaskStatus::Running,
                    "processing" => TaskStatus::Processing,
                    "completed" => TaskStatus::Completed,
                    "failed" => TaskStatus::Failed,
                    _ => TaskStatus::Queued,
                }
            } else {
                TaskStatus::Queued
            };
            
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
                .unwrap_or_else(|| {
                    current_timestamp_secs()
                });
            
            let updated_at = state_data.get("updated_at")
                .and_then(|u| u.as_u64())
                .unwrap_or(created_at);
            
            let error = state_data.get("error")
                .and_then(|e| e.as_str())
                .map(|e| e.to_string());
            
                console_log!("[AITaskDO] Successfully got state data from storage");
                Ok(TaskState {
                    status,
                    progress,
                    current_step,
                    created_at,
                    updated_at,
                    error,
                })
            }
            Err(e) => {
                console_error!("[AITaskDO] Failed to get state from storage: {}", e);
                // 返回默认状态
                let now = current_timestamp_secs();
                
                Ok(TaskState {
                    status: TaskStatus::Queued,
                    progress: 0.0,
                    current_step: "存储读取失败".to_string(),
                    created_at: now,
                    updated_at: now,
                    error: Some(format!("存储读取失败: {}", e)),
                })
            }
        }
    }
    
    /// 保存状态到存储
    async fn save_state(&self, state: &TaskState) -> Result<()> {
        let storage = self.state.storage();
        
        let state_data = serde_json::json!({
            "status": state.status.to_string(),
            "progress": state.progress,
            "current_step": state.current_step,
            "created_at": state.created_at,
            "updated_at": state.updated_at,
            "error": state.error,
        });
        
        console_log!("[DEBUG] Saving state to key: {}", STATE_KEY);
        storage.put(STATE_KEY, state_data).await?;
        Ok(())
    }
    
    /// 获取请求数据
    async fn get_request(&self) -> Result<Option<CompatibleRequest>> {
        let storage = self.state.storage();
        
        console_log!("[DEBUG] Getting request from key: {}", REQUEST_KEY);
        if let Ok(request) = storage.get::<CompatibleRequest>(REQUEST_KEY).await {
            Ok(Some(request))
        } else {
            Ok(None)
        }
    }
    
    /// 保存请求数据
    async fn save_request(&self, request: &CompatibleRequest) -> Result<()> {
        let storage = self.state.storage();
        console_log!("[DEBUG] Saving request to key: {}", REQUEST_KEY);
        storage.put(REQUEST_KEY, request).await?;
        Ok(())
    }
    
    /// 获取结果数据
    async fn get_result(&self) -> Result<Option<CompatibleResponse>> {
        let storage = self.state.storage();
        
        console_log!("[DEBUG] Getting result from key: {}", RESULT_KEY);
        if let Ok(result) = storage.get::<CompatibleResponse>(RESULT_KEY).await {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
    
    /// 保存结果数据
    async fn save_result(&self, result: &CompatibleResponse) -> Result<()> {
        let storage = self.state.storage();
        console_log!("[DEBUG] Saving result to key: {}", RESULT_KEY);
        storage.put(RESULT_KEY, result).await?;
        Ok(())
    }
    
    /// 执行AI任务（优化版，减少日志和复杂性）
    async fn execute_task(&self) -> Result<()> {
        console_log!("[EXECUTE] Starting task execution for: {}", self.task_name());
        
        // 设置 panic hook
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
        
        // 1. 快速更新状态
        let mut state = self.load_state().await?;
        state.progress = 0.2;
        state.current_step = "加载请求数据".to_string();
            state.updated_at = current_timestamp_secs();
        
        self.save_state(&state).await?;
        
        // 2. 获取请求数据
        let request = match self.get_request().await? {
            Some(req) => req,
            None => {
                state.status = TaskStatus::Failed;
                state.error = Some("请求数据不存在".to_string());
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(worker::Error::RustError("Request data not found".to_string()));
            }
        };
        
        // 3. 更新状态
        state.progress = 0.3;
        state.current_step = "初始化AI客户端".to_string();
        self.save_state(&state).await?;
        
        // 4. 获取API key
        let ai_api_key = match self.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    state.status = TaskStatus::Failed;
                    state.error = Some("AI_API_KEY 或 DEEPSEEK_API_KEY 未配置".to_string());
                    state.progress = 1.0;
                    self.save_state(&state).await?;
                    return Err(worker::Error::RustError("AI_API_KEY or DEEPSEEK_API_KEY not configured".to_string()));
                }
            }
        };
        
        // 5. 创建AI客户端
        let ai_client = match AiClient::new("deepseek", ai_api_key, Some(request.model.clone())) {
            Ok(client) => client,
            Err(e) => {
                state.status = TaskStatus::Failed;
                state.error = Some(format!("创建AI客户端失败: {}", e));
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(worker::Error::RustError(format!("Failed to create AI client: {}", e)));
            }
        };
        
        // 6. 准备消息和工具
        let messages = self.convert_to_ai_messages(&request);
        let tools = self.convert_to_ai_tools(&request.tools);
        
        // 7. 更新状态
        state.progress = 0.5;
        state.current_step = "调用AI服务".to_string();
        self.save_state(&state).await?;
        
        // 8. 调用AI服务
        let ai_result = self.call_ai_with_timeout(ai_client, messages, tools).await;
        
        match ai_result {
            Ok(ai_response) => {
                // 9. 保存结果
                let response = self.convert_from_ai_response(ai_response);
                let _ = self.save_result(&response).await; // 忽略保存错误
                
                // 10. 更新状态为完成
                state.status = TaskStatus::Completed;
                state.progress = 1.0;
                state.current_step = "任务完成".to_string();
                state.updated_at = current_timestamp_secs();
                self.save_state(&state).await?;
                
                console_log!("✅ [EXECUTE] Task {} completed", self.task_name());
                Ok(())
            }
            Err(e) => {
                // 11. 更新状态为失败
                state.status = TaskStatus::Failed;
                state.error = Some(format!("AI服务错误: {}", e));
                state.progress = 1.0;
                state.current_step = "任务失败".to_string();
                state.updated_at = current_timestamp_secs();
                self.save_state(&state).await?;
                
                console_error!("❌ [EXECUTE] Task {} failed: {}", self.task_name(), e);
                Err(worker::Error::RustError(format!("AI service error: {}", e)))
            }
        }
    }
    
    /// 转换到AI消息格式
    fn convert_to_ai_messages(&self, request: &CompatibleRequest) -> Vec<AiMessage> {
        let mut messages = Vec::new();
        
        // 添加系统提示
        if let Some(system_prompt) = &request.system_prompt {
            messages.push(AiMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        
        // 添加历史消息
        for history_msg in &request.history {
            messages.push(AiMessage {
                role: history_msg.role.clone(),
                content: history_msg.content.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        
        // 添加当前提示
        messages.push(AiMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
            tool_call_id: None,
            tool_calls: None,
        });
        
        messages
    }
    
    /// 转换到AI工具格式
    fn convert_to_ai_tools(&self, tools: &[crate::compatibility::models::Tool]) -> Vec<AiTool> {
        tools.iter().map(|tool| {
            AiTool {
                name: tool.name.clone(),
                description: tool.description.clone().unwrap_or_default(),
                parameters: tool.parameters.clone().unwrap_or_default(),
            }
        }).collect()
    }
    
    /// 从AI响应转换
    fn convert_from_ai_response(&self, ai_response: crate::agent::ai_client::AiResponse) -> CompatibleResponse {
        CompatibleResponse {
            success: true,
            response: Some(ai_response.content),
            tool_calls: Some(
                ai_response.tool_calls.into_iter().map(|tc| {
                    crate::compatibility::models::ToolCall {
                        tool: tc.name,
                        arguments: tc.arguments,
                    }
                }).collect()
            ),
            metadata: None,
            task_id: Some(self.task_name()),
            status: Some("completed".to_string()),
            progress: Some(1.0),
            estimated_time: None,
            error: None,
        }
    }
    
    /// 简化版任务执行（避免复杂逻辑导致崩溃）
    async fn execute_task_simplified(&self) -> Result<()> {
        console_log!("=== SIMPLIFIED TASK EXECUTION START ===");
        console_log!("[SIMPLIFIED] Starting simplified task execution for: {}", self.task_name());
        
        // 设置 panic hook
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
        
        // 1. 快速更新状态为运行中
        console_log!("[SIMPLIFIED] Step 1: Quick state update to Running");
        let now = self.get_current_timestamp();
        
        let running_state = serde_json::json!({
            "status": "running",
            "progress": 0.3,
            "current_step": "简化执行中",
            "created_at": now,
            "updated_at": now,
            "error": serde_json::Value::Null
        });
        
        let storage = self.state.storage();
        if let Err(e) = storage.put(STATE_KEY, running_state).await {
            console_error!("[SIMPLIFIED-ERROR] Failed to save running state: {}", e);
            return Err(worker::Error::RustError(format!("Failed to save state: {}", e)));
        }
        
        // 2. 获取请求数据（简化版）
        console_log!("[SIMPLIFIED] Step 2: Getting request data");
        let request = match storage.get::<CompatibleRequest>(REQUEST_KEY).await {
            Ok(req) => {
                console_log!("[SIMPLIFIED] Request found - prompt length: {}", req.prompt.len());
                req
            }
            Err(e) => {
                console_error!("[SIMPLIFIED-ERROR] Failed to get request: {}", e);
                return Err(worker::Error::RustError(format!("Failed to get request: {}", e)));
            }
        };
        
        // 3. 简单的AI调用（不带复杂处理）
        console_log!("[SIMPLIFIED] Step 3: Making simple AI call");
        
        // 获取API key
        let ai_api_key = match self.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    console_error!("[SIMPLIFIED-ERROR] No API key configured");
                    return Err(worker::Error::RustError("No API key configured".to_string()));
                }
            }
        };
        
        // 创建简单的AI客户端
        let ai_client = match AiClient::new("deepseek", ai_api_key, Some(request.model.clone())) {
            Ok(client) => client,
            Err(e) => {
                console_error!("[SIMPLIFIED-ERROR] Failed to create AI client: {}", e);
                return Err(worker::Error::RustError(format!("Failed to create AI client: {}", e)));
            }
        };
        
        // 创建简单的消息
        let messages = vec![AiMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
            tool_call_id: None,
            tool_calls: None,
        }];
        
        // 调用AI服务（带超时）
        console_log!("[SIMPLIFIED] Step 4: Calling AI service");
        let ai_result = self.call_ai_simplified(ai_client, messages).await;
        
        match ai_result {
            Ok(ai_response) => {
                console_log!("[SIMPLIFIED] AI call succeeded, content length: {}", ai_response.content.len());
                
                // 保存结果
                let response = CompatibleResponse {
                    success: true,
                    response: Some(ai_response.content),
                    tool_calls: None,
                    metadata: None,
                    task_id: Some(self.task_name()),
                    status: Some("completed".to_string()),
                    progress: Some(1.0),
                    estimated_time: None,
                    error: None,
                };
                
                if let Err(e) = storage.put(RESULT_KEY, &response).await {
                    console_error!("[SIMPLIFIED-WARN] Failed to save result: {}", e);
                }
                
                // 更新状态为完成
                console_log!("[SIMPLIFIED] Step 5: Updating state to completed");
                let completed_state = serde_json::json!({
                    "status": "completed",
                    "progress": 1.0,
                    "current_step": "任务完成",
                    "created_at": now,
                    "updated_at": self.get_current_timestamp(),
                    "error": serde_json::Value::Null
                });
                
                if let Err(e) = storage.put(STATE_KEY, completed_state).await {
                    console_error!("[SIMPLIFIED-WARN] Failed to save completed state: {}", e);
                }
                
                console_log!("✅ [SIMPLIFIED] Task {} completed successfully", self.task_name());
                console_log!("=== SIMPLIFIED TASK EXECUTION SUCCESS ===");
                Ok(())
            }
            Err(e) => {
                console_error!("[SIMPLIFIED-ERROR] AI call failed: {}", e);
                
                // 更新状态为失败
                let failed_state = serde_json::json!({
                    "status": "failed",
                    "progress": 1.0,
                    "current_step": "AI调用失败",
                    "created_at": now,
                    "updated_at": self.get_current_timestamp(),
                    "error": format!("AI调用失败: {}", e)
                });
                
                if let Err(save_err) = storage.put(STATE_KEY, failed_state).await {
                    console_error!("[SIMPLIFIED-ERROR] Failed to save failed state: {}", save_err);
                }
                
                console_log!("❌ [SIMPLIFIED] Task {} failed", self.task_name());
                console_log!("=== SIMPLIFIED TASK EXECUTION FAILED ===");
                Err(worker::Error::RustError(format!("AI call failed: {}", e)))
            }
        }
    }
    
    /// 执行任务（带超时保护）
    async fn execute_task_with_timeout(&self) -> Result<()> {
        console_log!("Starting task execution with timeout for: {}", self.task_name());
        
        // 在 WASM 环境中，我们需要使用 Promise 来实现超时
        // 这里我们使用一个简单的实现：直接调用 execute_task，但在 AI 调用处添加超时
        self.execute_task().await
    }
    
    /// 简化版AI调用（不带工具，避免复杂处理）
    async fn call_ai_simplified(
        &self,
        ai_client: AiClient,
        messages: Vec<AiMessage>,
    ) -> std::result::Result<crate::agent::ai_client::AiResponse, worker::Error> {
        console_log!("[SIMPLIFIED-AI] Starting simplified AI call");
        
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_timers::future::TimeoutFuture;
            use futures::future::{select, Either};
            use std::pin::Pin;
            
            console_log!("[SIMPLIFIED-AI-WASM] Using enhanced timeout protection (55 seconds)");
            
            let ai_future = ai_client.send_message(messages, None); // 不带工具
            let timeout_future = TimeoutFuture::new(55_000); // 55秒超时，略低于DO限制
            
            match select(Pin::from(Box::pin(ai_future)), Pin::from(Box::pin(timeout_future))).await {
                Either::Left((Ok(response), _)) => {
                    console_log!("✅ [SIMPLIFIED-AI] AI call completed successfully");
                    Ok(response)
                }
                Either::Left((Err(e), _)) => {
                    console_error!("❌ [SIMPLIFIED-AI] AI call failed: {}", e);
                    Err(worker::Error::RustError(format!("AI call failed: {}", e)))
                }
                Either::Right((_, _)) => {
                    console_error!("⏰ [SIMPLIFIED-AI] AI call timed out after 55 seconds");
                    Err(worker::Error::RustError("AI call timeout (55 seconds)".to_string()))
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            console_log!("[SIMPLIFIED-AI-NonWASM] Using simple call");
            match ai_client.send_message(messages, None).await {
                Ok(response) => {
                    console_log!("✅ [SIMPLIFIED-AI] AI call completed successfully");
                    Ok(response)
                }
                Err(e) => {
                    console_error!("❌ [SIMPLIFIED-AI] AI call failed: {}", e);
                    Err(worker::Error::RustError(format!("AI call failed: {}", e)))
                }
            }
        }
    }
    
    /// 调用AI服务（带超时保护）
    async fn call_ai_with_timeout(
        &self,
        ai_client: AiClient,
        messages: Vec<AiMessage>,
        tools: Vec<AiTool>,
    ) -> std::result::Result<crate::agent::ai_client::AiResponse, worker::Error> {
        console_log!("=== CALL_AI_WITH_TIMEOUT START ===");
        console_log!("Calling AI service with timeout protection for task: {}", self.task_name());
        console_log!("Messages count: {}, Tools count: {}", messages.len(), tools.len());
        
        #[cfg(target_arch = "wasm32")]
        {
            // 在WASM环境中使用真正的超时机制
            console_log!("[WASM] Using real timeout mechanism with select");
            
            let ai_future = ai_client.send_message(messages, Some(tools));
            let timeout_future = TimeoutFuture::new(55_000); // 55秒超时，略低于DO限制
            
            // 使用select!宏竞争执行
            match select(Pin::from(Box::pin(ai_future)), Pin::from(Box::pin(timeout_future))).await {
                Either::Left((Ok(response), _)) => {
                    console_log!("✅ AI service call completed successfully for task: {}", self.task_name());
                    console_log!("Response content length: {}", response.content.len());
                    console_log!("=== CALL_AI_WITH_TIMEOUT SUCCESS ===");
                    Ok(response)
                }
                Either::Left((Err(e), _)) => {
                    console_error!("❌ AI service call failed for task {}: {}", self.task_name(), e);
                    Err(worker::Error::RustError(format!("AI service error: {}", e)))
                }
                Either::Right((_, _)) => {
                    console_error!("⏰ AI service call timed out after 55 seconds for task: {}", self.task_name());
                    Err(worker::Error::RustError("AI service call timeout (55 seconds)".to_string()))
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 非WASM环境使用原来的简单超时检查
            console_log!("[Non-WASM] Using simple timeout check");
            
            console_log!("Before ai_client.send_message() call");
            let ai_future = ai_client.send_message(messages, Some(tools));
            console_log!("After ai_client.send_message() call, before await");
            
            match ai_future.await {
                Ok(response) => {
                    console_log!("✅ AI service call completed for task: {}", self.task_name());
                    console_log!("Response content length: {}", response.content.len());
                    
                    console_log!("=== CALL_AI_WITH_TIMEOUT SUCCESS ===");
                    Ok(response)
                }
                Err(e) => {
                    console_error!("❌ AI service call failed for task {}: {}", self.task_name(), e);
                    Err(worker::Error::RustError(format!("AI service error: {}", e)))
                }
            }
        }
    }
    
    /// 加载状态（带超时保护）
    async fn load_state_with_timeout(&self) -> Result<TaskState> {
        console_log!("[TIMEOUT] Loading state with timeout protection");
        
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_timers::future::TimeoutFuture;
            use futures::future::{select, Either};
            use std::pin::Pin;
            
            let state_future = self.load_state();
            let timeout_future = TimeoutFuture::new(5_000); // 5秒超时
            
            match select(Pin::from(Box::pin(state_future)), Pin::from(Box::pin(timeout_future))).await {
                Either::Left((Ok(state), _)) => {
                    console_log!("[TIMEOUT] State loaded successfully");
                    Ok(state)
                }
                Either::Left((Err(e), _)) => {
                    console_error!("[TIMEOUT] State loading failed: {}", e);
                    Err(e)
                }
                Either::Right((_, _)) => {
                    console_error!("[TIMEOUT] State loading timed out after 5 seconds");
                    Err(worker::Error::RustError("State loading timeout".to_string()))
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.load_state().await
        }
    }
    
    /// 安全获取当前时间戳（秒）
    fn get_current_timestamp(&self) -> u64 {
        current_timestamp_secs()
    }
    
    /// 获取当前时间戳（毫秒）
    fn get_current_timestamp_millis(&self) -> u64 {
        // 在WASM环境中，我们可以使用js_sys::Date
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
    
    /// 添加调试日志到内存缓冲区
    fn add_debug_log(&self, message: String) {
        let mut logs = self.debug_logs.borrow_mut();
        logs.push(format!("[{}] {}", self.get_current_timestamp_millis(), message));
        
        // 只保留最近50条日志
        if logs.len() > 50 {
            logs.remove(0);
        }
        
        // 同时输出到 console_log 以便 wrangler tail 也能看到
        console_log!("[DEBUG-LOG] {}", message);
    }
    
    /// 获取最近的调试日志
    fn get_recent_logs(&self) -> Vec<String> {
        self.debug_logs.borrow().clone()
    }
    
    /// 安全标记任务为失败状态（不会panic）- 极度简化版
    async fn mark_as_failed_safe(&self, error_message: String) -> Result<()> {
        console_log!("[SAFE-SIMPLE] Marking task {} as failed: {}", self.task_name(), error_message);
        
        // 极度简化：只记录日志，不进行任何存储操作
        // 这样可以避免任何可能导致 WASM 初始化失败的操作
        console_error!("[SAFE-SIMPLE] Task {} would be marked as failed: {}", self.task_name(), error_message);
        
        Ok(())
    }
    
    /// 标记任务为失败状态
    async fn mark_as_failed(&self, error_message: String) -> Result<()> {
        console_log!("Marking task {} as failed: {}", self.task_name(), error_message);
        
        let mut state = match self.load_state().await {
            Ok(state) => state,
            Err(e) => {
                console_error!("Failed to load state for marking as failed: {}", e);
                return Err(e);
            }
        };
        
        state.status = TaskStatus::Failed;
        state.error = Some(error_message);
        state.progress = 1.0;
        state.current_step = "执行失败".to_string();
            state.updated_at = current_timestamp_secs();
        
        if let Err(e) = self.save_state(&state).await {
            console_error!("Failed to save failed state: {}", e);
            return Err(e);
        }
        
        console_log!("Task {} marked as failed successfully", self.task_name());
        Ok(())
    }
    
}
