//! 媒体生成工具 - 将 MediaProvider 封装为可调用的 Tool
//!
//! 支持：
//! - generate_image: 图片生成
//! - generate_audio: 语音合成
//! - generate_video: 视频生成

use std::sync::Arc;
use serde_json::Value;
use crate::agent_runtime::tool_bus::Tool;
use crate::agent::providers::ProviderRegistry;
use crate::agent::error::Result;

/// 图片生成工具
pub struct GenerateImageTool {
    provider_registry: Arc<ProviderRegistry>,
}

impl GenerateImageTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait::async_trait]
impl Tool for GenerateImageTool {
    fn name(&self) -> &str {
        "generate_image"
    }

    fn description(&self) -> &str {
        "根据文字描述生成图片，支持风景、人物、艺术创作等。可用的 Provider: google (Imagen), jimeng (即梦)"
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        use crate::agent::providers::media_provider::{MediaProvider, ImageOptions};

        let prompt = args.get("prompt")
            .and_then(|v| v.as_str())
            .ok_or("缺少必要参数：prompt (图片描述)")?
            .to_string();

        let width = args.get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024) as u32;

        let height = args.get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(1024) as u32;

        let provider_name = args.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("google");

        // 获取 Provider
        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

        // 检查是否支持图片生成
        if !provider.supported_types().contains(&crate::agent::providers::media_provider::MediaType::Image) {
            return Err(format!("Provider '{}' 不支持图片生成", provider_name));
        }

        // 生成图片
        let options = ImageOptions {
            prompt,
            width: Some(width),
            height: Some(height),
            aspect_ratio: None,
            model: None,
            negative_prompt: None,
            num_images: Some(1),
        };

        match provider.generate_image(options).await {
            Ok(output) => {
                Ok(json!({
                    "success": true,
                    "media_type": "image",
                    "provider": provider_name,
                    "file_path": output.file_path,
                    "url": output.url,
                    "metadata": {
                        "width": output.metadata.width,
                        "height": output.metadata.height,
                        "format": output.metadata.format,
                        "prompt": output.metadata.prompt,
                    }
                }))
            }
            Err(e) => {
                Err(format!("图片生成失败：{}", e))
            }
        }
    }
}

/// 语音合成工具
pub struct GenerateAudioTool {
    provider_registry: Arc<ProviderRegistry>,
}

impl GenerateAudioTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait::async_trait]
impl Tool for GenerateAudioTool {
    fn name(&self) -> &str {
        "generate_audio"
    }

    fn description(&self) -> &str {
        "将文字转换为语音（TTS），支持多种音色和语言。可用的 Provider: minimax"
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        use crate::agent::providers::media_provider::{MediaProvider, AudioOptions};

        let text = args.get("text")
            .and_then(|v| v.as_str())
            .ok_or("缺少必要参数：text (要转换的文字)")?
            .to_string();

        let voice_id = args.get("voice_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let provider_name = args.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("minimax");

        // 获取 Provider
        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

        // 检查是否支持音频生成
        if !provider.supported_types().contains(&crate::agent::providers::media_provider::MediaType::Audio) {
            return Err(format!("Provider '{}' 不支持语音合成", provider_name));
        }

        // 生成音频
        let options = AudioOptions {
            text,
            voice_id,
            model: None,
            speed: None,
            pitch: None,
            volume: None,
        };

        match provider.generate_audio(options).await {
            Ok(output) => {
                Ok(json!({
                    "success": true,
                    "media_type": "audio",
                    "provider": provider_name,
                    "file_path": output.file_path,
                    "url": output.url,
                    "metadata": {
                        "duration_secs": output.metadata.duration_secs,
                        "format": output.metadata.format,
                        "model": output.metadata.model,
                    }
                }))
            }
            Err(e) => {
                Err(format!("语音合成失败：{}", e))
            }
        }
    }
}

/// 视频生成工具
pub struct GenerateVideoTool {
    provider_registry: Arc<ProviderRegistry>,
}

impl GenerateVideoTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait::async_trait]
impl Tool for GenerateVideoTool {
    fn name(&self) -> &str {
        "generate_video"
    }

