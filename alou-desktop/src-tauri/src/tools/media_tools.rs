//! 媒体生成工具 - 直接调用 Provider（绕过 HTTP 层）
//!
//! 支持：
//! - generate_image: 图片生成
//! - generate_audio: 语音合成
//! - generate_video: 视频生成
//! - get_video_status: 查询视频任务状态
//!
//! 工具直接持有 ProviderRegistry 和 ArchiveManager，
//! 不再通过 HTTP API 调用，消除不必要的跨进程开销。

use std::sync::Arc;
use serde_json::Value;
use crate::agent::providers::ProviderRegistry;
use crate::agent::providers::media_provider::{MediaProvider, ImageOptions, AudioOptions, VideoOptions, TaskStatus};
use crate::media_archive::MediaArchiveManager;
use crate::tools::trait_def::{Tool, ToolContext};

/// 图片生成工具
pub struct GenerateImageTool {
    provider_registry: Arc<ProviderRegistry>,
    archive_manager: Arc<MediaArchiveManager>,
}

impl GenerateImageTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>, archive_manager: Arc<MediaArchiveManager>) -> Self {
        Self { provider_registry, archive_manager }
    }
}

#[async_trait::async_trait]
impl Tool for GenerateImageTool {
    fn name(&self) -> &str { "generate_image" }

    fn description(&self) -> &str {
        "根据文字描述生成图片，支持风景、人物、艺术创作等。可用的 Provider: google (Imagen), jimeng (即梦)"
    }

    async fn execute(&self, args: Value, _context: &ToolContext) -> Result<Value, String> {
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

        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

        let options = ImageOptions {
            prompt: prompt.clone(),
            width: Some(width),
            height: Some(height),
            aspect_ratio: None,
            model: None,
            negative_prompt: None,
            num_images: Some(1),
        };

        match provider.generate_image(options).await {
            Ok(output) => {
                let mut result = serde_json::json!({
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
                });

                if let Some(file_path) = output.file_path.as_ref() {
                    let archive_id = format!("media:image:{}", chrono::Utc::now().timestamp());
                    let archive_data = serde_json::json!({
                        "type": "image",
                        "file_path": file_path,
                        "url": output.url,
                        "metadata": output.metadata,
                        "created_at": chrono::Utc::now().to_rfc3339(),
                    });

                    match self.archive_manager.archive(&archive_id, &archive_data).await {
                        Ok(()) => {
                            result["archive"] = serde_json::json!({
                                "archive_id": archive_id,
                                "status": "saved"
                            });
                        }
                        Err(e) => {
                            log::warn!("媒体存档失败 ({}): {}", archive_id, e);
                            result["archive_warning"] = serde_json::json!({
                                "error": e,
                                "message": "本地存档失败，但媒体文件已保存"
                            });
                        }
                    }
                }

                Ok(result)
            }
            Err(e) => Err(format!("图片生成失败：{}", e))
        }
    }
}

/// 语音合成工具
pub struct GenerateAudioTool {
    provider_registry: Arc<ProviderRegistry>,
    archive_manager: Arc<MediaArchiveManager>,
}

impl GenerateAudioTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>, archive_manager: Arc<MediaArchiveManager>) -> Self {
        Self { provider_registry, archive_manager }
    }
}

#[async_trait::async_trait]
impl Tool for GenerateAudioTool {
    fn name(&self) -> &str { "generate_audio" }

    fn description(&self) -> &str {
        "将文字转换为语音（TTS），支持多种音色和语言。可用的 Provider: minimax"
    }

    async fn execute(&self, args: Value, _context: &ToolContext) -> Result<Value, String> {
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

        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

        let options = AudioOptions {
            text: text.clone(),
            voice_id: voice_id.clone(),
            model: None,
            speed: None,
            pitch: None,
            volume: None,
            lyrics: None,
            is_instrumental: None,
        };

        match provider.generate_audio(options).await {
            Ok(output) => {
                let mut result = serde_json::json!({
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
                });

                if let Some(file_path) = output.file_path.as_ref() {
                    let archive_id = format!("media:audio:{}", chrono::Utc::now().timestamp());
                    let archive_data = serde_json::json!({
                        "type": "audio",
                        "file_path": file_path,
                        "url": output.url,
                        "metadata": {
                            "text": text.clone(),
                            "voice_id": voice_id.clone(),
                            "duration_secs": output.metadata.duration_secs,
                            "format": output.metadata.format,
                        },
                        "created_at": chrono::Utc::now().to_rfc3339(),
                    });

                    match self.archive_manager.archive(&archive_id, &archive_data).await {
                        Ok(()) => {
                            result["archive"] = serde_json::json!({
                                "archive_id": archive_id,
                                "status": "saved"
                            });
                        }
                        Err(e) => {
                            log::warn!("媒体存档失败 ({}): {}", archive_id, e);
                            result["archive_warning"] = serde_json::json!({
                                "error": e,
                                "message": "本地存档失败，但媒体文件已保存"
                            });
                        }
                    }
                }

                Ok(result)
            }
            Err(e) => Err(format!("语音合成失败：{}", e))
        }
    }
}

/// 视频生成工具
pub struct GenerateVideoTool {
    provider_registry: Arc<ProviderRegistry>,
    archive_manager: Arc<MediaArchiveManager>,
}

