//! 媒体生成工具 - 实现 ToolExecutor trait
//!
//! 支持：
//! - generate_image: 图片生成
//! - generate_audio: 语音合成
//! - generate_video: 视频生成
//! - get_video_status: 查询视频任务状态

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use crate::agent::providers::ProviderRegistry;
use crate::agent::providers::media_provider::{MediaProvider, ImageOptions, AudioOptions, VideoOptions, TaskStatus};
use crate::media_archive::MediaArchiveManager;
use super::{ToolMetadata, ToolExecutor, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolPriority, ToolStatus};

// ============================================================================
// 图片生成工具
// ============================================================================

/// 图片生成工具
pub struct GenerateImageTool {
    provider_registry: Arc<ProviderRegistry>,
    archive_manager: Arc<MediaArchiveManager>,
    metadata: ToolMetadata,
}

impl GenerateImageTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>, archive_manager: Arc<MediaArchiveManager>) -> Self {
        Self {
            provider_registry,
            archive_manager,
            metadata: ToolMetadata {
                id: "generate_image".to_string(),
                name: "Generate Image".to_string(),
                description: "根据文字描述生成图片，支持风景、人物、艺术创作等。可用 Provider: google (Imagen), jimeng (即梦)".to_string(),
                category: ToolCategory::Other,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec![],
                tags: vec!["media".to_string(), "image".to_string(), "ai".to_string()],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for GenerateImageTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let prompt = args.get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("缺少必要参数：prompt (图片描述)".to_string()))?
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
            .ok_or_else(|| ToolError::ExecutionFailed(
                format!("媒体 Provider '{}' 不存在或未启用", provider_name)
            ))?;

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

                Ok(ToolResult::success(result))
            }
            Err(e) => Err(ToolError::ExecutionFailed(format!("图片生成失败：{}", e)))
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if args.get("prompt").and_then(|v| v.as_str()).is_none() {
            return Err(ToolError::InvalidArguments("缺少必要参数：prompt".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        "根据文字描述生成图片。\n参数：prompt (必需), width, height, provider".to_string()
    }
}

// ============================================================================
// 语音合成工具
// ============================================================================

/// 语音合成工具
pub struct GenerateAudioTool {
    provider_registry: Arc<ProviderRegistry>,
    archive_manager: Arc<MediaArchiveManager>,
    metadata: ToolMetadata,
}

impl GenerateAudioTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>, archive_manager: Arc<MediaArchiveManager>) -> Self {
        Self {
            provider_registry,
            archive_manager,
            metadata: ToolMetadata {
                id: "generate_audio".to_string(),
                name: "Generate Audio".to_string(),
                description: "将文字转换为语音（TTS），支持多种音色和语言。可用 Provider: minimax".to_string(),
                category: ToolCategory::Other,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec![],
                tags: vec!["media".to_string(), "audio".to_string(), "tts".to_string()],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for GenerateAudioTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let text = args.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("缺少必要参数：text (要转换的文字)".to_string()))?
            .to_string();

        let voice_id = args.get("voice_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let provider_name = args.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("minimax");

        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| ToolError::ExecutionFailed(
                format!("媒体 Provider '{}' 不存在或未启用", provider_name)
            ))?;

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

                Ok(ToolResult::success(result))
            }
            Err(e) => Err(ToolError::ExecutionFailed(format!("语音合成失败：{}", e)))
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if args.get("text").and_then(|v| v.as_str()).is_none() {
            return Err(ToolError::InvalidArguments("缺少必要参数：text".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        "将文字转换为语音。\n参数：text (必需), voice_id, provider".to_string()
    }
}

// ============================================================================
// 视频生成工具
// ============================================================================

/// 视频生成工具
pub struct GenerateVideoTool {
    provider_registry: Arc<ProviderRegistry>,
    archive_manager: Arc<MediaArchiveManager>,
    metadata: ToolMetadata,
}

impl GenerateVideoTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>, archive_manager: Arc<MediaArchiveManager>) -> Self {
        Self {
            provider_registry,
            archive_manager,
            metadata: ToolMetadata {
                id: "generate_video".to_string(),
                name: "Generate Video".to_string(),
                description: "根据文字描述生成视频，支持动画、实景等风格。可用 Provider: minimax, jimeng (即梦)。注意：视频生成是异步任务，需要轮询状态".to_string(),
                category: ToolCategory::Other,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "0.3.1".to_string(),
                author: "Alou".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec![],
                tags: vec!["media".to_string(), "video".to_string(), "ai".to_string()],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for GenerateVideoTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let prompt = args.get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("缺少必要参数：prompt (视频描述)".to_string()))?
            .to_string();

        let duration = args.get("duration")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);

        let provider_name = args.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("minimax");

        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| ToolError::ExecutionFailed(
                format!("媒体 Provider '{}' 不存在或未启用", provider_name)
            ))?;

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

                Ok(ToolResult::success(result))
            }
            Err(e) => Err(ToolError::ExecutionFailed(format!("视频生成任务创建失败：{}", e)))
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if args.get("prompt").and_then(|v| v.as_str()).is_none() {
            return Err(ToolError::InvalidArguments("缺少必要参数：prompt".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        "根据文字描述生成视频。\n参数：prompt (必需), duration, provider".to_string()
    }
}

// ============================================================================
// 查询视频状态工具
// ============================================================================

/// 查询视频任务状态工具
pub struct GetVideoStatusTool {
    provider_registry: Arc<ProviderRegistry>,
    _archive_manager: Arc<MediaArchiveManager>,
    metadata: ToolMetadata,
}

impl GetVideoStatusTool {
    pub fn new(provider_registry: Arc<ProviderRegistry>, archive_manager: Arc<MediaArchiveManager>) -> Self {
        Self {
            provider_registry,
            _archive_manager: archive_manager,
            metadata: ToolMetadata {
                id: "get_video_status".to_string(),
                name: "Get Video Status".to_string(),
                description: "查询异步视频生成任务的状态。参数：task_id (任务 ID), provider (Provider 名称)".to_string(),
                category: ToolCategory::Other,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec![],
                tags: vec!["media".to_string(), "video".to_string(), "status".to_string()],
            },
        }
    }
}

#[async_trait]
impl ToolExecutor for GetVideoStatusTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let task_id = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("缺少必要参数：task_id (任务 ID)".to_string()))?
            .to_string();

        let provider_name = args.get("provider")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("缺少必要参数：provider (Provider 名称)".to_string()))?;

        let provider = self.provider_registry
            .get_media_provider(provider_name)
            .ok_or_else(|| ToolError::ExecutionFailed(
                format!("媒体 Provider '{}' 不存在或未启用", provider_name)
            ))?;

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
                Ok(ToolResult::success(result))
            }
            Err(e) => Err(ToolError::ExecutionFailed(format!("查询视频状态失败：{}", e)))
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if args.get("task_id").and_then(|v| v.as_str()).is_none() {
            return Err(ToolError::InvalidArguments("缺少必要参数：task_id".to_string()));
        }
        if args.get("provider").and_then(|v| v.as_str()).is_none() {
            return Err(ToolError::InvalidArguments("缺少必要参数：provider".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        "查询异步视频生成任务的状态。\n参数：task_id (必需), provider (必需)".to_string()
    }
}
