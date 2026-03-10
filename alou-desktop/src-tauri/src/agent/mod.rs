//! Agent 模块
//!
//! 提供 AI Agent 的核心功能，包括 API 调用、任务管理、Ralph Loop 执行等
//! 
//! # Architecture
//! 
//! Following "The Mythical Man-Month" principles:
//! - **Conceptual Integrity**: Unified agent design
//! - **Clear Interfaces**: Well-defined task and skill contracts
//! - **Agent Autonomy**: Independent decision-making capabilities
//! - **Swarm Coordination**: Multi-agent parallel execution

pub mod config;
pub mod ai_client;
pub mod providers;
pub mod task;
pub mod executor;
pub mod streaming;
pub mod commands;
pub mod error;
pub mod role;
pub mod memory;
pub mod swarm;        // Multi-agent swarm coordination
pub mod autonomy;     // Agent autonomy framework
pub mod agent_hook;   // Agent Hook system for real-time instruction injection
pub mod hook_commands; // Tauri commands for Agent Hook

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

