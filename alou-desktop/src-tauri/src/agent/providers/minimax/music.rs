//! MiniMax 音乐生成实现
//!
//! 参考官方文档：https://platform.minimaxi.com/docs/api-reference/music-generation
//!
//! API 端点：POST https://api.minimaxi.com/v1/music_generation
//!
//! 支持模型：
//! - music-2.5+（推荐）：支持纯音乐模式
//! - music-2.5
//!
//! 认证方式：Bearer Token (API Key)

use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::{
    MediaProvider, MediaType, MediaOutput, MediaMetadata, MediaTask, TaskStatus,
    AudioOptions, VideoOptions,
};
use crate::agent::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// MiniMax 音乐配置
#[derive(Debug, Clone)]
pub struct MiniMaxMusicConfig {
    pub api_key: String,
    pub group_id: String,
    pub base_url: String,
}

impl MiniMaxMusicConfig {
    pub fn new(api_key: String, group_id: String) -> Self {
        Self {
            api_key,
            group_id,
            base_url: "https://api.minimaxi.com".to_string(),
        }
    }
}

/// 音乐生成请求
#[derive(Debug, Serialize)]
pub struct MusicGenerationRequest {
    #[serde(rename = "model")]
    pub model: String,

    #[serde(rename = "prompt", skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    #[serde(rename = "lyrics", skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,

    #[serde(rename = "stream", skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(rename = "output_format", skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,

    #[serde(rename = "audio_setting", skip_serializing_if = "Option::is_none")]
    pub audio_setting: Option<AudioSetting>,

    #[serde(rename = "aigc_watermark", skip_serializing_if = "Option::is_none")]
    pub aigc_watermark: Option<bool>,

    #[serde(rename = "lyrics_optimizer", skip_serializing_if = "Option::is_none")]
    pub lyrics_optimizer: Option<bool>,

    #[serde(rename = "is_instrumental", skip_serializing_if = "Option::is_none")]
    pub is_instrumental: Option<bool>,
}

/// 音频设置
#[derive(Debug, Serialize)]
pub struct AudioSetting {
    #[serde(rename = "sample_rate", skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,

    #[serde(rename = "bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,

    #[serde(rename = "format", skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// 音乐生成响应
#[derive(Debug, Deserialize)]
pub struct MusicGenerationResponse {
    #[serde(rename = "data")]
    pub data: MusicData,

    #[serde(rename = "trace_id", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    #[serde(rename = "extra_info", skip_serializing_if = "Option::is_none")]
    pub extra_info: Option<ExtraInfo>,

    #[serde(rename = "analysis_info", skip_serializing_if = "Option::is_none")]
    pub analysis_info: Option<serde_json::Value>,

    #[serde(rename = "base_resp")]
    pub base_resp: BaseResp,
}

#[derive(Debug, Deserialize)]
pub struct MusicData {
    #[serde(rename = "audio")]
    pub audio: String,

    #[serde(rename = "status")]
    pub status: u32,
}

#[derive(Debug, Deserialize)]
pub struct ExtraInfo {
    #[serde(rename = "music_duration", skip_serializing_if = "Option::is_none")]
    pub music_duration: Option<u32>,

    #[serde(rename = "music_sample_rate", skip_serializing_if = "Option::is_none")]
    pub music_sample_rate: Option<u32>,

    #[serde(rename = "music_channel", skip_serializing_if = "Option::is_none")]
    pub music_channel: Option<u32>,

    #[serde(rename = "bitrate", skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,

    #[serde(rename = "music_size", skip_serializing_if = "Option::is_none")]
    pub music_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BaseResp {
    #[serde(rename = "status_code")]
    pub status_code: u32,

    #[serde(rename = "status_msg")]
    pub status_msg: String,
}

/// MiniMax 音乐 Provider
pub struct MiniMaxMusicProvider {
    config: MiniMaxMusicConfig,
    client: Client,
}

impl MiniMaxMusicProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        // MiniMax 使用 base_url 字段存储 group_id
        let group_id = config.base_url.clone()
            .unwrap_or_default();

