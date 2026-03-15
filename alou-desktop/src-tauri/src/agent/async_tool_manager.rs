//! 异步工具调用管理器
//! 
//! 支持异步工具调用，AI 可以在工具执行期间继续做其他事情
//! 
//! 流程：
//! 1. AI 返回 tool_calls
//! 2. 创建异步任务，立即返回 task_id
//! 3. AI 可以继续对话或执行其他任务
//! 4. 后台轮询工具状态
//! 5. 工具完成后通知 AI

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::{Duration, interval};
use serde::{Deserialize, Serialize};
use crate::agent::error::{AgentError, Result};

/// 异步工具任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AsyncToolStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

/// 异步工具任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncToolTask {
    pub task_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub status: AsyncToolStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub progress: f32,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
    pub requires_polling: bool,
    pub polling_interval: Option<u64>,
}

impl AsyncToolTask {
    pub fn new(task_id: String, tool_name: String, arguments: serde_json::Value, requires_polling: bool) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            task_id,
            tool_name,
            arguments,
            status: AsyncToolStatus::Pending,
            result: None,
            error: None,
            progress: 0.0,
            created_at: now,
            updated_at: now,
            completed_at: None,
            requires_polling,
            polling_interval: if requires_polling { Some(5) } else { None },
        }
    }
}

/// 工具执行器 trait
#[async_trait::async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute_tool(&self, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value>;
    async fn poll_tool_status(&self, tool_name: &str, task_id: &str) -> Result<serde_json::Value>;
}

/// 异步工具调用管理器
pub struct AsyncToolManager {
    tasks: Arc<RwLock<HashMap<String, AsyncToolTask>>>,
    tool_executor: Arc<dyn ToolExecutor>,
    polling_tasks: Arc<Mutex<Vec<String>>>,
}

impl AsyncToolManager {
    pub fn new(tool_executor: Arc<dyn ToolExecutor>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            tool_executor,
            polling_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建异步工具任务
    pub async fn create_task(
        &self,
        tool_name: String,
        arguments: serde_json::Value,
        requires_polling: bool,
    ) -> String {
        let task_id = uuid::Uuid::new_v4().to_string();
        let task = AsyncToolTask::new(task_id.clone(), tool_name.clone(), arguments.clone(), requires_polling);

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task);
        }

        log::info!("[AsyncTool] 创建异步任务：{} - {}", task_id, tool_name);

        if requires_polling {
            let mut polling = self.polling_tasks.lock().await;
            polling.push(task_id.clone());
            self.start_polling(task_id.clone());
        } else {
            self.execute_task_immediately(task_id.clone()).await;
        }

