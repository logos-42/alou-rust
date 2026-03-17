//! Tauri Commands - Session Actor 版本
//!
//! 使用 SessionActor 架构处理 AI 对话

use super::ai_client::AiClient;
use super::config::{ApiConfig, UserApiConfig};
use super::executor::RalphLoopExecutor;
// use super::streaming::StreamingExecutor;  // 暂时注释，模块不存在
use super::task::{TaskFinalResult, TaskManager};
use std::sync::Arc;
use crate::runtime::router::SessionRouter;
use crate::runtime::message::{SessionMessage, MessageMetadata};
use crate::agent::media_config::{MediaApiConfig, ProviderConfig as MediaProviderConfig};

/// Tauri 命令：执行 Agent 任务（同步返回结果）
#[tauri::command]
pub async fn execute_agent_task(
    message: String,
    agent_id: String,
    config: UserApiConfig,
    bridge_manager: tauri::State<'_, std::sync::Arc<crate::bridges::BridgeManager>>,
) -> std::result::Result<TaskFinalResult, String> {
    log::info!("[Command] 执行 Agent 任务：{}", message);
    let tool_bridge = bridge_manager.tool_bridge();
    let task_manager = Arc::new(TaskManager::new());
    let ai_client = match AiClient::new(&config) {
        Ok(client) => Arc::new(client),
        Err(e) => return Err(format!("创建 AI 客户端失败：{}", e)),
    };
    let executor = RalphLoopExecutor::new(
        ai_client.clone(),
        task_manager.clone(),
        tool_bridge.clone(),
        Arc::new(crate::tools::ToolRegistry::new()),
    );
    let task_id = task_manager.create_task(agent_id, message).await;
    match executor.execute(&task_id).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

/// Tauri 命令：执行 AI 对话
/// 🔥 直接执行模式：每个请求独立并发执行，互不阻塞
#[tauri::command]
pub async fn execute_ai_conversation(
    app_handle: tauri::AppHandle,
    agent_config: serde_json::Value,
    message: String,
    messages: Option<Vec<serde_json::Value>>,
    options: Option<serde_json::Value>,
    agent_id: Option<String>,
    session_id: Option<String>,
    bridge_manager: tauri::State<'_, std::sync::Arc<crate::bridges::BridgeManager>>,
    _session_router: tauri::State<'_, std::sync::Arc<SessionRouter>>,
) -> std::result::Result<serde_json::Value, String> {
    use super::task::TaskManager;

    log::info!("[Command] 执行 AI 对话：{}", &message[..20.min(message.len())]);

    let user_config = match serde_json::from_value::<UserApiConfig>(agent_config) {
        Ok(config) => config,
        Err(e) => return Err(format!("解析 agent 配置失败：{}", e)),
    };

    let ai_client = match AiClient::new(&user_config) {
        Ok(client) => Arc::new(client),
        Err(e) => return Err(format!("创建 AI 客户端失败：{}", e)),
    };

    let target_session_id = session_id.or(agent_id).unwrap_or_else(|| format!("session_{}", chrono::Utc::now().timestamp()));

    // 🔥 每个请求创建独立的任务管理器和执行器，实现真正的并发
    let task_manager = Arc::new(TaskManager::new());
    let tool_bridge = bridge_manager.tool_bridge();

    // 创建任务：使用完整的消息历史
    let task_id = if let Some(msgs) = &messages {
        // 将 JSON 数组转换为 AiMessage 列表
        use super::ai_client::AiMessage;
        let ai_messages: Vec<AiMessage> = msgs
            .iter()
            .filter_map(|m| {
                let role = m.get("role")?.as_str()?.to_string();
                let content = m.get("content")?.as_str()?.to_string();
                Some(AiMessage {
                    role,
                    content,
                    tool_call_id: None,
                    tool_calls: None,
                })
            })
            .collect();

        log::info!("[Command] 使用完整消息历史创建任务，消息数：{}", ai_messages.len());
        task_manager.create_task_with_messages(target_session_id.clone(), ai_messages).await
    } else {
        log::info!("[Command] 使用单条消息创建任务");
        task_manager.create_task(target_session_id.clone(), message.clone()).await
    };

    // 🔥 创建执行器（带 AppHandle 用于发送进度事件到前端）
    let executor = RalphLoopExecutor::new(
        ai_client,
        task_manager.clone(),
        tool_bridge,
        Arc::new(crate::tools::ToolRegistry::new()),
    )
    .with_app_handle(app_handle.clone());

    let use_stream = options.as_ref().and_then(|o| o.get("stream")).and_then(|s| s.as_bool()).unwrap_or(false);

    // 🔥 直接执行任务，不通过 SessionActor，实现真正的并发
    let result = if use_stream {
        // StreamingExecutor not implemented yet
        Ok(serde_json::json!({
            "success": true,
            "task_id": task_id,
            "stream": false,
            "session_id": target_session_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "note": "Streaming not implemented"
        }))
    } else {
        // 🔥 同步执行并等待完成（每个请求独立，互不阻塞）
        match executor.execute(&task_id).await {
            Ok(result) => {
                log::info!("[Command] 对话执行成功：{}", task_id);
                Ok(serde_json::json!({
                    "success": true,
                    "result": result,
                    "task_id": task_id,
                    "session_id": target_session_id,
                    "timestamp": chrono::Utc::now().timestamp()
                }))
            }
            Err(e) => Err(format!("AI 对话执行失败：{}", e)),
        }
    };

    result
}

#[tauri::command]
pub async fn get_agent_config() -> std::result::Result<ApiConfig, String> {
    match ApiConfig::load().await {
        Ok(config) => Ok(config),
        Err(e) => Err(format!("加载配置失败：{}", e)),
    }
}

#[tauri::command]
pub async fn update_agent_config(config: ApiConfig) -> std::result::Result<(), String> {
    match config.save().await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("保存配置失败：{}", e)),
    }
}

