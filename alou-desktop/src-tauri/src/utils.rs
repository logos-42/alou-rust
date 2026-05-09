use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Get application data directory
pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))
}

/// Normalize base URL by removing trailing slashes
pub fn normalize_base_url(url: &str) -> String {
    url.trim_end_matches('/').to_string()
}

/// Default IPFS API URL
pub fn default_ipfs_api_url() -> String {
    "http://127.0.0.1:5001".to_string()
}

/// Default IPFS Gateway URL
pub fn default_ipfs_gateway_url() -> String {
    "http://127.0.0.1:8080".to_string()
}

