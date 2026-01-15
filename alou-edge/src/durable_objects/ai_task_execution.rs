//! AI任务执行模块
//! 
//! 负责任务的执行逻辑、Alarm处理和超时控制

use super::ai_task_state::TaskState;
use worker::{Request, Response, Result, console_log, console_error};

#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use futures::future::{select, Either};
#[cfg(target_arch = "wasm32")]
use std::pin::Pin;

/// 任务执行trait
pub trait TaskExecutor {
    /// 处理任务初始化和启动（合并操作）
    async fn handle_init_and_start(&self, req: Request) -> Result<Response>;
    
    /// 处理任务开始执行
    async fn handle_start(&self) -> Result<Response>;
    
    /// 执行AI任务
    async fn execute_task(&self) -> Result<()>;
    
    /// 简化版任务执行
    async fn execute_task_simplified(&self) -> Result<()>;
    
    /// 执行任务（带超时保护）
    async fn execute_task_with_timeout(&self) -> Result<()>;
    
    /// 加载状态（带超时保护）
    async fn load_state_with_timeout(&self) -> Result<TaskState>;
}

/// 实现AITaskDO的Alarm逻辑
pub async fn handle_alarm<Executor: TaskExecutor>(executor: &Executor) -> Result<Response> {
    console_error!("!!! ALARM ACTIVE !!!");
    console_log!("🚨 AITaskDO ALARM STARTED");
    
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    
    let alarm_logic = || async {
        // 1. 加载当前状态
        console_log!("[ALARM] Step 1: Loading state");
        let state = match executor.load_state_with_timeout().await {
            Ok(state) => state,
            Err(e) => {
                console_error!("[ALARM-ERROR] Failed to load state: {}", e);
                return Response::error(format!("Failed to load state: {}", e), 500);
            }
        };
        
        // 2. 检查任务状态
        match state.status {
            crate::compatibility::models::TaskStatus::Completed => {
                console_log!("Task is already completed, alarm exiting.");
                return Response::ok("Already completed");
            }
            crate::compatibility::models::TaskStatus::Failed => {
                console_log!("Task is already failed, alarm exiting.");
                return Response::ok("Already failed");
            }
            crate::compatibility::models::TaskStatus::Running => {
                console_log!("Task is already running, alarm exiting.");
                return Response::ok("Busy");
            }
            crate::compatibility::models::TaskStatus::Processing => {
                console_log!("Task is processing, alarm exiting.");
                return Response::ok("Processing");
            }
            crate::compatibility::models::TaskStatus::Queued => {
                console_log!("Task is queued, starting execution via alarm");
            }
        }
        
        // 3. 执行任务
        console_log!("[ALARM] Starting task execution");
        
        #[cfg(target_arch = "wasm32")]
        {
            let task_future = executor.execute_task_simplified();
            let timeout_future = TimeoutFuture::new(30_000);
            
            match select(Pin::from(Box::pin(task_future)), Pin::from(Box::pin(timeout_future))).await {
                Either::Left((Ok(()), _)) => {
                    console_log!("✅ [ALARM] Task completed successfully");
                    Response::ok("Task completed")
                }
                Either::Left((Err(e), _)) => {
                    console_error!("❌ [ALARM] Task failed: {}", e);
                    Response::error(format!("Task execution failed: {}", e), 500)
                }
                Either::Right((_, _)) => {
                    console_error!("⏰ [ALARM] Alarm execution timed out");
                    Response::error("Alarm execution timeout", 500)
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            match executor.execute_task_simplified().await {
                Ok(_) => {
                    console_log!("✅ [ALARM] Task completed successfully");
                    Response::ok("Task completed")
                }
                Err(e) => {
                    console_error!("❌ [ALARM] Task failed: {}", e);
                    Response::error(format!("Task execution failed: {}", e), 500)
                }
            }
        }
    };
    
    // 捕获panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Box::pin(alarm_logic())
    }));
    
    match result {
        Ok(future) => future.await,
        Err(_) => {
            console_error!("[ALARM-PANIC] Task panicked in alarm");
            Response::error("Task execution panicked", 500)
        }
    }
}
