//! 即梦 Seedream Provider 实现
//!
//! 即梦 (Jimeng) Seedream 图片生成 API
//!
//! 官方文档：
//! - DeerAPI: https://apidoc.deerapi.com/seedream-%E5%9B%BE%E5%83%8F%E7%94%9F%E6%88%90-331149260e0
//! - 火山方舟：https://www.volcengine.com/docs/82379/1824121
//!
//! API 端点：
//! - 图片生成：POST https://api.deerapi.com/v1/images/generations
//!
//! 支持模型：
//! - doubao-seedream-4-0-250828: Seedream 4.0
//! - doubao-seedream-4-5-251128: Seedream 4.5
//! - doubao-seedream-5-0-260128: Seedream 5.0
//! - doubao-seedream-3-0-t2i-250415: Seedream 3.0 (text-to-image)
//!
//! 认证方式：Bearer Token (API Key)

use crate::agent::media_config::ProviderConfig;
use crate::agent::providers::media_provider::{
    MediaProvider, MediaType, MediaOutput, MediaMetadata,
    ImageOptions, MediaTask, TaskStatus,
};
use crate::agent::error::{AgentError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Seedream 配置
#[derive(Debug, Clone)]
pub struct SeedreamConfig {
    pub api_key: String,
    pub base_url: String,
    pub default_model: String,
}

impl SeedreamConfig {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.deerapi.com".to_string()),
            default_model: "doubao-seedream-5-0-260128".to_string(),
        }
    }
}

/// 图片生成请求
#[derive(Debug, Serialize)]
pub struct SeedreamImageRequest {
    #[serde(rename = "model")]
    pub model: String,

    #[serde(rename = "prompt")]
    pub prompt: String,

    #[serde(rename = "image", skip_serializing_if = "Option::is_none")]
    pub image: Option<Vec<String>>,

    #[serde(rename = "size", skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    #[serde(rename = "watermark", skip_serializing_if = "Option::is_none")]
    pub watermark: Option<bool>,
}

/// 图片生成响应
#[derive(Debug, Deserialize)]
pub struct SeedreamImageResponse {
    #[serde(rename = "model")]
    pub model: String,

    #[serde(rename = "created")]
    pub created: u64,

    #[serde(rename = "data")]
    pub data: Vec<ImageData>,

    #[serde(rename = "usage", skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ImageData {
    #[serde(rename = "url")]
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct UsageInfo {
    #[serde(rename = "generated_images")]
    pub generated_images: u32,

    #[serde(rename = "output_tokens")]
    pub output_tokens: u32,

    #[serde(rename = "total_tokens")]
    pub total_tokens: u32,
}

pub struct SeedreamProvider {
    config: SeedreamConfig,
    client: Client,
}

impl SeedreamProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let seedream_config = SeedreamConfig::new(
            config.api_key.clone(),
            config.base_url.clone(),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();

        Ok(Self {
            config: seedream_config,
            client,
        })
    }

    /// 生成图片
    pub async fn generate_image_request(
        &self,
        prompt: String,
        model: Option<String>,
        size: Option<String>,
        num_images: Option<u32>,
        reference_images: Option<Vec<String>>,
        watermark: Option<bool>,
    ) -> Result<Vec<String>> {
        let request = SeedreamImageRequest {
            model: model.unwrap_or_else(|| self.config.default_model.clone()),
            prompt,
            image: reference_images,
            size,
            n: num_images,
            watermark,
        };

        let url = format!("{}/v1/images/generations", self.config.base_url);

        log::info!("[Seedream] 生成图片，URL: {}", url);

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
                format!("Seedream API 错误 ({}): {}", status, error)
            ));
        }

        let result: SeedreamImageResponse = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        let urls: Vec<String> = result.data.iter().map(|img| img.url.clone()).collect();

        log::info!("[Seedream] 生成 {} 张图片", urls.len());

        Ok(urls)
    }

    /// 下载图片
    async fn download_image(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("下载图片失败：{}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("下载图片失败 ({}): {}", status, error)
            ));
        }

        let image_bytes = response
            .bytes()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("读取图片数据失败：{}", e)))?
            .to_vec();

        Ok(image_bytes)
    }
}

#[async_trait]
impl MediaProvider for SeedreamProvider {
    fn name(&self) -> &str {
        "seedream"
    }

    fn supported_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image]
    }

    async fn generate_image(&self, options: ImageOptions) -> Result<MediaOutput> {
        log::info!("[Seedream] 开始生成图片");

        let urls = self.generate_image_request(
            options.prompt.clone(),
            options.model,
            options.aspect_ratio.clone().map(|ar| "2K".to_string()),
            options.num_images,
            None,
            Some(false),
        ).await?;

        if urls.is_empty() {
            return Err(AgentError::ExternalApiError("未生成任何图片".to_string()));
        }

        let image_bytes = self.download_image(&urls[0]).await?;
        let file_path = crate::agent::media::storage::save_media(
            &image_bytes,
            "image",
            "png",
        )?;

        Ok(MediaOutput {
            media_type: MediaType::Image,
            provider: self.name().to_string(),
            url: Some(urls[0].clone()),
            file_path: Some(file_path),
            ipfs_cid: None,
            metadata: MediaMetadata::default(),
        })
    }

    async fn generate_audio(&self, _options: crate::agent::providers::media_provider::AudioOptions) -> Result<MediaOutput> {
        Err(AgentError::InvalidInput("Seedream 不支持音频生成".to_string()))
    }

    async fn generate_video(&self, _options: crate::agent::providers::media_provider::VideoOptions) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("Seedream 不支持视频生成".to_string()))
    }

    async fn get_task_status(&self, _task_id: &str) -> Result<MediaTask> {
        Err(AgentError::InvalidInput("Seedream 图片生成是同步的，不支持任务状态查询".to_string()))
    }
}
