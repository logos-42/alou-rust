//! 即梦 (Jimeng) Provider 实现
//!
//! 支持：图片生成 + 视频生成

use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::{
    MediaProvider, MediaType, MediaOutput, MediaMetadata,
    MediaTask, TaskStatus,
    ImageOptions, VideoOptions,
};
use crate::agent::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// 即梦配置
#[derive(Debug, Clone)]
pub struct JimengConfig {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: String,
}

impl JimengConfig {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
            base_url: "https://api.jimeng.pro".to_string(),
        }
    }
}

pub struct JimengProvider {
    config: JimengConfig,
}

impl JimengProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let api_secret = config.base_url.clone()
            .ok_or_else(|| AgentError::ConfigError("Jimeng requires api_secret (put in base_url field)".to_string()))?;

        let jimeng_config = JimengConfig::new(config.api_key.clone(), api_secret);

        Ok(Self {
            config: jimeng_config,
        })
    }
}

#[async_trait]
impl MediaProvider for JimengProvider {
    fn name(&self) -> &str {
        "jimeng"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image, MediaType::Video]
    }

    async fn generate_image(
        &self,
        options: ImageOptions,
    ) -> Result<MediaOutput> {
        let client = Client::new();
        
        #[derive(Serialize)]
        struct ImageRequest {
            prompt: String,
            width: Option<u32>,
            height: Option<u32>,
        }

        let request = ImageRequest {
            prompt: options.prompt,
            width: options.width,
            height: options.height,
        };

        let url = format!("{}/image/generation", self.config.base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("X-Secret", &self.config.api_secret)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("Jimeng Image API error: {}", error)
            ));
        }

        #[derive(Deserialize)]
        struct ImageResponse {
            image_url: String,
            image_bytes_base64: Option<String>,
        }

        let result: ImageResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        // 下载图片或解码 base64
        let image_bytes = if let Some(base64_str) = result.image_bytes_base64 {
            base64::decode(&base64_str)
                .map_err(|e| AgentError::ExternalApiError(e.to_string()))?
        } else {
            // 从 URL 下载
            client.get(&result.image_url)
                .send()
                .await
                .map_err(|e| AgentError::ExternalApiError(e.to_string()))?
                .bytes()
                .await
                .map_err(|e| AgentError::ExternalApiError(e.to_string()))?
                .to_vec()
        };

        let file_path = crate::agent::media::storage::save_media(
            &image_bytes,
            "image",
            "png",
        )?;

        Ok(MediaOutput {
            media_type: MediaType::Image,
            provider: self.name().to_string(),
            url: Some(result.image_url),
            file_path: Some(file_path),
            ipfs_cid: None,
            metadata: MediaMetadata {
                width: options.width,
                height: options.height,
                format: Some("png".to_string()),
                prompt: Some(options.prompt),
                ..Default::default()
            },
        })
    }

    async fn generate_video(
        &self,
        options: VideoOptions,
    ) -> Result<MediaTask> {
        let client = Client::new();
        
        #[derive(Serialize)]
        struct VideoRequest {
            prompt: String,
            duration: Option<u32>,
            resolution: Option<String>,
        }

        let request = VideoRequest {
            prompt: options.prompt.unwrap_or_default(),
            duration: options.duration_secs,
            resolution: options.resolution,
        };

        let url = format!("{}/video/generation", self.config.base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("X-Secret", &self.config.api_secret)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("Jimeng Video API error: {}", error)
            ));
        }

        #[derive(Deserialize)]
        struct VideoResponse {
            task_id: String,
        }

        let result: VideoResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        Ok(MediaTask {
            task_id: result.task_id,
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
        let client = Client::new();
        let url = format!("{}/video/status?task_id={}", self.config.base_url, task_id);

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("X-Secret", &self.config.api_secret)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::ExternalApiError(
                "Failed to get video status".to_string()
            ));
        }

        #[derive(Deserialize)]
        struct StatusResponse {
            task_id: String,
            status: String,
            video_url: Option<String>,
            progress: Option<f32>,
        }

        let result: StatusResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        let status = match result.status.as_str() {
            "completed" => TaskStatus::Completed,
            "failed" => TaskStatus::Failed,
            _ => TaskStatus::Processing,
        };

        Ok(MediaTask {
            task_id: result.task_id,
            provider: self.name().to_string(),
            media_type: MediaType::Video,
            status,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            result: if status == TaskStatus::Completed && result.video_url.is_some() {
                Some(MediaOutput {
                    media_type: MediaType::Video,
                    provider: self.name().to_string(),
                    url: result.video_url,
                    file_path: None,
                    ipfs_cid: None,
                    metadata: Default::default(),
                })
            } else {
                None
            },
            error: if status == TaskStatus::Failed { Some("Video generation failed".to_string()) } else { None },
            progress: result.progress.unwrap_or(0.0),
        })
    }
}
