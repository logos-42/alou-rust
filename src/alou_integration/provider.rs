// ============================================
// Dual Provider Support (Anthropic + DeepSeek)
// ============================================
//
// This module provides dual LLM provider support by wrapping
// alou-code's api crate and providing a unified interface
// for both Anthropic and DeepSeek models.
//
// Provider selection:
// - Set ANTHROPIC_API_KEY for Anthropic models
// - Use model names like "deepseek-chat" to use DeepSeek
// - DeepSeek uses ANTHROPIC_API_KEY + ANTHROPIC_BASE_URL=https://api.deepseek.com
//
// Usage:
//   let provider = AlouProvider::new(ProviderConfig::default());
//   let response = provider.send_message(&request).await?;

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use api::{
    ApiClient, AuthSource, MessageRequest, MessageResponse,
    providers::{ProviderKind, detect_provider_kind, metadata_for_model, resolve_model_alias},
};
use api::http_client::ProxyConfig;
use api::providers::anthropic::AnthropicClient;

use crate::error::Error;

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Model to use (e.g., "claude-opus-4-6", "deepseek-chat", "deepseek-coder")
    pub model: String,
    /// API key (if not using env vars)
    pub api_key: Option<String>,
    /// Base URL override
    pub base_url: Option<String>,
    /// Max tokens for responses
    pub max_tokens: u32,
    /// Temperature for sampling
    pub temperature: f32,
    /// Enable streaming
    pub stream: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            model: "deepseek-chat".to_string(),
            api_key: None,
            base_url: None,
            max_tokens: 4096,
            temperature: 0.7,
            stream: false,
        }
    }
}

impl ProviderConfig {
    /// Create config for Anthropic
    pub fn anthropic(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Create config for DeepSeek
    pub fn deepseek(model: impl Into<String>) -> Self {
        Self {
            model: format!("deepseek-{}", model.into().trim_start_matches("deepseek-")),
            ..Default::default()
        }
    }

    /// Get the effective model name (resolved alias)
    pub fn resolved_model(&self) -> String {
        resolve_model_alias(&self.model)
    }

    /// Detect provider from model name
    pub fn detect_provider(&self) -> ProviderKind {
        detect_provider_kind(&self.model)
    }
}

/// Unified provider for dual Anthropic/DeepSeek support
pub struct AlouProvider {
    config: ProviderConfig,
    anthropic_client: Option<AnthropicClient>,
}

impl AlouProvider {
    /// Create a new provider instance
    pub fn new(config: ProviderConfig) -> Result<Self, Error> {
        let resolved_model = resolve_model_alias(&config.model);
        let provider = detect_provider_kind(&config.model);

        let anthropic_client = match provider {
            ProviderKind::Anthropic => {
                let client = Self::create_anthropic_client(&config)?;
                Some(client)
            }
            ProviderKind::DeepSeek => {
                let mut deepseek_config = config.clone();
                // DeepSeek uses the same API shape as Anthropic
                deepseek_config.base_url = Some("https://api.deepseek.com".to_string());
                let client = Self::create_anthropic_client(&deepseek_config)?;
                Some(client)
            }
            _ => {
                // For other providers (OpenAI, xAI, etc.), use Anthropic-compatible client
                let client = Self::create_anthropic_client(&config)?;
                Some(client)
            }
        };

        Ok(Self {
            config,
            anthropic_client,
        })
    }

    fn create_anthropic_client(config: &ProviderConfig) -> Result<AnthropicClient, Error> {
        let resolved_model = resolve_model_alias(&config.model);
        let metadata = metadata_for_model(&resolved_model)
            .unwrap_or_else(|| api::providers::ProviderMetadata {
                provider: ProviderKind::Anthropic,
                auth_env: "ANTHROPIC_API_KEY",
                base_url_env: "ANTHROPIC_BASE_URL",
                default_base_url: "https://api.deepseek.com",
            });

        let auth_source = if let Some(ref api_key) = config.api_key {
            AuthSource::ApiKey(api_key.clone())
        } else {
            AuthSource::Env
        };

        let base_url = config.base_url.clone()
            .or_else(|| std::env::var(metadata.base_url_env).ok())
            .unwrap_or_else(|| metadata.default_base_url.to_string());

        let client = AnthropicClient::new(
            auth_source,
            &base_url,
            config.max_tokens,
            config.temperature,
            ProxyConfig::default(),
        )
        .map_err(|e| Error::Other(format!("Failed to create Anthropic client: {}", e)))?;

        Ok(client)
    }

    /// Send a message using the configured provider
    pub async fn send_message(&self, request: &MessageRequest) -> Result<MessageResponse, Error> {
        if let Some(ref client) = self.anthropic_client {
            client.send_message(request).await
                .map_err(|e| Error::Other(format!("Provider error: {}", e)))
        } else {
            Err(Error::Other("No provider client available".to_string()))
        }
    }

    /// Get the current provider kind
    pub fn provider_kind(&self) -> ProviderKind {
        detect_provider_kind(&self.config.model)
    }

    /// Check if using DeepSeek
    pub fn is_deepseek(&self) -> bool {
        self.provider_kind() == ProviderKind::DeepSeek
    }

    /// Check if using Anthropic
    pub fn is_anthropic(&self) -> bool {
        self.provider_kind() == ProviderKind::Anthropic
    }

    /// Get the resolved model name
    pub fn model_name(&self) -> String {
        resolve_model_alias(&self.config.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_deepseek_alias() {
        let config = ProviderConfig::deepseek("chat");
        assert_eq!(config.resolved_model(), "deepseek-chat");
    }

    #[test]
    fn test_resolve_deepseek_coder() {
        let config = ProviderConfig::deepseek("coder");
        assert_eq!(config.resolved_model(), "deepseek-coder");
    }

    #[test]
    fn test_resolve_anthropic_alias() {
        let config = ProviderConfig::anthropic("opus");
        assert_eq!(config.resolved_model(), "claude-opus-4-6");
    }

    #[test]
    fn test_detect_deepseek_provider() {
        let config = ProviderConfig::deepseek("chat");
        assert_eq!(config.detect_provider(), ProviderKind::DeepSeek);
    }

    #[test]
    fn test_detect_anthropic_provider() {
        let config = ProviderConfig::anthropic("sonnet");
        assert_eq!(config.detect_provider(), ProviderKind::Anthropic);
    }
}
