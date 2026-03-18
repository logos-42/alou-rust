/// Tauri commands for heartbeat management

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::heartbeat::manager::{HeartbeatManager, HeartbeatManagerState};
use crate::heartbeat::types::{HeartbeatConfig, HeartbeatState, HeartbeatResult};
use crate::heartbeat::config;

/// Start the heartbeat loop
#[tauri::command]
pub async fn start_heartbeat(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<serde_json::Value, String> {
    println!("[Heartbeat Command] Starting heartbeat...");
    
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    mgr.start().await?;
    
    Ok(serde_json::json!({
        "success": true,
        "message": "Heartbeat started successfully"
    }))
}

/// Stop the heartbeat loop
#[tauri::command]
pub async fn stop_heartbeat(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<serde_json::Value, String> {
    println!("[Heartbeat Command] Stopping heartbeat...");
    
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    mgr.stop().await?;
    
    Ok(serde_json::json!({
        "success": true,
        "message": "Heartbeat stopped successfully"
    }))
}

/// Trigger a heartbeat immediately
#[tauri::command]
pub async fn trigger_heartbeat_now(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<HeartbeatResult, String> {
    println!("[Heartbeat Command] Triggering heartbeat now...");
    
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    let result = mgr.trigger_heartbeat().await?;
    
    Ok(result)
}

/// Get current heartbeat state
#[tauri::command]
pub async fn get_heartbeat_state(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<HeartbeatState, String> {
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    Ok(mgr.get_state().await)
}

/// Get current heartbeat configuration
#[tauri::command]
pub async fn get_heartbeat_config(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<HeartbeatConfig, String> {
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    Ok(mgr.get_config().await)
}

/// Configuration update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHeartbeatConfigRequest {
    pub enabled: Option<bool>,
    pub interval_minutes: Option<u64>,
    pub model: Option<String>,
    pub heartbeat_file_path: Option<String>,
    pub cheap_model: Option<String>,
    pub expensive_model: Option<String>,
}

/// Update heartbeat configuration
#[tauri::command]
pub async fn update_heartbeat_config(
    manager_state: State<'_, HeartbeatManagerState>,
    request: UpdateHeartbeatConfigRequest,
) -> Result<HeartbeatConfig, String> {
    println!("[Heartbeat Command] Updating configuration...");
    
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    // Load current config
    let mut config = mgr.get_config().await;
    
    // Apply updates
    if let Some(enabled) = request.enabled {
        config.enabled = enabled;
    }
    
    if let Some(interval_minutes) = request.interval_minutes {
        config.interval_minutes = interval_minutes;
    }
    
    if let Some(model) = request.model {
        config.model = model;
    }
    
    if let Some(heartbeat_file_path) = request.heartbeat_file_path {
        config.heartbeat_file_path = heartbeat_file_path;
    }
    
    if let Some(cheap_model) = request.cheap_model {
        config.cheap_model = cheap_model;
    }
    
    if let Some(expensive_model) = request.expensive_model {
        config.expensive_model = expensive_model;
    }
    
    // Save updated config
    mgr.update_config(config.clone()).await?;
    
    Ok(config)
}

/// Initialize heartbeat system (called during app setup)
#[tauri::command]
pub async fn initialize_heartbeat(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<serde_json::Value, String> {
    println!("[Heartbeat Command] Initializing heartbeat system...");

    // Check if already initialized
    let manager = manager_state.inner().lock().await;
    if manager.is_some() {
        return Ok(serde_json::json!({
            "success": true,
            "message": "Heartbeat system already initialized"
        }));
    }

    drop(manager);

    // Create new manager
    let mut manager = manager_state.inner().lock().await;
    let mgr = HeartbeatManager::new()?;
    
    // 🔥 自动创建文档文件（如果不存在）
    mgr.ensure_documentation_files().await;
    
    *manager = Some(mgr);

    Ok(serde_json::json!({
        "success": true,
        "message": "Heartbeat system initialized successfully"
    }))
}

/// Check if heartbeat is enabled in configuration
#[tauri::command]
pub async fn is_heartbeat_enabled(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<serde_json::Value, String> {
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    let config = mgr.get_config().await;
    
    Ok(serde_json::json!({
        "enabled": config.enabled,
        "interval_minutes": config.interval_minutes,
        "model": config.model
    }))
}

/// Get heartbeat file content
#[tauri::command]
pub async fn get_heartbeat_file_content(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<serde_json::Value, String> {
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    let config = mgr.get_config().await;
    let file_path = config::expand_tilde(&config.heartbeat_file_path);
    
    let content = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| String::new());
    
    Ok(serde_json::json!({
        "path": file_path,
        "content": content,
        "is_empty": content.trim().is_empty()
    }))
}

/// Write content to heartbeat file
#[tauri::command]
pub async fn write_heartbeat_file(
    manager_state: State<'_, HeartbeatManagerState>,
    content: String,
) -> Result<serde_json::Value, String> {
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    let config = mgr.get_config().await;
    let file_path = config::expand_tilde(&config.heartbeat_file_path);
    
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    std::fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(serde_json::json!({
        "success": true,
        "path": file_path,
        "bytes_written": content.len()
    }))
}

/// Clear heartbeat file (for quick health check)
#[tauri::command]
pub async fn clear_heartbeat_file(
    manager_state: State<'_, HeartbeatManagerState>,
) -> Result<serde_json::Value, String> {
    let manager = manager_state.inner().lock().await;
    
    let mgr = manager.as_ref()
        .ok_or_else(|| "Heartbeat manager not initialized".to_string())?;
    
    let config = mgr.get_config().await;
    let file_path = config::expand_tilde(&config.heartbeat_file_path);
    
    std::fs::write(&file_path, "")
        .map_err(|e| format!("Failed to clear file: {}", e))?;
    
    Ok(serde_json::json!({
        "success": true,
        "path": file_path,
        "message": "Heartbeat file cleared"
    }))
}
