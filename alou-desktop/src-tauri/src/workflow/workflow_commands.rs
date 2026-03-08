//! 工作流执行器 Tauri 命令
//!
//! 提供前端与工作流执行器的桥接功能

use super::AsyncWorkflowExecutor;

/// 获取执行状态（Tauri命令）
#[tauri::command]
pub async fn get_execution_status(
    execution_id: String,
    executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    let execution = executor.get_execution(&execution_id).await?
        .ok_or_else(|| format!("Execution '{}' not found", execution_id))?;

    Ok(serde_json::json!({
        "execution_id": execution.execution_id,
        "workflow_id": execution.workflow_id,
        "status": execution.status,
        "current_step": execution.current_step,
        "progress": execution.progress,
        "step_results": execution.step_results,
        "started_at": execution.started_at,
        "completed_at": execution.completed_at,
        "total_execution_time_ms": execution.total_execution_time_ms,
        "error": execution.error
    }))
}

/// 暂停执行（Tauri命令）
#[tauri::command]
pub async fn pause_execution(
    execution_id: String,
    executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    executor.pause_execution(&execution_id).await?;
    Ok(serde_json::json!({
        "execution_id": execution_id,
        "status": "paused"
    }))
}

/// 恢复执行（Tauri命令）
#[tauri::command]
pub async fn resume_execution(
    execution_id: String,
    executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    executor.resume_execution(&execution_id).await?;
    Ok(serde_json::json!({
        "execution_id": execution_id,
        "status": "resumed"
    }))
}

/// 取消执行（Tauri命令）
#[tauri::command]
pub async fn cancel_execution(
    execution_id: String,
    executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    executor.cancel_execution(&execution_id).await?;
    Ok(serde_json::json!({
        "execution_id": execution_id,
        "status": "cancelled"
    }))
}

/// 获取执行日志（Tauri命令）
#[tauri::command]
pub async fn get_execution_logs(
    execution_id: String,
    _executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    // TODO: 实现日志获取
    Ok(serde_json::json!({
        "execution_id": execution_id,
        "logs": []
    }))
}

/// 获取性能指标（Tauri命令）
#[tauri::command]
pub async fn get_performance_metrics(
    _executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    // TODO: 实现性能指标收集
    Ok(serde_json::json!({
        "total_executions": 0,
        "active_executions": 0,
        "completed_executions": 0,
        "failed_executions": 0,
        "average_execution_time_ms": 0.0,
        "success_rate": 0.0
    }))
}

/// 获取Ralph Loop执行历史（Tauri命令）
#[tauri::command]
pub async fn get_ralph_loop_history(
    execution_id: String,
    executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    let history = executor.get_execution_history(&execution_id).await?
        .ok_or_else(|| format!("Execution history not found for: {}", execution_id))?;

    Ok(serde_json::json!({
        "execution_id": history.execution_id,
        "workflow_id": history.workflow_id,
        "total_iterations": history.total_iterations,
        "iterations": history.iterations,
        "started_at": history.started_at,
        "completed_at": history.completed_at,
        "final_status": history.final_status,
        "total_cost": history.total_cost,
        "total_execution_time_ms": history.total_execution_time_ms
    }))
}

/// 回滚Ralph Loop执行到指定迭代（Tauri命令）
#[tauri::command]
pub async fn rollback_ralph_loop_execution(
    execution_id: String,
    target_iteration: u32,
    executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    executor.rollback_to_iteration(&execution_id, target_iteration).await?;

    Ok(serde_json::json!({
        "execution_id": execution_id,
        "rolled_back_to_iteration": target_iteration,
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// 清理旧的Ralph Loop历史记录（Tauri命令）
#[tauri::command]
pub async fn cleanup_ralph_loop_histories(
    max_age_days: u32,
    executor: tauri::State<'_, AsyncWorkflowExecutor>,
) -> Result<serde_json::Value, String> {
    executor.cleanup_old_histories(max_age_days).await;

    Ok(serde_json::json!({
        "message": "Old Ralph Loop histories cleaned up",
        "max_age_days": max_age_days,
        "timestamp": chrono::Utc::now().timestamp()
    }))
}