//! Skills 工具
//!
//! 提供技能管理和执行功能，支持AI自动搜索和使用技能

mod types;
mod executor;
mod commands;

pub use types::*;
pub use executor::*;
pub use commands::*;

use crate::skills::{SkillStorage, SkillManifest, SkillSearchParams, SkillCategory};
use crate::skills::executor::{SkillExecutionContext, SkillExecutorFactory};
use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use std::sync::Arc;
use std::path::PathBuf;

/// Skills 工具
pub struct SkillsTool {
    metadata: ToolMetadata,
    storage: Arc<SkillStorage>,
}

impl SkillsTool {
    /// 创建新的 Skills 工具
    pub async fn new() -> Result<Self, String> {
        let storage_path = PathBuf::from("./data/skills.json");
        let storage = Arc::new(SkillStorage::new(storage_path).await?);
        storage.initialize().await?;

        Ok(Self {
            metadata: ToolMetadata {
                id: "skills".to_string(),
                name: "Skills Management Tool".to_string(),
                description: "管理和执行可复用的技能，支持AI自动搜索和执行".to_string(),
                category: ToolCategory::Skills,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "2.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "execute".to_string()],
            },
            storage,
        })
    }

    /// 获取存储引用
    pub fn storage(&self) -> Arc<SkillStorage> {
        self.storage.clone()
    }
}

#[async_trait]
impl ToolExecutor for SkillsTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        commands::execute_command(self, args, context).await
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }
        if args.get("action").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: action".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        commands::help_text()
    }
}