/// Tauri 命令：获取媒体配置
#[tauri::command]
pub async fn get_media_config() -> std::result::Result<MediaApiConfig, String> {
    MediaApiConfig::load()
}

/// Tauri 命令：更新媒体配置（合并模式）
#[tauri::command]
pub async fn update_media_config(
    providers: std::collections::HashMap<String, MediaProviderConfig>
) -> std::result::Result<(), String> {
    use crate::agent::media_config::MediaApiConfig;
    
    log::info!("[update_media_config] 接收到的配置：{:?}", providers);
    
    // 加载现有配置
    let mut existing = MediaApiConfig::load()?;
    log::info!("[update_media_config] 现有配置中的 providers: {:?}", existing.providers.keys());
    
    // 合并新配置
    for (name, config) in providers {
        log::info!("[update_media_config] 添加/更新 provider: {}", name);
        existing.providers.insert(name, config);
    }
    
    log::info!("[update_media_config] 合并后的 providers: {:?}", existing.providers.keys());
    
    // 保存合并后的配置
    let result = existing.save();
    log::info!("[update_media_config] 保存结果：{:?}", result);
    result
}

/// Tauri 命令：测试媒体 Provider 连接
#[tauri::command]
pub async fn test_media_provider_connection(
    provider_name: String,
    config: MediaProviderConfig,
) -> std::result::Result<TestResult, String> {
    use crate::agent::providers::media_factory::MediaProviderFactory;
    
    // 创建 Provider 实例
    let provider = match MediaProviderFactory::create_provider(&provider_name, &config) {
        Ok(p) => p,
        Err(e) => return Ok(TestResult { 
            success: false, 
            message: format!("创建 Provider 失败：{}", e) 
        }),
    };

    // 根据 Provider 类型测试
    let result = match provider_name.as_str() {
        "seedream" | "google" | "jimeng" => {
            // 图片 Provider - 测试生成一张小图
            use crate::agent::providers::media_provider::{MediaProvider, ImageOptions};
            let options = ImageOptions {
                prompt: "test".to_string(),
                width: Some(64),
                height: Some(64),
                ..Default::default()
            };
            provider.generate_image(options).await
                .map(|_| "图片生成测试成功".to_string())
                .map_err(|e| e.to_string())
        }
        "minimax" => {
            // 音频 Provider - 测试 TTS
            use crate::agent::providers::media_provider::{MediaProvider, AudioOptions};
            let options = AudioOptions {
                text: "test".to_string(),
                ..Default::default()
            };
            provider.generate_audio(options).await
                .map(|_| "语音合成测试成功".to_string())
                .map_err(|e| e.to_string())
        }
        "seedance" => {
            // 视频 Provider - 只测试配置有效性，不实际生成
            Ok("视频 Provider 配置有效".to_string())
        }
        _ => {
            Ok(format!("Provider {} 不支持测试", provider_name))
        }
    };

    match result {
        Ok(msg) => Ok(TestResult { success: true, message: msg }),
        Err(e) => Ok(TestResult { success: false, message: e }),
    }
}

