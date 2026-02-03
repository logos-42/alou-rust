//! Git助手工具 - 类型定义模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Git操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum GitAction {
    /// 执行原始Git命令（带安全检查）
    Execute {
        subcommand: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        working_dir: Option<String>,
    },
    
    /// 智能提交
    SmartCommit {
        message: String,
        #[serde(default)]
        working_dir: Option<String>,
        #[serde(default)]
        add_all: bool,
        #[serde(default)]
        allow_empty: bool,
    },
    
    /// 创建功能分支
    CreateFeatureBranch {
        branch_name: String,
        #[serde(default)]
        base_branch: Option<String>,
        #[serde(default)]
        working_dir: Option<String>,
    },
    
    /// 安全合并
    SafeMerge {
        source_branch: String,
        #[serde(default = "default_merge_strategy")]
        strategy: String,
        #[serde(default)]
        working_dir: Option<String>,
    },
    
    /// 状态检查
    StatusCheck {
        #[serde(default)]
        working_dir: Option<String>,
        #[serde(default)]
        detailed: bool,
    },
    
    /// 变更摘要
    DiffSummary {
        #[serde(default)]
        working_dir: Option<String>,
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        stat_only: bool,
    },
    
    /// 提交历史
    LogHistory {
        #[serde(default)]
        working_dir: Option<String>,
        #[serde(default = "default_log_count")]
        count: usize,
        #[serde(default)]
        branch: Option<String>,
        #[serde(default = "default_log_format")]
        format: String,
    },
    
    /// 暂存管理
    StashManagement {
        operation: String,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        working_dir: Option<String>,
    },
    
    /// 获取提示词
    GetPrompt {
        scenario: String,
        #[serde(default)]
        context: HashMap<String, String>,
    },
    
    /// 批量操作
    BatchOperation {
        operations: Vec<BatchCommitOp>,
        #[serde(default)]
        working_dir: Option<String>,
    },
    
    /// 撤销操作
    UndoOperation {
        undo_type: String,
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        force: bool,
        #[serde(default)]
        working_dir: Option<String>,
    },
    
    /// 远程同步
    RemoteSync {
        operation: String,
        #[serde(default = "default_remote")]
        remote: String,
        #[serde(default)]
        branch: Option<String>,
        #[serde(default)]
        working_dir: Option<String>,
    },
    
    /// 初始化仓库
    InitRepository {
        path: String,
        #[serde(default = "default_branch_name")]
        initial_branch: String,
    },
    
    /// 配置管理
    ConfigManagement {
        operation: String,
        key: String,
        #[serde(default)]
        value: Option<String>,
        #[serde(default)]
        global: bool,
    },
}

fn default_merge_strategy() -> String { "merge".to_string() }
fn default_log_count() -> usize { 10 }
fn default_log_format() -> String { "oneline".to_string() }
fn default_remote() -> String { "origin".to_string() }
fn default_branch_name() -> String { "main".to_string() }

/// 批量提交操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCommitOp {
    pub files: Vec<String>,
    pub message: String,
}

/// Git状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub is_git_repo: bool,
    pub current_branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged: Vec<FileStatus>,
    pub unstaged: Vec<FileStatus>,
    pub untracked: Vec<String>,
    pub conflicts: Vec<String>,
    pub clean: bool,
}

/// 文件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
    pub change_type: String,
}

/// Git提示词模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPrompt {
    pub scenario: String,
    pub description: String,
    pub suggested_commands: Vec<String>,
    pub safety_checklist: Vec<String>,
    pub best_practice: String,
}
