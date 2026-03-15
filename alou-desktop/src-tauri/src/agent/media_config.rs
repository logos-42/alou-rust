//! Media Config - 媒体 API 配置
//!
//! 配置所有媒体 Provider 的 API 密钥和参数

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 媒体 API 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaApiConfig {
    pub providers: HashMap<String, ProviderConfig>,
}

/// Provider 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub capabilities: Vec<Capability>,
    pub config: Option<serde_json::Value>,
}

impl MediaApiConfig {
    pub fn load() -> Result<Self, String> {
        // TODO: 从配置文件加载
        Ok(Self::default())
    }

    /// 获取启用的 Provider
    pub fn enabled_providers(&self) -> Vec<&String> {
        self.providers
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, _)| name)
            .collect()
    }

    /// 根据能力获取 Provider
    pub fn get_provider_by_capability(&self, capability: Capability) -> Option<&String> {
        for (name, config) in &self.providers {
            if config.enabled && config.capabilities.contains(&capability) {
                return Some(name);
            }
        }
        None
    }
}

/// 媒体能力
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Image,
    Audio,
    Video,
    Music,
    Tts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_api_config() {
        let mut config = MediaApiConfig::default();
        
        config.providers.insert("seedance".to_string(), ProviderConfig {
            name: "seedance".to_string(),
            api_key: "test_key".to_string(),
            base_url: None,
            model: Some("seedance-1.0-pro".to_string()),
            enabled: true,
            capabilities: vec![Capability::Video],
            config: None,
        });

        config.providers.insert("seedream".to_string(), ProviderConfig {
            name: "seedream".to_string(),
            api_key: "test_key".to_string(),
            base_url: None,
            model: Some("doubao-seedream-5-0-260128".to_string()),
            enabled: true,
            capabilities: vec![Capability::Image],
            config: None,
        });

        assert_eq!(config.enabled_providers().len(), 2);
        assert_eq!(
            config.get_provider_by_capability(Capability::Video),
            Some(&"seedance".to_string())
        );
        assert_eq!(
            config.get_provider_by_capability(Capability::Image),
            Some(&"seedream".to_string())
        );
    }
}
