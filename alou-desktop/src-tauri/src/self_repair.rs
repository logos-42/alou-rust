//! 自修复系统 — Alou 修改自身代码并验证的完整闭环
//!
//! 提供：
//! - `self_repair_check`: 检测当前项目编译错误
//! - `self_repair_build`: 构建项目（cargo build）
//! - `self_repair_restart`: 重启 Tauri 应用
//! - `self_repair_full_cycle`: 完整修复循环（check → 修 → build → 重启）

use std::path::PathBuf;
use std::process::Stdio;
use serde_json::json;
use tauri::Emitter;

/// 项目根目录检测
fn find_project_root() -> Result<PathBuf, String> {
    let candidates = vec![
        std::env::current_dir().unwrap_or_default(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    ];

    for dir in candidates {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(dir);
        }
        // 检查父目录的 src-tauri
        if dir.file_name().map(|n| n == "src-tauri").unwrap_or(false) {
            let parent = dir.parent().unwrap_or(&dir).to_path_buf();
            if parent.join("src-tauri/Cargo.toml").exists() {
                return Ok(parent.join("src-tauri"));
            }
        }
    }

    Err("无法找到项目根目录 (Cargo.toml)".to_string())
}

/// 检测编译错误
#[tauri::command]
pub async fn self_repair_check() -> Result<serde_json::Value, String> {
    let project_root = find_project_root()?;
    log::info!("[SelfRepair] 检测编译错误: {:?}", project_root);

    let output = tokio::process::Command::new("cargo")
        .args(&["check", "--message-format=json"])
        .current_dir(&project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("执行 cargo check 失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 解析 cargo check 的 JSON 输出，提取错误
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

    Ok(json!({
        "success": !has_errors,
        "has_errors": has_errors,
        "error_count": errors.len(),
        "errors": errors,
        "stderr": stderr.lines().take(20).collect::<Vec<_>>(),
    }))
}

/// 构建项目
#[tauri::command]
pub async fn self_repair_build(
    profile: Option<String>,  // "debug" or "release"
) -> Result<serde_json::Value, String> {
    let project_root = find_project_root()?;
    let profile = profile.unwrap_or_else(|| "debug".to_string());

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
        .map_err(|e| format!("执行 cargo build 失败: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    Ok(json!({
        "success": success,
        "profile": profile,
        "stderr_last_lines": stderr.lines().rev().take(10).collect::<Vec<_>>(),
    }))
}

/// 重启 Tauri 应用
#[tauri::command]
pub async fn self_repair_restart(
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    log::info!("[SelfRepair] 请求重启应用");

    // 通知前端即将重启
    let _ = app_handle.emit("self-repair:restarting", json!({
        "message": "应用即将重启以应用修复...",
        "timestamp": chrono::Utc::now().timestamp(),
    }));

    // 延迟 1 秒后重启，给前端时间显示通知
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        log::info!("[SelfRepair] 正在重启...");
        // 使用 tauri-plugin-shell 的 restart 或者直接重启进程
        // Tauri 2.x 推荐使用 app.restart()
        // 但这里用进程重启更可靠
        let current_exe = std::env::current_exe().unwrap_or_default();
        let _ = std::process::Command::new(current_exe)
            .spawn();
        std::process::exit(0);
    });

    Ok(json!({
        "success": true,
        "message": "应用将在 2 秒后重启",
    }))
}

/// 获取项目结构摘要（给 AI 的上下文）
#[tauri::command]
pub async fn self_repair_project_info() -> Result<serde_json::Value, String> {
    let project_root = find_project_root()?;

    let src_dir = project_root.join("src");
    let mut rust_files = Vec::new();
    let mut ts_files = Vec::new();

    // 扫描 Rust 文件
    if let Ok(entries) = walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        .collect::<Vec<_>>()
    {
        for entry in entries.iter().take(50) {
            let relative = entry.path().strip_prefix(&project_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();
            rust_files.push(relative);
        }
    }

    // 扫描前端 TS/TSX 文件
    let frontend_dir = project_root.parent()
        .map(|p| p.join("src"))
        .unwrap_or_default();
    if frontend_dir.exists() {
        if let Ok(entries) = walkdir::WalkDir::new(&frontend_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "tsx" || ext == "ts").unwrap_or(false))
            .collect::<Vec<_>>()
        {
            for entry in entries.iter().take(50) {
                let relative = entry.path().strip_prefix(project_root.parent().unwrap_or(project_root.as_path()))
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();
                ts_files.push(relative);
            }
        }
    }

    // 读取 git status
    let git_output = tokio::process::Command::new("git")
        .args(&["status", "--short"])
        .current_dir(&project_root)
        .output()
        .await
        .ok();

    let git_status = git_output
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().take(20).map(|l| l.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();

    Ok(json!({
        "project_root": project_root.to_string_lossy(),
        "rust_files_count": rust_files.len(),
        "rust_files": rust_files,
        "frontend_files_count": ts_files.len(),
        "frontend_files": ts_files,
        "git_status": git_status,
    }))
}

/// 完整修复循环: check → (通知 AI 修复) → build → restart
#[tauri::command]
pub async fn self_repair_full_cycle(
    app_handle: tauri::AppHandle,
    auto_restart: Option<bool>,
) -> Result<serde_json::Value, String> {
    let auto_restart = auto_restart.unwrap_or(false);

    log::info!("[SelfRepair] 开始完整修复循环 (auto_restart={})", auto_restart);

    // Step 1: Check
    let check_result = self_repair_check().await?;

    if !check_result.get("has_errors").and_then(|v| v.as_bool()).unwrap_or(false) {
        // 通知前端
        let _ = app_handle.emit("self-repair:cycle-complete", json!({
            "phase": "check",
            "result": "no_errors",
            "message": "项目编译无错误，无需修复",
        }));
        return Ok(json!({
            "success": true,
            "message": "项目编译无错误",
            "check": check_result,
        }));
    }

    // 有错误 — 通知前端当前状态
    let error_count = check_result.get("error_count").and_then(|v| v.as_u64()).unwrap_or(0);
    let _ = app_handle.emit("self-repair:cycle-progress", json!({
        "phase": "errors_detected",
        "error_count": error_count,
        "message": format!("检测到 {} 个编译错误，等待 AI 修复...", error_count),
    }));

    // 返回错误信息，让 AI Agent 在对话中修复
    Ok(json!({
        "success": false,
        "message": format!("检测到 {} 个编译错误，请使用 filesystem 工具修复后调用 self_repair_build", error_count),
        "check": check_result,
        "next_steps": vec![
            "1. 使用 filesystem 工具读取并修复报错的文件",
            "2. 调用 self_repair_check 验证修复",
            "3. 调用 self_repair_build 构建项目",
            if auto_restart { "4. 构建成功后自动重启" } else { "4. 调用 self_repair_restart 重启应用" },
        ],
    }))
}
