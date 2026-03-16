//! Suno AI Music Provider 实现
//!
//! Suno AI 音乐生成 API
//!
//! 官方文档：https://docs.sunoapi.org/
//!
//! API 端点：
//! - 音乐生成：POST https://api.sunoapi.org/generate
//! - 歌词生成：POST /lyrics/generate
//! - 获取状态：GET /music/{id}
//!
//! 支持模型：
//! - V4: 增强音质，4 分钟
//! - V4_5: 快速生成，8 分钟
//! - V4_5PLUS: 增强音调变化，8 分钟
//! - V5: 最新模型

use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::{
    MediaProvider, MediaType, MediaOutput, MediaMetadata,
    MediaTask, TaskStatus,
    AudioOptions,
};
use crate::agent::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Suno 配置
#[derive(Debug, Clone)]
pub struct SunoConfig {
    pub api_key: String,
    pub base_url: String,
}

impl SunoConfig {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.sunoapi.org".to_string()),
        }
    }
}

/// 音乐生成请求
#[derive(Debug, Serialize)]
pub struct SunoGenerateRequest {
    #[serde(rename = "prompt")]
    pub prompt: String,

    #[serde(rename = "tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    #[serde(rename = "title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "make_instrumental", skip_serializing_if = "Option::is_none")]
    pub make_instrumental: Option<bool>,

    #[serde(rename = "model", skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(rename = "continue_clip_id", skip_serializing_if = "Option::is_none")]
    pub continue_clip_id: Option<String>,

    #[serde(rename = "continue_at", skip_serializing_if = "Option::is_none")]
    pub continue_at: Option<f32>,
}

/// 生成响应
#[derive(Debug, Deserialize)]
pub struct SunoGenerateResponse {
    #[serde(rename = "clips")]
    pub clips: Option<Vec<SunoClip>>,

    #[serde(rename = "id")]
    pub id: Option<String>,

    #[serde(rename = "status")]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SunoClip {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "audio_url")]
    pub audio_url: Option<String>,

    #[serde(rename = "video_url")]
    pub video_url: Option<String>,

    #[serde(rename = "title")]
    pub title: Option<String>,

    #[serde(rename = "duration")]
    pub duration: Option<f32>,

    #[serde(rename = "created_at")]
    pub created_at: Option<String>,
}

/// 状态查询响应
#[derive(Debug, Deserialize)]
pub struct SunoStatusResponse {
    #[serde(rename = "clips")]
    pub clips: Option<Vec<SunoClip>>,
}

pub struct SunoProvider {
    config: SunoConfig,
    client: Client,
}

impl SunoProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let suno_config = SunoConfig::new(
            config.api_key.clone(),
            config.base_url.clone(),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        Ok(Self {
            config: suno_config,
            client,
        })
    }

    /// 生成音乐
    pub async fn generate_music(
        &self,
        prompt: String,
        tags: Option<String>,
        title: Option<String>,
        make_instrumental: Option<bool>,
        model: Option<String>,
    ) -> Result<String> {
        let request = SunoGenerateRequest {
            prompt,
            tags,
            title,
            make_instrumental,
            model,
            continue_clip_id: None,
            continue_at: None,
        };

        let url = format!("{}/generate", self.config.base_url);

        log::info!("[Suno] 生成音乐，URL: {}", url);

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
                format!("Suno API 错误 ({}): {}", status, error)
            ));
        }

        let result: SunoGenerateResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        // 获取任务 ID
        let task_id = result.id
            .or_else(|| result.clips.as_ref().and_then(|clips| clips.first().map(|c| c.id.clone())))
            .ok_or_else(|| AgentError::ExternalApiError("响应中没有任务 ID".to_string()))?;

        log::info!("[Suno] 任务创建成功，task_id: {}", task_id);

        Ok(task_id)
    }

    /// 查询任务状态
    pub async fn get_status(&self, task_id: &str) -> Result<MediaTask> {
        let url = format!("{}/music/{}", self.config.base_url, task_id);

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

        let status_response: SunoStatusResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        let clip = status_response.clips
            .and_then(|clips| clips.into_iter().find(|c| c.id == task_id))
            .ok_or_else(|| AgentError::ExternalApiError("未找到任务".to_string()))?;

        let status = match clip.status.as_str() {
            "complete" | "completed" => TaskStatus::Completed,
            "error" => TaskStatus::Failed,
            _ => TaskStatus::Processing,
        };

        let result = if status == TaskStatus::Completed && clip.audio_url.is_some() {
            Some(MediaOutput {
                media_type: MediaType::Audio,
                provider: "suno".to_string(),
                url: clip.audio_url.clone(),
                file_path: None,
                ipfs_cid: None,
                metadata: MediaMetadata {
                    duration: clip.duration,
                    format: Some("mp3".to_string()),
                    prompt: clip.title,
                    ..Default::default()
                },
            })
        } else {
            None
        };

        Ok(MediaTask {
            task_id: task_id.to_string(),
            provider: "suno".to_string(),
            media_type: MediaType::Audio,
            status,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            result,
            error: if status == TaskStatus::Failed { Some("音乐生成失败".to_string()) } else { None },
            progress: if status == TaskStatus::Completed { 100.0 } else { 50.0 },
        })
    }

    /// 下载音频
    async fn download_audio(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("下载音频失败：{}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("下载音频失败 ({}): {}", status, error)
            ));
        }

        let audio_bytes = response
            .bytes()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("读取音频数据失败：{}", e)))?
            .to_vec();

        Ok(audio_bytes)
    }
}

#[async_trait]
impl MediaProvider for SunoProvider {
    fn name(&self) -> &str {
        "suno"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![MediaType::Audio]
    }

    async fn generate_audio(&self, options: AudioOptions) -> Result<MediaOutput> {
        log::info!("[Suno] 开始生成音乐");

        let task_id = self.generate_music(
            options.text,
            options.model.clone(),
            None, // title
            options.voice_id.map(|v| v == "instrumental"),
            options.model,
        ).await?;

        // 等待完成
        let mut attempts = 0;
        let max_attempts = 120; // 10 分钟

        while attempts < max_attempts {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let task = self.get_status(&task_id).await?;

            match task.status {
                TaskStatus::Completed => {
                    if let Some(output) = task.result {
                        if let Some(url) = output.url {
                            let audio_bytes = self.download_audio(&url).await?;
                            let file_path = crate::agent::media::storage::save_media(
                                &audio_bytes,
                                "audio",
                                "mp3",
                            )?;

                            return Ok(MediaOutput {
                                media_type: MediaType::Audio,
                                provider: self.name().to_string(),
                                url: Some(url),
                                file_path: Some(file_path),
                                ipfs_cid: None,
                                metadata: output.metadata,
                            });
                        }
                    }
                    break;
                }
                TaskStatus::Failed => {
                    return Err(AgentError::ExternalApiError(
                        task.error.unwrap_or_else(|| "音乐生成失败".to_string())
                    ));
                }
                _ => {}
            }

            attempts += 1;
        }

        Err(AgentError::ExternalApiError("音乐生成超时".to_string()))
    }

    async fn generate_image(&self, _options: crate::agent::providers::media_provider::ImageOptions) -> Result<MediaOutput> {
        Err(AgentError::InvalidInput("Suno 不支持图片生成".to_string()))
    }

    async fn generate_video(&self, _options: crate::agent::providers::media_provider::VideoOptions) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("Suno 不支持视频生成".to_string()))
    }

    async fn get_task_status(&self, task_id: &str) -> Result<MediaTask> {
        self.get_status(task_id).await
    }
}
