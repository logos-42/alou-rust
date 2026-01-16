//! AI任务执行模块
//! 
//! 负责任务的执行逻辑、Alarm处理和超时控制

use super::ai_task_state::{TaskState, get_pending_tool_calls_key, get_conversation_history_key, get_tool_result_key};
use super::ai_task_ai::{DefaultAiCaller, AiCaller};
use super::ai_task::AITaskDO;
use crate::agent::ai_client::AiClient;
use crate::compatibility::models::CompatibleRequest;
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

/// 实现AITaskDO的Alarm逻辑（已移至AITaskDO内部）
pub async fn handle_alarm(_executor: &super::ai_task::AITaskDO) -> Result<Response> {
    // Alarm逻辑现在在AITaskDO.handle_alarm_logic中处理
    Response::ok("Alarm handled by AITaskDO")
}