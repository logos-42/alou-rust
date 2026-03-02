//! Task 相关类型
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub timeout: u64,
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            timeout: 300_000,
            retries: 1,
        }
    }
}

// Re-export types from types.rs for backward compatibility
pub use crate::swarm::types::{
    Task,
    TaskResult,
    TaskStatus,
};
