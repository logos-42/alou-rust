//! Agent 模块
//!
//! 提供 AI Agent 的核心功能，包括 API 调用、任务管理、Ralph Loop 执行等

pub mod config;
pub mod ai_client;
pub mod providers;
pub mod task;
pub mod executor;
pub mod streaming;
pub mod commands;
pub mod error;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent 状态
pub struct AgentState {
    pub config: Arc<RwLock<config::ApiConfig>>,
    pub task_manager: Arc<task::TaskManager>,
    pub tool_bridge: Arc<crate::bridges::ToolBridge>,
    pub tool_registry: Arc<crate::tools::ToolRegistry>,
}

impl AgentState {
    pub fn new(tool_bridge: Arc<crate::bridges::ToolBridge>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config::ApiConfig::default())),
            task_manager: Arc::new(task::TaskManager::new()),
            tool_bridge,
            tool_registry: Arc::new(crate::tools::ToolRegistry::new()),
        }
    }
}

/// 重新导出主要类型
pub use config::{ApiConfig, UserApiConfig, WorkersApiConfig, ExecutionStrategy};
pub use task::{Task, TaskStatus, TaskEvent, TaskFinalResult, TaskManager};
pub use executor::{RalphLoopExecutor, ExecutionResult, ExecutorError};
pub use streaming::{StreamingExecutor, StreamEvent};
pub use error::{AgentError, Result};
