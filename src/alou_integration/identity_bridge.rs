// ============================================
// DIAP Identity Bridge
// ============================================
//
// This module provides the bridge between the DIAP identity system
// and alou-code's session/authentication system.
//
// DIAP is an identity system that needs to be bound to the underlying
// layer. This bridge enables:
// - DIAP authentication for alou-code sessions
// - API key issuance via DIAP identity
// - Session binding to DIAP identities
//
// DIAP SDK: diap-rs-sdk
// Documentation: https://docs.diap.org

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiapIdentity {
    /// Unique identifier for the identity
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Associated wallet address (if any)
    pub wallet_address: Option<String>,
    /// Identity level/verification status
    pub level: IdentityLevel,
    /// Creation timestamp
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityLevel {
    Anonymous,
    Basic,
    Verified,
    Trusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// The API key string
    pub key: String,
    /// Associated identity
    pub identity_id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp (0 = never expires)
    pub expires_at: u64,
    /// Scopes/permissions
    pub scopes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Identity not found: {0}")]
    NotFound(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("DIAP SDK error: {0}")]
    SdkError(String),
}

impl From<IdentityError> for crate::error::Error {
    fn from(err: IdentityError) -> Self {
        crate::error::Error::Other(err.to_string())
    }
}

/// DIAP Identity Bridge
///
/// This bridge connects DIAP identity system to alou-code's session management.
/// It provides:
/// - Identity authentication
/// - API key management
/// - Session identity binding
pub struct DiapIdentityBridge {
    config: DiapBridgeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiapBridgeConfig {
    /// DIAP SDK configuration
    pub sdk_config: SdkConfig,
    /// Enable API key authentication
    pub api_key_auth: bool,
    /// Session binding required
    pub require_session_binding: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    /// DIAP API endpoint
    pub endpoint: String,
    /// API key for DIAP
    pub api_key: Option<String>,
    /// Connection timeout (ms)
    pub timeout_ms: u64,
}

impl Default for DiapBridgeConfig {
    fn default() -> Self {
        Self {
            sdk_config: SdkConfig {
                endpoint: "https://api.diap.org".to_string(),
                api_key: None,
                timeout_ms: 30000,
            },
            api_key_auth: true,
            require_session_binding: true,
        }
    }
}

impl DiapIdentityBridge {
    /// Create a new DIAP identity bridge
    pub fn new(config: DiapBridgeConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default_bridge() -> Self {
        Self::new(DiapBridgeConfig::default())
    }

    /// Authenticate with DIAP using a token
    pub async fn authenticate(&self, token: &str) -> Result<DiapIdentity, IdentityError> {
        // In a real implementation, this would call the DIAP SDK
        // For now, we provide the structure
        if token.is_empty() {
            return Err(IdentityError::InvalidCredentials("Empty token".to_string()));
        }

        // Simulate identity lookup - in production, this would use diap-rs-sdk
        Ok(DiapIdentity {
            id: format!("identity_{}", &token[..8.min(token.len())]),
            name: "User".to_string(),
            wallet_address: None,
            level: IdentityLevel::Basic,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        })
    }

    /// Issue an API key for an identity
    pub async fn issue_api_key(&self, identity: &DiapIdentity) -> Result<ApiKey, IdentityError> {
        if !self.config.api_key_auth {
            return Err(IdentityError::PermissionDenied("API key auth disabled".to_string()));
        }

        // In production, this would use diap-rs-sdk to issue a real API key
        let key = generate_api_key(identity);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(ApiKey {
            key,
            identity_id: identity.id.clone(),
            created_at: now,
            expires_at: 0, // Never expires
            scopes: vec!["read".to_string(), "write".to_string()],
        })
    }

    /// Validate an API key
    pub async fn validate_api_key(&self, key: &str) -> Result<DiapIdentity, IdentityError> {
        if key.is_empty() {
            return Err(IdentityError::InvalidCredentials("Empty API key".to_string()));
        }

        // In production, this would validate against DIAP's key storage
        // For now, extract identity from key format: diap_key_{identity_id}_{random}
        if !key.starts_with("diap_key_") {
            return Err(IdentityError::InvalidCredentials("Invalid key format".to_string()));
        }

        // Simulate identity lookup
        Ok(DiapIdentity {
            id: "validated_identity".to_string(),
            name: "API User".to_string(),
            wallet_address: None,
            level: IdentityLevel::Verified,
            created_at: 0,
        })
    }

    /// Bind a session to an identity
    pub async fn bind_session(&self, session_id: &str, identity: &DiapIdentity) -> Result<(), IdentityError> {
        if self.config.require_session_binding {
            tracing::info!("Binding session {} to identity {}", session_id, identity.id);
        }
        Ok(())
    }

    /// Get identity by ID
    pub async fn get_identity(&self, identity_id: &str) -> Result<DiapIdentity, IdentityError> {
        if identity_id.is_empty() {
            return Err(IdentityError::NotFound("Empty identity ID".to_string()));
        }

        Ok(DiapIdentity {
            id: identity_id.to_string(),
            name: "User".to_string(),
            wallet_address: None,
            level: IdentityLevel::Basic,
            created_at: 0,
        })
    }

    /// Check if DIAP is available
    pub async fn health_check(&self) -> bool {
        // In production, ping the DIAP API endpoint
        true
    }
}

fn generate_api_key(identity: &DiapIdentity) -> String {
    use std::time::SystemTime;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let random: u64 = (timestamp % 1000000) as u64;
    format!("diap_key_{}_{:06}", identity.id, random)
}

/// DIAP identity extractor for HTTP requests
pub fn extract_identity_from_request(headers: &http::HeaderMap) -> Option<String> {
    // Try Authorization header first
    if let Some(auth) = headers.get(http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
            if auth_str.starts_with("DiapKey ") {
                return Some(auth_str[8..].to_string());
            }
        }
    }

    // Try X-DIAP-Key header
    if let Some(key) = headers.get("X-DIAP-Key") {
        if let Ok(key_str) = key.to_str() {
            return Some(key_str.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticate() {
        let bridge = DiapIdentityBridge::default_bridge();
        let result = bridge.authenticate("test_token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_issue_api_key() {
        let bridge = DiapIdentityBridge::default_bridge();
        let identity = DiapIdentity {
            id: "test_id".to_string(),
            name: "Test User".to_string(),
            wallet_address: None,
            level: IdentityLevel::Basic,
            created_at: 0,
        };
        let result = bridge.issue_api_key(&identity).await;
        assert!(result.is_ok());
        let api_key = result.unwrap();
        assert!(api_key.key.starts_with("diap_key_"));
    }

    #[tokio::test]
    async fn test_validate_api_key() {
        let bridge = DiapIdentityBridge::default_bridge();
        let result = bridge.validate_api_key("diap_key_test_123456").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let bridge = DiapIdentityBridge::default_bridge();
        let result = bridge.validate_api_key("invalid_key").await;
        assert!(result.is_err());
    }
}
