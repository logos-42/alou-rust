//! Tauri Commands
//!
//! 提供给前端调用的 Tauri 命令
//! 
//! # New Commands (Skills & Task System)
//! 
//! - Skills: discover, load, execute, search
//! - Tasks: create, decompose, execute_swarm, get_status
//! - Agent: configure_autonomy, register_agent, get_agent_status

use super::ai_client::AiClient;
use super::config::{ApiConfig, UserApiConfig};
use super::executor::RalphLoopExecutor;
use super::streaming::StreamingExecutor;
use super::task::{TaskFinalResult, TaskManager};
use std::sync::Arc;

/// Tauri 命令：执行 Agent 任务（同步返回结果）
#[tauri::command]
pub async fn execute_agent_task(
    message: String,
    agent_id: String,
    config: UserApiConfig,
    bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::bridges::BridgeManager>>>,
) -> std::result::Result<TaskFinalResult, String> {
    log::info!("[Command] 执行 Agent 任务: {}", message);

    // 获取 BridgeManager
    let manager = bridge_manager.lock().await;
    let tool_bridge = manager.tool_bridge();

    // 创建任务管理器
    let task_manager = Arc::new(TaskManager::new());

    // 创建 AI 客户端
    let ai_client = match AiClient::new(&config) {
        Ok(client) => Arc::new(client),
        Err(e) => return Err(format!("创建 AI 客户端失败: {}", e)),
    };

    // 创建执行器
    let executor = RalphLoopExecutor::new(
        ai_client.clone(),
        task_manager.clone(),
        Arc::new(tool_bridge.clone()),
        Arc::new(crate::tools::ToolRegistry::new()),
    );

    // 创建任务
    let task_id = task_manager.create_task(agent_id, message).await;

    // 执行任务
    match executor.execute(&task_id).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

/// Tauri 命令：获取 Agent 配置
#[tauri::command]
pub async fn get_agent_config() -> std::result::Result<ApiConfig, String> {
    match ApiConfig::load().await {
        Ok(config) => Ok(config),
        Err(e) => Err(format!("加载配置失败: {}", e)),
    }
}

/// Tauri 命令：更新 Agent 配置
#[tauri::command]
pub async fn update_agent_config(config: ApiConfig) -> std::result::Result<(), String> {
    match config.save().await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("保存配置失败: {}", e)),
    }
}

/// Tauri 命令：测试 API 连接
#[tauri::command]
pub async fn test_api_connection(config: UserApiConfig) -> std::result::Result<TestResult, String> {
    let client = match AiClient::new(&config) {
        Ok(client) => client,
        Err(e) => {
            return Ok(TestResult {
                success: false,
                message: e.to_string(),
            });
        }
    };

    use super::ai_client::AiMessage;

    let test_message = AiMessage {
        role: "user".to_string(),
        content: "Hello, this is a test.".to_string(),
        tool_call_id: None,
        tool_calls: None,
    };

    match client.send_message(vec![test_message], None).await {
        Ok(_) => Ok(TestResult {
            success: true,
            message: "连接成功".to_string(),
        }),
        Err(e) => Ok(TestResult {
            success: false,
            message: e.to_string(),
        }),
    }
}

