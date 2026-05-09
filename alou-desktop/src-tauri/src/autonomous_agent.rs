//! Autonomous Agent State Management
//!
//! 自主智能体状态管理 - 仅包含状态存取函数

use std::path::PathBuf;
use std::fs;

/// 获取存储路径
fn get_storage_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let path = home.join(".alou").join("autonomous");
    
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

/// 获取智能体状态
#[tauri::command]
pub async fn get_autonomous_agent_state() -> Result<serde_json::Value, String> {
    let storage_path = get_storage_path().map_err(|e| e.to_string())?;
    let state_path = storage_path.join("state.json");
    
    if state_path.exists() {
        let content = fs::read_to_string(&state_path).map_err(|e| e.to_string())?;
        let value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(value)
    } else {
        Ok(serde_json::json!({
            "is_running": false,
            "current_task_id": null,
            "last_heartbeat": chrono::Utc::now().timestamp(),
            "tasks_completed": 0,
            "tasks_failed": 0,
            "errors": [],
            "config": {
                "heartbeat_interval_seconds": 30,
                "max_idle_seconds": 300,
                "auto_restart": true
            }
        }))
    }
}

/// 保存智能体状态
#[tauri::command]
pub async fn save_autonomous_agent_state(state: serde_json::Value) -> Result<bool, String> {
    let storage_path = get_storage_path().map_err(|e| e.to_string())?;
    let state_path = storage_path.join("state.json");
    
    let content = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    fs::write(&state_path, content).map_err(|e| e.to_string())?;
    
    Ok(true)
}
