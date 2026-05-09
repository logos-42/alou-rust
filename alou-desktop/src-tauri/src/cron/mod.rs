//! Cron 定时任务模块
//!
//! 提供 OpenClaw 风格的 Cron 定时任务系统，支持每天/每周/每小时自动执行任务
//!
//! # Architecture
//!
//! - **types**: 核心类型定义（CronJob, CronJobState, CronJobResult, CronConfig）
//! - **job**: CronJob 实现（should_run, create_isolated_session）
//! - **config**: 配置管理（从 ~/.alou/cron.json 加载/保存）
//! - **scheduler**: 核心调度器（每分钟检查并执行到期的任务）
//! - **commands**: Tauri 命令接口
//!
//! # Cron 表达式格式
//!
//! 格式：`分 时 日 月 星期`
//!
//! 示例：
//! - `0 8 * * *` = 每天早上 8 点
//! - `0 2 * * *` = 每天凌晨 2 点
//! - `*/30 * * * *` = 每 30 分钟
//! - `0 9 * * 1` = 每周一早上 9 点
//!
//! # 使用示例
//!
//! ```rust
//! // 创建调度器
//! let scheduler = CronScheduler::new()?;
//!
//! // 启动调度器
//! scheduler.start().await?;
//!
//! // 添加任务
//! let job = CronJob::new(
//!     "daily_summary".to_string(),
//!     "0 8 * * *".to_string(),
//!     "生成昨日任务总结".to_string(),
//!     true,  // session_isolation
//!     Some("~/.alou/DAILY_SUMMARY.md".to_string()),
//! );
//! scheduler.add_job(job).await?;
//! ```
//!
//! # Session 隔离
//!
//! 每个 Cron 任务可以使用独立的 session_id，确保任务之间的上下文不会相互干扰。
//! 启用 `session_isolation` 后，每次执行都会生成唯一的 session_id。

pub mod types;
pub mod job;
pub mod config;
pub mod scheduler;
pub mod commands;

// 重新导出主要类型
pub use commands::CronSchedulerState;

/// 初始化 Cron 模块
///
/// 创建 CronSchedulerState 并管理
pub fn initialize_cron() -> Result<std::sync::Arc<tokio::sync::Mutex<CronSchedulerState>>, String> {
    let state = CronSchedulerState::new()?;
    Ok(std::sync::Arc::new(tokio::sync::Mutex::new(state)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_initialization() {
        let result = initialize_cron();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cron_job_creation() {
        let job = CronJob::new(
            "test_job".to_string(),
            "0 8 * * *".to_string(),
            "Test prompt".to_string(),
            true,
            None,
        );

        assert_eq!(job.name, "test_job");
        assert_eq!(job.schedule, "0 8 * * *");
        assert!(job.is_enabled());
    }
}
