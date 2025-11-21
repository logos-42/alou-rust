// Kubo binary management module
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::utils::app_data_dir;

pub const KUBO_VERSION: &str = "v0.24.0";
const KUBO_VERSION_FILE: &str = "version.json";

#[derive(Serialize, Deserialize)]
pub struct KuboVersionInfo {
    pub version: String,
    pub source: String,
}

pub fn record_kubo_version(dir: &Path, source: &str) -> Result<(), String> {
    let info = KuboVersionInfo {
        version: KUBO_VERSION.to_string(),
        source: source.to_string(),
    };
    let file = dir.join(KUBO_VERSION_FILE);
    let json = serde_json::to_string_pretty(&info)
        .map_err(|e| format!("Failed to serialize version info: {}", e))?;
    fs::write(&file, json).map_err(|e| format!("Failed to write version info: {}", e))
}

pub fn read_kubo_version(dir: &Path) -> Option<KuboVersionInfo> {
    let file = dir.join(KUBO_VERSION_FILE);
    let data = fs::read_to_string(file).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "ipfs.exe"
    } else {
        "ipfs"
    }
}

pub fn bundled_kubo_binary(app: &AppHandle) -> Option<PathBuf> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidate = resource_dir.join("kubo").join(binary_name());
        if candidate.exists() {
            return Some(candidate);
        }
    }
    #[cfg(debug_assertions)]
    {
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("kubo")
            .join(binary_name());
        if dev_path.exists() {
            return Some(dev_path);
        }
    }
    None
}

#[tauri::command]
pub async fn download_kubo_binary(app: AppHandle) -> Result<String, String> {
    // Get app data directory for storing Kubo
    let app_data_dir = app_data_dir(&app)?;

    let kubo_dir = app_data_dir.join("kubo");
    std::fs::create_dir_all(&kubo_dir)
        .map_err(|e| format!("Failed to create kubo directory: {}", e))?;

    let kubo_bin = kubo_dir.join(binary_name());

    // Check if already downloaded
    if kubo_bin.exists() {
        record_kubo_version(&kubo_dir, "existing")?;
        return Ok(format!("Kubo binary already exists at: {:?}", kubo_bin));
    }

    // Determine download URL based on platform
    let (url, filename) = if cfg!(target_os = "windows") {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_windows-amd64.zip",
            "kubo.zip",
        )
    } else if cfg!(target_os = "macos") {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_darwin-amd64.tar.gz",
            "kubo.tar.gz",
        )
    } else {
        (
            "https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_linux-amd64.tar.gz",
            "kubo.tar.gz",
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
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    temp_file.display(),
                    extract_dir.display()
                ),
            ])
            .output()
            .map_err(|e| format!("Failed to extract zip: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to extract zip: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
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
            .args(&[
                "-xzf",
                temp_file.to_str().unwrap(),
                "-C",
                kubo_dir.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to extract tar.gz: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to extract: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
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

    record_kubo_version(&kubo_dir, "download")?;

    Ok(format!("Kubo binary downloaded to: {:?}", kubo_bin))
}

pub async fn ensure_kubo_binary(app: &AppHandle) -> Result<(), String> {
    let app_data_dir = app_data_dir(app)?;
    let kubo_dir = app_data_dir.join("kubo");
    std::fs::create_dir_all(&kubo_dir).map_err(|e| format!("Failed to create kubo dir: {}", e))?;

    let target_bin = kubo_dir.join(binary_name());

    if target_bin.exists() {
        if let Some(info) = read_kubo_version(&kubo_dir) {
            if info.version == KUBO_VERSION {
                return Ok(());
            }
        }
    }

    if let Some(bundled) = bundled_kubo_binary(app) {
        std::fs::copy(&bundled, &target_bin)
            .map_err(|e| format!("Failed to copy bundled Kubo: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target_bin, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }
        record_kubo_version(&kubo_dir, "bundled")?;
        return Ok(());
    }

    download_kubo_binary(app.clone()).await.map(|_| ())
}

