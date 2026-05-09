//! Tasks - 任务管理
//!
//! 负责任务的创建、执行、监控和持久化

pub mod types;
pub mod manager;
pub mod queue;
pub mod executor;
pub mod unified_task;

pub use manager::TasksManager;
pub use types::{Task, TaskStatus};
