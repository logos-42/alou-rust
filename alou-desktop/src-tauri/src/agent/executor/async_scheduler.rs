//! 异步任务调度器
//!
//! 管理长时间运行的异步任务，如视频生成、模型训练等
//! 提供自动轮询、状态跟踪和事件通知功能

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use log::{debug, error, info, warn};
use crate::agent::providers::ProviderRegistry;

/// 异步任务状态
#[derive(Debug, Clone)]
pub enum AsyncTaskStatus {
    /// 处理中
    Processing,
    /// 已完成，包含结果
    Completed(Value),
    /// 失败，包含错误信息
    Failed(String),
    /// 超时
    Timeout,
}

impl AsyncTaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AsyncTaskStatus::Processing => "processing",
            AsyncTaskStatus::Completed(_) => "completed",
            AsyncTaskStatus::Failed(_) => "failed",
            AsyncTaskStatus::Timeout => "timeout",
        }
    }
}

/// 异步任务句柄
#[derive(Debug, Clone)]
pub struct AsyncTaskHandle {
    pub task_id: String,
    pub tool_name: String,
    pub provider: String,
    pub created_at: Instant,
    pub last_poll_at: Instant,
    pub poll_count: u32,
    pub status: AsyncTaskStatus,
}

impl AsyncTaskHandle {
    pub fn elapsed_secs(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }
}

/// 异步任务调度器
pub struct AsyncTaskScheduler {
    /// 存储正在运行的异步任务
    pending_tasks: Arc<Mutex<HashMap<String, AsyncTaskHandle>>>,
    /// 轮询间隔（秒）
    poll_interval: Duration,
    /// 最大轮询时间（秒）
    max_poll_duration: Duration,
    /// Tauri 应用句柄，用于发送事件
    app_handle: AppHandle,
    /// Provider 注册表，用于查询任务状态
    provider_registry: Arc<ProviderRegistry>,
}

impl AsyncTaskScheduler {
    /// 创建新的调度器实例
    pub fn new(app_handle: AppHandle, provider_registry: Arc<ProviderRegistry>) -> Self {
        Self {
            pending_tasks: Arc::new(Mutex::new(HashMap::new())),
            poll_interval: Duration::from_secs(5),
            max_poll_duration: Duration::from_secs(300), // 5分钟
            app_handle,
            provider_registry,
        }
    }