    fn description(&self) -> &str {
        "根据文字描述生成视频，支持动画、实景等风格。可用的 Provider: minimax, jimeng (即梦)。注意：视频生成是异步任务，需要轮询状态"
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        use crate::agent::providers::media_provider::{MediaProvider, VideoOptions, TaskStatus};

        let prompt = args.get("prompt")
            .and_then(|v| v.as_str())
            .ok_or("缺少必要参数：prompt (视频描述)")?
            .to_string();

        let duration = args.get("duration")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);

        let provider_name = args.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("minimax");

        // 获取 Provider
        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

        // 检查是否支持视频生成
        if !provider.supported_types().contains(&crate::agent::providers::media_provider::MediaType::Video) {
            return Err(format!("Provider '{}' 不支持视频生成", provider_name));
        }

        // 生成视频（异步任务）
        let options = VideoOptions {
            prompt: Some(prompt),
            duration_secs: Some(duration as u32),
            resolution: None,
            model: None,
            image_url: None,
        };

        match provider.generate_video(options).await {
            Ok(task) => {
                // 如果是终态，直接返回结果
                let result = match task.status {
                    TaskStatus::Completed => {
                        if let Some(output) = task.result {
                            json!({
                                "success": true,
                                "media_type": "video",
                                "provider": provider_name,
                                "status": "completed",
                                "file_path": output.file_path,
                                "url": output.url,
                                "task_id": task.task_id,
                            })
                        } else {
                            json!({
                                "success": false,
                                "error": "视频生成完成但没有返回结果",
                                "task_id": task.task_id,
                            })
                        }
                    }
                    TaskStatus::Failed => {
                        json!({
                            "success": false,
                            "error": task.error.unwrap_or_else(|| "视频生成失败".to_string()),
                            "status": "failed",
                            "task_id": task.task_id,
                        })
                    }
                    _ => {
                        // 异步任务，需要轮询
                        json!({
                            "success": true,
                            "media_type": "video",
                            "provider": provider_name,
                            "status": "processing",
                            "task_id": task.task_id,
                            "progress": task.progress,
                            "requires_polling": true,
                            "message": "视频生成是异步任务，需要轮询状态。请使用 get_video_status 工具查询进度。"
                        })
                    }
                };

                Ok(result)
            }
            Err(e) => {
                Err(format!("视频生成任务创建失败：{}", e))
            }
        }
    }
}

/// 查询视频任务状态工具
pub struct GetVideoStatusTool {
    provider_registry: Arc<ProviderRegistry>,
}

impl GetVideoStatusTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait::async_trait]
impl Tool for GetVideoStatusTool {
    fn name(&self) -> &str {
        "get_video_status"
    }

    fn description(&self) -> &str {
        "查询异步视频生成任务的状态。参数：task_id (任务 ID), provider (Provider 名称)"
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        use crate::agent::providers::media_provider::{MediaProvider, TaskStatus};

        let task_id = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少必要参数：task_id (任务 ID)")?
            .to_string();

        let provider_name = args.get("provider")
            .and_then(|v| v.as_str())
            .ok_or("缺少必要参数：provider (Provider 名称)")?;

        // 获取 Provider
        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

        // 查询状态
        match provider.get_task_status(&task_id).await {
            Ok(task) => {
                let result = match task.status {
                    TaskStatus::Completed => {
                        if let Some(output) = task.result {
                            json!({
                                "success": true,
                                "status": "completed",
                                "task_id": task.task_id,
                                "progress": 100.0,
                                "file_path": output.file_path,
                                "url": output.url,
                            })
                        } else {
                            json!({
                                "success": false,
                                "error": "视频生成完成但没有返回结果",
                                "task_id": task.task_id,
                            })
                        }
                    }
                    TaskStatus::Failed => {
                        json!({
                            "success": false,
                            "error": task.error.unwrap_or_else(|| "视频生成失败".to_string()),
                            "status": "failed",
                            "task_id": task.task_id,
                        })
                    }
                    TaskStatus::Processing => {
                        json!({
                            "success": true,
                            "status": "processing",
                            "task_id": task.task_id,
                            "progress": task.progress,
                            "message": "视频生成中，请稍后..."
                        })
                    }
                    TaskStatus::Pending => {
                        json!({
                            "success": true,
                            "status": "pending",
                            "task_id": task.task_id,
                            "progress": 0.0,
                            "message": "任务排队中，请稍后..."
                        })
                    }
                };

                Ok(result)
            }
            Err(e) => {
                Err(format!("查询任务状态失败：{}", e))
            }
        }
    }
}
