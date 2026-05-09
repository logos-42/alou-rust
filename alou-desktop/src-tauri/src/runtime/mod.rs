//! Session Actor 运行时模块
//!
//! # 架构原则
//!
//! 1. **Agent 决策，Workflow 执行** - 职责分离
//! 2. **1 session = 1 actor** - 无锁并发
//! 3. **Actor 独占状态** - 不 Clone
//! 4. **资源池无状态** - RAII 借用
//! 5. **容量限制** - 所有集合带 max_size
//! 6. **公平调度** - 防止饥饿和独占
//!
//! # 渐进式迁移
//!
//! - 新功能：使用 `actor_commands` 模块
//! - 旧功能：保持原有调用方式
//! - 逐步迁移：通过 `use_new_architecture` 标志控制

pub mod session;
pub mod message;
pub mod actor;
pub mod handle;
pub mod router;
pub mod actor_commands;  // ← 新增：渐进式迁移模块
#[cfg(test)]
mod concurrency_test;  // ← 并发测试

pub use session::SessionRuntime;
pub use message::SessionMessage;
pub use actor::SessionActor;
pub use handle::ActorHandle;
pub use router::SessionRouter;
pub use actor_commands::{
    AgentCommand,
    CommandResult,
    SessionCommandManager,
    execute_agent_task_compatible,
};
