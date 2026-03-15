//! Google Imagen Provider 实现

use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::{
    MediaProvider, MediaType, MediaOutput, MediaMetadata,
    ImageOptions,
};
use crate::agent::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Google 配置
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub api_key: String,
    pub project_id: String,
    pub location: String,
}

pub struct GoogleProvider {
    config: GoogleConfig,
}

impl GoogleProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let project_id = config.base_url.clone()
            .ok_or_else(|| AgentError::ConfigError("Google requires project_id (put in base_url field)".to_string()))?;
        
        let location = config.config
            .as_ref()
            .and_then(|c| c["location"].as_str())
            .unwrap_or("us-central1");

        let google_config = GoogleConfig {
            api_key: config.api_key.clone(),
            project_id: project_id.clone(),
            location: location.to_string(),
        };

        Ok(Self {
            config: google_config,
        })
    }
}

#[async_trait]
impl MediaProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image]
    }

    async fn generate_image(
        &self,
        options: ImageOptions,
    ) -> Result<MediaOutput> {
        let client = Client::new();
        
        #[derive(Serialize)]
        struct ImagenRequest {
            instances: Vec<ImagenInstance>,
            parameters: ImagenParameters,
        }

        #[derive(Serialize)]
        struct ImagenInstance {
            prompt: String,
        }

        #[derive(Serialize)]
        struct ImagenParameters {
            sample_count: u32,
            aspect_ratio: String,
            safety_setting: String,
        }

        let request = ImagenRequest {
            instances: vec![ImagenInstance { prompt: options.prompt }],
            parameters: ImagenParameters {
                sample_count: 1,
                aspect_ratio: options.aspect_ratio.unwrap_or_else(|| "1:1".to_string()),
                safety_setting: "block_some".to_string(),
            },
        };

        let url = format!(
            "https://us-central1-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/imagegeneration@006:predict",
            self.config.project_id, self.config.location
        );

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_token().await?))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("Google Imagen API error: {}", error)
            ));
        }

        #[derive(Deserialize)]
        struct ImagenResponse {
            predictions: Vec<ImagenPrediction>,
        }

        #[derive(Deserialize)]
        struct ImagenPrediction {
            bytes_base64_encoded: String,
            mime_type: String,
        }

        let result: ImagenResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

        if let Some(prediction) = result.predictions.first() {
            let image_bytes = base64::decode(&prediction.bytes_base64_encoded)
                .map_err(|e| AgentError::ExternalApiError(e.to_string()))?;

            // 保存到本地文件
            let file_path = crate::agent::media::storage::save_media(
                &image_bytes,
                "image",
                "png",
            )?;

            Ok(MediaOutput {
                media_type: MediaType::Image,
                provider: self.name().to_string(),
                url: None,
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
        } else {
            Err(AgentError::ExternalApiError("No image generated".to_string()))
        }
    }
}

impl GoogleProvider {
    async fn get_token(&self) -> Result<String> {
        // 简化版本：直接使用 API key
        // 生产环境应该使用 Google OAuth
        Ok(self.config.api_key.clone())
    }
}
