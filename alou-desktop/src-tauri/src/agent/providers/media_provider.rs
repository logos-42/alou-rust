//! Media Provider - 媒体 Provider 接口
//!
//! 定义所有媒体 Provider 的统一接口


use async_trait::async_trait;

use serde::{Deserialize, Serialize};

/// 媒体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Audio,
    Video,
    Music,
}

/// 媒体输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaOutput {
    pub media_type: MediaType,
    pub provider: String,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub ipfs_cid: Option<String>,
    pub metadata: MediaMetadata,
}

impl Default for MediaOutput {
    fn default() -> Self {
        Self {
            media_type: MediaType::Image,
            provider: String::new(),
            url: None,
            file_path: None,
            ipfs_cid: None,
            metadata: MediaMetadata::default(),
        }
    }
}

/// 媒体任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTask {
    pub task_id: String,
    pub provider: String,
    pub media_type: MediaType,
    pub status: TaskStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub result: Option<MediaOutput>,
    pub error: Option<String>,
    pub progress: f32,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// 媒体元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration: Option<f32>,
    pub duration_secs: Option<f32>,
    pub format: Option<String>,
    pub prompt: Option<String>,
    pub model: Option<String>,
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
    pub speed: Option<f32>,
    pub pitch: Option<f32>,
    pub volume: Option<f32>,
    pub lyrics: Option<String>,
    pub is_instrumental: Option<bool>,
}

/// 视频选项
#[derive(Debug, Clone, Default)]
pub struct VideoOptions {
    pub prompt: String,
    pub duration: Option<f64>,
    pub duration_secs: Option<u32>,
    pub model: Option<String>,
    pub resolution: Option<String>,
}

/// 媒体 Provider trait
#[async_trait]
pub trait MediaProvider: Send + Sync {
    /// 获取 Provider 名称
    fn name(&self) -> &str;
    
    /// 获取支持的媒体类型
    fn supported_types(&self) -> Vec<MediaType>;
    
    /// 生成图片
    async fn generate_image(&self, options: ImageOptions) -> Result<MediaOutput, crate::agent::error::AgentError>;
    
    /// 生成音频
    async fn generate_audio(&self, options: AudioOptions) -> Result<MediaOutput, crate::agent::error::AgentError>;
    
    /// 生成视频（异步任务）
    async fn generate_video(&self, options: VideoOptions) -> Result<MediaTask, crate::agent::error::AgentError>;
    
    /// 获取任务状态
    async fn get_task_status(&self, task_id: &str) -> Result<MediaTask, crate::agent::error::AgentError>;
}

/// 占位符 Provider（用于测试和默认值）
pub struct PlaceholderProvider;

#[async_trait]
impl MediaProvider for PlaceholderProvider {
    fn name(&self) -> &str {
        "placeholder"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![]
    }

    async fn generate_image(&self, _options: ImageOptions) -> Result<MediaOutput, crate::agent::error::AgentError> {
        Err(crate::agent::error::AgentError::InvalidInput(
            "Placeholder provider does not support image generation".to_string()
        ))
    }

    async fn generate_audio(&self, _options: AudioOptions) -> Result<MediaOutput, crate::agent::error::AgentError> {
        Err(crate::agent::error::AgentError::InvalidInput(
            "Placeholder provider does not support audio generation".to_string()
        ))
    }

    async fn generate_video(&self, _options: VideoOptions) -> Result<MediaTask, crate::agent::error::AgentError> {
        Err(crate::agent::error::AgentError::InvalidInput(
            "Placeholder provider does not support video generation".to_string()
        ))
    }

    async fn get_task_status(&self, _task_id: &str) -> Result<MediaTask, crate::agent::error::AgentError> {
        Err(crate::agent::error::AgentError::InvalidInput(
            "Placeholder provider does not support task status".to_string()
        ))
    }
}
