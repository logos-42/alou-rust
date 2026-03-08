//! 任务队列工具
//!
//! 提供任务队列的Tauri命令和工具接口

use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::tools::task_queue::{
    TaskQueueManager, Task, TaskPriority, TaskStatus, TaskResult, TaskStats,
};

/// 任务队列工具类型别名
pub type TaskQueueManagerRef = Arc<Mutex<TaskQueueManager>>;

/// 添加任务参数
#[derive(Serialize, Deserialize)]
pub struct AddTaskParams {
    title: String,
    description: String,
    priority: String,
    executor: Option<String>,
}

/// 更新状态参数
#[derive(Serialize, Deserialize)]
pub struct UpdateStatusParams {
    task_id: String,
    status: String,
}

/// 设置结果参数
#[derive(Serialize, Deserialize)]
pub struct SetResultParams {
    task_id: String,
    success: bool,
    output: Option<serde_json::Value>,
    error: Option<String>,
    execution_time_ms: u64,
}

/// 任务队列工具命令实现

#[tauri::command]
pub async fn add_task(
    state: State<'_, TaskQueueManagerRef>,
    params: AddTaskParams,
) -> Result<String, String> {
    let mut manager = state.inner().lock().await;
    
    let priority = match params.priority.as_str() {
        "Critical" => TaskPriority::Critical,
        "High" => TaskPriority::High,
        "Medium" => TaskPriority::Medium,
        "Low" => TaskPriority::Low,
        _ => TaskPriority::Medium,
    };
    
    let task = Task::new(
        params.title,
        params.description,
        priority,
        params.executor,
    );
    
    let task_id = manager.add_task(task).await;
    Ok(task_id)
}

#[tauri::command]
pub async fn get_next_task(
    state: State<'_, TaskQueueManagerRef>,
    executor: Option<String>,
) -> Result<Option<Task>, String> {
    let mut manager = state.inner().lock().await;
    let task = manager.next_task(executor.as_deref()).await;
    Ok(task)
}

#[tauri::command]
pub async fn update_task_status(
    state: State<'_, TaskQueueManagerRef>,
    params: UpdateStatusParams,
) -> Result<bool, String> {
    let mut manager = state.inner().lock().await;
    
    let status = match params.status.as_str() {
        "Pending" => TaskStatus::Pending,
        "InProgress" => TaskStatus::InProgress,
        "Completed" => TaskStatus::Completed,
        "Failed" => TaskStatus::Failed,
        "Cancelled" => TaskStatus::Cancelled,
        _ => return Err("无效的状态".to_string()),
    };
    
    let result = manager.update_status(&params.task_id, status).await;
    Ok(result)
}

#[tauri::command]
pub async fn set_task_result(
    state: State<'_, TaskQueueManagerRef>,
    params: SetResultParams,
) -> Result<bool, String> {
    let mut manager = state.inner().lock().await;
    
    let result = TaskResult {
        success: params.success,
        output: params.output,
        error: params.error,
        executed_at: chrono::Utc::now().timestamp(),
        execution_time_ms: params.execution_time_ms,
    };
    
    let result = manager.set_result(&params.task_id, result).await;
    Ok(result)
}

#[tauri::command]
pub async fn list_tasks(
    state: State<'_, TaskQueueManagerRef>,
    filter: Option<String>,
) -> Result<Vec<Task>, String> {
    let manager = state.inner().lock().await;
    
    let filter_status = match filter {
        Some(s) => match s.as_str() {
            "Pending" => Some(TaskStatus::Pending),
            "InProgress" => Some(TaskStatus::InProgress),
            "Completed" => Some(TaskStatus::Completed),
            "Failed" => Some(TaskStatus::Failed),
            "Cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        },
        None => None,
    };
    
    let tasks = manager.list_tasks(filter_status);
    Ok(tasks)
}

#[tauri::command]
pub async fn get_task_stats(
    state: State<'_, TaskQueueManagerRef>,
) -> Result<TaskStats, String> {
    let manager = state.inner().lock().await;
    let stats = manager.stats();
    Ok(stats)
}

#[tauri::command]
pub async fn get_task_by_id(
    state: State<'_, TaskQueueManagerRef>,
    task_id: String,
) -> Result<Option<Task>, String> {
    let manager = state.inner().lock().await;
    let tasks = manager.list_tasks(None);
    let task = tasks.into_iter().find(|t| t.id == task_id);
    Ok(task)
}

/// 初始化任务队列工具
pub fn initialize_task_queue_tool() -> Result<TaskQueueManagerRef, Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let manager = runtime.block_on(async { TaskQueueManager::new(None) })?;
    Ok(Arc::new(Mutex::new(manager)))
}