impl GenerateVideoTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>, archive_manager: Arc<MediaArchiveManager>) -> Self {
        Self { provider_registry, archive_manager }
    }
}

#[async_trait::async_trait]
impl Tool for GenerateVideoTool {
    fn name(&self) -> &str { "generate_video" }

    fn description(&self) -> &str {
        "根据文字描述生成视频，支持动画、实景等风格。可用的 Provider: minimax, jimeng (即梦)。注意：视频生成是异步任务，需要轮询状态"
    }

    async fn execute(&self, args: Value, _context: &ToolContext) -> Result<Value, String> {
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

        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

        let options = VideoOptions {
            prompt: prompt.clone(),
            duration: Some(duration as f64),
            duration_secs: Some(duration as u32),
            resolution: None,
            model: None,
        };

        match provider.generate_video(options).await {
            Ok(task) => {
                let mut result = match task.status {
                    TaskStatus::Completed => {
                        if let Some(output) = task.result.as_ref() {
                            serde_json::json!({
                                "success": true,
                                "media_type": "video",
                                "provider": provider_name,
                                "status": "completed",
                                "file_path": output.file_path,
                                "url": output.url,
                                "task_id": task.task_id,
                            })
                        } else {
                            serde_json::json!({
                                "success": false,
                                "error": "视频生成完成但没有返回结果",
                                "task_id": task.task_id,
                            })
                        }
                    }
                    TaskStatus::Failed => {
                        serde_json::json!({
                            "success": false,
                            "error": task.error.as_ref().unwrap_or(&"视频生成失败".to_string()),
                            "status": "failed",
                            "task_id": task.task_id,
                        })
                    }
                    _ => {
                        serde_json::json!({
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

                if task.status == TaskStatus::Completed {
                    if let Some(output) = task.result.as_ref() {
                        if let Some(file_path) = output.file_path.as_ref() {
                            let archive_id = format!("media:video:{}", chrono::Utc::now().timestamp());
                            let archive_data = serde_json::json!({
                                "type": "video",
                                "file_path": file_path,
                                "url": output.url,
                                "task_id": task.task_id,
                                "metadata": {
                                    "prompt": prompt,
                                    "duration": duration,
                                    "provider": provider_name,
                                },
                                "created_at": chrono::Utc::now().to_rfc3339(),
                            });

                            match self.archive_manager.archive(&archive_id, &archive_data).await {
                                Ok(()) => {
                                    result["archive"] = serde_json::json!({
                                        "archive_id": archive_id,
                                        "status": "saved"
                                    });
                                }
                                Err(e) => {
                                    log::warn!("视频存档失败 ({}): {}", archive_id, e);
                                    result["archive_warning"] = serde_json::json!({
                                        "error": e,
                                        "message": "本地存档失败，但媒体文件已保存"
                                    });
                                }
                            }
                        }
                    }
                }

                Ok(result)
            }
            Err(e) => Err(format!("视频生成任务创建失败：{}", e))
        }
    }
}

/// 查询视频任务状态工具
pub struct GetVideoStatusTool {
    provider_registry: Arc<ProviderRegistry>,
    _archive_manager: Arc<MediaArchiveManager>,
}

impl GetVideoStatusTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>, archive_manager: Arc<MediaArchiveManager>) -> Self {
        Self { provider_registry, _archive_manager: archive_manager }
    }
}

#[async_trait::async_trait]
impl Tool for GetVideoStatusTool {
    fn name(&self) -> &str { "get_video_status" }

    fn description(&self) -> &str {
        "查询异步视频生成任务的状态。参数：task_id (任务 ID), provider (Provider 名称)"
    }

    async fn execute(&self, args: Value, _context: &ToolContext) -> Result<Value, String> {
        let task_id = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少必要参数：task_id (任务 ID)")?
            .to_string();

        let provider_name = args.get("provider")
            .and_then(|v| v.as_str())
            .ok_or("缺少必要参数：provider (Provider 名称)")?;

        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

        match provider.get_task_status(&task_id).await {
            Ok(task) => {
                let result = match task.status {
                    TaskStatus::Completed => {
                        if let Some(output) = task.result {
                            serde_json::json!({
                                "success": true,
                                "status": "completed",
                                "task_id": task.task_id,
                                "progress": 100.0,
                                "file_path": output.file_path,
                                "url": output.url,
                            })
                        } else {
                            serde_json::json!({
                                "success": false,
                                "status": "completed",
                                "error": "任务完成但没有返回结果",
                                "task_id": task.task_id,
                            })
                        }
                    }
                    TaskStatus::Failed => {
                        serde_json::json!({
                            "success": false,
                            "status": "failed",
                            "error": task.error.as_ref().unwrap_or(&"视频生成失败".to_string()),
                            "task_id": task.task_id,
                        })
                    }
                    _ => {
                        serde_json::json!({
                            "success": true,
                            "status": "processing",
                            "task_id": task.task_id,
                            "progress": task.progress,
                            "requires_polling": true,
                            "message": "视频仍在生成中，请稍后再次查询"
                        })
                    }
                };
                Ok(result)
            }
            Err(e) => Err(format!("查询视频状态失败：{}", e))
        }
    }
}