        let minimax_config = MiniMaxMusicConfig::new(
            config.api_key.clone(),
            group_id,
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();

        Ok(Self {
            config: minimax_config,
            client,
        })
    }

    /// 生成音乐
    pub async fn generate_music(
        &self,
        prompt: String,
        lyrics: Option<String>,
        model: Option<String>,
        is_instrumental: Option<bool>,
    ) -> Result<Vec<u8>> {
        let model_name = model.unwrap_or_else(|| "music-2.5+".to_string());

        // 根据模型版本和模式决定参数
        let (final_prompt, final_lyrics) = if is_instrumental.unwrap_or(false) {
            // 纯音乐模式：prompt 必填，lyrics 不填
            if prompt.is_empty() {
                return Err(AgentError::InvalidInput(
                    "纯音乐模式需要提供".to_string()
                ));
            }
            (Some(prompt), None)
        } else if !lyrics.as_ref().map(|l| l.is_empty()).unwrap_or(true) {
            // 有歌词模式：lyrics 必填，prompt 可选
            (None, lyrics)
        } else {
            // 默认情况：使用 prompt 作为音乐描述
            (Some(prompt), None)
        };

        let request = MusicGenerationRequest {
            model: model_name,
            prompt: final_prompt,
            lyrics: final_lyrics,
            stream: Some(false),
            output_format: Some("hex".to_string()),
            audio_setting: Some(AudioSetting {
                sample_rate: Some(44100),
                bitrate: Some(256000),
                format: Some("mp3".to_string()),
            }),
            aigc_watermark: Some(false),
            lyrics_optimizer: None,
            is_instrumental,
        };

        let url = format!("{}/v1/music_generation", self.config.base_url);

        log::info!("[MiniMax Music] 生成音乐，URL: {}", url);
        log::debug!("[MiniMax Music] 请求参数：{:?}", request);

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
            log::error!("[MiniMax Music] HTTP 错误：{}", error);
            return Err(AgentError::ExternalApiError(
                format!("MiniMax Music API 错误 ({}): {}", status, error)
            ));
        }

        let response_text = response.text().await
            .map_err(|e| AgentError::ExternalApiError(format!("读取响应失败：{}", e)))?;

        log::debug!("[MiniMax Music] 响应内容：{}", response_text);

        let generation_response: MusicGenerationResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        // 检查业务状态码
        if generation_response.base_resp.status_code != 0 {
            log::error!("[MiniMax Music] 业务错误：code={}, msg={}", 
                generation_response.base_resp.status_code,
                generation_response.base_resp.status_msg
            );
            return Err(AgentError::ExternalApiError(
                format!("MiniMax Music 错误 ({}): {}", 
                    generation_response.base_resp.status_code,
                    generation_response.base_resp.status_msg
                )
            ));
        }

        // 检查数据状态
        if generation_response.data.status != 2 {
            return Err(AgentError::ExternalApiError(
                format!("MiniMax Music 生成失败，status: {}", generation_response.data.status)
            ));
        }

        // 解析 hex 编码的音频数据
        let audio_hex = &generation_response.data.audio;
        let audio_bytes = hex::decode(audio_hex.trim())
            .map_err(|e| AgentError::ExternalApiError(format!("音频数据解码失败：{}", e)))?;

        log::info!("[MiniMax Music] 音乐生成成功，音频大小：{} bytes", audio_bytes.len());

        if let Some(extra) = &generation_response.extra_info {
            log::info!("[MiniMax Music] 音频信息 - 时长：{}ms, 采样率：{}Hz, 声道：{}",
                extra.music_duration.unwrap_or(0),
                extra.music_sample_rate.unwrap_or(0),
                extra.music_channel.unwrap_or(0)
            );
        }

