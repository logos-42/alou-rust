//! Cron 调度器
//!
//! 核心调度逻辑，管理所有 Cron jobs 的执行

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use chrono::{DateTime, Utc};
use crate::cron::types::{CronJob, CronJobResult, CronJobState, CronConfig};
use crate::cron::config::CronConfigManager;

/// Cron 调度器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerState {
    /// 未启动
    Stopped,
    /// 正在运行
    Running,
    /// 已暂停
    Paused,
}

/// Cron 调度器
///
/// 负责：
/// - 管理所有 Cron jobs
/// - 每分钟检查是否有任务需要运行
/// - 执行任务（调用 agent 系统）
/// - 记录执行结果
pub struct CronScheduler {
    /// 配置
    config: Arc<Mutex<CronConfig>>,
    /// 调度器状态
    state: Arc<Mutex<SchedulerState>>,
    /// 执行历史
    history: Arc<Mutex<Vec<CronJobResult>>>,
    /// 配置管理器
    config_manager: CronConfigManager,
    /// 运行标记（用于停止调度循环）
    running: Arc<Mutex<bool>>,
    /// 最大历史记录数量
    max_history_size: usize,
}

impl CronScheduler {
    /// 创建新的调度器
    pub fn new() -> Result<Self, String> {
        let config_manager = CronConfigManager::new()?;
        let config = config_manager.load_config()?;

        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(SchedulerState::Stopped)),
            history: Arc::new(Mutex::new(Vec::new())),
            config_manager,
            running: Arc::new(Mutex::new(false)),
            max_history_size: 1000,
        })
    }

    /// 从现有配置创建调度器
    pub fn with_config(config: CronConfig) -> Result<Self, String> {
        let config_manager = CronConfigManager::new()?;

        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(SchedulerState::Stopped)),
            history: Arc::new(Mutex::new(Vec::new())),
            config_manager,
            running: Arc::new(Mutex::new(false)),
            max_history_size: 1000,
        })
    }

    /// 加载配置
    pub async fn load_config(&self) -> Result<(), String> {
        let config = self.config_manager.load_config()?;
        let mut state = self.config.lock().await;
        *state = config;
        Ok(())
    }

    /// 保存配置
    pub async fn save_config(&self) -> Result<(), String> {
        let config = self.config.lock().await;
        self.config_manager.save_config(&config)
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if *state == SchedulerState::Running {
            return Err("调度器已在运行中".to_string());
        }
        *state = SchedulerState::Running;
        drop(state);

        // 设置运行标记
        *self.running.lock().await = true;

        // 克隆 Arc 用于 spawn
        let self_arc = Arc::new(self.clone_for_scheduler());
        let running = self.running.clone();

        // 启动调度循环
        tokio::spawn(async move {
            println!("[CronScheduler] 调度器已启动");

            loop {
                // 每分钟检查
                sleep(Duration::from_secs(60)).await;

                // 检查是否应该停止
                if !*running.lock().await {
                    println!("[CronScheduler] 调度器已停止");
                    break;
                }

                // 检查并运行任务
                if let Err(e) = self_arc.check_and_run_jobs().await {
                    eprintln!("[CronScheduler] 检查任务失败：{}", e);
                }
            }
        });

        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if *state == SchedulerState::Stopped {
            return Err("调度器未运行".to_string());
        }
        *state = SchedulerState::Stopped;
        drop(state);

        // 设置停止标记
        *self.running.lock().await = false;

        println!("[CronScheduler] 调度器已停止");
        Ok(())
    }

    /// 暂停调度器
    pub async fn pause(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if *state == SchedulerState::Stopped {
            return Err("调度器未运行".to_string());
        }
        *state = SchedulerState::Paused;
        drop(state);

        println!("[CronScheduler] 调度器已暂停");
        Ok(())
    }

    /// 恢复调度器
    pub async fn resume(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if *state == SchedulerState::Stopped {
            return Err("调度器未运行".to_string());
        }
        *state = SchedulerState::Running;
        drop(state);

        println!("[CronScheduler] 调度器已恢复");
        Ok(())
    }

    /// 获取调度器状态
    pub async fn get_state(&self) -> SchedulerState {
        *self.state.lock().await
    }

    /// 检查并运行到期的任务
    pub async fn check_and_run_jobs(&self) -> Result<(), String> {
        // 检查调度器状态
        let state = *self.state.lock().await;
        if state != SchedulerState::Running {
            return Ok(()); // 暂停或停止状态下不执行
        }

        let now = Utc::now();
        let config = self.config.lock().await;

        println!("[CronScheduler] 检查任务 (当前时间：{})", now.format("%Y-%m-%d %H:%M:%S"));

        for job in &config.jobs {
            // 检查任务是否启用
            if !job.is_enabled() {
                continue;
            }

            // 检查是否应该运行
            if job.should_run(now) {
                println!("[CronScheduler] 任务 '{}' 到期，开始执行", job.name);

                // 异步执行任务（不阻塞其他任务）
                let self_clone = Arc::new(self.clone_for_scheduler());
                let job_clone = job.clone();
                tokio::spawn(async move {
                    match self_clone.run_job(&job_clone).await {
                        Ok(result) => {
                            println!(
                                "[CronScheduler] 任务 '{}' 执行完成：{:?}",
                                result.job_name, result.status
                            );
                        }
                        Err(e) => {
                            eprintln!("[CronScheduler] 任务 '{}' 执行失败：{}", job_clone.name, e);
                        }
                    }
                });
            }
        }

        Ok(())
    }

    /// 立即运行指定任务
    pub async fn run_job_now(&self, job_name: &str) -> Result<CronJobResult, String> {
        let job = {
            let config = self.config.lock().await;
            config
                .jobs
                .iter()
                .find(|j| j.name == job_name)
                .ok_or_else(|| format!("未找到任务：{}", job_name))?
                .clone()
        };

        self.run_job(&job).await
    }

    /// 执行单个任务
    ///
    /// # Arguments
    /// * `job` - 要执行的任务
    ///
    /// # Returns
    /// * `Result<CronJobResult, String>` - 执行结果
    pub async fn run_job(&self, job: &CronJob) -> Result<CronJobResult, String> {
        let start_time = Utc::now();
        println!("[CronScheduler] 开始执行任务：{}", job.name);

        // 创建会话 ID（如果启用会话隔离）
        let session_id = if job.session_isolation {
            Some(job.create_isolated_session_uuid())
        } else {
            None
        };

        println!("[CronScheduler] 使用 Session ID: {:?}", session_id);

        // TODO: 调用 agent 系统执行任务
        // 这里需要集成到现有的 agent 系统
        let result = self.execute_agent_task(&job.prompt, session_id.clone()).await;

        let end_time = Utc::now();
        let execution_time_ms = (end_time - start_time).num_milliseconds() as u64;

        let job_result = match result {
            Ok(output) => {
                // 保存结果到文件（如果指定了）
                if let Some(ref result_file) = job.result_file {
                    if let Err(e) = self.save_result_to_file(result_file, &output).await {
                        eprintln!("[CronScheduler] 保存结果文件失败：{}", e);
                    }
                }

                CronJobResult::success(
                    job.name.clone(),
                    output,
                    start_time,
                    Some(execution_time_ms),
                    session_id,
                )
            }
            Err(error) => CronJobResult::failure(
                job.name.clone(),
                error,
                start_time,
                session_id,
            ),
        };

        // 添加到历史记录
        self.add_to_history(job_result.clone()).await;

        Ok(job_result)
    }

    /// 执行 Agent 任务（调用 agent 系统）
    ///
    /// # Arguments
    /// * `prompt` - 提示词
    /// * `session_id` - 会话 ID（用于会话隔离）
    ///
    /// # Returns
    /// * `Result<String, String>` - 执行结果
    async fn execute_agent_task(&self, prompt: &str, session_id: Option<String>) -> Result<String, String> {
        println!("[CronScheduler] 调用 Agent 系统执行任务");
        println!("[CronScheduler] Prompt: {}", prompt);

        // TODO: 这里需要集成到现有的 agent 系统
        // 目前返回一个模拟结果
        //
        // 实际实现应该：
        // 1. 使用 session_id 创建独立的 agent session
        // 2. 调用 agent 的 execute_task 方法
        // 3. 等待任务完成并返回结果

        // 模拟执行（临时实现）
        tokio::time::sleep(Duration::from_secs(1)).await;

        let response = format!(
            "[Cron Job 执行结果]\n任务：Cron 定时任务\n提示词：{}\n执行时间：{}\n状态：成功",
            prompt,
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        Ok(response)
    }

    /// 保存结果到文件
    async fn save_result_to_file(&self, file_path: &str, content: &str) -> Result<(), String> {
        // 展开 ~ 为 home 目录
        let expanded_path = if file_path.starts_with("~/") {
            let home_dir = dirs::home_dir()
                .ok_or_else(|| "无法获取 home 目录".to_string())?;
            home_dir.join(&file_path[2..])
        } else {
            PathBuf::from(file_path)
        };

        // 确保目录存在
        if let Some(parent) = expanded_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败：{}", e))?;
            }
        }

        // 追加内容到文件
        let mut file_content = String::new();
        if expanded_path.exists() {
            file_content = std::fs::read_to_string(&expanded_path)
                .unwrap_or_default();
        }

        file_content.push_str(&format!(
            "\n\n---\n执行时间：{}\n---\n{}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S"),
            content
        ));

        std::fs::write(&expanded_path, &file_content)
            .map_err(|e| format!("写入文件失败：{}", e))?;

        println!("[CronScheduler] 结果已保存到：{}", expanded_path.display());
        Ok(())
    }

    /// 添加到历史记录
    async fn add_to_history(&self, result: CronJobResult) {
        let mut history = self.history.lock().await;
        history.push(result);

        // 限制历史记录大小
        if history.len() > self.max_history_size {
            let remove_count = history.len() - self.max_history_size;
            history.drain(0..remove_count);
        }
    }

    /// 获取执行历史
    pub async fn get_job_history(&self) -> Vec<CronJobResult> {
        self.history.lock().await.clone()
    }

    /// 获取历史数量
    pub async fn get_history_count(&self) -> usize {
        self.history.lock().await.len()
    }

    /// 清除历史记录
    pub async fn clear_history(&self) {
        let mut history = self.history.lock().await;
        history.clear();
    }

    /// 获取配置
    pub async fn get_config(&self) -> CronConfig {
        self.config.lock().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, config: CronConfig) -> Result<(), String> {
        let mut state = self.config.lock().await;
        *state = config;
        drop(state);

        // 保存到文件
        self.save_config().await
    }

    /// 添加任务
    pub async fn add_job(&self, job: CronJob) -> Result<(), String> {
        let mut config = self.config.lock().await;
        config.add_job(job);
        drop(config);

        self.save_config().await
    }

    /// 删除任务
    pub async fn remove_job(&self, job_name: &str) -> Result<Option<CronJob>, String> {
        let mut config = self.config.lock().await;
        let removed = config.remove_job(job_name);
        drop(config);

        if removed.is_some() {
            self.save_config().await?;
        }

        Ok(removed)
    }

    /// 列出所有任务
    pub async fn list_jobs(&self) -> Vec<CronJob> {
        self.config.lock().await.jobs.clone()
    }

    /// 克隆用于调度器的引用（内部使用）
    fn clone_for_scheduler(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            history: self.history.clone(),
            config_manager: CronConfigManager::new().expect("Failed to create config manager"),
            running: self.running.clone(),
            max_history_size: self.max_history_size,
        }
    }
}

impl Default for CronScheduler {
    fn default() -> Self {
        Self::new().expect("Failed to create CronScheduler")
    }
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = CronScheduler::new();
        assert!(scheduler.is_ok());
    }

    #[tokio::test]
    async fn test_scheduler_state() {
        let scheduler = CronScheduler::new().unwrap();

        // 初始状态应该是 Stopped
        assert_eq!(scheduler.get_state().await, SchedulerState::Stopped);

        // 启动
        scheduler.start().await.unwrap();
        assert_eq!(scheduler.get_state().await, SchedulerState::Running);

        // 暂停
        scheduler.pause().await.unwrap();
        assert_eq!(scheduler.get_state().await, SchedulerState::Paused);

        // 恢复
        scheduler.resume().await.unwrap();
        assert_eq!(scheduler.get_state().await, SchedulerState::Running);

        // 停止
        scheduler.stop().await.unwrap();
        assert_eq!(scheduler.get_state().await, SchedulerState::Stopped);
    }
}