#[tauri::command]
pub async fn test_api_connection(config: UserApiConfig) -> std::result::Result<TestResult, String> {
    let client = match AiClient::new(&config) {
        Ok(client) => client,
        Err(e) => return Ok(TestResult { success: false, message: e.to_string() }),
    };
    use super::ai_client::AiMessage;
    let test_message = AiMessage {
        role: "user".to_string(),
        content: "Hello, this is a test.".to_string(),
        tool_call_id: None,
        tool_calls: None,
    };
    match client.send_message(vec![test_message], None).await {
        Ok(_) => Ok(TestResult { success: true, message: "连接成功".to_string() }),
        Err(e) => Ok(TestResult { success: false, message: e.to_string() }),
    }
}

#[tauri::command]
pub async fn health_check() -> std::result::Result<serde_json::Value, String> {
    Ok(serde_json::json!({"status": "healthy", "mode": "local", "timestamp": chrono::Utc::now().timestamp()}))
}

#[tauri::command]
pub async fn get_available_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo { id: "deepseek".to_string(), name: "DeepSeek".to_string(), models: vec!["deepseek-v3.2".to_string(), "deepseek-v3.2-speciale".to_string(), "deepseek-r1".to_string()], requires_base_url: false },
        ProviderInfo { id: "openai".to_string(), name: "OpenAI".to_string(), models: vec!["gpt-5.4".to_string(), "gpt-5.3".to_string(), "gpt-5".to_string(), "o3-pro".to_string(), "o4-mini".to_string()], requires_base_url: false },
        ProviderInfo { id: "claude".to_string(), name: "Claude".to_string(), models: vec!["claude-sonnet-4-6-20260218".to_string(), "claude-opus-4-6-20260218".to_string(), "claude-sonnet-4-5-20251101".to_string()], requires_base_url: false },
        ProviderInfo { id: "kimi".to_string(), name: "Kimi".to_string(), models: vec!["kimi-k2.5".to_string(), "kimi-k2-thinking".to_string(), "kimi-k2".to_string()], requires_base_url: false },
        ProviderInfo { id: "qwen".to_string(), name: "Qwen".to_string(), models: vec!["qwen3.5-plus".to_string(), "qwen3.5".to_string(), "qwen3-max".to_string()], requires_base_url: false },
        ProviderInfo { id: "glm".to_string(), name: "GLM".to_string(), models: vec!["glm-5".to_string(), "glm-4.5-flash".to_string(), "glm-4-plus".to_string()], requires_base_url: false },
        ProviderInfo { id: "gemini".to_string(), name: "Gemini".to_string(), models: vec!["gemini-3.1-pro".to_string(), "gemini-3-pro".to_string(), "gemini-2.5-pro".to_string()], requires_base_url: false },
        ProviderInfo { id: "minimax".to_string(), name: "MiniMax".to_string(), models: vec!["minimax-m2.5".to_string(), "minimax-m2.1".to_string(), "abab6.5s-chat".to_string()], requires_base_url: false },
    ]
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub models: Vec<String>,
    pub requires_base_url: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
}
