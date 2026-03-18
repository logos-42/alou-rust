//! 媒体 API 路由 - 处理媒体生成请求

use axum::{
    extract::State as AxumState,
    http::Method,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use crate::agent::providers::ProviderRegistry;
use tauri::State;

/// 媒体生成请求
#[derive(Deserialize)]
pub struct MediaGenerateRequest {
    pub tool: String,
    pub args: serde_json::Value,
    #[serde(default)]
    pub timeout: Option<u64>,
}

/// 媒体生成响应
#[derive(Serialize)]
pub struct MediaGenerateResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: Option<u64>,
}

/// API 共享状态
pub struct MediaApiState {
    pub provider_registry: Arc<ProviderRegistry>,
}

/// 添加媒体 API 路由
pub async fn add_media_routes(
    router: Router<Arc<Mutex<Option<MediaApiState>>>>,
) -> Router<Arc<Mutex<Option<MediaApiState>>>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    router
        .route("/api/media/generate", post(generate_media))
        .route("/api/media/tools", get(list_media_tools))
        .layer(cors)
}

/// 生成媒体（图片/音频/视频）
async fn generate_media(
    AxumState(state): AxumState<Arc<Mutex<Option<MediaApiState>>>>,
    Json(payload): Json<MediaGenerateRequest>,
) -> Json<MediaGenerateResponse> {
    let state_guard = state.lock().await;
    let api_state = match state_guard.as_ref() {
        Some(s) => s,
        None => {
            return Json(MediaGenerateResponse {
                success: false,
                result: None,
                error: Some("API server not initialized".to_string()),
                execution_time_ms: None,
            });
        }
    };

    let start_time = std::time::Instant::now();

    // 获取 ProviderRegistry
    let provider_registry = api_state.provider_registry.clone();

    // 根据工具类型调用不同的媒体生成方法
    let result: Result<serde_json::Value, String> = match payload.tool.as_str() {
        "generate_image" => {
            execute_generate_image(&provider_registry, payload.args).await
        }
        "generate_audio" => {
            execute_generate_audio(&provider_registry, payload.args).await
        }
        "generate_video" => {
            execute_generate_video(&provider_registry, payload.args).await
        }
        "get_video_status" => {
            execute_get_video_status(&provider_registry, payload.args).await
        }
        _ => Err(format!("未知的媒体工具：{}", payload.tool)),
    };

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    match result {
        Ok(data) => Json(MediaGenerateResponse {
            success: true,
            result: Some(data),
            error: None,
            execution_time_ms: Some(execution_time_ms),
        }),
        Err(e) => Json(MediaGenerateResponse {
            success: false,
            result: None,
            error: Some(e),
            execution_time_ms: Some(execution_time_ms),
        }),
    }
}

