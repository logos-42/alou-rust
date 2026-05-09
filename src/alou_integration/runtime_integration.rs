// ============================================
// Runtime Integration Bridge
// ============================================
//
// This module provides integration with alou-code's runtime crate,
// which handles:
// - Session management and persistence
// - Permission enforcement
// - MCP client lifecycle
// - System prompt assembly
// - Usage tracking
//
// The bridge provides:
// - Session creation and management
// - Permission enforcement
// - Config loading
// - MCP lifecycle management

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use runtime::{
    ConfigLoader, Session, PermissionMode, PermissionPolicy,
    ConversationRuntime, RuntimeError,
};
use api::ProviderClient;

use crate::error::Error;
use super::provider::AlouProvider;

/// Runtime bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Working directory
    pub cwd: PathBuf,
    /// Home directory for config storage
    pub home_dir: PathBuf,
    /// Session storage path
    pub session_path: Option<PathBuf>,
    /// Default permission mode
    pub default_permission: PermissionMode,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let home_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".alou");
        Self {
            cwd: cwd.clone(),
            home_dir: home_dir.clone(),
            session_path: Some(home_dir.join("sessions")),
            default_permission: PermissionMode::DangerFullAccess,
            debug: false,
        }
    }
}

/// Bridge to alou-code runtime
pub struct AlouRuntimeBridge {
    config: RuntimeConfig,
    config_loader: ConfigLoader,
}

impl AlouRuntimeBridge {
    /// Create a new runtime bridge
    pub fn new(config: RuntimeConfig) -> Self {
        let config_loader = ConfigLoader::new(&config.cwd, &config.home_dir);
        Self {
            config,
            config_loader,
        }
    }

    /// Create with default configuration
    pub fn default_bridge() -> Self {
        Self::new(RuntimeConfig::default())
    }

    /// Load configuration
    pub fn load_config(&self) -> Result<runtime::Config, Error> {
        self.config_loader
            .load()
            .map_err(|e| Error::Other(format!("Config load error: {}", e)))
    }

    /// Get session storage path
    pub fn session_path(&self) -> PathBuf {
        self.config.session_path.clone()
            .unwrap_or_else(|| self.config.home_dir.join("sessions"))
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        model: &str,
        system_prompt: Option<&str>,
    ) -> Result<Session, Error> {
        let session_path = self.session_path();
        std::fs::create_dir_all(&session_path)
            .map_err(|e| Error::Other(format!("Failed to create session dir: {}", e)))?;

        let session = Session::new(
            model,
            system_prompt,
            &session_path,
        ).map_err(|e| Error::Other(format!("Failed to create session: {}", e)))?;

        Ok(session)
    }

    /// Get default permission policy
    pub fn default_permission_policy(&self) -> PermissionPolicy {
        PermissionPolicy::DangerFullAccess
    }

    /// Check if debug mode is enabled
    pub fn is_debug(&self) -> bool {
        self.config.debug
    }
}

impl Default for AlouRuntimeBridge {
    fn default() -> Self {
        Self::default_bridge()
    }
}

/// Conversation runtime wrapper
pub struct AlouConversationRuntime {
    runtime: ConversationRuntime,
}

impl AlouConversationRuntime {
    /// Create a new conversation runtime
    pub async fn new(
        provider: AlouProvider,
        session: Session,
        permission_mode: PermissionMode,
    ) -> Result<Self, Error> {
        let runtime = ConversationRuntime::new(
            provider.into_inner(),
            session,
            permission_mode,
        ).map_err(|e| Error::Other(format!("Failed to create runtime: {}", e)))?;

        Ok(Self { runtime })
    }

    /// Process a user message
    pub async fn process_message(&mut self, message: &str) -> Result<String, Error> {
        self.runtime
            .process_message(message)
            .await
            .map_err(|e| Error::Other(format!("Process error: {}", e)))
    }

    /// Get conversation history
    pub fn conversation_history(&self) -> Vec<String> {
        // This would return the actual conversation history from runtime
        vec![]
    }

    /// Reset the conversation
    pub async fn reset(&mut self) -> Result<(), Error> {
        self.runtime
            .reset()
            .map_err(|e| Error::Other(format!("Reset error: {}", e)))
    }
}

/// Extension trait for ProviderClient to work with ConversationRuntime
trait ProviderClientExt {
    fn into_inner(self) -> Arc<dyn ProviderClient + Send + Sync>;
}

impl ProviderClientExt for AlouProvider {
    fn into_inner(self) -> Arc<dyn ProviderClient + Send + Sync> {
        // This is a placeholder - the actual implementation would
        // extract the inner Anthropic client
        unimplemented!("ProviderClient extraction requires concrete type")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RuntimeConfig::default();
        assert!(config.session_path.is_some());
    }

    #[test]
    fn test_session_path() {
        let bridge = AlouRuntimeBridge::default_bridge();
        let path = bridge.session_path();
        assert!(path.to_string_lossy().contains("sessions"));
    }
}
