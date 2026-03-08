//! Autonomous Agent Main Loop
//!
//! 自主智能体主循环 - 让 Alou 像小剑一样自动执行工作
//!
//! 功能：
//! - 主循环持续运行
//! - 心跳检查（定期检查任务、日历、提醒）
//! - 自动发现工作
//! - 进度自动汇报
//! - 记忆持久化

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, interval};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::Utc;
use log::{info, warn, error};

use crate::tools::task_queue::{TaskQueueManager, Task, TaskPriority, TaskStatus};
use crate::tools::ToolRegistry;

/// 自主智能体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousLoopConfig {
    pub heartbeat_interval_seconds: u64,      // 心跳间隔（默认30秒）
    pub task_check_interval_seconds: u64,    // 任务检查间隔（默认10秒）
    pub memory_save_interval_seconds: u64,    // 记忆保存间隔（默认60秒）
    pub progress_report_interval_seconds: u64, // 进度汇报间隔（默认300秒）
    pub max_idle_seconds: u64,                // 最大空闲时间（默认300秒）
    pub auto_restart: bool,                  // 是否自动重启
    pub enabled: bool,                       // 是否启用
}

impl Default for AutonomousLoopConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_seconds: 30,
            task_check_interval_seconds: 10,
            memory_save_interval_seconds: 60,
            progress_report_interval_seconds: 300,
            max_idle_seconds: 300,
            auto_restart: true,
            enabled: true,
        }
    }
}

/// 自主智能体状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousLoopState {
    pub is_running: bool,
    pub is_paused: bool,
    pub current_task_id: Option<String>,
    pub last_heartbeat: i64,
    pub last_task_check: i64,
    pub last_memory_save: i64,
    pub last_progress_report: i64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub total_iterations: u64,
    pub errors: Vec<String>,
    pub config: AutonomousLoopConfig,
}

impl Default for AutonomousLoopState {
    fn default() -> Self {
        Self {
            is_running: false,
            is_paused: false,
            current_task_id: None,
            last_heartbeat: Utc::now().timestamp(),
            last_task_check: Utc::now().timestamp(),
            last_memory_save: Utc::now().timestamp(),
            last_progress_report: Utc::now().timestamp(),
            tasks_completed: 0,
            tasks_failed: 0,
            total_iterations: 0,
            errors: Vec::new(),
            config: AutonomousLoopConfig::default(),
        }
    }
}

/// 自主智能体主循环
pub struct AutonomousLoop {
    state: Arc<Mutex<AutonomousLoopState>>,
    task_queue: Arc<Mutex<TaskQueueManager>>,
    tool_registry: Arc<Mutex<ToolRegistry>>,
}

