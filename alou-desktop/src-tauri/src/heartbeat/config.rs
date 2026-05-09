/// Heartbeat configuration management

use std::path::PathBuf;
use std::fs;
use serde_json;
use crate::heartbeat::types::HeartbeatConfig;

/// Get the Alou config directory path
pub fn get_alou_config_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Failed to get home directory".to_string())?;
    
    let config_dir = home_dir.join(".alou");
    
    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&config_dir) {
        return Err(format!("Failed to create config directory: {}", e));
    }
    
    Ok(config_dir)
}

/// Get the heartbeat config file path
pub fn get_heartbeat_config_path() -> Result<PathBuf, String> {
    let config_dir = get_alou_config_dir()?;
    Ok(config_dir.join("heartbeat.json"))
}

/// Get the default heartbeat file path
pub fn get_default_heartbeat_file_path() -> Result<PathBuf, String> {
    let config_dir = get_alou_config_dir()?;
    Ok(config_dir.join("HEARTBEAT.md"))
}

/// Load heartbeat configuration from file
pub fn load_config() -> Result<HeartbeatConfig, String> {
    let config_path = get_heartbeat_config_path()?;
    
    if !config_path.exists() {
        // Create default config if file doesn't exist
        let default_config = HeartbeatConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }
    
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let config: HeartbeatConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;
    
    Ok(config)
}

/// Save heartbeat configuration to file
pub fn save_config(config: &HeartbeatConfig) -> Result<(), String> {
    let config_path = get_heartbeat_config_path()?;
    
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

/// Update specific configuration fields
pub fn update_config(updates: ConfigUpdates) -> Result<HeartbeatConfig, String> {
    let mut config = load_config()?;
    
    if let Some(enabled) = updates.enabled {
        config.enabled = enabled;
    }
    
    if let Some(interval_minutes) = updates.interval_minutes {
        config.interval_minutes = interval_minutes;
    }
    
    if let Some(model) = updates.model {
        config.model = model;
    }
    
    if let Some(heartbeat_file_path) = updates.heartbeat_file_path {
        config.heartbeat_file_path = heartbeat_file_path;
    }
    
    if let Some(cheap_model) = updates.cheap_model {
        config.cheap_model = cheap_model;
    }
    
    if let Some(expensive_model) = updates.expensive_model {
        config.expensive_model = expensive_model;
    }
    
    save_config(&config)?;
    
    Ok(config)
}

/// Configuration update parameters
#[derive(Debug, Clone, Default)]
pub struct ConfigUpdates {
    pub enabled: Option<bool>,
    pub interval_minutes: Option<u64>,
    pub model: Option<String>,
    pub heartbeat_file_path: Option<String>,
    pub cheap_model: Option<String>,
    pub expensive_model: Option<String>,
}

/// Expand tilde in path to home directory
pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Some(home_dir) = dirs::home_dir() {
            let home_str = home_dir.to_string_lossy();
            return path.replacen("~", &home_str, 1);
        }
    }
    path.to_string()
}

/// Get the absolute path for the heartbeat file
pub fn get_heartbeat_file_path(config: &HeartbeatConfig) -> Result<PathBuf, String> {
    let expanded = expand_tilde(&config.heartbeat_file_path);
    Ok(PathBuf::from(expanded))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expand_tilde() {
        let result = expand_tilde("~/.alou/HEARTBEAT.md");
        assert!(result.ends_with(".alou/HEARTBEAT.md"));
        assert!(result.starts_with('/'));
    }
    
    #[test]
    fn test_default_config() {
        let config = HeartbeatConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_minutes, 30);
        assert_eq!(config.model, "deepseek-chat");
    }
}
