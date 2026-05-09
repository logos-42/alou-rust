//! Kappa Loop Tauri 命令接口

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use tauri::State;

use super::{KappaLoop, KappaLoopConfig};

/// Kappa Loop 全局状态类型
pub type KappaLoopState = Arc<Mutex<KappaLoop>>;

/// 启动卡帕斯循环
#[tauri::command]
pub async fn start_kappa_loop(
    kappa_state: State<'_, KappaLoopState>,
) -> Result<serde_json::Value, String> {
    let loop_ref = kappa_state.inner().lock().await;
    match loop_ref.start().await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": "卡帕斯循环已启动"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e
        })),
    }
}

/// 停止卡帕斯循环
#[tauri::command]
pub async fn stop_kappa_loop(
    kappa_state: State<'_, KappaLoopState>,
) -> Result<serde_json::Value, String> {
    let loop_ref = kappa_state.inner().lock().await;
    match loop_ref.stop().await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": "卡帕斯循环已停止"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e
        })),
    }
}

/// 暂停卡帕斯循环
#[tauri::command]
pub async fn pause_kappa_loop(
    kappa_state: State<'_, KappaLoopState>,
) -> Result<serde_json::Value, String> {
    let loop_ref = kappa_state.inner().lock().await;
    match loop_ref.pause().await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": "卡帕斯循环已暂停"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e
        })),
    }
}

/// 恢复卡帕斯循环
#[tauri::command]
pub async fn resume_kappa_loop(
    kappa_state: State<'_, KappaLoopState>,
) -> Result<serde_json::Value, String> {
    let loop_ref = kappa_state.inner().lock().await;
    match loop_ref.resume().await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": "卡帕斯循环已恢复"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "error": e
        })),
    }
}

/// 获取卡帕斯循环状态
#[tauri::command]
pub async fn get_kappa_loop_state(
    kappa_state: State<'_, KappaLoopState>,
) -> Result<serde_json::Value, String> {
    let loop_ref = kappa_state.inner().lock().await;
    let state = loop_ref.get_state().await;

    Ok(json!({
        "is_running": state.is_running,
        "is_paused": state.is_paused,
        "current_iteration": state.current_iteration,
        "total_cycles_completed": state.total_cycles_completed,
        "total_improvements": state.total_improvements,
        "total_regressions": state.total_regressions,
        "total_failures": state.total_failures,
        "health_score": state.health_score,
        "circuit_breaker_open": state.circuit_breaker_open,
        "last_error": state.last_error,
        "experiments_log": state.experiments_log,
    }))
}

/// 手动触发一次自修复
#[tauri::command]
pub async fn trigger_kappa_self_repair(
    kappa_state: State<'_, KappaLoopState>,
) -> Result<serde_json::Value, String> {
    let loop_ref = kappa_state.inner().lock().await;
    loop_ref.trigger_self_repair().await
}

/// 更新卡帕斯循环配置
#[tauri::command]
pub async fn update_kappa_loop_config(
    project_root: Option<String>,
    target_files: Option<Vec<String>>,
    max_iterations: Option<u32>,
    dry_run: Option<bool>,
    strict: Option<bool>,
    auto_push: Option<bool>,
    cycle_interval_secs: Option<u64>,
    enable_web: Option<bool>,
    max_consecutive_failures: Option<u32>,
    health_score_threshold: Option<f32>,
    kappa_state: State<'_, KappaLoopState>,
) -> Result<serde_json::Value, String> {
    let loop_ref = kappa_state.inner().lock().await;

    let mut new_config = KappaLoopConfig::default();

    if let Some(root) = project_root {
        new_config.project_root = std::path::PathBuf::from(root);
    }
    if let Some(files) = target_files {
        new_config.target_files = files;
    }
    if let Some(iterations) = max_iterations {
        new_config.max_iterations = iterations;
    }
    if let Some(dry) = dry_run {
        new_config.dry_run = dry;
    }
    if let Some(s) = strict {
        new_config.strict = s;
    }
    if let Some(push) = auto_push {
        new_config.auto_push = push;
    }
    if let Some(secs) = cycle_interval_secs {
        new_config.cycle_interval_secs = secs;
    }
    if let Some(web) = enable_web {
        new_config.enable_web = web;
    }
    if let Some(max_fail) = max_consecutive_failures {
        new_config.max_consecutive_failures = max_fail;
    }
    if let Some(threshold) = health_score_threshold {
        new_config.health_score_threshold = threshold;
    }

    loop_ref.update_config(new_config).await;

    Ok(json!({
        "success": true,
        "message": "卡帕斯循环配置已更新"
    }))
}
