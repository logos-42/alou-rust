//! Git助手工具 - 命令执行模块

use super::types::{GitStatus, FileStatus};
use crate::tools::{ToolError};
use tokio::process::Command;

/// Git命令执行器
pub struct GitExecutor;

impl GitExecutor {
    /// 执行Git命令
    pub async fn run_git(args: &[&str], working_dir: Option<&str>) -> Result<(bool, String, String), ToolError> {
        let mut cmd = Command::new("git");
        cmd.args(args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute git: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        Ok((success, stdout, stderr))
    }

    /// 检查是否在Git仓库中
    pub async fn is_git_repo(working_dir: Option<&str>) -> bool {
        match Self::run_git(&["rev-parse", "--git-dir"], working_dir).await {
            Ok((success, _, _)) => success,
            Err(_) => false,
        }
    }

    /// 解析git status输出
    pub async fn parse_status(working_dir: Option<&str>) -> Result<GitStatus, ToolError> {
        let (success, stdout, _) = Self::run_git(&["status", "--porcelain", "-b"], working_dir).await?;
        
        if !success {
            return Ok(GitStatus {
                is_git_repo: false,
                current_branch: String::new(),
                ahead: 0,
                behind: 0,
                staged: vec![],
                unstaged: vec![],
                untracked: vec![],
                conflicts: vec![],
                clean: false,
            });
        }

        let mut status = GitStatus {
            is_git_repo: true,
            current_branch: String::new(),
            ahead: 0,
            behind: 0,
            staged: vec![],
            unstaged: vec![],
            untracked: vec![],
            conflicts: vec![],
            clean: true,
        };

        for line in stdout.lines() {
            if line.starts_with("##") {
                let branch_info = &line[3..];
                if let Some(pos) = branch_info.find("...") {
                    status.current_branch = branch_info[..pos].to_string();
                } else if let Some(pos) = branch_info.find('[') {
                    status.current_branch = branch_info[..pos].to_string();
                } else {
                    status.current_branch = branch_info.to_string();
                }
            } else if !line.is_empty() {
                status.clean = false;
                let state = &line[0..2];
                let file = line[3..].to_string();

                match state {
                    "??" => status.untracked.push(file),
                    "UU" | "AA" | "DD" | "AU" | "UA" | "DU" | "UD" => status.conflicts.push(file),
                    s if s.starts_with('M') || s.starts_with('A') || s.starts_with('D') || s.starts_with('R') => {
                        status.staged.push(FileStatus {
                            path: file,
                            status: s.to_string(),
                            change_type: "staged".to_string(),
                        });
                    }
                    s if s.ends_with('M') || s.ends_with('D') => {
                        status.unstaged.push(FileStatus {
                            path: file,
                            status: s.to_string(),
                            change_type: "unstaged".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(status)
    }

    /// 获取当前分支名
    pub async fn get_current_branch(working_dir: Option<&str>) -> Result<String, ToolError> {
        let (_, stdout, _) = Self::run_git(&["branch", "--show-current"], working_dir).await?;
        Ok(stdout.trim().to_string())
    }

    /// 获取最新提交短哈希
    pub async fn get_short_hash(working_dir: Option<&str>) -> Result<String, ToolError> {
        let (_, stdout, _) = Self::run_git(&["rev-parse", "--short", "HEAD"], working_dir).await?;
        Ok(stdout.trim().to_string())
    }
}
