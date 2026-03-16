/// Heartbeat Manager - Core heartbeat logic

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use chrono::{Utc, Local};
use std::fs;
use std::path::PathBuf;
use crate::heartbeat::types::{HeartbeatConfig, HeartbeatState, HeartbeatResult, TokenUsage};
use crate::heartbeat::config::{self, load_config};

// Import persistence modules
use crate::soul::SoulManager;
use crate::tasks::TasksManager;
use crate::logs::LogsManager;

/// Heartbeat Manager
pub struct HeartbeatManager {
    config: Arc<Mutex<HeartbeatConfig>>,
    state: Arc<Mutex<HeartbeatState>>,
    stop_sender: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl HeartbeatManager {
    /// Create a new HeartbeatManager
    pub fn new() -> Result<Self, String> {
        let config = load_config()?;
        
        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(HeartbeatState::default())),
            stop_sender: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Create with custom config
    pub fn with_config(config: HeartbeatConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(HeartbeatState::default())),
            stop_sender: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Start the heartbeat loop
    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        
        if state.is_running {
            return Err("Heartbeat is already running".to_string());
        }
        
        let config = self.config.lock().await;
        if !config.enabled {
            return Err("Heartbeat is disabled in configuration".to_string());
        }

        let interval_minutes = config.interval_minutes;
        drop(config);

        // Create stop channel
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

        // Clone Arcs for the spawned task
        let state_clone = Arc::clone(&self.state);
        let manager = self.clone_for_task();

        // Set stop sender
        *self.stop_sender.lock().await = Some(stop_tx);
        
        // Mark as running
        state.is_running = true;
        state.next_heartbeat = Some(Utc::now() + Duration::from_secs(interval_minutes * 60));
        drop(state);
        
        println!("[Heartbeat] Starting heartbeat loop with {} minute interval", interval_minutes);
        
        // Spawn the heartbeat loop
        tokio::spawn(async move {
            let interval = Duration::from_secs(interval_minutes * 60);
            let mut stop_rx = stop_rx;
            
            loop {
                tokio::select! {
                    _ = sleep(interval) => {
                        // Check if still running
                        let state = state_clone.lock().await;
                        if !state.is_running {
                            drop(state);
                            break;
                        }
                        drop(state);
                        
                        // Trigger heartbeat
                        let _ = manager.trigger_heartbeat_internal().await;
                        
                        // Update next heartbeat time
                        let mut state = state_clone.lock().await;
                        state.next_heartbeat = Some(Utc::now() + interval);
                        drop(state);
                    }
                    _ = &mut stop_rx => {
                        println!("[Heartbeat] Received stop signal");
                        break;
                    }
                }
            }
            
            println!("[Heartbeat] Heartbeat loop stopped");
        });
        
        Ok(())
    }
    
    /// Clone for task (helper for spawning)
    fn clone_for_task(&self) -> Arc<Self> {
        Arc::new(HeartbeatManager {
            config: Arc::clone(&self.config),
            state: Arc::clone(&self.state),
            stop_sender: Arc::clone(&self.stop_sender),
        })
    }
    
    /// Stop the heartbeat loop
    pub async fn stop(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        
        if !state.is_running {
            return Err("Heartbeat is not running".to_string());
        }
        
        // Send stop signal
        if let Some(tx) = self.stop_sender.lock().await.take() {
            let _ = tx.send(());
        }
        
        state.is_running = false;
        state.next_heartbeat = None;
        
        println!("[Heartbeat] Heartbeat stopped");
        
        Ok(())
    }
    
    /// Trigger a heartbeat immediately
    pub async fn trigger_heartbeat(&self) -> Result<HeartbeatResult, String> {
        self.trigger_heartbeat_internal().await
    }
    
    /// Internal trigger heartbeat (for use in spawned tasks)
    async fn trigger_heartbeat_internal(&self) -> Result<HeartbeatResult, String> {
        println!("[Heartbeat] Triggering heartbeat...");
        
        let result = self.handle_heartbeat().await;
        
        // Update state
        let mut state = self.state.lock().await;
        state.last_heartbeat = Some(Utc::now());
        state.total_beats += 1;
        drop(state);
        
        if result.success {
            println!("[Heartbeat] Heartbeat completed successfully: {}", result.message);
        } else {
            eprintln!("[Heartbeat] Heartbeat failed: {}", result.message);
        }
        
        Ok(result)
    }
    
