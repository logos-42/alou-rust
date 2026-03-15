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
        let streaming_executor = StreamingExecutor::new(task_manager.clone());
        let _stream = streaming_executor.execute_stream(task_id.clone()).await;
        Ok(serde_json::json!({
            "success": true,
            "task_id": task_id,
            "stream": true,
            "session_id": target_session_id,
            "timestamp": chrono::Utc::now().timestamp()
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
        ProviderInfo { id: "deepseek".to_string(), name: "DeepSeek".to_string(), models: vec!["deepseek-chat".to_string(), "deepseek-reasoner".to_string()], requires_base_url: false },
        ProviderInfo { id: "openai".to_string(), name: "OpenAI".to_string(), models: vec!["gpt-4".to_string(), "gpt-4-turbo".to_string(), "gpt-3.5-turbo".to_string(), "gpt-5".to_string()], requires_base_url: false },
        ProviderInfo { id: "claude".to_string(), name: "Claude".to_string(), models: vec!["claude-3-opus-20240229".to_string(), "claude-3-sonnet-20240229".to_string(), "claude-4-5-sonnet-20241022".to_string()], requires_base_url: false },
        ProviderInfo { id: "kimi".to_string(), name: "Kimi".to_string(), models: vec!["kimi-k2-turbo-preview".to_string()], requires_base_url: false },
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
