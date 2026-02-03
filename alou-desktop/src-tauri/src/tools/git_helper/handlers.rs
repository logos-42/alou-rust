//! Git助手工具 - 操作处理模块

use super::types::*;
use super::executor::GitExecutor;
use super::prompts::{generate_prompt, suggest_commit_message};
use crate::tools::{ToolResult, ToolError};
use std::collections::HashMap;
use std::path::PathBuf;

/// 处理执行命令
pub async fn handle_execute(
    subcommand: String,
    args: Vec<String>,
    working_dir: Option<String>,
) -> Result<ToolResult, ToolError> {
    let dangerous_commands = vec!["reset", "clean", "push", "filter-branch"];
    if dangerous_commands.contains(&subcommand.as_str()) {
        return Err(ToolError::PermissionDenied(
            format!("'{}' is a potentially dangerous command.", subcommand)
        ));
    }

    let mut cmd_args = vec![subcommand.as_str()];
    cmd_args.extend(args.iter().map(|s| s.as_str()));

    let (success, stdout, stderr) = GitExecutor::run_git(&cmd_args, working_dir.as_deref()).await?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({
            "stdout": stdout,
            "stderr": stderr,
            "command": format!("git {}", cmd_args.join(" ")),
        }),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(stdout.clone()),
        warnings: if stderr.is_empty() { vec![] } else { vec![stderr] },
        context: None,
    })
}

/// 处理智能提交
pub async fn handle_smart_commit(
    message: String,
    working_dir: Option<String>,
    add_all: bool,
    allow_empty: bool,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let status = GitExecutor::parse_status(working_dir.as_deref()).await?;
    
    if status.clean && !allow_empty {
        return Ok(ToolResult {
            success: true,
            data: serde_json::json!({"status": "nothing_to_commit"}),
            error: None,
            execution_time_ms: 0,
            output: Some("Nothing to commit, working tree clean".to_string()),
            warnings: vec![],
            context: None,
        });
    }

    if !status.conflicts.is_empty() {
        return Err(ToolError::ExecutionFailed(
            format!("Unresolved conflicts: {}", status.conflicts.join(", "))
        ));
    }

    if add_all {
        GitExecutor::run_git(&["add", "-A"], working_dir.as_deref()).await?;
    }

    let mut commit_args = vec!["commit", "-m", &message];
    if allow_empty {
        commit_args.push("--allow-empty");
    }
    
    let (success, _, stderr) = GitExecutor::run_git(&commit_args, working_dir.as_deref()).await?;
    let short_hash = GitExecutor::get_short_hash(working_dir.as_deref()).await?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({
            "commit_hash": short_hash,
            "message": message,
            "files_changed": status.staged.len() + if add_all { status.unstaged.len() + status.untracked.len() } else { 0 },
        }),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(format!("[{}] {}", short_hash, message)),
        warnings: if stderr.is_empty() { vec![] } else { vec![stderr] },
        context: None,
    })
}

/// 处理创建分支
pub async fn handle_create_branch(
    branch_name: String,
    base_branch: Option<String>,
    working_dir: Option<String>,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    if let Some(ref base) = base_branch {
        GitExecutor::run_git(&["fetch", "origin", base], working_dir.as_deref()).await.ok();
    }

    let checkout_target = base_branch.as_deref().unwrap_or("HEAD");
    GitExecutor::run_git(&["checkout", checkout_target], working_dir.as_deref()).await?;

    let (success, _, stderr) = GitExecutor::run_git(
        &["checkout", "-b", &branch_name],
        working_dir.as_deref()
    ).await?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({
            "branch": branch_name,
            "base_branch": base_branch,
            "current_branch": branch_name,
        }),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(format!("Switched to a new branch '{}'", branch_name)),
        warnings: vec![],
        context: None,
    })
}

