//! Cron 类型定义
//!
//! 定义 Cron 系统的核心数据类型

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Cron 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CronJobState {
    /// 等待执行
    Pending,
    /// 正在运行
    Running,
    /// 已完成
    Completed,
    /// 执行失败
    Failed,
}

impl std::fmt::Display for CronJobState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CronJobState::Pending => write!(f, "pending"),
            CronJobState::Running => write!(f, "running"),
            CronJobState::Completed => write!(f, "completed"),
            CronJobState::Failed => write!(f, "failed"),
        }
    }
}

/// Cron 任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobResult {
    /// 任务名称
    pub job_name: String,
    /// 执行状态
    pub status: CronJobState,
    /// 输出内容
    pub output: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间
    pub executed_at: DateTime<Utc>,
    /// 执行时长（毫秒）
    pub execution_time_ms: Option<u64>,
    /// Session ID（用于会话隔离）
    pub session_id: Option<String>,
}

impl CronJobResult {
    /// 创建成功的执行结果
    pub fn success(
        job_name: String,
        output: String,
        executed_at: DateTime<Utc>,
        execution_time_ms: Option<u64>,
        session_id: Option<String>,
    ) -> Self {
        Self {
            job_name,
            status: CronJobState::Completed,
            output: Some(output),
            error: None,
            executed_at,
            execution_time_ms,
            session_id,
        }
    }

    /// 创建失败的执行结果
    pub fn failure(
        job_name: String,
        error: String,
        executed_at: DateTime<Utc>,
        session_id: Option<String>,
    ) -> Self {
        Self {
            job_name,
            status: CronJobState::Failed,
            output: None,
            error: Some(error),
            executed_at,
            execution_time_ms: None,
            session_id,
        }
    }
}

/// Cron 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CronConfig {
    /// Cron 任务列表
    pub jobs: Vec<CronJob>,
    /// 是否启用 Cron 调度器
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 检查间隔（秒），默认 60 秒
    #[serde(default = "default_check_interval")]
    pub check_interval_seconds: u64,
}

fn default_enabled() -> bool {
    false
}

fn default_check_interval() -> u64 {
    60
}

impl CronConfig {
    /// 创建默认配置
    pub fn default_with_template() -> Self {
        Self {
            jobs: vec![
                CronJob {
                    name: "每日总结".to_string(),
                    schedule: "0 8 * * *".to_string(),
                    prompt: "生成昨日任务总结".to_string(),
                    session_isolation: true,
                    result_file: Some("~/.alou/DAILY_SUMMARY.md".to_string()),
                    enabled: true,
                },
                CronJob {
                    name: "任务续行".to_string(),
                    schedule: "0 2 * * *".to_string(),
                    prompt: "继续 TASKS.md 中未完成的任务".to_string(),
                    session_isolation: true,
                    result_file: None,
                    enabled: true,
                },
            ],
            enabled: false,
            check_interval_seconds: 60,
        }
    }

    /// 添加任务
    pub fn add_job(&mut self, job: CronJob) {
        // 检查是否已存在同名任务
        if let Some(existing) = self.jobs.iter_mut().find(|j| j.name == job.name) {
            *existing = job;
        } else {
            self.jobs.push(job);
        }
    }

    /// 删除任务
    pub fn remove_job(&mut self, name: &str) -> Option<CronJob> {
        if let Some(pos) = self.jobs.iter().position(|j| j.name == name) {
            Some(self.jobs.remove(pos))
        } else {
            None
        }
    }

    /// 获取任务
    pub fn get_job(&self, name: &str) -> Option<&CronJob> {
        self.jobs.iter().find(|j| j.name == name)
    }

    /// 获取可变任务引用
    pub fn get_job_mut(&mut self, name: &str) -> Option<&mut CronJob> {
        self.jobs.iter_mut().find(|j| j.name == name)
    }
}

/// Cron 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    /// 任务名称
    pub name: String,
    /// Cron 表达式（格式：分 时 日 月 星期）
    /// 示例："0 8 * * *" = 每天早上 8 点
    pub schedule: String,
    /// 执行提示词（发送给 AI Agent）
    pub prompt: String,
    /// 是否使用独立会话（每个任务使用不同的 session_id）
    #[serde(default = "default_session_isolation")]
    pub session_isolation: bool,
    /// 结果文件路径（可选，保存到指定文件）
    pub result_file: Option<String>,
    /// 是否启用此任务
    #[serde(default = "default_job_enabled")]
    pub enabled: bool,
}

fn default_session_isolation() -> bool {
    true
}

fn default_job_enabled() -> bool {
    true
}

impl CronJob {
    /// 创建新的 Cron 任务
    pub fn new(
        name: String,
        schedule: String,
        prompt: String,
        session_isolation: bool,
        result_file: Option<String>,
    ) -> Self {
        Self {
            name,
            schedule,
            prompt,
            session_isolation,
            result_file,
            enabled: true,
        }
    }

    /// 检查任务是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