impl AutonomousLoop {
    /// 创建新的自主循环实例
    pub fn new(
        task_queue: Arc<Mutex<TaskQueueManager>>,
        tool_registry: Arc<Mutex<ToolRegistry>>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(AutonomousLoopState::default())),
            task_queue,
            tool_registry,
        }
    }

    /// 启动主循环
    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        
        if state.is_running {
            return Err("自主循环已经在运行".to_string());
        }
        
        state.is_running = true;
        state.is_paused = false;
        state.last_heartbeat = Utc::now().timestamp();
        info!("🚀 Alou 自主循环已启动");
        
        drop(state);
        
        // 运行主循环
        self.run_main_loop().await;
        
        Ok(())
    }

    /// 停止主循环
    pub async fn stop(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        
        if !state.is_running {
            return Err("自主循环未运行".to_string());
        }
        
        state.is_running = false;
        state.is_paused = false;
        info!("🛑 Alou 自主循环已停止");
        
        Ok(())
    }

    /// 暂停主循环
    pub async fn pause(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        
        if !state.is_running {
            return Err("自主循环未运行".to_string());
        }
        
        state.is_paused = true;
        info!("⏸️ Alou 自主循环已暂停");
        
        Ok(())
    }

    /// 恢复主循环
    pub async fn resume(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        
        if !state.is_running {
            return Err("自主循环未运行".to_string());
        }
        
        state.is_paused = false;
        state.last_heartbeat = Utc::now().timestamp();
        info!("▶️ Alou 自主循环已恢复");
        
        Ok(())
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> AutonomousLoopState {
        self.state.lock().await.clone()
    }

    /// 主循环
    async fn run_main_loop(&self) {
        info!("🔄 Alou 主循环开始运行");
        
        let mut heartbeat_interval = interval(Duration::from_secs(30));
        let mut task_check_interval = interval(Duration::from_secs(10));
        let mut memory_save_interval = interval(Duration::from_secs(60));
        let mut progress_report_interval = interval(Duration::from_secs(300));
        
        loop {
            // 检查是否停止
            {
                let state = self.state.lock().await;
                if !state.is_running {
                    break;
                }
            }
            
            tokio::select! {
                // 心跳检查
                _ = heartbeat_interval.tick() => {
                    self.heartbeat_check().await;
                }
                
                // 任务检查
                _ = task_check_interval.tick() => {
                    self.task_check_and_execute().await;
                }
                
                // 记忆保存
                _ = memory_save_interval.tick() => {
                    self.memory_save().await;
                }
                
                // 进度汇报
                _ = progress_report_interval.tick() => {
                    self.progress_report().await;
                }
                
                // 定期休眠
                _ = sleep(Duration::from_secs(5)) => {}
            }
        }
        
        info!("🔚 Alou 主循环已结束");
    }

    /// 心跳检查
    async fn heartbeat_check(&self) {
        let mut state = self.state.lock().await;
        state.last_heartbeat = Utc::now().timestamp();
        state.total_iterations += 1;
        
        // 检查是否空闲太久
        let idle_time = Utc::now().timestamp() - state.last_task_check;
        if idle_time > state.config.max_idle_seconds as i64 {
            warn!("⚠️ Alou 空闲时间过长 ({})", idle_time);
            // 发现工作
            self.discover_and_queue_work().await;
        }
        
        info!("💓 心跳: 迭代#{} 运行中", state.total_iterations);
    }

    /// 任务检查与执行
    async fn task_check_and_execute(&self) {
        let mut state = self.state.lock().await;
        
        if state.is_paused {
            return;
        }
        
        state.last_task_check = Utc::now().timestamp();
        
        drop(state);
        
        // 获取待执行任务
        let task = {
            let mut queue = self.task_queue.lock().await;
            queue.next_task(None).await
        };
        
        if let Some(task) = task {
            self.execute_task(task).await;
        } else {
            // 没有待执行任务，发现新工作
            self.discover_and_queue_work().await;
        }
    }

    /// 执行任务
    async fn execute_task(&self, task: Task) {
        info!("📋 执行任务: {}", task.title);
        
        {
            let mut state = self.state.lock().await;
            state.current_task_id = Some(task.id.clone());
        }
        
        // 更新任务状态为执行中
        {
            let mut queue = self.task_queue.lock().await;
            queue.update_status(&task.id, TaskStatus::InProgress).await;
        }
        
        // 执行任务（这里可以调用 AgentExecutor 或工具）
        let result = self.run_task_workflow(task.clone()).await;
        
        // 更新任务状态
        {
            let mut queue = self.task_queue.lock().await;
            
            // Clone error before moving
            let error_msg = result.error.clone().unwrap_or_default();
            
            if result.success {
                queue.update_status(&task.id, TaskStatus::Completed).await;
                queue.set_result(&task.id, crate::tools::task_queue::TaskResult {
                    success: true,
                    output: result.output,
                    error: None,
                    executed_at: Utc::now().timestamp(),
                    execution_time_ms: result.duration_ms,
                }).await;
                
                let mut state = self.state.lock().await;
                state.tasks_completed += 1;
                info!("✅ 任务完成: {}", task.title);
            } else {
                queue.update_status(&task.id, TaskStatus::Failed).await;
                queue.set_result(&task.id, crate::tools::task_queue::TaskResult {
                    success: false,
                    output: None,
                    error: Some(error_msg.clone()),
                    executed_at: Utc::now().timestamp(),
                    execution_time_ms: result.duration_ms,
                }).await;
                
                let mut state = self.state.lock().await;
                state.tasks_failed += 1;
                state.errors.push(error_msg.clone());
                error!("❌ 任务失败: {} - {}", task.title, error_msg);
            }
        }
        
        {
            let mut state = self.state.lock().await;
            state.current_task_id = None;
        }
    }

    /// 运行任务工作流（真正调用 AI 执行任务）
    async fn run_task_workflow(&self, task: Task) -> TaskExecutionResult {
        let start_time = Utc::now().timestamp();
        
        info!("🔧 执行 AI 任务: {}", task.title);
        
        // 读取本地 API 配置
        let api_config = match crate::agent::config::ApiConfig::load().await {
            Ok(config) => config,
            Err(e) => {
                warn!("[AutonomousLoop] 无法加载 API 配置: {}，跳过 AI 执行", e);
                return TaskExecutionResult {
                    success: false,
                    output: None,
                    error: Some(format!("无法加载 API 配置: {} (请先在 APP 中配置 API Key)", e)),
                    duration_ms: 0,
                };
            }
        };
        
        // 获取激活的 API 配置
        let user_api = match api_config.get_active_api() {
            Some(api) if !api.api_key.is_empty() => api.clone(),
            _ => {
                // 没有配置 API Key，返回友好提示
                warn!("[AutonomousLoop] 未配置 API Key，无法执行 AI 任务");
                let duration_ms = (Utc::now().timestamp() - start_time) as u64;
                return TaskExecutionResult {
                    success: false,
                    output: None,
                    error: Some("未配置 API Key。请在 APP 中点击 API 配置，填入你的 API Key 后再启动自主循环。".to_string()),
                    duration_ms,
                };
            }
        };
        
        // 构建任务消息
        let task_message = format!(
            "请执行以下任务：\n\n标题：{}\n描述：{}\n\n请分析任务并执行，完成后给出执行结果。",
            task.title, task.description
        );
        
        // 创建 AI 客户端
        let ai_client = match crate::agent::ai_client::AiClient::new(&user_api) {
            Ok(client) => std::sync::Arc::new(client),
            Err(e) => {
                error!("[AutonomousLoop] 创建 AI 客户端失败: {}", e);
                let duration_ms = (Utc::now().timestamp() - start_time) as u64;
                return TaskExecutionResult {
                    success: false,
                    output: None,
                    error: Some(format!("创建 AI 客户端失败: {}", e)),
                    duration_ms,
                };
            }
        };
        
        // 创建任务管理器和执行器
        let task_manager = std::sync::Arc::new(crate::agent::task::TaskManager::new());
        let tool_registry = std::sync::Arc::new(crate::tools::ToolRegistry::new());
        let tool_bridge = std::sync::Arc::new(
            crate::bridges::ToolBridge::new_sync(crate::bridges::ToolBridgeConfig::default())
        );
        
        let executor = crate::agent::executor::RalphLoopExecutor::new(
            ai_client,
            task_manager.clone(),
            tool_bridge,
            tool_registry,
        );
        
        // 创建任务并执行
        let task_id = task_manager.create_task(
            "autonomous_loop".to_string(),
            task_message,
        ).await;
        
        let duration_ms = match executor.execute(&task_id).await {
            Ok(result) => {
                let elapsed = (Utc::now().timestamp() - start_time) as u64;
                info!("[AutonomousLoop] AI 任务完成: {} - {}", task.title, result.result);
                return TaskExecutionResult {
                    success: true,
                    output: Some(json!({
                        "message": result.result,
                        "task_id": task.id,
                        "ai_task_id": result.task_id,
                        "iterations": result.iteration_count,
                    })),
                    error: None,
                    duration_ms: elapsed,
                };
            }
            Err(e) => {
                error!("[AutonomousLoop] AI 任务执行失败: {}", e);
                (Utc::now().timestamp() - start_time) as u64
            }
        };
        
        TaskExecutionResult {
            success: false,
            output: None,
            error: Some(format!("AI 任务执行失败: {}", task.title)),
            duration_ms,
        }
    }

    /// 发现并队列工作
    async fn discover_and_queue_work(&self) {
        info!("🔍 发现新工作...");
        
        let mut queue = self.task_queue.lock().await;
        
        // 检查是否有待处理的工作
        // 这里可以集成各种发现机制：
        // - 检查日历
        // - 检查提醒
        // - 检查邮箱
        // - 检查项目状态
        
        // 示例：添加一个自动发现的任务
        let auto_task = Task::new(
            "自主检查系统状态".to_string(),
            "Alou 自动检查任务队列和系统状态".to_string(),
            TaskPriority::Low,
            Some("autonomous_loop".to_string()),
        );
        
        queue.add_task(auto_task).await;
        
        info!("📝 已添加自动发现的任务");
    }

    /// 记忆保存
    async fn memory_save(&self) {
        let mut state = self.state.lock().await;
        state.last_memory_save = Utc::now().timestamp();
        
        info!("💾 记忆已保存 (完成: {}, 失败: {})", 
              state.tasks_completed, state.tasks_failed);
    }

    /// 进度汇报
    async fn progress_report(&self) {
        let mut state = self.state.lock().await;
        state.last_progress_report = Utc::now().timestamp();
        
        let status = if state.is_paused { "已暂停" } else { "运行中" };
        
        let report = json!({
            "status": status,
            "iterations": state.total_iterations,
            "tasks_completed": state.tasks_completed,
            "tasks_failed": state.tasks_failed,
            "current_task": state.current_task_id,
            "last_heartbeat": state.last_heartbeat,
            "uptime_seconds": Utc::now().timestamp() - state.last_heartbeat,
        });
        
        info!("📊 进度汇报: {}", report);
        
        // 这里可以发送到飞书、邮件等
        // send_progress_to_feishu(report).await;
    }

    /// 添加新任务
    pub async fn add_task(&self, title: String, description: String, priority: TaskPriority) {
        let task = Task::new(
            title.clone(),
            description,
            priority,
            Some("autonomous_loop".to_string()),
        );
        
        let mut queue = self.task_queue.lock().await;
        queue.add_task(task).await;
        
        info!("📥 新任务已添加: {}", title);
    }
}

/// 任务执行结果
struct TaskExecutionResult {
    success: bool,
    output: Option<serde_json::Value>,
    error: Option<String>,
    duration_ms: u64,
}
