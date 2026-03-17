//! 即梦 Seedance Provider 实现
//!
//! 即梦 (Jimeng) Seedance 1.0 视频生成 API
//!
//! 官方文档：
//! - 火山方舟：https://www.volcengine.com/docs/82379/1520757
//! - 302.AI 代理：https://doc.302.ai/344076582e0
//!
//! API 端点（火山方舟官方）：
//! - 创建任务：POST https://ark.cn-beijing.volces.com/api/v3/videos/generations
//! - 查询状态：GET https://ark.cn-beijing.volces.com/api/v3/videos/generations/{task_id}
//!
//! API 端点（302.AI 代理）：
//! - 创建任务：POST https://api.302.ai/doubao/doubao-seedance
//! - 查询状态：GET https://api.302.ai/doubao/doubao-seedance/{task_id}
//!
//! 支持模型：
//! - seedance-1.0-pro: 标准版
//! - seedance-1.0-pro-fast: 快速版
//! - seedance-1.0-lite: 精简版
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
        // 根据 API Key 前缀自动判断使用哪个端点
        // 火山方舟 API Key 通常以 "ark-" 开头或不带前缀
        // 302.AI API Key 通常较短
        let default_url = if api_key.starts_with("ark-") || !api_key.contains("302") {
            // 火山方舟官方 API
            "https://ark.cn-beijing.volces.com/api/v3".to_string()
        } else {
            // 302.AI 代理 API
            "https://api.302.ai/doubao".to_string()
        };
        
        Self {
            api_key,
            base_url: base_url.unwrap_or(default_url),
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
        // 判断使用哪个 API 格式
        let is_volcengine = self.config.base_url.contains("volces.com");
        
        let url = if is_volcengine {
            format!("{}/videos/generations", self.config.base_url)
        } else {
            format!("{}/doubao-seedance", self.config.base_url)
        };

        log::info!("[Seedance] 创建视频任务，URL: {}", url);
        log::info!("[Seedance] 使用 API 类型：{}", if is_volcengine { "火山方舟官方" } else { "302.AI 代理" });

        // 构建请求体
        let request_body = if is_volcengine {
            // 火山方舟官方 API 格式
            serde_json::json!({
                "model": model.unwrap_or_else(|| self.config.default_model.clone()),
                "prompt": prompt,
                "duration": duration.unwrap_or(5),
                "ratio": ratio.unwrap_or_else(|| "adaptive".to_string()),
                "generate_audio": true
            })
        } else {
            // 302.AI 代理 API 格式
            serde_json::json!({
                "model": model.unwrap_or_else(|| self.config.default_model.clone()),
                "content": vec![serde_json::json!({"type": "text", "text": prompt})],
                "duration": duration,
                "ratio": ratio,
                "generate_audio": true,
                "service_tier": "default"
            })
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("网络请求失败：{}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            log::error!("[Seedance] API 响应错误 ({}): {}", status, error);
            return Err(AgentError::ExternalApiError(
                format!("Seedance API 错误 ({}): {}", status, error)
            ));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        log::info!("[Seedance] API 响应：{:?}", response_json);

        // 根据不同 API 格式解析 task_id
        let task_id = if is_volcengine {
            // 火山方舟：{"id": "xxx", "data": {...}}
            response_json["id"].as_str().unwrap_or_default().to_string()
        } else {
            // 302.AI: {"id": "xxx"}
            response_json["id"].as_str().unwrap_or_default().to_string()
        };

        if task_id.is_empty() {
            return Err(AgentError::ExternalApiError(
                "未能获取 task_id".to_string()
            ));
        }

        log::info!("[Seedance] 任务创建成功，task_id: {}", task_id);

        Ok(task_id)
    }

    /// 查询任务状态
    pub async fn get_status(&self, task_id: &str) -> Result<MediaTask> {
        // 判断使用哪个 API 格式
        let is_volcengine = self.config.base_url.contains("volces.com");
        
        let url = if is_volcengine {
            format!("{}/videos/generations/{}", self.config.base_url, task_id)
        } else {
            format!("{}/doubao-seedance/{}", self.config.base_url, task_id)
        };

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("网络请求失败：{}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            log::error!("[Seedance] 查询状态失败 ({}): {}", status, error);
            return Err(AgentError::ExternalApiError(
                format!("查询状态失败 ({}): {}", status, error)
            ));
        }

        let status_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AgentError::ExternalApiError(format!("解析响应失败：{}", e)))?;

        log::info!("[Seedance] 任务状态响应：{:?}", status_response);

        // 根据不同 API 格式解析状态
        let status_str = if is_volcengine {
            // 火山方舟：{"data": {"status": "xxx"}}
            status_response["data"]["status"].as_str().unwrap_or("pending")
        } else {
            // 302.AI: {"status": "xxx"}
            status_response["status"].as_str().unwrap_or("pending")
        };

        let status = match status_str {
            "succeeded" | "completed" | "success" => TaskStatus::Completed,
            "failed" | "error" => TaskStatus::Failed,
            "running" | "processing" => TaskStatus::Processing,
            _ => TaskStatus::Pending,
        };

        // 解析结果
        let result = if status == TaskStatus::Completed {
            let video_info = if is_volcengine {
                // 火山方舟：{"data": {"video": {"play_addr": "xxx", "cover": "xxx"}}}
                &status_response["data"]["video"]
            } else {
                // 302.AI: {"video": {"play_addr": "xxx", "cover": "xxx"}}
                &status_response["video"]
            };

            let play_addr = video_info["play_addr"].as_str().unwrap_or_default().to_string();
            let cover = video_info["cover"].as_str().unwrap_or_default().to_string();

            if !play_addr.is_empty() {
                Some(MediaOutput {
                    media_type: MediaType::Video,
                    provider: "seedance".to_string(),
                    url: Some(play_addr),
                    file_path: None,
                    ipfs_cid: None,
                    metadata: MediaMetadata {
                        width: video_info["width"].as_u64().map(|v| v as u32),
                        height: video_info["height"].as_u64().map(|v| v as u32),
                        duration: video_info["duration"].as_f64().map(|v| v as f32),
                        ..Default::default()
                    },
                })
            } else {
                None
            }
        } else {
            None
        };

        // 解析错误
        let error = if status == TaskStatus::Failed {
            let error_info = if is_volcengine {
                &status_response["data"]["error"]
            } else {
                &status_response["error"]
            };
            Some(error_info["message"].as_str().unwrap_or("视频生成失败").to_string())
        } else {
            None
        };

        // 解析进度
        let progress = if is_volcengine {
            status_response["data"]["progress"].as_u64().unwrap_or(0) as f32
        } else {
            status_response["progress"].as_u64().unwrap_or(0) as f32
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
            progress,
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
