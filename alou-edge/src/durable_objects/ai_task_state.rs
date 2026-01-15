//! AI任务状态管理模块
//! 
//! 负责任务状态的存储、加载和管理

use crate::compatibility::models::TaskStatus;
use crate::utils::time::current_timestamp_secs;

// 存储键名函数 - 使用任务ID前缀避免冲突
pub fn get_state_key(task_id: &str) -> String {
    format!("{}:task_state", task_id)
}

pub fn get_request_key(task_id: &str) -> String {
    format!("{}:task_request", task_id)
}

pub fn get_result_key(task_id: &str) -> String {
    format!("{}:task_result", task_id)
}

pub fn get_tool_result_key(task_id: &str) -> String {
    format!("{}:last_tool_result", task_id)
}

/// 任务缓存（内存中的状态副本）
#[derive(Clone)]
pub struct TaskCache {
    pub task_id: String,
    pub status: TaskStatus,
    pub progress: f32,
    pub current_step: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 任务状态
#[derive(Clone)]
pub struct TaskState {
    pub status: TaskStatus,
    pub progress: f32,
    pub current_step: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub error: Option<String>,
}

impl TaskState {
    /// 创建初始状态
    pub fn new_initial() -> Self {
        let now = current_timestamp_secs();
        Self {
            status: TaskStatus::Queued,
            progress: 0.0,
            current_step: "等待执行".to_string(),
            created_at: now,
            updated_at: now,
            error: None,
        }
    }
    
    /// 创建失败状态
    pub fn new_failed(error: String) -> Self {
        let now = current_timestamp_secs();
        Self {
            status: TaskStatus::Failed,
            progress: 1.0,
            current_step: "执行失败".to_string(),
            created_at: now,
            updated_at: now,
            error: Some(error),
        }
    }
}
