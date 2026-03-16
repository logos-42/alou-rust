//! Google Imagen 图片生成实现
//! 
//! 参考文档：
//! - https://cloud.google.com/vertex-ai/docs/generative-ai/image/overview
//! - https://cloud.google.com/vertex-ai/docs/generative-ai/image/generate-images
//! 
//! API 端点：
//! POST https://{LOCATION}-aiplatform.googleapis.com/v1/projects/{PROJECT_ID}/locations/{LOCATION}/publishers/google/models/imagegeneration@006:predict
//! 
//! 认证方式：
//! - 使用 Google Cloud OAuth 2.0
//! - 或使用 API Key（简化方式）
//! 
//! 支持的功能：
//! - 文本到图像生成
//! - 图像编辑
//! - 多种分辨率和纵横比

use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::{
    MediaProvider, MediaType, MediaOutput, MediaMetadata, MediaTask, TaskStatus,
    ImageOptions, AudioOptions, VideoOptions,
};
use crate::agent::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Google Imagen 配置
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub api_key: String,
    pub project_id: String,
    pub location: String,
}

pub struct GoogleProvider {
    config: GoogleConfig,
    client: Client,
}

impl GoogleProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let project_id = config.base_url.clone()
            .ok_or_else(|| AgentError::ConfigError(
                "Google Imagen 需要配置 project_id（填入 base_url 字段）".to_string()
            ))?;

        let location = config.config
            .as_ref()
            .and_then(|c| c["location"].as_str())
            .unwrap_or("us-central1");

        let google_config = GoogleConfig {
            api_key: config.api_key.clone(),
            project_id: project_id.clone(),
            location: location.to_string(),
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Ok(Self {
            config: google_config,
            client,
        })
    }

    /// 获取 API 端点 URL
    fn get_endpoint_url(&self) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/imagegeneration@006:predict",
            self.config.location,
            self.config.project_id,
            self.config.location
        )
    }

    /// 获取认证 Token
    async fn get_token(&self) -> Result<String> {
        // 简化版本：直接使用 API key
        // 生产环境应该使用 Google OAuth 2.0
        Ok(self.config.api_key.clone())
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
        log::info!("[Google Imagen] 开始生成图片");
        log::debug!("[Google Imagen] 参数：{:?}", options);

        // 构建请求
        let request = ImagenRequest {
            instances: vec![ImagenInstance {
                prompt: options.prompt.clone(),
                negative_prompt: options.negative_prompt.clone(),
            }],
            parameters: ImagenParameters {
                sample_count: options.num_images.unwrap_or(1),
                aspect_ratio: options.aspect_ratio.clone().unwrap_or_else(|| "1:1".to_string()),
                safety_setting: "block_some".to_string(),
                add_watermark: Some(true),
                language: Some("en".to_string()),
            },
        };

        let url = self.get_endpoint_url();
        log::info!("[Google Imagen] 请求 URL: {}", url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_token().await?))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("[Google Imagen] 网络请求失败：{}", e);
                AgentError::ExternalApiError(format!("网络请求失败：{}", e))
            })?;

        let status = response.status();
        log::info!("[Google Imagen] 响应状态码：{}", status);

        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            log::error!("[Google Imagen] API 错误：{}", error);

            // 尝试解析 Google 错误格式
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error) {
                if let Some(error_obj) = error_json.get("error") {
                    let message = error_obj.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error");
                    return Err(AgentError::ExternalApiError(
                        format!("Google Imagen API 错误：{}", message)
                    ));
                }
            }

            return Err(AgentError::ExternalApiError(
                format!("Google Imagen API 错误 ({}): {}", status, error)
            ));
        }

        let response_text = response.text().await
            .map_err(|e| {
                log::error!("[Google Imagen] 读取响应失败：{}", e);
                AgentError::ExternalApiError(format!("读取响应失败：{}", e))
            })?;

        log::debug!("[Google Imagen] 响应内容：{}", response_text);

        let imagen_response: ImagenResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("[Google Imagen] 解析响应失败：{}, 响应内容：{}", e, response_text);
                AgentError::ExternalApiError(format!("解析响应失败：{}", e))
            })?;

        if imagen_response.predictions.is_empty() {
            log::error!("[Google Imagen] 没有生成图片");
            return Err(AgentError::ExternalApiError("没有生成图片".to_string()));
        }

        let prediction = &imagen_response.predictions[0];

        // 解码 base64 图片
        let image_bytes = base64::decode(&prediction.bytes_base64_encoded)
            .map_err(|e| {
                log::error!("[Google Imagen] 解码图片失败：{}", e);
                AgentError::ExternalApiError(format!("解码图片失败：{}", e))
            })?;

        // 保存到本地文件
        let file_path = crate::agent::media::storage::save_media(
            &image_bytes,
            "image",
            "png",
        )?;

        log::info!("[Google Imagen] 图片生成成功，保存到：{}", file_path);

        // 解析 MIME 类型获取格式
        let format = prediction.mime_type
            .split('/')
            .nth(1)
            .unwrap_or("png")
            .to_string();

        Ok(MediaOutput {
            media_type: MediaType::Image,
            provider: self.name().to_string(),
            url: None,
            file_path: Some(file_path),
            ipfs_cid: None,
            metadata: MediaMetadata {
                width: options.width,
                height: options.height,
                format: Some(format),
                prompt: Some(options.prompt),
                duration: None,
                duration_secs: None,
                model: None,
            },
        })
    }

    async fn generate_audio(&self, _options: AudioOptions) -> Result<MediaOutput> {
        Err(AgentError::InvalidInput("Google provider does not support audio generation".to_string()))
    }

    async fn generate_video(&self, _options: VideoOptions) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("Google provider does not support video generation".to_string()))
    }

    async fn get_task_status(&self, _task_id: &str) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("Google provider does not support task status".to_string()))
    }
}

