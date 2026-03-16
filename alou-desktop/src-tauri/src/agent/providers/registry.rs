//! Provider 注册表
//!
//! 启动时创建所有 Provider，运行时复用

use crate::agent::error::{AgentError, Result};
use crate::agent::media_config::{Capability, MediaApiConfig, ProviderConfig};
use crate::agent::providers::media_provider::MediaProvider;
use std::collections::HashMap;
use std::sync::Arc;

/// Provider 注册表
pub struct ProviderRegistry {
    media_providers: HashMap<String, Arc<dyn MediaProvider>>,
    config: MediaApiConfig,
}

impl ProviderRegistry {
    pub fn new(config: &MediaApiConfig) -> Result<Self> {
        let mut media_providers = HashMap::new();

        // 根据配置创建 Provider
        for (name, provider_config) in &config.providers {
            if !provider_config.enabled {
                continue;
            }

            if let Ok(provider) = Self::create_media_provider(name, provider_config) {
                media_providers.insert(name.clone(), provider);
            }
        }

        Ok(Self {
            media_providers,
            config: config.clone(),
        })
    }

    fn create_media_provider(
        name: &str,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MediaProvider>> {
        match name {
            "minimax" => {
                let provider = crate::agent::providers::minimax::MiniMaxProvider::new(config)?;
                Ok(Arc::new(provider))
            }
            "google" => {
                let provider = crate::agent::providers::google::GoogleProvider::new(config)?;
                Ok(Arc::new(provider))
            }
            "jimeng" => {
                let provider = crate::agent::providers::jimeng::JimengProvider::new(config)?;
                Ok(Arc::new(provider))
            }
            "haimian" => {
                let provider = crate::agent::providers::haimian::HaimianMusicProvider::new(config)?;
                Ok(Arc::new(provider))
            }
            "seedance" => {
                let provider = crate::agent::providers::seedance::SeedanceProvider::new(config)?;
                Ok(Arc::new(provider))
            }
            "seedream" => {
                let provider = crate::agent::providers::seedream::SeedreamProvider::new(config)?;
                Ok(Arc::new(provider))
            }
            "suno" => {
                let provider = crate::agent::providers::suno::SunoProvider::new(config)?;
                Ok(Arc::new(provider))
            }
            // "minimax_music" => {
            //     let provider = crate::agent::providers::minimax::MiniMaxMusicProvider::new(config)?;
            //     Ok(Arc::new(provider))
            // }
            _ => Err(AgentError::ConfigError(format!(
                "Unknown media provider: {}",
                name
            ))),
        }
    }

    /// 获取 Media Provider
    pub fn get_media_provider(&self, name: &str) -> Option<Arc<dyn MediaProvider>> {
        self.media_providers.get(name).cloned()
    }

    /// 根据能力获取 Provider
    pub fn get_provider_by_capability(&self, capability: Capability) -> Option<String> {
        for (name, config) in &self.config.providers {
            if config.enabled && config.capabilities.contains(&capability) {
                return Some(name.clone());
            }
        }
        None
    }

    /// 获取所有可用的 Provider
    pub fn available_providers(&self) -> Vec<String> {
        self.media_providers.keys().cloned().collect()
    }

    /// 获取配置
    pub fn config(&self) -> &MediaApiConfig {
        &self.config
    }
}
