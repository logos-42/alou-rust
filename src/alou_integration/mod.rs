// ============================================
// Alou-Code Integration Module
// ============================================
//
// This module provides integration between the existing alou-rust codebase
// and the alou-code workflow engine from ultraworkers/alou-code.
//
// Key features:
// - Dual Provider support (Anthropic + DeepSeek)
// - DIAP Identity System integration
// - Tools crate integration (40+ tools)
// - Runtime crate integration (sessions, permissions)
// - Commands crate integration (slash commands)
//
// Architecture:
// - identity_bridge: DIAP identity system binding
// - provider: Dual provider selector (Anthropic/DeepSeek)
// - tools_integration: Bridge to alou-code tools system
// - runtime_integration: Bridge to alou-code runtime

pub mod identity_bridge;
pub mod provider;
pub mod tools_integration;
pub mod runtime_integration;

pub use identity_bridge::DiapIdentityBridge;
pub use provider::{AlouProvider, ProviderConfig};
pub use tools_integration::AlouToolsBridge;
pub use runtime_integration::AlouRuntimeBridge;

use serde::{Deserialize, Serialize};

/// Integration configuration for alou-code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlouIntegrationConfig {
    /// Provider configuration
    pub provider: ProviderConfig,
    /// DIAP identity enabled
    pub diap_enabled: bool,
    /// Session storage path
    pub session_path: Option<String>,
    /// Workspace directories
    pub workspace_dirs: Vec<String>,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for AlouIntegrationConfig {
    fn default() -> Self {
        Self {
            provider: ProviderConfig::default(),
            diap_enabled: true,
            session_path: None,
            workspace_dirs: vec![],
            debug: false,
        }
    }
}