        Ok(audio_bytes)
    }
}

#[async_trait]
impl MediaProvider for MiniMaxMusicProvider {
    fn name(&self) -> &str {
        "minimax_music"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![MediaType::Audio]
    }

    async fn generate_audio(&self, options: AudioOptions) -> Result<MediaOutput> {
        log::info!("[MiniMax Music] 开始生成音乐");

        // 从 options 中提取参数
        // text: 用作 prompt（音乐风格描述）
        // lyrics: 歌词内容
        // model: 模型版本
        // is_instrumental: 是否纯音乐模式
        let prompt = options.text.clone();
        let lyrics = options.lyrics.clone();
        let model = options.model.clone();
        let is_instrumental = options.is_instrumental;

        let audio_bytes = self.generate_music(
            prompt,
            lyrics,
            model,
            is_instrumental,
        ).await?;

        // 保存到本地文件
        let file_path = crate::agent::media::storage::save_media(
            &audio_bytes,
            "audio",
            "mp3",
        )?;

        // 计算时长（估算）
        let duration_secs = audio_bytes.len() as f32 / 16000.0;

        Ok(MediaOutput {
            media_type: MediaType::Audio,
            provider: self.name().to_string(),
            url: None,
            file_path: Some(file_path),
            ipfs_cid: None,
            metadata: MediaMetadata {
                duration_secs: Some(duration_secs),
                format: Some("mp3".to_string()),
                model: options.model,
                ..Default::default()
            },
        })
    }

    async fn generate_image(&self, _options: crate::agent::providers::media_provider::ImageOptions) -> Result<MediaOutput> {
        Err(AgentError::InvalidInput("MiniMax Music 不支持图片生成".to_string()))
    }

    async fn generate_video(&self, _options: crate::agent::providers::media_provider::VideoOptions) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("MiniMax Music 不支持视频生成".to_string()))
    }

    async fn get_task_status(&self, _task_id: &str) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("MiniMax Music 不支持任务状态查询".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::providers::media_provider::AudioOptions;

    #[tokio::test]
    #[ignore]
    async fn test_generate_music_instrumental() {
        let config = ProviderConfig {
            api_key: std::env::var("MINIMAX_API_KEY").unwrap_or_default(),
            base_url: Some(std::env::var("MINIMAX_GROUP_ID").unwrap_or_default()),
            model: None,
            enabled: true,
            capabilities: vec![],
            config: None,
        };

        let provider = MiniMaxMusicProvider::new(&config).unwrap();

        // 测试纯音乐生成
        let options = AudioOptions {
            text: "一首抒情的钢琴曲，带有柔和的弦乐伴奏".to_string(),
            voice_id: None,
            model: Some("music-2.5+".to_string()),
            speed: None,
            pitch: None,
            volume: None,
            lyrics: None,
            is_instrumental: Some(true),
        };

        let result = provider.generate_audio(options).await;
        println!("纯音乐生成结果：{:?}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate_music_with_lyrics() {
        let config = ProviderConfig {
            api_key: std::env::var("MINIMAX_API_KEY").unwrap_or_default(),
            base_url: Some(std::env::var("MINIMAX_GROUP_ID").unwrap_or_default()),
            model: None,
            enabled: true,
            capabilities: vec![],
            config: None,
        };

        let provider = MiniMaxMusicProvider::new(&config).unwrap();

        // 测试带歌词的音乐生成
        let options = AudioOptions {
            text: "流行摇滚风格，快节奏".to_string(),
            voice_id: None,
            model: Some("music-2.5+".to_string()),
            speed: None,
            pitch: None,
            volume: None,
            lyrics: Some("Verse 1\n今天天气真好\n心情也很棒\n\nChorus\n让我们一起唱歌\n一起跳舞".to_string()),
            is_instrumental: None,
        };

        let result = provider.generate_audio(options).await;
        println!("带歌词音乐生成结果：{:?}", result);
    }
}
