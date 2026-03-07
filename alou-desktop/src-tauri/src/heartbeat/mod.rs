/// Heartbeat module for Alou Desktop
/// 
/// Provides OpenClaw-style heartbeat mechanism for 7x24 unattended operation.
/// 
/// # Features
/// - Configurable heartbeat interval (30-60 minutes recommended)
/// - HEARTBEAT.md file-based task triggering
/// - Model layering (cheap model for routine heartbeats)
/// - Quick health check response for empty files
/// - Maintenance task execution for files with content

pub mod types;
pub mod config;
pub mod manager;
pub mod commands;

// Re-export commonly used types
pub use types::{HeartbeatConfig, HeartbeatState, HeartbeatResult, TokenUsage};
pub use manager::{HeartbeatManagerState, initialize_heartbeat_manager};

// Re-export commands for Tauri registration
pub use commands::{
    start_heartbeat,
    stop_heartbeat,
    trigger_heartbeat_now,
    get_heartbeat_state,
    get_heartbeat_config,
    update_heartbeat_config,
    initialize_heartbeat,
    is_heartbeat_enabled,
    get_heartbeat_file_content,
    write_heartbeat_file,
    clear_heartbeat_file,
};

/// Module initialization
/// Call this during app setup to initialize the heartbeat system
pub fn init() -> HeartbeatManagerState {
    initialize_heartbeat_manager()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_exports() {
        // Verify all expected types are exported
        let _config = HeartbeatConfig::default();
        let _state = HeartbeatState::default();
    }
}
