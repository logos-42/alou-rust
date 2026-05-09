//! Autonomous Loop Commands
//!
//! 自主循环命令 - Tauri 命令接口

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use tauri::State;

/// 启动自主循环
#[tauri::command]
pub async fn start_autonomous_loop(
    loop_state: State<'_, Arc<Mutex<crate::autonomous_loop::AutonomousLoop>>>,
) -> Result<serde_json::Value, String> {
    let loop_ref = loop_state.inner().lock().await;
    match loop_ref.start().await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": "自主循环已启动"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e
        })),
    }
}

/// 停止自主循环
#[tauri::command]
pub async fn stop_autonomous_loop(
    loop_state: State<'_, Arc<Mutex<crate::autonomous_loop::AutonomousLoop>>>,
) -> Result<serde_json::Value, String> {
    let loop_ref = loop_state.inner().lock().await;
    match loop_ref.stop().await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": "自主循环已停止"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e
        })),
    }
}

/// 暂停自主循环
#[tauri::command]
pub async fn pause_autonomous_loop(
    loop_state: State<'_, Arc<Mutex<crate::autonomous_loop::AutonomousLoop>>>,
) -> Result<serde_json::Value, String> {
    let loop_ref = loop_state.inner().lock().await;
    match loop_ref.pause().await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": "自主循环已暂停"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e
        })),
    }
}

/// 恢复自主循环
#[tauri::command]
pub async fn resume_autonomous_loop(
    loop_state: State<'_, Arc<Mutex<crate::autonomous_loop::AutonomousLoop>>>,
) -> Result<serde_json::Value, String> {
    let loop_ref = loop_state.inner().lock().await;
    match loop_ref.resume().await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": "自主循环已恢复"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e
        })),
    }
}

/// 获取自主循环状态
#[tauri::command]
pub async fn get_autonomous_loop_state(
    loop_state: State<'_, Arc<Mutex<crate::autonomous_loop::AutonomousLoop>>>,
) -> Result<serde_json::Value, String> {
    let loop_ref = loop_state.inner().lock().await;
    let state = loop_ref.get_state().await;
    
    Ok(json!({
        "is_running": state.is_running,
        "is_paused": state.is_paused,
        "current_task_id": state.current_task_id,
        "last_heartbeat": state.last_heartbeat,
        "tasks_completed": state.tasks_completed,
        "tasks_failed": state.tasks_failed,
        "total_iterations": state.total_iterations,
        "config": {
            "heartbeat_interval_seconds": state.config.heartbeat_interval_seconds,
            "task_check_interval_seconds": state.config.task_check_interval_seconds,
            "memory_save_interval_seconds": state.config.memory_save_interval_seconds,
            "progress_report_interval_seconds": state.config.progress_report_interval_seconds,
            "auto_restart": state.config.auto_restart,
            "enabled": state.config.enabled,
        }
    }))
}

/// 添加自主任务
#[tauri::command]
pub async fn add_autonomous_task(
    title: String,
    description: String,
    priority: String,
    loop_state: State<'_, Arc<Mutex<crate::autonomous_loop::AutonomousLoop>>>,
) -> Result<serde_json::Value, String> {
    let loop_ref = loop_state.inner().lock().await;
    
    let task_priority = match priority.to_lowercase().as_str() {
        "critical" => crate::tools::task_queue::TaskPriority::Critical,
        "high" => crate::tools::task_queue::TaskPriority::High,
        "low" => crate::tools::task_queue::TaskPriority::Low,
        _ => crate::tools::task_queue::TaskPriority::Medium,
    };
    
    loop_ref.add_task(title, description, task_priority).await;
    
    Ok(json!({
        "success": true,
        "message": "任务已添加"
    }))
}