/// 处理安全合并
pub async fn handle_safe_merge(
    source_branch: String,
    strategy: String,
    working_dir: Option<String>,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let status = GitExecutor::parse_status(working_dir.as_deref()).await?;
    if !status.clean {
        return Err(ToolError::ExecutionFailed(
            "Working tree not clean. Commit or stash changes first.".to_string()
        ));
    }

    let current_branch = GitExecutor::get_current_branch(working_dir.as_deref()).await?;

    let (success, _, stderr) = match strategy.as_str() {
        "rebase" => GitExecutor::run_git(&["rebase", &source_branch], working_dir.as_deref()).await,
        "squash" => GitExecutor::run_git(&["merge", "--squash", &source_branch], working_dir.as_deref()).await,
        _ => GitExecutor::run_git(&["merge", &source_branch], working_dir.as_deref()).await,
    }?;

    let status_after = GitExecutor::parse_status(working_dir.as_deref()).await?;
    let has_conflicts = !status_after.conflicts.is_empty();

    Ok(ToolResult {
        success: success && !has_conflicts,
        data: serde_json::json!({
            "strategy": strategy,
            "source_branch": source_branch,
            "target_branch": current_branch,
            "has_conflicts": has_conflicts,
            "conflicts": status_after.conflicts,
        }),
        error: if has_conflicts { 
            Some(format!("Merge conflicts detected: {}", status_after.conflicts.join(", ")))
        } else if !success {
            Some(stderr.clone())
        } else {
            None
        },
        execution_time_ms: 0,
        output: Some(if has_conflicts {
            format!("Merge conflicts! Files: {}", status_after.conflicts.join(", "))
        } else {
            format!("Successfully merged")
        }),
        warnings: if has_conflicts { vec!["Manual conflict resolution required".to_string()] } else { vec![] },
        context: None,
    })
}

/// 处理状态检查
pub async fn handle_status_check(
    working_dir: Option<String>,
    _detailed: bool,
) -> Result<ToolResult, ToolError> {
    let status = GitExecutor::parse_status(working_dir.as_deref()).await?;

    if !status.is_git_repo {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let output = if status.clean {
        format!("On branch {}\nNothing to commit", status.current_branch)
    } else {
        let mut parts = vec![];
        if !status.staged.is_empty() { parts.push(format!("{} staged", status.staged.len())); }
        if !status.unstaged.is_empty() { parts.push(format!("{} unstaged", status.unstaged.len())); }
        if !status.untracked.is_empty() { parts.push(format!("{} untracked", status.untracked.len())); }
        if !status.conflicts.is_empty() { parts.push(format!("{} conflicts!", status.conflicts.len())); }
        format!("On branch {}\n{}", status.current_branch, parts.join(", "))
    };

    Ok(ToolResult {
        success: true,
        data: serde_json::to_value(&status).unwrap(),
        error: None,
        execution_time_ms: 0,
        output: Some(output),
        warnings: if status.conflicts.is_empty() { vec![] } else { 
            vec![format!("Conflicts: {}", status.conflicts.join(", "))]
        },
        context: None,
    })
}

/// 处理差异摘要
pub async fn handle_diff_summary(
    working_dir: Option<String>,
    target: Option<String>,
    stat_only: bool,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let target_ref = target.as_deref().unwrap_or("HEAD");
    let args = if stat_only {
        vec!["diff", "--stat", target_ref]
    } else {
        vec!["diff", target_ref]
    };

    let (success, stdout, stderr) = GitExecutor::run_git(&args, working_dir.as_deref()).await?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({"diff": stdout, "target": target_ref}),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(stdout.clone()),
        warnings: vec![],
        context: None,
    })
}

