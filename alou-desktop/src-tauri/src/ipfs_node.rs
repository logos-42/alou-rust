// IPFS node management module
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{State, Manager};

// Windows 平台特定配置 - 用于隐藏终端窗口
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use crate::ipfs_api::test_ipfs_api_ready;
use crate::kubo::{binary_name, ensure_kubo_binary};
use crate::utils::app_data_dir;

pub struct IpfsState {
    pub process: Option<std::process::Child>,
    pub data_dir: PathBuf,
}

#[tauri::command]
pub async fn start_ipfs_node(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    launch_ipfs_node(state.inner(), &app).await
}

#[tauri::command]
pub async fn stop_ipfs_node(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
) -> Result<String, String> {
    let mut ipfs_state = state.lock().await;

    if let Some(mut process) = ipfs_state.process.take() {
        process
            .kill()
            .map_err(|e| format!("Failed to stop IPFS: {}", e))?;
        Ok("IPFS node stopped".to_string())
    } else {
        Ok("IPFS node was not running".to_string())
    }
}

#[tauri::command]
pub async fn get_ipfs_info(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let ipfs_state = state.lock().await;

    // Get app data directory where Kubo is stored
    let app_data_dir = app_data_dir(&app)?;

    let kubo_bin = if cfg!(target_os = "windows") {
        app_data_dir.join("kubo").join("ipfs.exe")
    } else {
        app_data_dir.join("kubo").join("ipfs")
    };

    if !kubo_bin.exists() {
        return Err("Kubo binary not found. Please download it first.".to_string());
    }

    let data_dir = if ipfs_state.data_dir.as_os_str().is_empty() {
        app_data_dir.join("ipfs")
    } else {
        ipfs_state.data_dir.clone()
    };

    let mut cmd = Command::new(&kubo_bin);
    cmd.arg("id")
        .env("IPFS_PATH", &data_dir);
    
    // Windows: 隐藏终端窗口
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    
    let output = cmd.output()
        .map_err(|e| format!("Failed to get IPFS info: {}", e))?;

    if !output.status.success() {
        return Err("IPFS node is not running".to_string());
    }

    let info: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse IPFS info: {}", e))?;

    Ok(info)
}

/// 检查 IPFS 守护进程状态
#[tauri::command]
pub async fn get_ipfs_daemon_status(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
) -> Result<serde_json::Value, String> {
    let mut ipfs_state = state.lock().await;
    
    let mut status = serde_json::json!({
        "process_exists": false,
        "process_running": false,
        "data_dir": ipfs_state.data_dir.to_string_lossy().to_string()
    });
    
    if let Some(ref mut process) = ipfs_state.process {
        status["process_exists"] = serde_json::json!(true);
        
        match process.try_wait() {
            Ok(None) => {
                status["process_running"] = serde_json::json!(true);
            }
            Ok(Some(exit_status)) => {
                status["process_running"] = serde_json::json!(false);
                status["exit_code"] = serde_json::json!(exit_status.code());
            }
            Err(e) => {
                status["process_running"] = serde_json::json!(false);
                status["error"] = serde_json::json!(format!("无法检查进程状态: {}", e));
            }
        }
    }
    
    Ok(status)
}

pub async fn launch_ipfs_node(
    state: &tauri::async_runtime::Mutex<IpfsState>,
    app: &tauri::AppHandle,
) -> Result<String, String> {
    let mut ipfs_state = state.lock().await;

    if let Some(child) = ipfs_state.process.as_mut() {
        match child.try_wait() {
            Ok(None) => {
                return Ok("IPFS node already running".to_string());
            }
            Ok(Some(_)) => {
                ipfs_state.process = None;
            }
            Err(e) => {
                return Err(format!("Failed to inspect IPFS process: {}", e));
            }
        }
    }

    let app_data_dir = app_data_dir(app)?;
    let kubo_bin = app_data_dir.join("kubo").join(binary_name());
    if !kubo_bin.exists() {
        return Err(format!(
            "Kubo binary not found. Please download it first using download_kubo_binary command. Expected at: {:?}",
            kubo_bin
        ));
    }

    let data_dir = app_data_dir.join("ipfs");
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create IPFS data dir: {}", e))?;

    // 检查端口是否被占用（通过尝试连接 API 端口）
    // 如果 API 可以访问，说明已经有 IPFS 实例在运行
    let api_url = "http://127.0.0.1:5001";
    match test_ipfs_api_ready(api_url).await {
        Ok(true) => {
            // API 可访问，检查是否是我们的进程
            // 如果我们的进程不存在或已退出，说明是其他实例
            let mut is_our_instance = false;
            if let Some(ref mut proc) = ipfs_state.process {
                if let Ok(None) = proc.try_wait() {
                    // 我们的进程还在运行
                    is_our_instance = true;
                }
            }
            
            if !is_our_instance {
                return Err(format!(
                    "检测到另一个 IPFS 实例正在运行，端口 5001 已被占用。\n\
                    这可能是 IPFS Desktop 或其他 IPFS 守护进程。\n\
                    \n\
                    解决方案：\n\
                    1. 关闭其他 IPFS 应用程序（如 IPFS Desktop）\n\
                    2. 或者停止其他 IPFS 守护进程（运行 'ipfs shutdown' 或关闭 IPFS Desktop）\n\
                    3. 然后重试启动 IPFS 节点\n\
                    \n\
                    注意：Alou 可以使用已运行的 IPFS 实例，但建议使用 Alou 自己的实例以避免冲突。"
                ));
            }
        }
        Ok(false) | Err(_) => {
            // API 不可访问，端口可能空闲，或者节点正在启动中
            // 继续启动流程
        }
    }

    // Initialize IPFS if not already initialized
    let repo_config = data_dir.join("config");
    if !repo_config.exists() {
        let mut init_cmd = Command::new(&kubo_bin);
        init_cmd.arg("init")
            .arg("--profile=server")
            .env("IPFS_PATH", &data_dir);
        
        // Windows: 隐藏初始化命令的终端窗口
        #[cfg(target_os = "windows")]
        init_cmd.creation_flags(CREATE_NO_WINDOW);
        
        let init_output = init_cmd.output()
            .map_err(|e| format!("Failed to initialize IPFS: {}", e))?;

        if !init_output.status.success() {
            let stderr = String::from_utf8_lossy(&init_output.stderr);
            if !stderr.contains("already") {
                return Err(format!("IPFS init failed: {}", stderr));
            }
        }
    }

    // Start IPFS daemon
    let mut cmd = Command::new(&kubo_bin);
    cmd.arg("daemon")
        .env("IPFS_PATH", &data_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // Windows: 隐藏终端窗口，使 Kubo 完全在后台运行
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    
    let child = cmd.spawn()
        .map_err(|e| format!("Failed to start IPFS daemon: {}", e))?;

    ipfs_state.process = Some(child);
    ipfs_state.data_dir = data_dir;

    // 等待一小段时间，让守护进程开始初始化
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    Ok("IPFS node started".to_string())
}

pub async fn bootstrap_ipfs(app_handle: tauri::AppHandle) {
    if let Err(e) = ensure_kubo_binary(&app_handle).await {
        eprintln!("Failed to ensure Kubo binary: {}", e);
        return;
    }

    let state = app_handle.state::<tauri::async_runtime::Mutex<IpfsState>>();
    if let Err(e) = launch_ipfs_node(state.inner(), &app_handle).await {
        eprintln!("Failed to auto-start IPFS: {}", e);
    }
}

