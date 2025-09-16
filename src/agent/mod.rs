pub mod deepseek;
pub mod types;
pub mod core;

// 重新导出常用类型
pub use deepseek::{DeepSeekClient, DeepSeekConfig, ChatMessage, ChatResponse};
pub use types::{
    Agent, AgentConfig, AgentContext, AgentState, AgentMessage, MessageType,
    ToolCall, ToolCallStatus, ToolInfo, BehaviorConfig, WorkspaceConfig, ToolStrategy
};
pub use core::McpAgent;
