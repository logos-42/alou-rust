//! Session Actor 运行时模块
//!
//! 包含 Agent Runtime 的核心组件：
//! - SessionActor: Actor 模型实现
//! - SessionRuntime: 状态管理
//! - Hook 系统：事件总线式生命周期拦截
//! - Command 系统：主动控制命令
//! - EventLog: 事件日志系统（Debug/Replay/Metrics）

pub mod session;
pub mod message;
pub mod actor;
pub mod handle;
pub mod router;
pub mod hook;
pub mod command;
pub mod event_log;

pub use session::SessionRuntime;
pub use message::SessionMessage;
pub use actor::SessionActor;
pub use handle::ActorHandle;
pub use router::SessionRouter;
pub use hook::{HookManager, HookContext, HookEvent, HookResult, AgentHook};
pub use command::{AgentCommand, CommandQueue, CommandContext, CommandResult, CommandHandler};
pub use event_log::{EventLog, EventEntry, EventType, EventLevel, EventQuery, SessionMetrics, SessionDataset};
