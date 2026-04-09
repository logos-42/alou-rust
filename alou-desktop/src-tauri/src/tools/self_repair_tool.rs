//! 自修复工具 — 让 AI Agent 可以检测、修复、构建和重启自身
//!
//! 实现 ToolExecutor trait，注册到 ToolBridge 的标准工具路径

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;

/// 自修复工具
pub struct SelfRepairTool {
    metadata: ToolMetadata,
}

impl SelfRepairTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "self_repair".to_string(),
                name: "Self Repair Tool".to_string(),
                description: "自修复系统：检测编译错误、构建项目、重启应用。让 Alou 修改并验证自身代码".to_string(),
                category: ToolCategory::Development,
                priority: ToolPriority::Critical,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["macos".to_string(), "linux".to_string(), "windows".to_string()],
                permissions: vec!["execute".to_string(), "filesystem".to_string()],
                tags: vec!["self-repair".to_string(), "build".to_string(), "restart".to_string()],
            },
        }
    }

    /// 查找项目根目录
    fn find_project_root() -> Result<PathBuf, ToolError> {
        let candidates = vec![
            std::env::current_dir().unwrap_or_default(),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        ];

        for dir in candidates {
            if dir.join("Cargo.toml").exists() {
                return Ok(dir);
            }
            if dir.file_name().map(|n| n == "src-tauri").unwrap_or(false) {
                if let Some(parent) = dir.parent() {
                    let tauri_dir = parent.join("src-tauri");
                    if tauri_dir.join("Cargo.toml").exists() {
                        return Ok(tauri_dir);
                    }
                }
            }
        }

        Err(ToolError::ExecutionFailed("无法找到项目根目录 (Cargo.toml)".to_string()))
    }

    /// 检测编译错误
    async fn do_check(&self) -> Result<ToolResult, ToolError> {
        let project_root = Self::find_project_root()?;
        log::info!("[SelfRepair] 检测编译错误: {:?}", project_root);

        let output = tokio::process::Command::new("cargo")
            .args(&["check", "--message-format=json"])
            .current_dir(&project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("执行 cargo check 失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut errors = Vec::new();
        for line in stdout.lines() {
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
                if msg.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(message) = msg.get("message") {
                        let level = message.get("level").and_then(|l| l.as_str()).unwrap_or("");
                        if level == "error" {
                            errors.push(json!({
                                "message": message.get("message").and_then(|m| m.as_str()).unwrap_or(""),
                                "code": message.get("code").and_then(|c| c.get("code")).and_then(|c| c.as_str()).unwrap_or(""),
                                "spans": message.get("spans").map(|spans| {
                                    spans.as_array().unwrap_or(&vec![]).iter().map(|s| json!({
                                        "file": s.get("file_name").and_then(|f| f.as_str()).unwrap_or(""),
                                        "line_start": s.get("line_start"),
                                        "line_end": s.get("line_end"),
                                        "text": s.get("text").and_then(|t| t.get("text")).and_then(|t| t.as_str()).unwrap_or(""),
                                    })).collect::<Vec<_>>()
                                }).unwrap_or(vec![]),
                            }));
                        }
                    }
                }
            }
        }

        let has_errors = !errors.is_empty();

        Ok(ToolResult::success(json!({
            "has_errors": has_errors,
            "error_count": errors.len(),
            "errors": errors,
            "stderr_preview": stderr.lines().take(20).collect::<Vec<_>>(),
        })))
    }

    /// 构建项目
    async fn do_build(&self, profile: &str) -> Result<ToolResult, ToolError> {
        let project_root = Self::find_project_root()?;
        log::info!("[SelfRepair] 构建项目: {:?} (profile={})", project_root, profile);

        let args = if profile == "release" {
            vec!["build", "--release"]
        } else {
            vec!["build"]
        };

        let output = tokio::process::Command::new("cargo")
            .args(&args)
            .current_dir(&project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("执行 cargo build 失败: {}", e)))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();

        Ok(ToolResult::success(json!({
            "success": success,
            "profile": profile,
            "output_tail": stderr.lines().rev().take(10).collect::<Vec<_>>(),
        })))
    }

    /// 请求重启
    async fn do_restart(&self) -> Result<ToolResult, ToolError> {
        log::info!("[SelfRepair] 请求重启应用");

        Ok(ToolResult::success(json!({
            "action": "restart_requested",
            "message": "应用需要重启以加载修复后的代码。请在前端触发重启，或调用 Tauri command self_repair_restart",
            "note": "Rust 代码修改必须 rebuild + restart 才能生效",
        })))
    }

    /// 项目信息
    async fn do_project_info(&self) -> Result<ToolResult, ToolError> {
        let project_root = Self::find_project_root()?;
        let src_dir = project_root.join("src");

        let mut rust_files = Vec::new();
        for entry in walkdir::WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map(|ext| ext == "rs").unwrap_or(false) {
                let relative = entry.path().strip_prefix(&project_root)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();
                rust_files.push(relative);
                if rust_files.len() >= 50 { break; }
            }
        }

        let frontend_dir = project_root.parent()
            .map(|p| p.join("src"))
            .unwrap_or_default();
        let mut ts_files = Vec::new();
        if frontend_dir.exists() {
            for entry in walkdir::WalkDir::new(&frontend_dir).into_iter().filter_map(|e| e.ok()) {
                let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "tsx" || ext == "ts" {
                    let relative = entry.path().strip_prefix(project_root.parent().unwrap_or(project_root.as_path()))
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .to_string();
                    ts_files.push(relative);
                    if ts_files.len() >= 50 { break; }
                }
            }
        }

        let git_output = tokio::process::Command::new("git")
            .args(&["status", "--short"])
            .current_dir(&project_root)
            .output()
            .await
            .ok();

        let git_status = git_output
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().take(20).map(|l| l.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();

        Ok(ToolResult::success(json!({
            "project_root": project_root.to_string_lossy(),
            "rust_files": rust_files,
            "frontend_files": ts_files,
            "git_status": git_status,
        })))
    }

    /// 完整修复循环
    async fn do_full_cycle(&self, auto_restart: bool) -> Result<ToolResult, ToolError> {
        log::info!("[SelfRepair] 完整修复循环 (auto_restart={})", auto_restart);

        let check_result = self.do_check().await?;

        let has_errors = check_result.data.get("has_errors")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !has_errors {
            return Ok(ToolResult::success(json!({
                "phase": "check",
                "result": "no_errors",
                "message": "项目编译无错误，无需修复",
            })));
        }

        let error_count = check_result.data.get("error_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(ToolResult::success(json!({
            "phase": "errors_detected",
            "error_count": error_count,
            "message": format!("检测到 {} 个编译错误，请使用 filesystem 工具修复后调用 self_repair (action=check) 验证", error_count),
            "errors": check_result.data.get("errors"),
            "next_steps": vec![
                "1. 使用 filesystem (operation=read) 读取出错的文件",
                "2. 使用 filesystem (operation=edit) 修复代码",
                "3. 调用 self_repair (action=check) 验证修复",
                "4. 调用 self_repair (action=build) 构建项目",
                if auto_restart { "5. 构建成功后自动重启" } else { "5. 调用 self_repair (action=restart) 重启" },
            ],
        })))
    }
}