/// 处理日志历史
pub async fn handle_log_history(
    working_dir: Option<String>,
    count: usize,
    branch: Option<String>,
    format: String,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let format_arg = match format.as_str() {
        "oneline" => "--oneline",
        "short" => "--pretty=short",
        "full" => "--pretty=full",
        _ => "--oneline",
    };

    let count_str = count.to_string();
    let mut args = vec!["log", format_arg, "-n", &count_str];
    if let Some(ref b) = branch {
        args.push(b.as_str());
    }

    let (success, stdout, stderr) = GitExecutor::run_git(&args, working_dir.as_deref()).await?;

    let commits: Vec<HashMap<String, String>> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let mut map = HashMap::new();
            if format == "oneline" {
                if let Some(pos) = line.find(' ') {
                    map.insert("hash".to_string(), line[..pos].to_string());
                    map.insert("message".to_string(), line[pos+1..].to_string());
                }
            } else {
                map.insert("line".to_string(), line.to_string());
            }
            map
        })
        .collect();

    Ok(ToolResult {
        success,
        data: serde_json::json!({"commits": commits, "count": commits.len()}),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(stdout.clone()),
        warnings: vec![],
        context: None,
    })
}

/// 处理暂存管理
pub async fn handle_stash(
    operation: String,
    message: Option<String>,
    working_dir: Option<String>,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let (success, stdout, stderr) = match operation.as_str() {
        "save" => {
            let msg = message.as_deref().unwrap_or("WIP");
            GitExecutor::run_git(&["stash", "push", "-m", msg], working_dir.as_deref()).await
        }
        "pop" => GitExecutor::run_git(&["stash", "pop"], working_dir.as_deref()).await,
        "list" => GitExecutor::run_git(&["stash", "list"], working_dir.as_deref()).await,
        "clear" => GitExecutor::run_git(&["stash", "clear"], working_dir.as_deref()).await,
        "drop" => GitExecutor::run_git(&["stash", "drop"], working_dir.as_deref()).await,
        _ => return Err(ToolError::InvalidArguments(format!("Unknown stash operation: {}", operation))),
    }?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({"operation": operation, "output": stdout}),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(stdout.clone()),
        warnings: vec![],
        context: None,
    })
}

/// 处理获取提示词
pub async fn handle_get_prompt(
    scenario: String,
    _context: HashMap<String, String>,
) -> Result<ToolResult, ToolError> {
    let prompt = generate_prompt(&scenario);

    Ok(ToolResult {
        success: true,
        data: serde_json::json!(prompt),
        error: None,
        execution_time_ms: 0,
        output: Some(format!("Git提示词 [{}]: {}", scenario, prompt.description)),
        warnings: vec![],
        context: None,
    })
}

/// 处理批量操作
pub async fn handle_batch_operation(
    operations: Vec<BatchCommitOp>,
    working_dir: Option<String>,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let mut results = vec![];
    let mut errors = vec![];

    for op in operations {
        for file in &op.files {
            let (success, _, stderr) = GitExecutor::run_git(&["add", file], working_dir.as_deref()).await?;
            if !success {
                errors.push(format!("Failed to add {}: {}", file, stderr));
                continue;
            }
        }

        let (success, stdout, stderr) = GitExecutor::run_git(
            &["commit", "-m", &op.message],
            working_dir.as_deref()
        ).await?;

        if success {
            results.push(serde_json::json!({
                "files": op.files,
                "message": op.message,
                "output": stdout,
            }));
        } else {
            errors.push(format!("Failed to commit '{}': {}", op.message, stderr));
        }
    }

    Ok(ToolResult {
        success: errors.is_empty() || !results.is_empty(),
        data: serde_json::json!({"commits": results, "errors": errors}),
        error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
        execution_time_ms: 0,
        output: Some(format!("Batch complete: {}/{} commits", results.len(), results.len() + errors.len())),
        warnings: errors,
        context: None,
    })
}

