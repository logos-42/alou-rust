//! Git助手工具 (Git Helper Tool)
//!
//! 为智能体提供Git操作的封装和最佳实践提示，通过终端工具调用系统Git命令。
//!
//! 模块结构：
//! - types: 类型定义（GitAction, GitStatus等）
//! - executor: Git命令执行器
//! - handlers: 操作处理函数
//! - prompts: 提示词生成

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;

pub mod types;
pub mod executor;
pub mod handlers;
pub mod prompts;

use types::GitAction;

/// Git助手工具
pub struct GitHelperTool {
    metadata: ToolMetadata,
}

impl GitHelperTool {
    /// 创建新的Git助手工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "git_helper".to_string(),
                name: "Git Helper Tool".to_string(),
                description: "智能Git操作助手，提供封装的工作流和安全检查".to_string(),
                category: ToolCategory::Terminal,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec!["git".to_string()],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["execute".to_string()],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for GitHelperTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let action: GitAction = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        match action {
            GitAction::Execute { subcommand, args: cmd_args, working_dir } => {
                handlers::handle_execute(subcommand, cmd_args, working_dir).await
            }
            GitAction::SmartCommit { message, working_dir, add_all, allow_empty } => {
                handlers::handle_smart_commit(message, working_dir, add_all, allow_empty).await
            }
            GitAction::CreateFeatureBranch { branch_name, base_branch, working_dir } => {
                handlers::handle_create_branch(branch_name, base_branch, working_dir).await
            }
            GitAction::SafeMerge { source_branch, strategy, working_dir } => {
                handlers::handle_safe_merge(source_branch, strategy, working_dir).await
            }
            GitAction::StatusCheck { working_dir, detailed } => {
                handlers::handle_status_check(working_dir, detailed).await
            }
            GitAction::DiffSummary { working_dir, target, stat_only } => {
                handlers::handle_diff_summary(working_dir, target, stat_only).await
            }
            GitAction::LogHistory { working_dir, count, branch, format } => {
                handlers::handle_log_history(working_dir, count, branch, format).await
            }
            GitAction::StashManagement { operation, message, working_dir } => {
                handlers::handle_stash(operation, message, working_dir).await
            }
            GitAction::GetPrompt { scenario, context } => {
                handlers::handle_get_prompt(scenario, context).await
            }
            GitAction::BatchOperation { operations, working_dir } => {
                handlers::handle_batch_operation(operations, working_dir).await
            }
            GitAction::UndoOperation { undo_type, target, force, working_dir } => {
                handlers::handle_undo(undo_type, target, force, working_dir).await
            }
            GitAction::RemoteSync { operation, remote, branch, working_dir } => {
                handlers::handle_remote_sync(operation, remote, branch, working_dir).await
            }
            GitAction::InitRepository { path, initial_branch } => {
                handlers::handle_init(path, initial_branch).await
            }
            GitAction::ConfigManagement { operation, key, value, global } => {
                handlers::handle_config(operation, key, value, global).await
            }
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if serde_json::from_value::<GitAction>(args.clone()).is_ok() {
            Ok(())
        } else {
            Err(ToolError::InvalidArguments("Invalid git action arguments".to_string()))
        }
    }

    fn help(&self) -> String {
        include_str!("./help.md").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_git_helper_creation() {
        let tool = GitHelperTool::new();
        assert_eq!(tool.metadata().id, "git_helper");
    }

    #[tokio::test]
    async fn test_suggest_commit_message() {
        use prompts::suggest_commit_message;
        let files = vec!["src/main.rs".to_string(), "tests/test.rs".to_string()];
        let suggestions = suggest_commit_message(&files);
        assert!(!suggestions.is_empty());
    }
}
