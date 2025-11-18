// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{Manager, State};

// IPFS module - uncomment when ipfs.rs is ready
// mod ipfs;

struct IpfsState {
    process: Option<std::process::Child>,
    data_dir: PathBuf,
}

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))
}

#[tauri::command]
async fn download_kubo_binary(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use std::io::Write;
    
    // Get app data directory for storing Kubo
    let app_data_dir = app_data_dir(&app)?;
    
    let kubo_dir = app_data_dir.join("kubo");
    std::fs::create_dir_all(&kubo_dir)
        .map_err(|e| format!("Failed to create kubo directory: {}", e))?;
    
    let kubo_bin = if cfg!(target_os = "windows") {
        kubo_dir.join("ipfs.exe")
    } else {
        kubo_dir.join("ipfs")
    };

    // Check if already downloaded
    if kubo_bin.exists() {
        return Ok(format!("Kubo binary already exists at: {:?}", kubo_bin));
    }

    // Determine download URL based on platform
    let (url, filename) = if cfg!(target_os = "windows") {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_windows-amd64.zip",
            "kubo.zip"
        )
    } else if cfg!(target_os = "macos") {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_darwin-amd64.tar.gz",
            "kubo.tar.gz"
        )
    } else {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_linux-amd64.tar.gz",
            "kubo.tar.gz"
        )
    };

    // Download Kubo using async request
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download Kubo: {}", e))?;
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let temp_file = kubo_dir.join(filename);
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    drop(file);

    // Extract
    if filename.ends_with(".zip") {
        // Windows: extract zip using PowerShell or zip crate
        let extract_dir = kubo_dir.join("extract");
        std::fs::create_dir_all(&extract_dir)
            .map_err(|e| format!("Failed to create extract dir: {}", e))?;
        
        // Try using PowerShell on Windows
        let output = std::process::Command::new("powershell")
            .args(&[
                "-Command",
                &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", 
                    temp_file.display(), extract_dir.display())
            ])
            .output()
            .map_err(|e| format!("Failed to extract zip: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Failed to extract zip: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Find and copy binary (path may vary)
        let possible_paths = vec![
            extract_dir.join("kubo").join("kubo").join("ipfs.exe"),
            extract_dir.join("kubo").join("ipfs.exe"),
            extract_dir.join("ipfs.exe"),
        ];
        
        let mut found = false;
        for extracted_bin in &possible_paths {
            if extracted_bin.exists() {
                std::fs::copy(&extracted_bin, &kubo_bin)
                    .map_err(|e| format!("Failed to copy binary: {}", e))?;
                found = true;
                break;
            }
        }
        
        if !found {
            return Err("Binary not found in extracted archive".to_string());
        }
        
        // Cleanup
        std::fs::remove_dir_all(&extract_dir).ok();
    } else {
        // Unix: extract tar.gz
        let output = std::process::Command::new("tar")
            .args(&["-xzf", temp_file.to_str().unwrap(), "-C", kubo_dir.to_str().unwrap()])
            .output()
            .map_err(|e| format!("Failed to extract tar.gz: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Failed to extract: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Find and copy binary
        let possible_paths = vec![
            kubo_dir.join("kubo").join("kubo").join("ipfs"),
            kubo_dir.join("kubo").join("ipfs"),
            kubo_dir.join("ipfs"),
        ];
        
        let mut found = false;
        for extracted_bin in &possible_paths {
            if extracted_bin.exists() {
                std::fs::copy(&extracted_bin, &kubo_bin)
                    .map_err(|e| format!("Failed to copy binary: {}", e))?;
                found = true;
                break;
            }
        }
        
        if !found {
            return Err("Binary not found in extracted archive".to_string());
        }
        
        // Cleanup extracted directory
        for path in &possible_paths {
            if let Some(parent) = path.parent() {
                if parent != kubo_dir && parent.exists() {
                    std::fs::remove_dir_all(parent).ok();
                }
            }
        }
    }
    
    // Set executable permission (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&kubo_bin, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    
    // Cleanup temp file
    std::fs::remove_file(&temp_file).ok();
    
    Ok(format!("Kubo binary downloaded to: {:?}", kubo_bin))
}

#[tauri::command]
async fn start_ipfs_node(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let mut ipfs_state = state.lock().await;
    
    // Get app data directory where Kubo is stored (not in bundle)
    let app_data_dir = app_data_dir(&app)?;
    
    let kubo_bin = if cfg!(target_os = "windows") {
        app_data_dir.join("kubo").join("ipfs.exe")
    } else {
        app_data_dir.join("kubo").join("ipfs")
    };

    if !kubo_bin.exists() {
        return Err(format!("Kubo binary not found. Please download it first using download_kubo_binary command. Expected at: {:?}", kubo_bin));
    }

    // Get data directory for IPFS
    let data_dir = app_data_dir.join("ipfs");

    std::fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create IPFS data dir: {}", e))?;

    // Initialize IPFS if not already initialized
    let init_output = Command::new(&kubo_bin)
        .arg("init")
        .arg("--profile=server")
        .env("IPFS_PATH", &data_dir)
        .output()
        .map_err(|e| format!("Failed to initialize IPFS: {}", e))?;

    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        // Ignore error if already initialized
        if !stderr.contains("already") {
            return Err(format!("IPFS init failed: {}", stderr));
        }
    }

    // Start IPFS daemon
    let child = Command::new(&kubo_bin)
        .arg("daemon")
        .env("IPFS_PATH", &data_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start IPFS daemon: {}", e))?;

    ipfs_state.process = Some(child);
    ipfs_state.data_dir = data_dir;

    Ok("IPFS node started".to_string())
}

#[tauri::command]
async fn stop_ipfs_node(
    state: State<'_, tauri::async_runtime::Mutex<IpfsState>>,
) -> Result<String, String> {
    let mut ipfs_state = state.lock().await;
    
    if let Some(mut process) = ipfs_state.process.take() {
        process.kill().map_err(|e| format!("Failed to stop IPFS: {}", e))?;
        Ok("IPFS node stopped".to_string())
    } else {
        Ok("IPFS node was not running".to_string())
    }
}

#[tauri::command]
async fn get_ipfs_info(
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

    let output = Command::new(&kubo_bin)
        .arg("id")
        .env("IPFS_PATH", &ipfs_state.data_dir)
        .output()
        .map_err(|e| format!("Failed to get IPFS info: {}", e))?;

    if !output.status.success() {
        return Err("IPFS node is not running".to_string());
    }

    let info: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse IPFS info: {}", e))?;

    Ok(info)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(tauri::async_runtime::Mutex::new(IpfsState {
            process: None,
            data_dir: PathBuf::new(),
        }))
        .invoke_handler(tauri::generate_handler![
            download_kubo_binary,
            start_ipfs_node,
            stop_ipfs_node,
            get_ipfs_info
        ])
        .setup(|app| {
            // Set window title
            if let Some(window) = app.get_webview_window("main") {
                window.set_title("Alou").unwrap();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
