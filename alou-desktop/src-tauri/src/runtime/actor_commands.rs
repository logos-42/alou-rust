//! Session Actor 命令接口
//!
//! # 说明
//!
//! 本模块提供 SessionCommandManager 用于发送简单命令到 SessionActor
//! 但实际使用中，推荐直接使用 execute_agent_task Tauri command

use std::sync::Arc;
use crate::bridges::BridgeManager;
use crate::tools::ToolRegistry;

/// Session 命令管理器（简化版）
pub struct SessionCommandManager {
    #[allow(dead_code)]
    router: crate::runtime::router::SessionRouter,
    #[allow(dead_code)]
    tool_registry: Arc<ToolRegistry>,
}

impl SessionCommandManager {
    #[allow(dead_code)]
    pub fn new(bridge_manager: Arc<BridgeManager>, tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            router: crate::runtime::router::SessionRouter::new(bridge_manager, tool_registry.clone()),
            tool_registry,
        }
    }

    /// 获取活跃 session 数量
    pub fn active_session_count(&self) -> usize {
        self.router.active_count()
    }
}

/// Agent 命令（保留用于未来扩展）
#[derive(Debug, Clone)]
pub enum AgentCommand {
    Chat {
        content: String,
        metadata: Option<serde_json::Value>,
    },
    #[allow(dead_code)]
    ToolCall {
        tool_name: String,
        arguments: serde_json::Value,
    },
    #[allow(dead_code)]
    Workflow {
        workflow_id: String,
        input: serde_json::Value,
    },
    #[allow(dead_code)]
    GroupChat {
        group_id: String,
        content: String,
        from_agent: Option<String>,
    },
    #[allow(dead_code)]
    Cancel,
    #[allow(dead_code)]
    GetStatus,
}

/// 命令结果（保留用于未来扩展）
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// 新旧架构兼容的 Agent 执行函数（简化版）
#[allow(dead_code)]
pub async fn execute_agent_task_compatible(
    message: String,
    agent_id: String,
    _config: crate::agent::config::UserApiConfig,
    _bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::bridges::BridgeManager>>>,
    _use_new_architecture: bool,
) -> std::result::Result<crate::agent::task::TaskFinalResult, String> {
    // 简化：直接使用旧架构
    // 未来可以在这里切换到 SessionActor
    Err("Not implemented: use execute_agent_task directly".to_string())
}
