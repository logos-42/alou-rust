//! MiniMax TTS (文本转语音) 实现
//! 
//! 参考文档：https://platform.minimaxi.com/document
//! 
//! API 端点：
//! - TTS: POST https://api.minimax.chat/v1/audio/speech
//! - 声音列表：GET https://api.minimax.chat/v1/system/voices
//! 
//! 认证方式：Bearer Token (API Key)
//! 请求头：
//! - Authorization: Bearer {API_KEY}
//! - Content-Type: application/json


use async_trait::async_trait;

use super::MiniMaxConfig;
use crate::agent::error::{AgentError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// TTS 请求结构
/// 参考官方文档：https://platform.minimaxi.com/document
#[derive(Debug, Serialize)]
pub struct SpeechRequest {
    /// 模型名称
    /// - speech-01-turbo: 快速模型，适合实时场景
    /// - speech-01-standard: 标准模型，音质更好
    #[serde(rename = "model")]
    pub model: String,
    
    /// 要合成的文本内容
    #[serde(rename = "text")]
    pub text: String,
    
    /// 音色 ID，如：female-shaonv, male-qn-qingse
    #[serde(rename = "voice_id", skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    
    /// 语速，范围：0.5-2.0，默认 1.0
    #[serde(rename = "speed", skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    
    /// 音量，范围：0.1-10.0，默认 1.0
    #[serde(rename = "vol", skip_serializing_if = "Option::is_none")]
    pub vol: Option<f32>,
    
    /// 音调，范围：-1.0-1.0，默认 0.0
    #[serde(rename = "pitch", skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f32>,
    
    /// 输出格式：mp3, wav, pcm
    #[serde(rename = "format", skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    
    /// 采样率：8000, 16000, 22050, 24000, 32000, 44100, 48000
    #[serde(rename = "sample_rate", skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    
    /// 文本语言：zh(中文), en(英文), auto(自动检测)
    #[serde(rename = "lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    
    /// 是否启用文本归一化，默认 true
    #[serde(rename = "text_normalize", skip_serializing_if = "Option::is_none")]
    pub text_normalize: Option<bool>,
}

/// TTS 响应结构
#[derive(Debug, Deserialize)]
pub struct SpeechResponse {
    /// 错误码，0 表示成功
    #[serde(rename = "code")]
    pub code: u32,
    
    /// 错误信息
    #[serde(rename = "msg")]
    pub msg: String,
    
    /// 音频数据 (base64 编码)
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<SpeechData>,
    
    /// 请求 ID
    #[serde(rename = "request_id", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpeechData {
    /// 音频时长（秒）
    #[serde(rename = "duration")]
    pub duration: f32,
    
    /// 音频数据 (base64)
    #[serde(rename = "audio")]
    pub audio: String,
    
    /// 使用的音色 ID
    #[serde(rename = "voice_id", skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
}

/// 声音列表响应
#[derive(Debug, Deserialize)]
pub struct VoicesResponse {
    #[serde(rename = "code")]
    pub code: u32,
    
    #[serde(rename = "msg")]
    pub msg: String,
    
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<VoicesData>,
}

#[derive(Debug, Deserialize)]
pub struct VoicesData {
    #[serde(rename = "voices")]
    pub voices: Vec<VoiceInfo>,
}

/// 声音信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    /// 音色 ID
    #[serde(rename = "voice_id")]
    pub voice_id: String,
    
    /// 音色名称
    #[serde(rename = "name")]
    pub name: String,
    
    /// 性别：male, female
    #[serde(rename = "gender")]
    pub gender: String,
    
    /// 音色描述
    #[serde(rename = "description")]
    pub description: String,
    
    /// 适用场景
    #[serde(rename = "scene", skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,
}

pub struct MiniMaxTts {
    config: MiniMaxConfig,
    client: Client,
}

impl MiniMaxTts {
    pub fn new(config: MiniMaxConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        
        Self { config, client }
    }

