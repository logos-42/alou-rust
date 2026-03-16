//! 即梦 Seedance Provider 实现
//!
//! 即梦 (Jimeng) Seedance 1.0 视频生成 API
//!
//! 官方文档：
//! - 302.AI API: https://doc.302.ai/305249446e0
//! - 火山方舟：https://www.volcengine.com/docs/82379/1520757
//!
//! API 端点：
//! - 创建任务：POST https://api.302.ai/doubao/doubao-seedance
//! - 查询状态：GET https://api.302.ai/doubao/doubao-seedance/{task_id}
//!
//! 支持模型：
//! - seedance-1.0-pro: 标准版，2.2 PTC/1M tokens
//! - seedance-1.0-pro-fast: 快速版，0.6 PTC/1M tokens
//! - seedance-1.0-lite: 精简版，1.5 PTC/1M tokens
//!
//! 认证方式：Bearer Token (API Key)

use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::{
    MediaProvider, MediaType, MediaOutput, MediaMetadata,
    MediaTask, TaskStatus,
    VideoOptions,
};
use crate::agent::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Seedance 配置
#[derive(Debug, Clone)]
pub struct SeedanceConfig {
    pub api_key: String,
    pub base_url: String,
    pub default_model: String,
}

impl SeedanceConfig {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.302.ai/doubao".to_string()),
            default_model: "seedance-1.0-pro".to_string(),
        }
    }
}

/// 视频生成请求
#[derive(Debug, Serialize)]
pub struct SeedanceVideoRequest {
    #[serde(rename = "model")]
    pub model: String,

    #[serde(rename = "content")]
    pub content: Vec<ContentItem>,

    #[serde(rename = "resolution", skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,

    #[serde(rename = "ratio", skip_serializing_if = "Option::is_none")]
    pub ratio: Option<String>,

    #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,

    #[serde(rename = "generate_audio", skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,

    #[serde(rename = "service_tier", skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
}

/// 内容项（文字或图片）
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentItem {
    Text { text: String },
    Image { image_url: String },
}

/// 创建任务响应
#[derive(Debug, Deserialize)]
pub struct SeedanceCreateResponse {
    #[serde(rename = "id")]
    pub id: String,
}

/// 任务状态响应
#[derive(Debug, Deserialize)]
pub struct SeedanceStatusResponse {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "video", skip_serializing_if = "Option::is_none")]
    pub video: Option<SeedanceVideo>,

    #[serde(rename = "error", skip_serializing_if = "Option::is_none")]
    pub error: Option<SeedanceError>,

    #[serde(rename = "progress", skip_serializing_if = "Option::is_none")]
    pub progress: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SeedanceVideo {
    #[serde(rename = "play_addr")]
    pub play_addr: String,

    #[serde(rename = "cover")]
    pub cover: String,

    #[serde(rename = "duration")]
    pub duration: Option<f32>,

    #[serde(rename = "width")]
    pub width: Option<u32>,

    #[serde(rename = "height")]
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SeedanceError {
    #[serde(rename = "code")]
    pub code: String,

    #[serde(rename = "message")]
    pub message: String,
}

pub struct SeedanceProvider {
    config: SeedanceConfig,
    client: Client,
}

impl SeedanceProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let seedance_config = SeedanceConfig::new(
            config.api_key.clone(),
            config.base_url.clone(),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        Ok(Self {
            config: seedance_config,
            client,
        })
    }

    /// 创建视频生成任务
    pub async fn create_video_task(
        &self,
        prompt: String,
        model: Option<String>,
        duration: Option<u32>,
        resolution: Option<String>,
        ratio: Option<String>,
    ) -> Result<String> {
        let request = SeedanceVideoRequest {
            model: model.unwrap_or_else(|| self.config.default_model.clone()),
            content: vec![ContentItem::Text { text: prompt }],
            resolution,
            ratio,
            duration,
            generate_audio: Some(true),
            service_tier: Some("default".to_string()),
        };

        let url = format!("{}/doubao-seedance", self.config.base_url);

        log::info!("[Seedance] 创建视频任务，URL: {}", url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("网络请求失败：{}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("Seedance API 错误 ({}): {}", status, error)
            ));
        }

        let result: SeedanceCreateResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        log::info!("[Seedance] 任务创建成功，task_id: {}", result.id);

        Ok(result.id)
    }

    /// 查询任务状态
    pub async fn get_status(&self, task_id: &str) -> Result<MediaTask> {
        let url = format!("{}/doubao-seedance/{}", self.config.base_url, task_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("网络请求失败：{}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("查询状态失败 ({}): {}", status, error)
            ));
        }

        let status_response: SeedanceStatusResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        let status = match status_response.status.as_str() {
            "succeeded" | "completed" => TaskStatus::Completed,
            "failed" => TaskStatus::Failed,
            "running" => TaskStatus::Processing,
            _ => TaskStatus::Pending,
        };

        let result = if status == TaskStatus::Completed && status_response.video.is_some() {
            let video = status_response.video.unwrap();
            Some(MediaOutput {
                media_type: MediaType::Video,
                provider: "seedance".to_string(),
                url: Some(video.play_addr),
                file_path: None,
                ipfs_cid: None,
                metadata: serde_json::to_value(MediaMetadata {
                    duration: video.duration,
                    width: video.width,
                    height: video.height,
                    format: Some("mp4".to_string()),
                    prompt: None,
                }).unwrap_or_default(),
            })
        } else {
            None
        };

        let error = if status == TaskStatus::Failed {
            status_response.error.map(|e| e.message)
        } else {
            None
        };

        Ok(MediaTask {
            task_id: task_id.to_string(),
            provider: "seedance".to_string(),
            media_type: MediaType::Video,
            status,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            result,
            error,
            progress: status_response.progress.unwrap_or(0) as f32,
        })
    }

    /// 等待任务完成
    pub async fn wait_for_completion(
        &self,
        task_id: String,
        interval_secs: u64,
        timeout_secs: u64,
    ) -> Result<MediaTask> {
        let start_time = std::time::Instant::now();

        loop {
            if start_time.elapsed().as_secs() >= timeout_secs {
                return Err(AgentError::ExternalApiError(
                    format!("视频生成超时（{}秒）", timeout_secs)
                ));
            }

            let task = self.get_status(&task_id).await?;

            match task.status {
                TaskStatus::Completed => return Ok(task),
                TaskStatus::Failed => {
                    return Err(AgentError::ExternalApiError(
                        task.error.unwrap_or_else(|| "视频生成失败".to_string())
                    ));
                }
                _ => {}
            }

            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }
}

#[async_trait]
impl MediaProvider for SeedanceProvider {
    fn name(&self) -> &str {
        "seedance"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![MediaType::Video]
    }

    async fn generate_video(&self, options: VideoOptions) -> Result<MediaTask> {
        log::info!("[Seedance] 开始生成视频");

        let task_id = self.create_video_task(
            options.prompt,
            options.model,
            options.duration.map(|d| d as u32),
            None,
            Some("adaptive".to_string()),
        ).await?;

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
        self.get_status(task_id).await
    }

    async fn generate_image(&self, _options: crate::agent::providers::media_provider::ImageOptions) -> Result<MediaOutput> {
        Err(AgentError::InvalidInput("Seedance 不支持图片生成".to_string()))
    }

    async fn generate_audio(&self, _options: crate::agent::providers::media_provider::AudioOptions) -> Result<MediaOutput> {
        Err(AgentError::InvalidInput("Seedance 不支持单独音频生成".to_string()))
    }
}