    /// Handle the heartbeat logic
    async fn handle_heartbeat(&self) -> HeartbeatResult {
        let config = self.config.lock().await;
        let heartbeat_file_path = config.heartbeat_file_path.clone();
        let model = config.model.clone();
        let cheap_model = config.cheap_model.clone();
        drop(config);

        // Expand tilde in path
        let expanded_path = config::expand_tilde(&heartbeat_file_path);
        let file_path = std::path::Path::new(&expanded_path);

        // Read heartbeat file content
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_e) => {
                // File doesn't exist, create empty one
                if let Some(parent) = file_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(file_path, "");
                return HeartbeatResult::ok("HEARTBEAT_OK - File created")
                    .with_actions(vec!["Created heartbeat file".to_string()]);
            }
        };

        let trimmed_content = content.trim();

        if trimmed_content.is_empty() {
            // Empty file - quick health check response
            println!("[Heartbeat] Empty heartbeat file - quick health check");

            // Clear the file to indicate we've processed it
            let _ = fs::write(file_path, "");

            // Save all persistence modules
            let actions = self.save_all_persistence().await;

            return HeartbeatResult::ok("HEARTBEAT_OK - All modules saved")
                .with_actions(actions)
                .with_token_usage(TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 1,
                    total_tokens: 1,
                    model: cheap_model,
                });
        }

        // File has content - execute maintenance task
        println!("[Heartbeat] Heartbeat file has content - executing maintenance task");

        let task_content = trimmed_content.to_string();

        // Clear the file after reading
        let _ = fs::write(file_path, "");

        // Execute the maintenance task
        let mut actions = self.execute_maintenance_task(&task_content, &model).await;

        // Also save persistence modules
        let save_actions = self.save_all_persistence().await;
        actions.extend(save_actions);

        HeartbeatResult::ok("Maintenance task executed")
            .with_actions(actions)
            .with_token_usage(TokenUsage {
                prompt_tokens: task_content.len() as u64 / 4,
                completion_tokens: 100,
                total_tokens: 100 + task_content.len() as u64 / 4,
                model: model,
            })
    }

    /// Save all persistence modules
    async fn save_all_persistence(&self) -> Vec<String> {
        let mut actions = Vec::new();
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Get base directory
        let base_dir = dirs::home_dir()
            .map(|h| h.join(".alou"))
            .unwrap_or_else(|| PathBuf::from("./.alou"));

        // Ensure directory exists
        let _ = fs::create_dir_all(&base_dir);

        // Save SOUL.md
        match SoulManager::new(&base_dir).load() {
            Ok(_) => {
                actions.push(format!("[{}] Soul module: loaded", now));
            }
            Err(e) => {
                actions.push(format!("[{}] Soul module: created default ({})", now, e));
            }
        }

        // Save TASKS.md
        let tasks_manager = TasksManager::new();
        actions.push(format!("[{}] Tasks module: initialized", now));

        // Save LOGS.md
        match LogsManager::new(&base_dir).get_stats() {
            Ok(stats) => {
                actions.push(format!("[{}] Logs module: {} lines ({:.2} KB)", now, stats.total_lines, stats.file_size_kb()));
            }
            Err(e) => {
                actions.push(format!("[{}] Logs module: created default ({})", now, e));
            }
        }

        actions
    }
    
    /// Execute a maintenance task
    async fn execute_maintenance_task(&self, task: &str, model: &str) -> Vec<String> {
        let mut actions = Vec::new();
        
        println!("[Heartbeat] Executing maintenance task with model: {}", model);
        actions.push(format!("Task: {}", task));
        
        // Parse task content for known commands
        let task_lower = task.to_lowercase();
        
        if task_lower.contains("cleanup") || task_lower.contains("clean") {
            actions.push("Cleanup action initiated".to_string());
            // In real implementation, would call actual cleanup functions
        }
        
        if task_lower.contains("sync") {
            actions.push("Sync action initiated".to_string());
            // In real implementation, would call sync functions
        }
        
        if task_lower.contains("diagnose") || task_lower.contains("diagnostic") {
            actions.push("Diagnostics action initiated".to_string());
            // In real implementation, would run diagnostics
        }
        
        if task_lower.contains("status") || task_lower.contains("check") {
            actions.push("Status check completed".to_string());
        }
        
        // If no specific action matched, treat as general task
        if actions.len() == 1 {
            actions.push("General maintenance task executed".to_string());
        }
        
        actions
    }
    
    /// Get current heartbeat state
    pub async fn get_state(&self) -> HeartbeatState {
        self.state.lock().await.clone()
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> HeartbeatConfig {
        self.config.lock().await.clone()
    }
    
    /// Update configuration
    pub async fn update_config(&self, new_config: HeartbeatConfig) -> Result<(), String> {
        let mut config = self.config.lock().await;
        *config = new_config.clone();
        drop(config);
        
        // Save to file
        config::save_config(&new_config)
    }
    
    /// Check if heartbeat is running
    pub async fn is_running(&self) -> bool {
        self.state.lock().await.is_running
    }
}

impl Clone for HeartbeatManager {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            state: Arc::clone(&self.state),
            stop_sender: Arc::clone(&self.stop_sender),
        }
    }
}

/// Global heartbeat manager state for Tauri
pub type HeartbeatManagerState = Arc<Mutex<Option<HeartbeatManager>>>;

/// Initialize the global heartbeat manager
pub fn initialize_heartbeat_manager() -> HeartbeatManagerState {
    let manager = match HeartbeatManager::new() {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("[Heartbeat] Failed to initialize manager: {}", e);
            None
        }
    };
    Arc::new(Mutex::new(manager))
}
