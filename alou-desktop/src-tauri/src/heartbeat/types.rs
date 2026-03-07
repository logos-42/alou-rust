/// Heartbeat module type definitions

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Heartbeat configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Whether heartbeat is enabled
    pub enabled: bool,
    /// Heartbeat interval in minutes (30-60 recommended)
    pub interval_minutes: u64,
    /// Model to use for heartbeat tasks
    pub model: String,
    /// Path to the heartbeat file (HEARTBEAT.md)
    pub heartbeat_file_path: String,
    /// Cheap model for routine heartbeats
    #[serde(default = "default_cheap_model")]
    pub cheap_model: String,
    /// Expensive model for complex tasks
    #[serde(default = "default_expensive_model")]
    pub expensive_model: String,
}

fn default_cheap_model() -> String {
    "deepseek-chat".to_string()
}

fn default_expensive_model() -> String {
    "claude-sonnet-4".to_string()
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 30,
            model: "deepseek-chat".to_string(),
            heartbeat_file_path: "~/.alou/HEARTBEAT.md".to_string(),
            cheap_model: "deepseek-chat".to_string(),
            expensive_model: "claude-sonnet-4".to_string(),
        }
    }
}

/// Heartbeat state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatState {
    /// Whether the heartbeat loop is currently running
    pub is_running: bool,
    /// Timestamp of the last heartbeat
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Timestamp of the next scheduled heartbeat
    pub next_heartbeat: Option<DateTime<Utc>>,
    /// Total number of heartbeats executed
    pub total_beats: u64,
}

impl Default for HeartbeatState {
    fn default() -> Self {
        Self {
            is_running: false,
            last_heartbeat: None,
            next_heartbeat: None,
            total_beats: 0,
        }
    }
}

/// Result of a heartbeat execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResult {
    /// Whether the heartbeat was successful
    pub success: bool,
    /// Result message
    pub message: String,
    /// Actions taken during the heartbeat
    pub actions_taken: Vec<String>,
    /// Token usage information
    pub token_used: Option<TokenUsage>,
}

impl HeartbeatResult {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            actions_taken: vec![],
            token_used: None,
        }
    }

    pub fn with_actions(mut self, actions: Vec<String>) -> Self {
        self.actions_taken = actions;
        self
    }

    pub fn with_token_usage(mut self, usage: TokenUsage) -> Self {
        self.token_used = Some(usage);
        self
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub model: String,
}

/// Heartbeat action types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HeartbeatAction {
    /// Quick health check (empty HEARTBEAT.md)
    HealthCheck,
    /// Execute maintenance task
    MaintenanceTask { task: String },
    /// Run diagnostics
    Diagnostics,
    /// Clean up temporary files
    Cleanup,
    /// Sync data
    Sync,
}