/// Tauri 命令：执行 AI 对话（支持流式响应）
#[tauri::command]
pub async fn execute_ai_conversation(
    app_handle: tauri::AppHandle,
    agent_config: serde_json::Value,
    message: String,
    messages: Option<Vec<serde_json::Value>>, // 完整的对话历史消息数组（role + content）
    options: Option<serde_json::Value>,
    agent_id: Option<String>, // 智能体 ID（可选，主要用于 agent_document 工具）
    bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::bridges::BridgeManager>>>,
) -> std::result::Result<serde_json::Value, String> {
    log::info!("[Command] 执行 AI 对话: {}", &message[..20.min(message.len())]);
    
    // 获取 BridgeManager
    let manager = bridge_manager.lock().await;
    let tool_bridge = manager.tool_bridge();

    // 解析配置
    let user_config = match serde_json::from_value::<UserApiConfig>(agent_config) {
        Ok(config) => config,
        Err(e) => return Err(format!("解析 agent 配置失败: {}", e)),
    };

    // 创建任务管理器
    let task_manager = Arc::new(TaskManager::new());

    // 创建 AI 客户端
    let ai_client = match AiClient::new(&user_config) {
        Ok(client) => Arc::new(client),
        Err(e) => return Err(format!("创建 AI 客户端失败: {}", e)),
    };

    // 创建执行器（附带 AppHandle，以便向前端推送进度事件）
    let executor = RalphLoopExecutor::new(
        ai_client.clone(),
        task_manager.clone(),
        Arc::new(tool_bridge.clone()),
        Arc::new(crate::tools::ToolRegistry::new()),
    )
    .with_app_handle(app_handle);

    // 使用传入的 agent_id，如果没有则使用默认值
    let agent_id = agent_id.unwrap_or_else(|| "conversation".to_string());

    // 创建任务：优先使用 messages 数组（包含完整上下文），否则退回单条消息
    use super::ai_client::AiMessage;
    let task_id = if let Some(msgs) = messages {
        // 将 JSON 数组转换为 AiMessage 列表
        let ai_messages: Vec<AiMessage> = msgs
            .into_iter()
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
        if ai_messages.is_empty() {
            task_manager.create_task(agent_id.clone(), message).await
        } else {
            task_manager.create_task_with_messages(agent_id.clone(), ai_messages).await
        }
    } else {
        task_manager.create_task(agent_id.clone(), message).await
    };

    // 检查是否需要流式响应
    let use_stream = options
        .as_ref()
        .and_then(|o| o.get("stream"))
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    if use_stream {
        // 流式执行
        let streaming_executor = StreamingExecutor::new(task_manager.clone());
        let _stream = streaming_executor.execute_stream(task_id.clone()).await;
        
        log::info!("[Command] 启动流式执行: {}", task_id);
        
        Ok(serde_json::json!({
            "success": true,
            "task_id": task_id,
            "stream": true,
            "execution_mode": "local",
            "timestamp": chrono::Utc::now().timestamp()
        }))
    } else {
        // 同步执行
        match executor.execute(&task_id).await {
            Ok(result) => {
                log::info!("[Command] 对话执行成功: {}", task_id);
                Ok(serde_json::json!({
                    "success": true,
                    "result": result,
                    "execution_mode": "local",
                    "timestamp": chrono::Utc::now().timestamp()
                }))
            }
            Err(e) => {
                log::error!("[Command] 对话执行失败: {}", e);
                Err(format!("AI 对话执行失败: {}", e))
            }
        }
    }
}

/// Tauri 命令：健康检查
#[tauri::command]
pub async fn health_check() -> std::result::Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "healthy",
        "mode": "local",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// Tauri 命令：获取可用 Provider 列表
#[tauri::command]
pub async fn get_available_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            models: vec![
                "deepseek-chat".to_string(),
                "deepseek-reasoner".to_string(),
            ],
            requires_base_url: false,
        },
        ProviderInfo {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            models: vec![
                "gpt-4".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-3.5-turbo".to_string(),
                "gpt-5".to_string(),
            ],
            requires_base_url: false,
        },
        ProviderInfo {
            id: "claude".to_string(),
            name: "Claude".to_string(),
            models: vec![
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-4-5-sonnet-20241022".to_string(),
            ],
            requires_base_url: false,
        },
        ProviderInfo {
            id: "kimi".to_string(),
            name: "Kimi".to_string(),
            models: vec!["kimi-k2-turbo-preview".to_string()],
            requires_base_url: false,
        },
    ]
}

/// Provider 信息
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub models: Vec<String>,
    pub requires_base_url: bool,
}

/// 测试结果
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
}
