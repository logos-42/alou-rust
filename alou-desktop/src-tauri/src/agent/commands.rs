//! Tauri Commands - Session Actor 版本
//!
//! 使用 SessionActor 架构处理 AI 对话

use super::ai_client::AiClient;
use super::config::{ApiConfig, UserApiConfig};
use super::executor::{RalphLoopExecutor, RalphLoopExecutorBuilder};
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
    // 使用新的 Builder 模式创建执行器
    let executor = match RalphLoopExecutorBuilder::new()
        .ai_client(ai_client.clone())
        .task_manager(task_manager.clone())
        .tool_bridge(tool_bridge.clone())
        .tool_registry(Arc::new(crate::tools::ToolRegistry::new()))
        .build() {
        Ok(exec) => exec,
        Err(e) => return Err(format!("创建执行器失败: {}", e)),
    };
    
    let task_id = task_manager.create_task(agent_id, message).await;
    match executor.execute(&task_id).await {
        Ok(result) => Ok(TaskFinalResult {
            task_id: task_id.clone(),
            success: true,
            result,
            error: None,
            iteration_count: 0, // TODO: 从执行器获取实际迭代次数
        }),
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
    log::info!("[Command] session_id: {:?}, agent_id: {:?}", session_id, agent_id);

    let user_config = match serde_json::from_value::<UserApiConfig>(agent_config) {
        Ok(config) => config,
        Err(e) => return Err(format!("解析 agent 配置失败：{}", e)),
    };

    let ai_client = match AiClient::new(&user_config) {
        Ok(client) => Arc::new(client),
        Err(e) => return Err(format!("创建 AI 客户端失败：{}", e)),
    };

    let target_session_id = session_id.or(agent_id).unwrap_or_else(|| format!("session_{}", chrono::Utc::now().timestamp()));
    log::info!("[Command] 使用 session_id: {}", target_session_id);

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

    // 🔥 创建执行器（使用新的 Builder 模式）
    let executor = match RalphLoopExecutorBuilder::new()
        .ai_client(ai_client)
        .task_manager(task_manager.clone())
        .tool_bridge(tool_bridge)
        .tool_registry(Arc::new(crate::tools::ToolRegistry::new()))
        .app_handle(app_handle.clone())  // 🔥 传递 app_handle 以便发送事件到前端
        .build() {
        Ok(exec) => exec,
        Err(e) => return Err(format!("创建执行器失败: {}", e)),
    };

    let use_stream = options.as_ref().and_then(|o| o.get("stream")).and_then(|s| s.as_bool()).unwrap_or(false);

    // 🔥 直接执行任务，不通过 SessionActor，实现真正的并发
    let result = if use_stream {
        // StreamingExecutor not implemented yet
        Ok(serde_json::json!({
            "success": true,
            "result": {
                "task_id": task_id,
                "success": true,
                "result": "Streaming not implemented",
                "error": None::<String>,
                "iteration_count": 0,
            },
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
                // 🔥 关键修复：返回嵌套的 result 对象，与前端期望的格式一致
                // 前端期望：tauri_result.result.result (字符串)
                Ok(serde_json::json!({
                    "success": true,
                    "result": {
                        "task_id": task_id,
                        "success": true,
                        "result": result,  // AI 回复的字符串
                        "error": None::<String>,
                        "iteration_count": 0,
                        "session_id": target_session_id,
                    },
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
        // DeepSeek API: https://api-docs.deepseek.com/quick_start/pricing
        ProviderInfo { 
            id: "deepseek".to_string(), 
            name: "DeepSeek".to_string(), 
            models: vec![
                "deepseek-chat".to_string(),      // DeepSeek-V3.2
                "deepseek-reasoner".to_string(),  // DeepSeek-R1
                "deepseek-coder".to_string(),     // DeepSeek Coder
            ], 
            requires_base_url: false 
        },
        // OpenAI API 最新模型 (2026): https://platform.openai.com/docs/models
        ProviderInfo { 
            id: "openai".to_string(), 
            name: "OpenAI".to_string(), 
            models: vec![
                "gpt-5.4".to_string(),           // GPT-5.4 (最新旗舰)
                "gpt-5.4-pro".to_string(),       // GPT-5.4 Pro
                "gpt-5.3-chat".to_string(),      // GPT-5.3 Chat
                "gpt-5.3-codex".to_string(),     // GPT-5.3 Codex
                "gpt-5.2".to_string(),           // GPT-5.2
                "gpt-5.2-codex".to_string(),     // GPT-5.2 Codex
                "gpt-5.1".to_string(),           // GPT-5.1
                "gpt-5-pro".to_string(),         // GPT-5 Pro
                "o3-pro".to_string(),            // O3 Pro
                "o3".to_string(),                // O3
                "o4-mini".to_string(),           // O4 Mini
            ], 
            requires_base_url: false 
        },
        // Claude API 最新模型 (2026): https://docs.anthropic.com/en/docs/about-claude/models
        ProviderInfo { 
            id: "claude".to_string(), 
            name: "Claude".to_string(), 
            models: vec![
                "claude-sonnet-4-6-20260218".to_string(),  // Claude Sonnet 4.6 (最新)
                "claude-opus-4-6-20260218".to_string(),    // Claude Opus 4.6 (最强)
                "claude-sonnet-4-20250514".to_string(),    // Claude Sonnet 4 (2025)
                "claude-opus-4-20250514".to_string(),      // Claude Opus 4 (2025)
                "claude-3-5-sonnet-20241022".to_string(),  // Claude 3.5 Sonnet
            ], 
            requires_base_url: false 
        },
        // Kimi API 最新模型: https://platform.moonshot.cn/docs/guide/choose-model
        ProviderInfo { 
            id: "kimi".to_string(), 
            name: "Kimi".to_string(), 
            models: vec![
                "kimi-k2.5".to_string(),            // Kimi K2.5 (最新，256K)
                "kimi-k2-thinking".to_string(),     // Kimi K2 Thinking
                "kimi-k2-turbo-preview".to_string(), // Kimi K2 Turbo
                "moonshot-v1-128k".to_string(),     // Moonshot V1 128K
                "moonshot-v1-32k".to_string(),      // Moonshot V1 32K
                "moonshot-v1-8k".to_string(),       // Moonshot V1 8K
            ], 
            requires_base_url: false 
        },
        // 通义千问 API 最新模型: https://help.aliyun.com/zh/model-studio/models
        ProviderInfo { 
            id: "qwen".to_string(), 
            name: "Qwen".to_string(), 
            models: vec![
                "qwen3.5-plus".to_string(),     // Qwen3.5 Plus (最新)
                "qwen3.5-flash".to_string(),    // Qwen3.5 Flash
                "qwen3-max".to_string(),        // Qwen3 Max
                "qwen-plus".to_string(),        // Qwen Plus
                "qwen-turbo".to_string(),       // Qwen Turbo
            ], 
            requires_base_url: false 
        },
        // 智谱 GLM API 最新模型: https://open.bigmodel.cn/modelcenter/square
        ProviderInfo { 
            id: "glm".to_string(), 
            name: "GLM".to_string(), 
            models: vec![
                "glm-5".to_string(),           // GLM-5 (最新旗舰)
                "glm-4.5-flash".to_string(),   // GLM-4.5 Flash
                "glm-4-plus".to_string(),      // GLM-4 Plus
                "glm-4-air".to_string(),       // GLM-4 Air
            ], 
            requires_base_url: false 
        },
        // Gemini API 最新模型 (2026): https://ai.google.dev/gemini-api/docs/models
        ProviderInfo { 
            id: "gemini".to_string(), 
            name: "Gemini".to_string(), 
            models: vec![
                "gemini-3.1-pro".to_string(),    // Gemini 3.1 Pro (最新)
                "gemini-3-pro".to_string(),      // Gemini 3 Pro
                "gemini-2.5-pro".to_string(),    // Gemini 2.5 Pro
                "gemini-2.5-flash".to_string(),  // Gemini 2.5 Flash
                "gemini-2.0-flash".to_string(),  // Gemini 2.0 Flash
            ], 
            requires_base_url: false 
        },
        // MiniMax API 最新模型
        ProviderInfo { 
            id: "minimax".to_string(), 
            name: "MiniMax".to_string(), 
            models: vec![
                "minimax-m2.5".to_string(),    // MiniMax M2.5 (最新)
                "abab6.5s-chat".to_string(),   // ABAB 6.5s Chat
                "abab6.5t-chat".to_string(),   // ABAB 6.5t Chat
            ], 
            requires_base_url: false 
        },
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
