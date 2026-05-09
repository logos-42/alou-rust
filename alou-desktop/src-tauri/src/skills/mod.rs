//! Skills 系统模块
//!
//! 提供统一的技能定义、存储和执行功能
//! 支持 AI 自动发现和使用技能

pub mod manifest;
pub mod storage;
pub mod executor;
pub mod agent_skill;
pub mod builtin;
pub mod prompt;
pub mod toolchain;
pub mod script;

pub use manifest::*;
pub use storage::SkillStorage;
pub use executor::{SkillExecutor, SkillExecutionContext, SkillExecutionResult};
pub use agent_skill::AgentSkillExecutor;
