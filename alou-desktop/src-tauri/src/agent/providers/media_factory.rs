//! Media Provider Factory - 媒体 Provider 工厂
//!
//! 根据配置创建不同的媒体 Provider 实例
//! 参考官方 API 文档：
//! - MiniMax: https://platform.minimaxi.com/document
//! - Google Imagen: https://cloud.google.com/vertex-ai/docs/generative-ai/image/overview
//! - Jimeng (即梦): https://api.jimeng.pro/docs

use std::sync::Arc;
use crate::agent::media_config::ProviderConfig;
use crate::agent::error::{AgentError, Result};
use super::media_provider::MediaProvider;

/// Provider 信息
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub description: String,
    pub supported_types: Vec<super::media_provider::MediaType>,
    pub capabilities: Vec<String>,
}

/// 媒体 Provider 工厂
pub struct MediaProviderFactory;

impl MediaProviderFactory {
    /// 根据名称创建 Provider
    pub fn create_provider(name: &str, config: &ProviderConfig) -> Result<Arc<dyn MediaProvider>> {
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
            _ => Err(AgentError::ConfigError(
                format!("未知的媒体 Provider: {}", name)
            )),
        }
    }

    /// 列出所有可用的 Provider
    pub fn list_providers() -> Vec<ProviderInfo> {
        vec![
            ProviderInfo {
                name: "minimax".to_string(),
                description: "MiniMax 是中国领先的 AI 大模型公司，提供高质量的 TTS 和视频生成服务".to_string(),
                supported_types: vec![
                    super::media_provider::MediaType::Audio,
                    super::media_provider::MediaType::Video,
                ],
                capabilities: vec![
                    "tts".to_string(),
                    "video_generation".to_string(),
                    "chinese_optimized".to_string(),
                ],
            },
            ProviderInfo {
                name: "google".to_string(),
                description: "Google Imagen 是 Google 的图像生成模型，以高质量和安全性著称".to_string(),
                supported_types: vec![
                    super::media_provider::MediaType::Image,
                ],
                capabilities: vec![
                    "image_generation".to_string(),
                    "high_quality".to_string(),
                    "safe_content".to_string(),
                ],
            },
            ProviderInfo {
                name: "jimeng".to_string(),
                description: "即梦是字节跳动旗下的 AI 创作平台，提供图片和视频生成服务".to_string(),
                supported_types: vec![
                    super::media_provider::MediaType::Image,
                    super::media_provider::MediaType::Video,
                ],
                capabilities: vec![
                    "image_generation".to_string(),
                    "video_generation".to_string(),
                    "chinese_optimized".to_string(),
                ],
            },
            ProviderInfo {
                name: "haimian".to_string(),
                description: "海绵音乐是字节跳动推出的 AI 音乐生成平台，支持个性化音乐创作".to_string(),
                supported_types: vec![
                    super::media_provider::MediaType::Audio,
                ],
                capabilities: vec![
                    "music_generation".to_string(),
                    "chinese_optimized".to_string(),
                    "lyrics_support".to_string(),
                ],
            },
        ]
    }

    /// 根据能力获取推荐的 Provider
    pub fn get_provider_by_capability(capability: &str) -> Option<String> {
        match capability {
            "image" => Some("google".to_string()),
            "image_cn" => Some("jimeng".to_string()),
            "audio" | "tts" => Some("minimax".to_string()),
            "music" | "music_generation" => Some("haimian".to_string()),
            "video" => Some("jimeng".to_string()),
            "video_cn" => Some("jimeng".to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_providers() {
        let providers = MediaProviderFactory::list_providers();
        assert_eq!(providers.len(), 3);
        
        let names: Vec<String> = providers.iter().map(|p| p.name.clone()).collect();
        assert!(names.contains(&"minimax".to_string()));
        assert!(names.contains(&"google".to_string()));
        assert!(names.contains(&"jimeng".to_string()));
    }

    #[test]
    fn test_get_provider_by_capability() {
        assert_eq!(MediaProviderFactory::get_provider_by_capability("image"), Some("google".to_string()));
        assert_eq!(MediaProviderFactory::get_provider_by_capability("tts"), Some("minimax".to_string()));
        assert_eq!(MediaProviderFactory::get_provider_by_capability("video"), Some("jimeng".to_string()));
    }
}