/// 处理撤销操作
pub async fn handle_undo(
    undo_type: String,
    target: Option<String>,
    force: bool,
    working_dir: Option<String>,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let (success, stdout, stderr) = match undo_type.as_str() {
        "unstage" => {
            let t = target.as_deref().unwrap_or(".");
            GitExecutor::run_git(&["reset", "HEAD", t], working_dir.as_deref()).await
        }
        "restore" => {
            let t = target.as_deref().unwrap_or(".");
            GitExecutor::run_git(&["restore", t], working_dir.as_deref()).await
        }
        "reset" => {
            if !force {
                return Err(ToolError::PermissionDenied(
                    "Reset is destructive. Use force=true to confirm.".to_string()
                ));
            }
            let t = target.as_deref().unwrap_or("HEAD");
            GitExecutor::run_git(&["reset", "--hard", t], working_dir.as_deref()).await
        }
        "revert" => {
            let t = target.clone().ok_or_else(|| ToolError::InvalidArguments("Revert requires target commit".to_string()))?;
            GitExecutor::run_git(&["revert", &t], working_dir.as_deref()).await
        }
        _ => return Err(ToolError::InvalidArguments(format!("Unknown undo type: {}", undo_type))),
    }?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({"undo_type": undo_type, "target": target}),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(stdout.clone()),
        warnings: vec![],
        context: None,
    })
}

/// 处理远程同步
pub async fn handle_remote_sync(
    operation: String,
    remote: String,
    branch: Option<String>,
    working_dir: Option<String>,
) -> Result<ToolResult, ToolError> {
    if !GitExecutor::is_git_repo(working_dir.as_deref()).await {
        return Err(ToolError::ExecutionFailed("Not a git repository".to_string()));
    }

    let (success, stdout, stderr) = match operation.as_str() {
        "fetch" => GitExecutor::run_git(&["fetch", &remote], working_dir.as_deref()).await,
        "pull" => {
            let args = if let Some(ref b) = branch {
                vec!["pull", &remote, b.as_str()]
            } else {
                vec!["pull", &remote]
            };
            GitExecutor::run_git(&args, working_dir.as_deref()).await
        }
        "push" => {
            let args = if let Some(ref b) = branch {
                vec!["push", &remote, b.as_str()]
            } else {
                vec!["push", &remote]
            };
            GitExecutor::run_git(&args, working_dir.as_deref()).await
        }
        _ => return Err(ToolError::InvalidArguments(format!("Unknown remote operation: {}", operation))),
    }?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({"operation": operation, "remote": remote}),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(stdout.clone()),
        warnings: vec![],
        context: None,
    })
}

/// 处理初始化仓库
pub async fn handle_init(
    path: String,
    initial_branch: String,
) -> Result<ToolResult, ToolError> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        tokio::fs::create_dir_all(&path).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create directory: {}", e)))?;
    }

    let path_str = path.to_str().ok_or_else(|| ToolError::InvalidArguments("Invalid path".to_string()))?;

    let (success, stdout, stderr) = GitExecutor::run_git(
        &["init", "-b", &initial_branch],
        Some(path_str)
    ).await?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({"path": path_str, "initial_branch": initial_branch}),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(stdout.clone()),
        warnings: vec![],
        context: None,
    })
}

/// 处理配置管理
pub async fn handle_config(
    operation: String,
    key: String,
    value: Option<String>,
    global: bool,
) -> Result<ToolResult, ToolError> {
    let (success, stdout, stderr) = match operation.as_str() {
        "get" => GitExecutor::run_git(&["config", &key], None).await,
        "set" => {
            let val = value.ok_or_else(|| ToolError::InvalidArguments("Set requires value".to_string()))?;
            let scope = if global { "--global" } else { "--local" };
            GitExecutor::run_git(&["config", scope, &key, &val], None).await
        }
        _ => return Err(ToolError::InvalidArguments(format!("Unknown config operation: {}", operation))),
    }?;

    Ok(ToolResult {
        success,
        data: serde_json::json!({"operation": operation, "key": key}),
        error: if success { None } else { Some(stderr.clone()) },
        execution_time_ms: 0,
        output: Some(stdout.trim().to_string()),
        warnings: vec![],
        context: None,
    })
}
