//! 海绵音乐 (Haimian Music) Provider 实现
//! 
//! 海绵音乐是字节跳动推出的 AI 音乐生成平台
//! 
//! 官方网站：https://www.haimianyinyue.com/
//! 
//! API 端点（参考）：
//! - 音乐生成：POST https://api.haimianyinyue.com/v1/music/generation
//! - 状态查询：GET https://api.haimianyinyue.com/v1/music/status
//! - 音乐下载：GET https://api.haimianyinyue.com/v1/music/download
//! 
//! 认证方式：Bearer Token (API Key)
//! 
//! 注意：具体 API 文档请参考字节跳动开发者平台
//! https://developer.byteplus.com

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

/// 海绵音乐配置
#[derive(Debug, Clone)]
pub struct HaimianConfig {
    pub api_key: String,
    pub api_secret: Option<String>,
    pub base_url: String,
}

impl HaimianConfig {
    pub fn new(api_key: String, api_secret: Option<String>) -> Self {
        Self {
            api_key,
            api_secret,
            base_url: "https://api.haimianyinyue.com".to_string(),
        }
    }
}

/// 音乐生成请求
#[derive(Debug, Serialize)]
pub struct MusicGenerationRequest {
    #[serde(rename = "model")]
    pub model: String,
    
    #[serde(rename = "prompt")]
    pub prompt: String,
    
    #[serde(rename = "style", skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    
    #[serde(rename = "mood", skip_serializing_if = "Option::is_none")]
    pub mood: Option<String>,
    
    #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    
    #[serde(rename = "with_vocals", skip_serializing_if = "Option::is_none")]
    pub with_vocals: Option<bool>,
    
    #[serde(rename = "lyrics", skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    
    #[serde(rename = "genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    
    #[serde(rename = "tempo", skip_serializing_if = "Option::is_none")]
    pub tempo: Option<u32>,
}

/// 音乐生成响应
#[derive(Debug, Deserialize)]
pub struct MusicGenerationResponse {
    #[serde(rename = "code")]
    pub code: u32,
    
    #[serde(rename = "msg")]
    pub msg: String,
    
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<MusicTaskData>,
    
    #[serde(rename = "request_id", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MusicTaskData {
    #[serde(rename = "task_id")]
    pub task_id: String,
    
    #[serde(rename = "status")]
    pub status: String,
    
    #[serde(rename = "music_url", skip_serializing_if = "Option::is_none")]
    pub music_url: Option<String>,
    
    #[serde(rename = "progress", skip_serializing_if = "Option::is_none")]
    pub progress: Option<u32>,
    
    #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
}

/// 状态查询响应
#[derive(Debug, Deserialize)]
pub struct MusicStatusResponse {
    #[serde(rename = "code")]
    pub code: u32,
    
    #[serde(rename = "msg")]
    pub msg: String,
    
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<MusicStatusData>,
}

#[derive(Debug, Deserialize)]
pub struct MusicStatusData {
    #[serde(rename = "task_id")]
    pub task_id: String,
    
    #[serde(rename = "status")]
    pub status: String,
    
    #[serde(rename = "music_url", skip_serializing_if = "Option::is_none")]
    pub music_url: Option<String>,
    
    #[serde(rename = "download_url", skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    
    #[serde(rename = "progress", skip_serializing_if = "Option::is_none")]
    pub progress: Option<u32>,
    
    #[serde(rename = "error_message", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    
    #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
    
    #[serde(rename = "style", skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    
    #[serde(rename = "format", skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

pub struct HaimianMusicProvider {
    config: HaimianConfig,
    client: Client,
}

impl HaimianMusicProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let api_secret = config.base_url.clone();

        let haimian_config = HaimianConfig::new(
            config.api_key.clone(),
            api_secret,
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        Ok(Self {
            config: haimian_config,
            client,
        })
    }

