//! Media Provider - 媒体 Provider 接口

use serde::{Deserialize, Serialize};

/// 媒体类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Audio,
    Video,
}

/// 媒体输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaOutput {
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub metadata: serde_json::Value,
}

/// 媒体任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTask {
    pub id: String,
    pub status: TaskStatus,
    pub result: Option<MediaOutput>,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// 媒体 Provider trait
pub trait MediaProvider: Send + Sync {
    fn name(&self) -> &str;
    fn supported_types(&self) -> Vec<MediaType>;
}

/// 占位符 Provider
pub struct PlaceholderProvider;

impl MediaProvider for PlaceholderProvider {
    fn name(&self) -> &str {
        "placeholder"
    }
    
    fn supported_types(&self) -> Vec<MediaType> {
        vec![]
    }
}

/// Provider Registry（占位符）
pub struct ProviderRegistry;

impl ProviderRegistry {
    pub fn new(_config: &crate::agent::media_config::MediaApiConfig) -> Result<Self, String> {
        Ok(Self)
    }
    
    pub fn get_media_provider(&self, _name: &str) -> Option<Box<dyn MediaProvider>> {
        Some(Box::new(PlaceholderProvider))
    }
}

/// 媒体元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration: Option<f64>,
    pub format: Option<String>,
    pub prompt: Option<String>,
}

/// 图片选项
#[derive(Debug, Clone, Default)]
pub struct ImageOptions {
    pub prompt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub aspect_ratio: Option<String>,
    pub model: Option<String>,
    pub negative_prompt: Option<String>,
    pub num_images: Option<u32>,
}

/// 音频选项
#[derive(Debug, Clone, Default)]
pub struct AudioOptions {
    pub text: String,
    pub voice_id: Option<String>,
    pub model: Option<String>,
}

/// 视频选项
#[derive(Debug, Clone, Default)]
pub struct VideoOptions {
    pub prompt: String,
    pub duration: Option<f64>,
    pub model: Option<String>,
}