/// Imagen 请求结构
#[derive(Debug, Serialize)]
struct ImagenRequest {
    instances: Vec<ImagenInstance>,
    parameters: ImagenParameters,
}

#[derive(Debug, Serialize)]
struct ImagenInstance {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
struct ImagenParameters {
    sample_count: u32,
    aspect_ratio: String,
    safety_setting: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    add_watermark: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

/// Imagen 响应结构
#[derive(Debug, Deserialize)]
struct ImagenResponse {
    predictions: Vec<ImagenPrediction>,
}

#[derive(Debug, Deserialize)]
struct ImagenPrediction {
    bytes_base64_encoded: String,
    mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_attributes: Option<SafetyAttributes>,
}

#[derive(Debug, Deserialize)]
struct SafetyAttributes {
    #[serde(skip_serializing_if = "Option::is_none")]
    blocked: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::providers::media_provider::ImageOptions;

    #[tokio::test]
    #[ignore] // 需要真实 API Key，手动运行
    async fn test_generate_image() {
        let config = ProviderConfig {
            api_key: std::env::var("GOOGLE_API_KEY").unwrap_or_default(),
            base_url: Some(std::env::var("GOOGLE_PROJECT_ID").unwrap_or_default()),
            model: Some("imagegeneration@006".to_string()),
            enabled: true,
            capabilities: vec![],
            config: Some(serde_json::json!({
                "location": "us-central1"
            })),
        };

        let provider = GoogleProvider::new(&config).unwrap();
        
        let options = ImageOptions {
            prompt: "A beautiful sunset over the ocean".to_string(),
            width: Some(1024),
            height: Some(1024),
            aspect_ratio: Some("1:1".to_string()),
            model: None,
            negative_prompt: None,
            num_images: Some(1),
        };

        let result = provider.generate_image(options).await.unwrap();
        
        assert!(result.file_path.is_some());
        println!("图片路径：{:?}", result.file_path);
    }
}