/// 执行图片生成
async fn execute_generate_image(
    provider_registry: &Arc<crate::agent::providers::ProviderRegistry>,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
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
    let provider = provider_registry
        .get_media_provider(provider_name)
        .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

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
            Ok(serde_json::json!({
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
        Err(e) => Err(format!("图片生成失败：{}", e))
    }
}

/// 执行音频生成
async fn execute_generate_audio(
    provider_registry: &Arc<crate::agent::providers::ProviderRegistry>,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
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
    let provider = provider_registry
        .get_media_provider(provider_name)
        .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

    // 生成音频
    let options = AudioOptions {
        text,
        voice_id,
        model: None,
        speed: None,
        pitch: None,
        volume: None,
        lyrics: None,
        is_instrumental: None,
    };

    match provider.generate_audio(options).await {
        Ok(output) => {
            Ok(serde_json::json!({
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
        Err(e) => Err(format!("语音合成失败：{}", e))
    }
}

/// 执行视频生成
async fn execute_generate_video(
    provider_registry: &Arc<crate::agent::providers::ProviderRegistry>,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
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
    let provider = provider_registry
        .get_media_provider(provider_name)
        .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

    // 生成视频（异步任务）
    let options = VideoOptions {
        prompt: prompt,
        duration: Some(duration as f64),
        duration_secs: Some(duration as u32),
        resolution: None,
        model: None,
    };

    match provider.generate_video(options).await {
        Ok(task) => {
            let result = match task.status {
                TaskStatus::Completed => {
                    if let Some(output) = task.result {
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
                        "error": task.error.unwrap_or_else(|| "视频生成失败".to_string()),
                        "status": "failed",
                        "task_id": task.task_id,
                    })
                }
                _ => {
                    // 异步任务，需要轮询
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

            Ok(result)
        }
        Err(e) => Err(format!("视频生成任务创建失败：{}", e))
    }
}

/// 执行视频状态查询
async fn execute_get_video_status(
    provider_registry: &Arc<crate::agent::providers::ProviderRegistry>,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use crate::agent::providers::media_provider::{MediaProvider, TaskStatus};

    let task_id = args.get("task_id")
        .and_then(|v| v.as_str())
        .ok_or("缺少必要参数：task_id (任务 ID)")?
        .to_string();

    let provider_name = args.get("provider")
        .and_then(|v| v.as_str())
        .ok_or("缺少必要参数：provider (Provider 名称)")?;

    // 获取 Provider
    let provider = provider_registry
        .get_media_provider(provider_name)
        .ok_or_else(|| format!("媒体 Provider '{}' 不存在或未启用", provider_name))?;

    // 查询状态
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
                            "error": "视频生成完成但没有返回结果",
                            "task_id": task.task_id,
                        })
                    }
                }
                TaskStatus::Failed => {
                    serde_json::json!({
                        "success": false,
                        "error": task.error.unwrap_or_else(|| "视频生成失败".to_string()),
                        "status": "failed",
                        "task_id": task.task_id,
                    })
                }
                TaskStatus::Processing => {
                    serde_json::json!({
                        "success": true,
                        "status": "processing",
                        "task_id": task.task_id,
                        "progress": task.progress,
                        "message": "视频生成中，请稍后..."
                    })
                }
                TaskStatus::Pending => {
                    serde_json::json!({
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
        Err(e) => Err(format!("查询任务状态失败：{}", e))
    }
}

/// 列出可用的媒体工具
async fn list_media_tools(
    AxumState(state): AxumState<Arc<Mutex<Option<MediaApiState>>>>,
) -> Json<Vec<serde_json::Value>> {
    Json(vec![
        serde_json::json!({
            "name": "generate_image",
            "description": "根据文字描述生成图片",
            "parameters": {
                "type": "object",
                "properties": {
                    "prompt": {"type": "string", "description": "图片描述"},
                    "width": {"type": "integer", "default": 1024},
                    "height": {"type": "integer", "default": 1024},
                    "provider": {"type": "string", "enum": ["google", "jimeng"]}
                },
                "required": ["prompt"]
            }
        }),
        serde_json::json!({
            "name": "generate_audio",
            "description": "将文字转换为语音",
            "parameters": {
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "文字内容"},
                    "voice_id": {"type": "string"},
                    "provider": {"type": "string", "enum": ["minimax"]}
                },
                "required": ["text"]
            }
        }),
        serde_json::json!({
            "name": "generate_video",
            "description": "根据文字描述生成视频（异步任务）",
            "parameters": {
                "type": "object",
                "properties": {
                    "prompt": {"type": "string", "description": "视频描述"},
                    "duration": {"type": "integer", "default": 5},
                    "provider": {"type": "string", "enum": ["minimax", "jimeng"]}
                },
                "required": ["prompt"]
            }
        }),
        serde_json::json!({
            "name": "get_video_status",
            "description": "查询异步视频生成任务的状态",
            "parameters": {
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "任务 ID"},
                    "provider": {"type": "string", "enum": ["minimax", "jimeng"]}
                },
                "required": ["task_id", "provider"]
            }
        }),
    ])
}

/// 启动媒体 API 服务器
pub async fn start_media_api_server(
    provider_registry: Arc<ProviderRegistry>,
) -> Result<u16, String> {
    let api_state = MediaApiState { provider_registry };
    let state: Arc<Mutex<Option<MediaApiState>>> = Arc::new(Mutex::new(Some(api_state)));

    let router = add_media_routes(Router::new()).await.with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind media API: {}", e))?;

    let addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to get media API address: {}", e))?;

    log::info!("媒体 API 服务器启动在：{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("Failed to serve media API");
    });

    Ok(addr.port())
}