    /// 注册并开始轮询一个新的异步任务
    pub async fn start_polling(
        &self,
        task_id: String,
        tool_name: String,
        provider: String,
        initial_payload: Value,
    ) {
        info!("[AsyncScheduler] 开始轮询任务: {}, 工具: {}", task_id, tool_name);

        // 创建任务句柄
        let handle = AsyncTaskHandle {
            task_id: task_id.clone(),
            tool_name: tool_name.clone(),
            provider: provider.clone(),
            created_at: Instant::now(),
            last_poll_at: Instant::now(),
            poll_count: 0,
            status: AsyncTaskStatus::Processing,
        };

        // 存储到 pending_tasks
        {
            let mut tasks = self.pending_tasks.lock().await;
            tasks.insert(task_id.clone(), handle.clone());
        }

        // 发送任务创建事件到前端
        let _ = self.app_handle.emit("async:task:created", serde_json::json!({
            "task_id": &task_id,
            "tool_name": &tool_name,
            "provider": &provider,
            "status": "processing",
            "payload": initial_payload,
        }));

        // 启动轮询任务
        let pending_tasks = self.pending_tasks.clone();
        let poll_interval = self.poll_interval;
        let max_poll_duration = self.max_poll_duration;
        let app_handle = self.app_handle.clone();
        let provider_registry = self.provider_registry.clone();
        let task_id_for_spawn = task_id.clone();
        let provider_for_spawn = provider.clone();

        tokio::spawn(async move {
            let start = Instant::now();
            let mut interval = tokio::time::interval(poll_interval);
            
            loop {
                interval.tick().await;
                
                // 检查是否超时
                if start.elapsed() > max_poll_duration {
                    warn!("[AsyncScheduler] 任务轮询超时: {}", task_id_for_spawn);
                    
                    // 更新任务状态为超时
                    {
                        let mut tasks = pending_tasks.lock().await;
                        if let Some(task) = tasks.get_mut(&task_id_for_spawn) {
                            task.status = AsyncTaskStatus::Timeout;
                        }
                    }
                    
                    // 发送超时事件
                    let _ = app_handle.emit("async:task:completed", serde_json::json!({
                        "task_id": &task_id_for_spawn,
                        "status": "timeout",
                        "error": "任务轮询超时（超过5分钟）",
                        "elapsed": start.elapsed().as_secs(),
                    }));
                    
                    // 从 pending_tasks 中移除
                    pending_tasks.lock().await.remove(&task_id_for_spawn);
                    break;
                }

                // 执行状态检查
                let status_result = Self::check_video_status(
                    &task_id_for_spawn, 
                    &provider_for_spawn,
                    &provider_registry
                ).await;
                
                // 更新任务状态
                let mut tasks = pending_tasks.lock().await;
                let task = match tasks.get_mut(&task_id_for_spawn) {
                    Some(t) => t,
                    None => {
                        warn!("[AsyncScheduler] 任务已不存在: {}", task_id_for_spawn);
                        break;
                    }
                };
                
                task.last_poll_at = Instant::now();
                task.poll_count += 1;

                match &status_result {
                    Ok(status_json) => {
                        let status = status_json.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
                        debug!("[AsyncScheduler] 任务 {} 状态: {}", task_id_for_spawn, status);

                        // 发送进度事件
                        let _ = app_handle.emit("async:task:progress", serde_json::json!({
                            "task_id": &task_id_for_spawn,
                            "status": status,
                            "elapsed": start.elapsed().as_secs(),
                            "poll_count": task.poll_count,
                            "details": status_json,
                        }));

                        match status {
                            "SUCCESS" | "completed" | "success" => {
                                task.status = AsyncTaskStatus::Completed(status_json.clone());
                                
                                // 发送完成事件
                                let _ = app_handle.emit("async:task:completed", serde_json::json!({
                                    "task_id": &task_id_for_spawn,
                                    "status": "completed",
                                    "result": status_json,
                                    "elapsed": start.elapsed().as_secs(),
                                }));
                                
                                // 移除任务
                                tasks.remove(&task_id_for_spawn);
                                info!("[AsyncScheduler] 任务完成: {}, 耗时 {} 秒", task_id_for_spawn, start.elapsed().as_secs());
                                break;
                            }
                            "FAILED" | "failed" | "error" | "FAILURE" => {
                                let error_msg = status_json.get("error").and_then(|e| e.as_str())
                                    .or_else(|| status_json.get("failure_reason").and_then(|r| r.as_str()))
                                    .unwrap_or("任务执行失败");
                                
                                task.status = AsyncTaskStatus::Failed(error_msg.to_string());
                                
                                // 发送失败事件
                                let _ = app_handle.emit("async:task:completed", serde_json::json!({
                                    "task_id": &task_id_for_spawn,
                                    "status": "failed",
                                    "error": error_msg,
                                    "elapsed": start.elapsed().as_secs(),
                                }));
                                
                                tasks.remove(&task_id_for_spawn);
                                error!("[AsyncScheduler] 任务失败: {}, 原因: {}", task_id_for_spawn, error_msg);
                                break;
                            }
                            _ => {
                                // 继续处理中，继续轮询
                                debug!("[AsyncScheduler] 任务 {} 仍在处理中，继续轮询", task_id_for_spawn);
                            }
                        }
                    }
                    Err(e) => {
                        error!("[AsyncScheduler] 检查任务状态失败: {}, 错误: {:?}", task_id_for_spawn, e);
                        // 继续轮询，不中断
                    }
                }
                
                // 释放锁
                drop(tasks);
            }
        });
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> Option<AsyncTaskHandle> {
        let tasks = self.pending_tasks.lock().await;
        tasks.get(task_id).cloned()
    }

    /// 获取所有进行中的任务
    pub async fn get_all_pending_tasks(&self) -> Vec<AsyncTaskHandle> {
        let tasks = self.pending_tasks.lock().await;
        tasks.values().cloned().collect()
    }

    /// 取消任务轮询
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let mut tasks = self.pending_tasks.lock().await;
        if tasks.remove(task_id).is_some() {
            info!("[AsyncScheduler] 已取消任务轮询: {}", task_id);
            
            // 发送取消事件
            let _ = self.app_handle.emit("async:task:completed", serde_json::json!({
                "task_id": task_id,
                "status": "cancelled",
            }));
            
            true
        } else {
            false
        }
    }

    /// 检查视频任务状态
    /// 这里通过调用 provider 的 get_task_status 方法
    async fn check_video_status(
        task_id: &str, 
        provider: &str,
        provider_registry: &Arc<crate::agent::providers::ProviderRegistry>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        use crate::agent::providers::media_provider::{MediaProvider, TaskStatus};

        let provider = provider_registry
            .get_media_provider(provider)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider))?;

        match provider.get_task_status(task_id).await {
            Ok(task) => {
                let result = match task.status {
                    TaskStatus::Completed => {
                        if let Some(output) = task.result {
                            serde_json::json!({
                                "success": true,
                                "status": "completed",
                                "task_id": task.task_id,
                                "progress": 100.0,
                                "file_path": output.file_path,
                                "url": output.url,
                                "metadata": output.metadata,
                            })
                        } else {
                            serde_json::json!({
                                "success": false,
                                "status": "failed",
                                "task_id": task.task_id,
                                "error": "视频生成完成但没有返回结果",
                            })
                        }
                    }
                    TaskStatus::Failed => {
                        serde_json::json!({
                            "success": false,
                            "status": "failed",
                            "task_id": task.task_id,
                            "error": task.error.unwrap_or_else(|| "视频生成失败".to_string()),
                        })
                    }
                    TaskStatus::Processing => {
                        serde_json::json!({
                            "success": true,
                            "status": "processing",
                            "task_id": task.task_id,
                            "progress": task.progress,
                            "message": "视频生成中，请稍后...",
                        })
                    }
                    TaskStatus::Pending => {
                        serde_json::json!({
                            "success": true,
                            "status": "pending",
                            "task_id": task.task_id,
                            "progress": 0.0,
                            "message": "任务排队中，请稍后...",
                        })
                    }
                };
                Ok(result)
            }
            Err(e) => Err(format!("查询状态失败: {}", e).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用例...
}