#[async_trait]
impl ToolExecutor for SelfRepairTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match action {
            "check" => self.do_check().await,
            "build" => {
                let profile = args.get("profile").and_then(|v| v.as_str()).unwrap_or("debug");
                self.do_build(profile).await
            }
            "restart" => self.do_restart().await,
            "project_info" => self.do_project_info().await,
            "full_cycle" => {
                let auto_restart = args.get("auto_restart").and_then(|v| v.as_bool()).unwrap_or(false);
                self.do_full_cycle(auto_restart).await
            }
            _ => Err(ToolError::InvalidArguments(format!(
                "未知自修复操作: '{}'. 可用: check, build, restart, project_info, full_cycle", action
            ))),
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Args must be an object".to_string()));
        }

        let action = args.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if action.is_empty() {
            return Err(ToolError::InvalidArguments("action 字段不能为空".to_string()));
        }

        if !matches!(action, "check" | "build" | "restart" | "project_info" | "full_cycle") {
            return Err(ToolError::InvalidArguments(format!(
                "未知操作: '{}'. 可用: check, build, restart, project_info, full_cycle", action
            )));
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Self Repair Tool - Alou 自修复系统

让 AI Agent 可以检测、修复、构建和重启自身代码的完整闭环。

操作类型:
  check        - 检测编译错误 (cargo check --message-format=json)
  build        - 构建项目 (cargo build)
  restart      - 请求重启应用
  project_info - 获取项目结构 (Rust/TS 文件列表 + git status)
  full_cycle   - 完整修复循环 (check → 修复 → build → restart)

自修复循环步骤:
  1. self_repair (action=check)        → 获取编译错误
  2. filesystem (operation=read)       → 读取出错文件
  3. filesystem (operation=edit)       → 修复代码
  4. self_repair (action=check)        → 验证修复
  5. self_repair (action=build)        → 构建项目
  6. self_repair (action=restart)      → 重启应用

示例:
  {"action": "check"}
  {"action": "build", "profile": "debug"}
  {"action": "project_info"}
  {"action": "full_cycle", "auto_restart": false}
"#.to_string()
    }
}
