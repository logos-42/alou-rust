//! Agent Scheduler - 智能体调度器
//!
//! # 架构愿景
//!
//! 从 **Agent Runtime** 升级为 **Agent Operating System**
//!
//! # 核心功能
//!
//! - 智能体优先级调度
//! - 资源配额管理
//! - 并发控制
//! - 任务队列管理
//! - 负载均衡
//!
//! # 架构原则
//!
//! 1. **公平调度** - 防止单个 agent 独占资源
//! 2. **优先级驱动** - 支持关键任务优先
//! 3. **资源隔离** - 防止资源竞争
//! 4. **可观测性** - 所有调度决策可追踪

pub mod types;
pub mod queue;
pub mod scheduler;
pub mod quota;

pub use types::{AgentPriority, AgentState, AgentTask, TaskType};
pub use queue::TaskQueue;
pub use scheduler::AgentScheduler;
pub use quota::{ResourceQuota, QuotaManager};
