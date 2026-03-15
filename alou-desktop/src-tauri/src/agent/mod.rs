//! Agent 模块
//!
//! 包含 AI Agent 核心功能、Provider 实现、媒体生成等

pub mod config;
pub mod error;
pub mod ai_client;
pub mod providers;
pub mod media_config;
pub mod media;
pub mod executor;
pub mod task;
pub mod context_layers;
pub mod commands;
pub mod hook_commands;
pub mod async_tool_manager;  // 异步工具调用管理器

// 重新导出常用类型
pub use config::ApiConfig;
pub use error::AgentError;
pub use ai_client::AiClient;
pub use providers::ProviderRegistry;
pub use async_tool_manager::{AsyncToolManager, AsyncToolStatus, AsyncToolTask, ToolExecutor};
