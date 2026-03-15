//! Tasks - 任务管理（占位符）
//!
//! TODO: 实现实际的任务管理

use std::sync::Arc;
use tokio::sync::RwLock;

/// 任务管理器（占位符）
pub struct TasksManager {
    tasks: Arc<RwLock<Vec<TaskInfo>>>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

impl TasksManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(vec![])),
        }
    }
    
    pub async fn list_tasks(&self) -> Vec<TaskInfo> {
        self.tasks.read().await.clone()
    }
}

impl Default for TasksManager {
    fn default() -> Self {
        Self::new()
    }
}
