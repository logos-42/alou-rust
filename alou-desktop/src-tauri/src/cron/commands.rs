//! Tauri 命令
//!
//! 提供给前端调用的 Cron 管理命令

use tauri::State;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::cron::types::{CronJob, CronConfig};
use crate::cron::scheduler::{CronScheduler, SchedulerState};

/// Cron 调度器状态（用于 Tauri State）
pub struct CronSchedulerState {
    pub scheduler: Arc<CronScheduler>,
}

impl CronSchedulerState {
    pub fn new() -> Result<Self, String> {
        let scheduler = Arc::new(CronScheduler::new()?);
        Ok(Self { scheduler })
    }
}

impl Default for CronSchedulerState {
    fn default() -> Self {
        Self::new().expect("Failed to create CronSchedulerState")
    }
}

/// 启动 Cron 调度器
#[tauri::command]
pub async fn start_cron_scheduler(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    match scheduler_state.scheduler.start().await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Cron 调度器已启动"
        })),
        Err(e) => Err(format!("启动失败：{}", e)),
    }
}

/// 停止 Cron 调度器
#[tauri::command]
pub async fn stop_cron_scheduler(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    match scheduler_state.scheduler.stop().await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Cron 调度器已停止"
        })),
        Err(e) => Err(format!("停止失败：{}", e)),
    }
}

/// 暂停 Cron 调度器
#[tauri::command]
pub async fn pause_cron_scheduler(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    match scheduler_state.scheduler.pause().await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Cron 调度器已暂停"
        })),
        Err(e) => Err(format!("暂停失败：{}", e)),
    }
}

/// 恢复 Cron 调度器
#[tauri::command]
pub async fn resume_cron_scheduler(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    match scheduler_state.scheduler.resume().await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "Cron 调度器已恢复"
        })),
        Err(e) => Err(format!("恢复失败：{}", e)),
    }
}

/// 获取 Cron 调度器状态
#[tauri::command]
pub async fn get_cron_scheduler_state(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    let scheduler_state_value = match scheduler_state.scheduler.get_state().await {
        SchedulerState::Stopped => "stopped",
        SchedulerState::Running => "running",
        SchedulerState::Paused => "paused",
    };

    Ok(serde_json::json!({
        "state": scheduler_state_value,
        "history_count": scheduler_state.scheduler.get_history_count().await
    }))
}

/// 添加 Cron 任务
#[tauri::command]
pub async fn add_cron_job(
    job: CronJob,
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    scheduler_state.scheduler.add_job(job).await?;

    Ok(serde_json::json!({
        "success": true,
        "message": "任务已添加"
    }))
}

/// 删除 Cron 任务
#[tauri::command]
pub async fn remove_cron_job(
    name: String,
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    match scheduler_state.scheduler.remove_job(&name).await? {
        Some(_) => Ok(serde_json::json!({
            "success": true,
            "message": format!("任务 '{}' 已删除", name)
        })),
        None => Err(format!("未找到任务：{}", name)),
    }
}

/// 列出所有 Cron 任务
#[tauri::command]
pub async fn list_cron_jobs(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    let jobs = scheduler_state.scheduler.list_jobs().await;

    Ok(serde_json::json!({
        "jobs": jobs,
        "count": jobs.len()
    }))
}

/// 立即运行指定 Cron 任务
#[tauri::command]
pub async fn run_cron_job_now(
    name: String,
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    let result = scheduler_state.scheduler.run_job_now(&name).await?;

    Ok(serde_json::json!({
        "success": result.status == crate::cron::types::CronJobState::Completed,
        "result": result
    }))
}

/// 获取 Cron 任务执行历史
#[tauri::command]
pub async fn get_cron_job_history(
    limit: Option<usize>,
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    let mut history = scheduler_state.scheduler.get_job_history().await;

    // 按执行时间倒序排序
    history.sort_by(|a, b| b.executed_at.cmp(&a.executed_at));

    // 限制返回数量
    if let Some(limit) = limit {
        history.truncate(limit);
    }

    Ok(serde_json::json!({
        "history": history,
        "count": history.len()
    }))
}

/// 获取 Cron 配置
#[tauri::command]
pub async fn get_cron_config(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    let config = scheduler_state.scheduler.get_config().await;

    Ok(serde_json::json!({
        "config": config
    }))
}

/// 更新 Cron 配置
#[tauri::command]
pub async fn update_cron_config(
    config: CronConfig,
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    scheduler_state.scheduler.update_config(config).await?;

    Ok(serde_json::json!({
        "success": true,
        "message": "配置已更新"
    }))
}

/// 清除 Cron 执行历史
#[tauri::command]
pub async fn clear_cron_job_history(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    scheduler_state.scheduler.clear_history().await;

    Ok(serde_json::json!({
        "success": true,
        "message": "历史记录已清除"
    }))
}

/// 获取配置文件路径
#[tauri::command]
pub async fn get_cron_config_path() -> Result<serde_json::Value, String> {
    use crate::cron::config::CronConfigManager;

    let path = CronConfigManager::get_config_path()?;
    Ok(serde_json::json!({
        "path": path.to_string_lossy().to_string()
    }))
}

/// 创建默认配置文件
#[tauri::command]
pub async fn create_default_cron_config(
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    let config = CronConfig::default_with_template();
    scheduler_state.scheduler.update_config(config).await?;

    Ok(serde_json::json!({
        "success": true,
        "message": "默认配置已创建",
        "config": scheduler_state.scheduler.get_config().await
    }))
}

/// 启用/禁用任务
#[tauri::command]
pub async fn toggle_cron_job(
    name: String,
    enabled: bool,
    state: State<'_, Arc<Mutex<CronSchedulerState>>>,
) -> Result<serde_json::Value, String> {
    let scheduler_state = state.lock().await;
    let mut config = scheduler_state.scheduler.get_config().await;

    if let Some(job) = config.get_job_mut(&name) {
        job.enabled = enabled;
        scheduler_state.scheduler.update_config(config).await?;

        Ok(serde_json::json!({
            "success": true,
            "message": format!("任务 '{}' 已{}", name, if enabled { "启用" } else { "禁用" })
        }))
    } else {
        Err(format!("未找到任务：{}", name))
    }
}