    /// 文本转语音
    /// 
    /// # 参数
    /// * `text` - 要合成的文本内容
    /// * `voice_id` - 音色 ID，None 则使用默认音色
    /// 
    /// # 返回
    /// * `Ok(Vec<u8>)` - MP3 格式的音频数据
    /// * `Err(AgentError)` - 错误信息
    /// 
    /// # 示例
    /// ```rust
    /// let audio_bytes = tts.synthesize("你好，世界".to_string(), None).await?;
    /// ```
    pub async fn synthesize(
        &self,
        text: String,
        voice_id: Option<String>,
    ) -> Result<Vec<u8>> {
        let request = SpeechRequest {
            model: "speech-01-turbo".to_string(),
            text,
            voice_id,
            speed: Some(1.0),
            vol: Some(1.0),
            pitch: Some(0.0),
            format: Some("mp3".to_string()),
            sample_rate: Some(32000),
            lang: Some("auto".to_string()),
            text_normalize: Some(true),
        };

        // 官方 API 端点
        let url = format!("{}/v1/audio/speech", self.config.base_url);

        log::info!("[MiniMax TTS] 发送请求到：{}", url);
        log::debug!("[MiniMax TTS] 请求参数：{:?}", request);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("[MiniMax TTS] 网络请求失败：{}", e);
                AgentError::ExternalApiError(format!("网络请求失败：{}", e))
            })?;

        let status = response.status();
        log::info!("[MiniMax TTS] 响应状态码：{}", status);

        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            log::error!("[MiniMax TTS] API 错误：{}", error);
            return Err(AgentError::ExternalApiError(
                format!("MiniMax TTS API 错误 ({}): {}", status, error)
            ));
        }

        let response_text = response.text().await
            .map_err(|e| {
                log::error!("[MiniMax TTS] 读取响应失败：{}", e);
                AgentError::ExternalApiError(format!("读取响应失败：{}", e))
            })?;

        log::debug!("[MiniMax TTS] 响应内容：{}", response_text);

        // 解析 JSON 响应
        let speech_response: SpeechResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("[MiniMax TTS] 解析响应失败：{}, 响应内容：{}", e, response_text);
                AgentError::ExternalApiError(format!("解析响应失败：{}", e))
            })?;

        // 检查错误码
        if speech_response.code != 0 {
            log::error!("[MiniMax TTS] 业务错误：code={}, msg={}", 
                speech_response.code, speech_response.msg);
            return Err(AgentError::ExternalApiError(
                format!("MiniMax TTS 错误 ({}): {}", speech_response.code, speech_response.msg)
            ));
        }

        // 获取音频数据
        let audio_base64 = speech_response
            .data
            .ok_or_else(|| {
                log::error!("[MiniMax TTS] 响应中没有音频数据");
                AgentError::ExternalApiError("响应中没有音频数据".to_string())
            })?
            .audio;

        // 解码 base64
        let audio_bytes = base64::decode(&audio_base64)
            .map_err(|e| {
                log::error!("[MiniMax TTS] 解码音频失败：{}", e);
                AgentError::ExternalApiError(format!("解码音频失败：{}", e))
            })?;

        log::info!("[MiniMax TTS] 音频生成成功，大小：{} bytes", audio_bytes.len());

        Ok(audio_bytes)
    }

    /// 获取可用声音列表
    pub async fn list_voices(&self) -> Result<Vec<VoiceInfo>> {
        let url = format!("{}/v1/system/voices", self.config.base_url);

        log::info!("[MiniMax TTS] 获取声音列表：{}", url);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| {
                log::error!("[MiniMax TTS] 获取声音列表失败：{}", e);
                AgentError::ExternalApiError(format!("网络请求失败：{}", e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("获取声音列表失败 ({}): {}", status, error)
            ));
        }

        let voices_response: VoicesResponse = response
            .json()
            .await
            .map_err(|e| {
                log::error!("[MiniMax TTS] 解析声音列表失败：{}", e);
                AgentError::ExternalApiError(format!("解析响应失败：{}", e))
            })?;

        if voices_response.code != 0 {
            return Err(AgentError::ExternalApiError(
                format!("获取声音列表错误 ({}): {}", voices_response.code, voices_response.msg)
            ));
        }

        let voices = voices_response
            .data
            .map(|d| d.voices)
            .unwrap_or_default();

        log::info!("[MiniMax TTS] 获取到 {} 个声音", voices.len());

        Ok(voices)
    }

    /// 获取推荐音色（根据场景）
    pub fn get_recommended_voice(&self, scene: &str) -> &'static str {
        match scene {
            "narration" => "female-shaonv",      // 旁白：少女音
            "news" => "male-qn-qingse",          // 新闻：青年音色
            "story" => "female-wenrou",          // 故事：温柔女声
            "emotion" => "male-yujia",           // 情感：御姐音
            "child" => "female-child",           // 儿童：童声
            "anime" => "female-loli",            // 动漫：萝莉音
            _ => "female-shaonv",                 // 默认：少女音
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要真实 API Key，手动运行
    async fn test_synthesize() {
        let config = MiniMaxConfig {
            api_key: std::env::var("MINIMAX_API_KEY").unwrap_or_default(),
            group_id: std::env::var("MINIMAX_GROUP_ID").unwrap_or_default(),
            base_url: "https://api.minimax.chat".to_string(),
        };

        let tts = MiniMaxTts::new(config);
        
        let result = tts.synthesize("你好，这是 MiniMax TTS 测试".to_string(), None).await;
        assert!(result.is_ok());
        
        let audio_bytes = result.unwrap();
        assert!(audio_bytes.len() > 0);
        
        // 保存测试文件
        std::fs::write("/tmp/test_tts.mp3", audio_bytes).unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_voices() {
        let config = MiniMaxConfig {
            api_key: std::env::var("MINIMAX_API_KEY").unwrap_or_default(),
            group_id: std::env::var("MINIMAX_GROUP_ID").unwrap_or_default(),
            base_url: "https://api.minimax.chat".to_string(),
        };

        let tts = MiniMaxTts::new(config);
        let voices = tts.list_voices().await.unwrap();
        
        assert!(voices.len() > 0);
        
        for voice in voices {
            println!("音色：{} - {} ({})", voice.name, voice.voice_id, voice.gender);
        }
    }
}
