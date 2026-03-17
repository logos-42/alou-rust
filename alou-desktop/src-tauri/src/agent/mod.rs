//! Agent 模块
//!
//! 包含 AI Agent 核心功能、Provider 实现、媒体生成等

pub mod config;
pub mod error;
pub mod ai_client;
pub mod ai_client_pool;  // AI Client 缓存池
pub mod providers;
pub mod media_config;
pub mod media;
pub mod executor;
pub mod task;
pub mod context_layers;
pub mod commands;
pub mod hook_commands;
pub mod agent_hook;  // 新增
pub mod async_tool_manager;  // 异步工具调用管理器
pub mod perception;  // 智能感知层
pub mod memory;  // 记忆系统
pub mod goal;  // 目标管理系统

// 可选的存储后端（需要启用相应 feature）
#[cfg(feature = "redis-storage")]
pub mod goal_storage_redis;
#[cfg(feature = "postgres-storage")]
pub mod goal_storage_postgres;
#[cfg(all(feature = "redis-storage", feature = "postgres-storage"))]
pub mod goal_storage_hybrid;

// 重新导出常用类型
pub use config::ApiConfig;
pub use error::AgentError;
pub use ai_client::AiClient;
pub use ai_client_pool::AiClientPool;
pub use providers::ProviderRegistry;
pub use async_tool_manager::{AsyncToolManager, AsyncToolStatus, AsyncToolTask, ToolExecutor};
pub use perception::{PerceptionEngine, RetrievedContext, Intent};
pub use memory::{MemoryManager, Memory, MemoryType, Importance};
pub use goal::{GoalTracker, Goal, GoalStatus, Priority, GoalStorage, InMemoryGoalStorage, GoalSummary as GoalInfo};

#[cfg(feature = "redis-storage")]
pub use goal_storage_redis::RedisGoalStorage;
#[cfg(feature = "postgres-storage")]
pub use goal_storage_postgres::{PostgresGoalStorage, GoalStats};
#[cfg(all(feature = "redis-storage", feature = "postgres-storage"))]
pub use goal_storage_hybrid::{HybridGoalStorage, HybridStorageConfig};