    pub async fn generate_music(
        &self,
        prompt: String,
        style: Option<String>,
        duration: Option<u32>,
    ) -> Result<String> {
        let request = MusicGenerationRequest {
            model: "haimian-music-v1".to_string(),
            prompt,
            style,
            mood: None,
            duration,
            with_vocals: Some(true),
            lyrics: None,
            genre: None,
            tempo: None,
        };

        let url = format!("{}/v1/music/generation", self.config.base_url);

        log::info!("[Haimian Music] 生成音乐，URL: {}", url);

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
                format!("Haimian Music API 错误 ({}): {}", status, error)
            ));
        }

        let response_text = response.text().await
            .map_err(|e| AgentError::ExternalApiError(format!("读取响应失败：{}", e)))?;

        let generation_response: MusicGenerationResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        if generation_response.code != 0 {
            return Err(AgentError::ExternalApiError(
                format!("Haimian Music 错误 ({}): {}", generation_response.code, generation_response.msg)
            ));
        }

        let task_id = generation_response
            .data
            .ok_or_else(|| AgentError::ExternalApiError("响应中没有任务数据".to_string()))?
            .task_id;

        log::info!("[Haimian Music] 任务创建成功，task_id: {}", task_id);

        Ok(task_id)
    }

    pub async fn get_status(&self, task_id: String) -> Result<MediaTask> {
        let url = format!("{}/v1/music/status?task_id={}", self.config.base_url, task_id);

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

        let status_response: MusicStatusResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        if status_response.code != 0 {
            return Err(AgentError::ExternalApiError(
                format!("查询状态错误 ({}): {}", status_response.code, status_response.msg)
            ));
        }

        let status_data = status_response
            .data
            .ok_or_else(|| AgentError::ExternalApiError("响应中没有状态数据".to_string()))?;

        let status = match status_data.status.as_str() {
            "completed" => TaskStatus::Completed,
            "failed" => TaskStatus::Failed,
            "processing" => TaskStatus::Processing,
            _ => TaskStatus::Pending,
        };

        let result = if status == TaskStatus::Completed && status_data.music_url.is_some() {
            Some(MediaOutput {
                media_type: MediaType::Audio,
                provider: "haimian".to_string(),
                url: status_data.music_url.clone(),
                file_path: None,
                ipfs_cid: None,
                metadata: MediaMetadata {
                    duration_secs: status_data.duration,
                    format: status_data.format.clone().or(Some("mp3".to_string())),
                    prompt: None,
                    ..Default::default()
                },
            })
        } else {
            None
        };

        let error = if status == TaskStatus::Failed {
            status_data.error_message.clone().or(Some("音乐生成失败".to_string()))
        } else {
            None
        };

        Ok(MediaTask {
            task_id,
            provider: "haimian".to_string(),
            media_type: MediaType::Audio,
            status,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            result,
            error,
            progress: status_data.progress.unwrap_or(0) as f32,
        })
    }

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
                    format!("音乐生成超时（{}秒）", timeout_secs)
                ));
            }

            let task = self.get_status(task_id.clone()).await?;

            match task.status {
                TaskStatus::Completed => return Ok(task),
                TaskStatus::Failed => {
                    return Err(AgentError::ExternalApiError(
                        task.error.unwrap_or_else(|| "音乐生成失败".to_string())
                    ));
                }
                _ => {}
            }

            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }

    async fn download_music(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("下载音乐失败：{}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("下载音乐失败 ({}): {}", status, error)
            ));
        }

        let music_bytes = response
            .bytes()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("读取音乐数据失败：{}", e)))?
            .to_vec();

        Ok(music_bytes)
    }
}

#[async_trait]
impl MediaProvider for HaimianMusicProvider {
    fn name(&self) -> &str {
        "haimian"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![MediaType::Audio]
    }

    async fn generate_audio(&self, options: AudioOptions) -> Result<MediaOutput> {
        log::info!("[Haimian Music] 开始生成音乐");

        let task_id = self.generate_music(
            options.text.clone(),
            options.model.clone(),
            options.speed.map(|s| (s * 60.0) as u32),
        ).await?;

        let task = self.wait_for_completion(task_id, 5, 300).await?;

        if let Some(output) = task.result {
            if let Some(url) = output.url.clone() {
                let music_bytes = self.download_music(&url).await?;
                let file_path = crate::agent::media::storage::save_media(
                    &music_bytes,
                    "audio",
                    "mp3",
                )?;

                return Ok(MediaOutput {
                    media_type: MediaType::Audio,
                    provider: self.name().to_string(),
                    url: Some(url),
                    file_path: Some(file_path),
                    ipfs_cid: None,
                    metadata: MediaMetadata {
                        duration_secs: output.metadata.duration_secs,
                        format: output.metadata.format,
                        model: options.model,
                        ..Default::default()
                    },
                });
            }
        }

        Err(AgentError::ExternalApiError("音乐生成失败，未返回有效结果".to_string()))
    }

    async fn generate_image(&self, _options: crate::agent::providers::media_provider::ImageOptions) -> Result<MediaOutput> {
        Err(AgentError::InvalidInput("Haimian Music 不支持图片生成".to_string()))
    }

    async fn generate_video(&self, _options: crate::agent::providers::media_provider::VideoOptions) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("Haimian Music 不支持视频生成".to_string()))
    }

    async fn get_task_status(&self, _task_id: &str) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("get_task_status 仅支持视频任务".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_generate_music() {
        let config = ProviderConfig {
            api_key: std::env::var("HAIMIAN_API_KEY").unwrap_or_default(),
            base_url: None,
            model: None,
            enabled: true,
            capabilities: vec![],
            config: None,
        };

        let provider = HaimianMusicProvider::new(&config).unwrap();
        
        let options = AudioOptions {
            text: "创作一首抒情的钢琴曲".to_string(),
            voice_id: None,
            model: Some("pop".to_string()),
            speed: None,
            pitch: None,
            volume: None,
        };

        let result = provider.generate_audio(options).await;
        println!("结果：{:?}", result);
    }
}
