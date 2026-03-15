//! MiniMax Provider 实现
//!
//! 支持：语音合成 (TTS) + 视频生成

use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::{
    MediaProvider, MediaType, MediaOutput, MediaMetadata,
    MediaTask, TaskStatus,
    ImageOptions, AudioOptions, VideoOptions,
};
use crate::agent::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod tts;
mod video;

pub use tts::MiniMaxTts;
pub use video::MiniMaxVideo;

/// MiniMax 配置
#[derive(Debug, Clone)]
pub struct MiniMaxConfig {
    pub api_key: String,
    pub group_id: String,
    pub base_url: String,
}

impl MiniMaxConfig {
    pub fn new(api_key: String, group_id: String) -> Self {
        Self {
            api_key,
            group_id,
            base_url: "https://api.minimax.chat/v1".to_string(),
        }
    }
}

/// MiniMax Provider
pub struct MiniMaxProvider {
    config: MiniMaxConfig,
    tts: MiniMaxTts,
    video: MiniMaxVideo,
}

impl MiniMaxProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let group_id = config.base_url.clone()
            .ok_or_else(|| AgentError::ConfigError("MiniMax requires group_id (put in base_url field)".to_string()))?;
        
        let minimax_config = MiniMaxConfig::new(config.api_key.clone(), group_id);

        Ok(Self {
            config: minimax_config.clone(),
            tts: MiniMaxTts::new(minimax_config.clone()),
            video: MiniMaxVideo::new(minimax_config),
        })
    }
}

#[async_trait]
impl MediaProvider for MiniMaxProvider {
    fn name(&self) -> &str {
        "minimax"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![MediaType::Audio, MediaType::Video]
    }

    async fn generate_audio(
        &self,
        options: AudioOptions,
    ) -> Result<MediaOutput> {
        let audio_bytes = self.tts.synthesize(options.text, options.voice_id).await?;

        // 保存到本地文件
        let file_path = crate::agent::media::storage::save_media(
            &audio_bytes,
            "audio",
            "mp3",
        )?;

        Ok(MediaOutput {
            media_type: MediaType::Audio,
            provider: self.name().to_string(),
            url: None,
            file_path: Some(file_path),
            ipfs_cid: None,
            metadata: MediaMetadata {
                duration_secs: Some(audio_bytes.len() as f32 / 16000.0),
                format: Some("mp3".to_string()),
                model: options.model,
                ..Default::default()
            },
        })
    }

    async fn generate_video(
        &self,
        options: VideoOptions,
    ) -> Result<MediaTask> {
        let task_id = self.video
            .generate(options.prompt.unwrap_or_default(), options.duration_secs.unwrap_or(5))
            .await?;

        Ok(MediaTask {
            task_id,
            provider: self.name().to_string(),
            media_type: MediaType::Video,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            result: None,
            error: None,
            progress: 0.0,
        })
    }

    async fn get_task_status(&self, task_id: &str) -> Result<MediaTask> {
        self.video.get_status(task_id).await
    }
}
