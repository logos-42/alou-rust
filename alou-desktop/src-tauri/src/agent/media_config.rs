//! Media Config - 媒体配置（占位符）
//!
//! TODO: 实现实际的媒体配置

use serde::{Deserialize, Serialize};

/// 媒体 API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaApiConfig {
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub enabled: bool,
}

impl Default for MediaApiConfig {
    fn default() -> Self {
        Self {
            providers: vec![],
        }
    }
}

impl MediaApiConfig {
    pub fn load() -> Result<Self, String> {
        // TODO: 从配置文件加载
        Ok(Self::default())
    }
}

/// 媒体能力
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Image,
    Audio,
    Video,
}
