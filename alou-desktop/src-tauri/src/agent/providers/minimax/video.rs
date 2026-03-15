//! MiniMax Video 生成

use super::MiniMaxConfig;
use crate::agent::providers::media_provider::{MediaTask, TaskStatus};
use crate::agent::error::{AgentError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// 视频生成请求
#[derive(Debug, Serialize)]
pub struct VideoRequest {
    pub model: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub duration: u32,
    pub resolution: String,
}

/// 视频生成响应
#[derive(Debug, Deserialize)]
pub struct VideoResponse {
    pub task_id: String,
    pub video_url: Option<String>,
    pub status: String,
}

pub struct MiniMaxVideo {
    config: MiniMaxConfig,
}

impl MiniMaxVideo {
    pub fn new(config: MiniMaxConfig) -> Self {
        Self { config }
    }

    /// 生成视频（异步）
    pub async fn generate(
        &self,
        prompt: String,
        duration: u32,
    ) -> Result<String> {
        let client = Client::new();
        
        let request = VideoRequest {
            model: "video-01".to_string(),
            prompt,
            negative_prompt: None,
            duration,
            resolution: "720p".to_string(),
        };

        let url = format!("{}/video/generation", self.config.base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("MiniMax Video API error: {}", error)
            ));
        }

        let result: VideoResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        Ok(result.task_id)
    }

    /// 查询视频生成状态
    pub async fn get_status(&self, task_id: String) -> Result<MediaTask> {
        let client = Client::new();
        let url = format!("{}/video/status?task_id={}", self.config.base_url, task_id);

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("MiniMax Video status API error: {}", error)
            ));
        }

        #[derive(Deserialize)]
        struct StatusResponse {
            pub task_id: String,
            pub video_url: Option<String>,
            pub status: String,
            pub progress: Option<f32>,
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
            provider: "minimax".to_string(),
            media_type: crate::agent::providers::media_provider::MediaType::Video,
            status,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            result: if status == TaskStatus::Completed && result.video_url.is_some() {
                Some(crate::agent::providers::media_provider::MediaOutput {
                    media_type: crate::agent::providers::media_provider::MediaType::Video,
                    provider: "minimax".to_string(),
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
