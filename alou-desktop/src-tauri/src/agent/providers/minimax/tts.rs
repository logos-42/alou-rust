//! MiniMax TTS (文本转语音)

use super::MiniMaxConfig;
use crate::agent::error::{AgentError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// TTS 请求
#[derive(Debug, Serialize)]
pub struct TtsRequest {
    pub model: String,
    pub text: String,
    pub voice_id: Option<String>,
    pub speed: Option<f32>,
    pub vol: Option<f32>,
    pub pitch: Option<f32>,
    pub format: String,
    pub sample_rate: u32,
}

pub struct MiniMaxTts {
    config: MiniMaxConfig,
}

impl MiniMaxTts {
    pub fn new(config: MiniMaxConfig) -> Self {
        Self { config }
    }

    /// 文本转语音
    pub async fn synthesize(
        &self,
        text: String,
        voice_id: Option<String>,
    ) -> Result<Vec<u8>> {
        let client = Client::new();
        
        let request = TtsRequest {
            model: "speech-01-turbo".to_string(),
            text,
            voice_id,
            speed: Some(1.0),
            vol: Some(1.0),
            pitch: Some(0.0),
            format: "mp3".to_string(),
            sample_rate: 32000,
        };

        let url = format!(
            "{}/audio/speech?GroupId={}",
            self.config.base_url, self.config.group_id
        );

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
                format!("MiniMax TTS API error: {}", error)
            ));
        }

        let audio_bytes = response
            .bytes()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        Ok(audio_bytes.to_vec())
    }

    /// 获取可用声音列表
    pub async fn list_voices(&self) -> Result<Vec<VoiceInfo>> {
        let client = Client::new();
        let url = format!("{}/voices/list?GroupId={}", self.config.base_url, self.config.group_id);

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::ExternalApiError(
                "Failed to list voices".to_string()
            ));
        }

        #[derive(Deserialize)]
        struct VoiceResponse {
            voices: Vec<VoiceInfo>,
        }

        let result: VoiceResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        Ok(result.voices)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub voice_id: String,
    pub name: String,
    pub gender: String,
    pub description: String,
}