        task_id
    }

    /// 立即执行任务（同步工具）
    async fn execute_task_immediately(&self, task_id: String) {
        let (tool_name, arguments) = {
            let tasks = self.tasks.read().await;
            let task = tasks.get(&task_id).unwrap();
            (task.tool_name.clone(), task.arguments.clone())
        };

        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = AsyncToolStatus::Running;
                task.updated_at = chrono::Utc::now().timestamp();
            }
        }

        let result = self.tool_executor.execute_tool(&tool_name, arguments).await;

        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                match result {
                    Ok(data) => {
                        task.status = AsyncToolStatus::Completed;
                        task.result = Some(data);
                        task.progress = 100.0;
                    }
                    Err(e) => {
                        task.status = AsyncToolStatus::Failed;
                        task.error = Some(e.to_string());
                    }
                }
                task.completed_at = Some(chrono::Utc::now().timestamp());
                task.updated_at = chrono::Utc::now().timestamp();
            }
        }

        log::info!("[AsyncTool] 任务完成：{} - {:?}", task_id, result.is_ok());
    }

    /// 启动后台轮询
    fn start_polling(&self, task_id: String) {
        let tasks = self.tasks.clone();
        let tool_executor = self.tool_executor.clone();
        let polling_tasks = self.polling_tasks.clone();

        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(5));
            
            loop {
                interval_timer.tick().await;

                {
                    let polling = polling_tasks.lock().await;
                    if !polling.contains(&task_id) {
                        break;
                    }
                }

                let (tool_name, task_status) = {
                    let tasks = tasks.read().await;
                    if let Some(task) = tasks.get(&task_id) {
                        (task.tool_name.clone(), task.status.clone())
                    } else {
                        break;
                    }
                };

                if matches!(task_status, AsyncToolStatus::Completed | AsyncToolStatus::Failed | AsyncToolStatus::Cancelled) {
                    polling_tasks.lock().await.retain(|id| id != &task_id);
                    break;
                }

                match tool_executor.poll_tool_status(&tool_name, &task_id).await {
                    Ok(status_data) => {
                        let mut tasks = tasks.write().await;
                        if let Some(task) = tasks.get_mut(&task_id) {
                            if let Some(status_str) = status_data.get("status").and_then(|v| v.as_str()) {
                                task.status = match status_str {
                                    "completed" => AsyncToolStatus::Completed,
                                    "failed" => AsyncToolStatus::Failed,
                                    _ => AsyncToolStatus::Running,
                                };
                            }

                            if let Some(progress) = status_data.get("progress").and_then(|v| v.as_f64()) {
                                task.progress = progress as f32;
                            }

                            if matches!(task.status, AsyncToolStatus::Completed) {
                                task.result = Some(status_data);
                                task.completed_at = Some(chrono::Utc::now().timestamp());
                                polling_tasks.lock().await.retain(|id| id != &task_id);
                                log::info!("[AsyncTool] 任务 {} 轮询完成", task_id);
                                break;
                            }

                            task.updated_at = chrono::Utc::now().timestamp();
                        }
                    }
                    Err(e) => {
                        log::error!("[AsyncTool] 轮询任务 {} 失败：{}", task_id, e);
                        let mut tasks = tasks.write().await;
                        if let Some(task) = tasks.get_mut(&task_id) {
                            task.status = AsyncToolStatus::Failed;
                            task.error = Some(e.to_string());
                            task.completed_at = Some(chrono::Utc::now().timestamp());
                            polling_tasks.lock().await.retain(|id| id != &task_id);
                            break;
                        }
                    }
                }
            }
        });
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> Option<AsyncToolTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 获取所有任务
    pub async fn list_tasks(&self) -> Vec<AsyncToolTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if matches!(task.status, AsyncToolStatus::Pending | AsyncToolStatus::Running) {
                task.status = AsyncToolStatus::Cancelled;
                task.updated_at = chrono::Utc::now().timestamp();
                task.completed_at = Some(chrono::Utc::now().timestamp());
                self.polling_tasks.lock().await.retain(|id| id != task_id);
                log::info!("[AsyncTool] 任务已取消：{}", task_id);
                return Ok(());
            }
        }
        Err(AgentError::InvalidInput(format!("任务 {} 不存在或无法取消", task_id)))
    }

    /// 等待任务完成
    pub async fn wait_for_completion(&self, task_id: &str, timeout_secs: u64) -> Result<AsyncToolTask> {
        let start_time = std::time::Instant::now();
        
        loop {
            if start_time.elapsed().as_secs() >= timeout_secs {
                return Err(AgentError::ExternalApiError(format!("等待任务 {} 完成超时", task_id)));
            }

            if let Some(task) = self.get_task_status(task_id).await {
                if matches!(task.status, AsyncToolStatus::Completed | AsyncToolStatus::Failed | AsyncToolStatus::Cancelled) {
                    return Ok(task);
                }
            } else {
                return Err(AgentError::InvalidInput(format!("任务 {} 不存在", task_id)));
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// 获取活跃任务数量
    pub async fn get_active_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.values().filter(|t| matches!(t.status, AsyncToolStatus::Pending | AsyncToolStatus::Running)).count()
    }
}
