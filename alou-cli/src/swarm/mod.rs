//! Agent Swarm 系统 - 基于人月神话的分布式任务协调
//! 
//! 核心设计原则:
//! - 概念完整性: 统一的接口和数据模型
//! - 减少沟通成本: 明确的任务边界和接口契约
//! - 外科手术式团队: Chief Agent + 多个独立 Task Agent

pub mod coordinator;
pub mod executor;
pub mod planner;
pub mod skill_registry;
pub mod task;
pub mod types;

pub use coordinator::{CoordinatorConfig, SwarmCoordinator};
pub use executor::{LocalExecutor, TaskExecutor};
pub use planner::{ExecutionPlan, ExecutionStrategy, TaskPlanner};
pub use skill_registry::SkillRegistry;
pub use task::{Task, TaskConfig, TaskInput, TaskResult, TaskStatus};
pub use types::*;
