//! Agent Runtime Tauri Commands - 完整版
//! 
//! 后端群聊集成关键命令
//! 设计原则：后端 = 真正的大脑，前端 = 纯 UI

use super::*;
use tauri::{State, AppHandle, Emitter};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// ============================================================================
// 状态管理
// ============================================================================

/// Agent Runtime 状态（Tauri 状态管理）
pub struct AgentRuntimeState {
    pub runtime: Arc<RwLock<Option<AgentRuntime>>>,
    pub app_handle: Arc<RwLock<Option<AppHandle>>>,
}

impl AgentRuntimeState {
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(RwLock::new(None)),
            app_handle: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 设置应用句柄（用于发送事件到前端）
    pub fn set_app_handle(&self, app_handle: AppHandle) {
        let mut handle = self.app_handle.blocking_write();
        *handle = Some(app_handle);
    }
    
    /// 发送事件到前端
    pub async fn emit_to_frontend(&self, event: &str, payload: serde_json::Value) {
        let handle = self.app_handle.read().await;
        if let Some(app) = handle.as_ref() {
            let _ = app.emit(event, payload);
            log::info!("发送事件到前端：{}", event);
        }
    }
}

impl Default for AgentRuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 请求/响应结构
// ============================================================================

/// 发送用户消息到群聊请求
#[derive(Debug, Deserialize, Serialize)]
pub struct SendUserMessageRequest {
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
}

/// Agent 加入群聊请求
#[derive(Debug, Deserialize)]
pub struct AgentJoinGroupRequest {
    pub agent_id: String,
    pub group_id: String,
}

/// 创建 Agent 请求
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub display_name: String,
    pub system_prompt: String,
    pub api_config: serde_json::Value,
}

/// Agent 信息响应
#[derive(Debug, Serialize)]
pub struct AgentInfoResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub status: String,
}

// ============================================================================
// 核心命令
// ============================================================================

/// 初始化 Agent Runtime
#[tauri::command]
pub async fn init_agent_runtime(
    state: State<'_, AgentRuntimeState>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    let mut runtime = state.runtime.write().await;

    if runtime.is_some() {
        log::warn!("Agent Runtime 已初始化");
        return Ok(true);
    }

    // Create required dependencies
    let tool_registry = Arc::new(crate::tools::ToolRegistry::new());
    
    // 创建 ProviderRegistry 用于媒体工具
    let provider_registry = match crate::agent::media_config::MediaApiConfig::load() {
        Ok(media_config) => {
            match crate::agent::providers::ProviderRegistry::new(&media_config) {
                Ok(registry) => {
                    log::info!("ProviderRegistry 创建成功");
                    Some(Arc::new(registry))
                }
                Err(e) => {
                    log::warn!("Failed to create ProviderRegistry: {}, using None", e);
                    None
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to load MediaApiConfig: {}, using None", e);
            None
        }
    };
    
    // 创建媒体存档管理器
    let archive_manager = match crate::media_archive::MediaArchiveManager::new() {
        Ok(am) => Some(Arc::new(am)),
        Err(e) => {
            log::warn!("Failed to create MediaArchiveManager: {}, using None", e);
            None
        }
    };
    
    // 创建 ToolBus 并注册媒体工具
    let mut tool_bus = crate::agent_runtime::tool_bus::ToolBus::new();
    if let (Some(provider_reg), Some(archive_mgr)) = (&provider_registry, &archive_manager) {
        tool_bus.register_media_tools(provider_reg.clone(), archive_mgr.clone());
    }
    let tool_bus = Arc::new(tool_bus);
    
    let bridge_manager = Arc::new(crate::bridges::BridgeManager::new_with_toolbus(
        crate::bridges::BridgeConfig {
            tool_bridge: crate::bridges::ToolBridgeConfig::default(),
            context_bridge: crate::bridges::ContextBridgeConfig::default(),
            enabled: true,
            max_concurrent_calls: 10,
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            debug_mode: false,
        },
        Some(tool_bus.clone())
    ));

    let rt = AgentRuntime::new(tool_registry, bridge_manager).await?;
    rt.start().await?;

    // 设置 app handle 用于事件发送
    state.set_app_handle(app_handle.clone());

    *runtime = Some(rt);
    log::info!("Agent Runtime 初始化完成");

    // Start media API server after AgentRuntime is initialized
    let _app_handle_for_media = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        use crate::media_api::start_media_api_server;
        // 使用已创建的 ProviderRegistry
        if let Some(provider_reg) = provider_registry {
            match start_media_api_server(provider_reg).await {
                Ok(port) => {
                    println!("Media API server started on port {}", port);
                    // Write port to config file for frontend to read
                    let config_dir = dirs::config_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("alou");
                    let _ = std::fs::create_dir_all(&config_dir);
                    let port_file = config_dir.join("media_api_port");
                    let _ = std::fs::write(port_file, port.to_string());
                }
                Err(e) => eprintln!("Failed to start media API server: {}", e),
            }
        } else {
            eprintln!("ProviderRegistry not available, skipping media API server startup");
        }
    });

    Ok(true)
}

/// 发送用户消息到群聊
// #[tauri::command] 已注释
pub async fn send_user_message(
    request: SendUserMessageRequest,
    state: State<'_, AgentRuntimeState>,
) -> Result<(), String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;

    // Store request data before moving
    let group_id = request.group_id.clone();
    let sender_id = request.sender_id.clone();
    let sender_name = request.sender_name.clone();
    let content = request.content.clone();

    // 创建群聊消息事件
    let event = message_bus::Event::GroupMessage(message_bus::GroupMessage {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        group_id: group_id.clone(),
        sender_id,
        sender_name,
        content,
        timestamp: chrono::Utc::now().timestamp_millis(),
    });

    // 发布到 MessageBus
    rt.state.message_bus.publish(event.clone()).await;

    // 同时发送到前端显示
    let frontend_request = SendUserMessageRequest {
        group_id,
        sender_id: request.sender_id,
        sender_name: request.sender_name,
        content: request.content,
    };
    state.emit_to_frontend("user-message-sent", serde_json::to_value(&frontend_request).unwrap()).await;

    log::info!("用户消息已发送到群聊：{}", request.group_id);
    Ok(())
}

/// Agent 加入群聊
// #[tauri::command] 已注释
pub async fn agent_join_group_chat(
    request: AgentJoinGroupRequest,
    state: State<'_, AgentRuntimeState>,
) -> Result<(), String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    // Agent Supervisor 加入群聊
    rt.state.agent_supervisor.join_group_chat(&request.agent_id, &request.group_id).await?;
    
    log::info!("Agent 已加入群聊：{} -> {}", request.agent_id, request.group_id);
    Ok(())
}

/// 注册后端 Agent
#[tauri::command]
pub async fn register_backend_agent(
    agent: crate::agent_runtime::agent_registry::AgentInfo,
    config: crate::agent_runtime::agent_registry::AgentConfig,
    state: State<'_, AgentRuntimeState>,
) -> Result<String, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    let agent_id = agent.id.clone();
    rt.state.agent_supervisor.spawn(agent, config).await;
    
    log::info!("Agent 注册成功：{}", agent_id);
    Ok(agent_id)
}

/// 获取活跃的 Agents
// #[tauri::command] 已注释
pub async fn get_active_agents(
    state: State<'_, AgentRuntimeState>,
) -> Result<Vec<String>, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    let agents = rt.state.agent_supervisor.list_active_agents().await;
    Ok(agents)
}

/// 停止 Agent
// #[tauri::command] 已注释
pub async fn stop_agent(
    agent_id: String,
    state: State<'_, AgentRuntimeState>,
) -> Result<(), String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    rt.state.agent_supervisor.stop(&agent_id).await
}

/// 获取群聊历史消息
#[tauri::command]
pub async fn get_group_history(
    group_id: String,
    limit: i32,
    state: State<'_, AgentRuntimeState>,
) -> Result<Vec<storage::StoredMessage>, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    rt.state.storage.get_group_history(&group_id, limit).await
}

/// 提交任务到队列
#[tauri::command]
pub async fn submit_task(
    agent_id: String,
    action: String,
    payload: serde_json::Value,
    state: State<'_, AgentRuntimeState>,
) -> Result<String, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    let task_action = match action.as_str() {
        "send_message" => {
            let group_id = payload.get("group_id")
                .and_then(|v| v.as_str())
                .ok_or("缺少 group_id")?
                .to_string();
            let content = payload.get("content")
                .and_then(|v| v.as_str())
                .ok_or("缺少 content")?
                .to_string();
            task_queue::TaskAction::SendMessage { group_id, content }
        }
        "check_activity" => {
            let group_id = payload.get("group_id")
                .and_then(|v| v.as_str())
                .ok_or("缺少 group_id")?
                .to_string();
            task_queue::TaskAction::CheckGroupActivity { group_id }
        }
        _ => return Err(format!("不支持的任务类型：{}", action)),
    };
    
    let task = task_queue::TaskQueue::create_task(agent_id, task_action, 5);
    let task_id = task.id.clone();
    
    rt.state.task_queue.submit(task).await?;
    
    Ok(task_id)
}

/// 列出可用工具
// #[tauri::command] 已注释
pub async fn list_tools(
    state: State<'_, AgentRuntimeState>,
) -> Result<Vec<tool_bus::ToolInfo>, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    Ok(rt.state.tool_bus.list_tools().await)
}

/// 获取运行时状态
#[tauri::command]
pub async fn get_runtime_status(
    state: State<'_, AgentRuntimeState>,
) -> Result<serde_json::Value, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    let active_agents = rt.state.agent_supervisor.list_active_agents().await;
    let task_queue_size = rt.state.task_queue.get_queue_size().await;
    
    Ok(serde_json::json!({
        "initialized": true,
        "active_agents": active_agents,
        "task_queue_size": task_queue_size,
    }))
}





// ============================================================================
// 群聊订阅管理（辅助命令）
// ============================================================================

/// 订阅群聊请求
#[derive(Debug, Deserialize)]
pub struct SubscribeGroupRequest {
    pub agent_id: String,
    pub group_id: String,
    pub mode: String,  // "pubsub", "iroh", "memory"
}

/// 群聊订阅响应
#[derive(Debug, Serialize)]
pub struct SubscribeGroupResponse {
    pub success: bool,
    pub group_id: String,
    pub mode: String,
}

/// Agent 订阅群聊
#[tauri::command]
pub async fn agent_subscribe_group(
    request: SubscribeGroupRequest,
    state: State<'_, AgentRuntimeState>,
) -> Result<SubscribeGroupResponse, String> {
    log::info!("Agent 订阅群聊：{} -> {} ({})", request.agent_id, request.group_id, request.mode);

    // 转换 mode 字符串为枚举
    let mode = match request.mode.to_lowercase().as_str() {
        "pubsub" => crate::agent_runtime::agent_registry::GroupChatMode::PubSub,
        "iroh" => crate::agent_runtime::agent_registry::GroupChatMode::Iroh,
        "memory" => crate::agent_runtime::agent_registry::GroupChatMode::Memory,
        _ => return Err(format!("不支持的群聊模式：{}", request.mode)),
    };

    // 发布订阅事件到 MessageBus
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    rt.state.message_bus.publish(
        message_bus::Event::System(message_bus::SystemEvent {
            event_type: "subscribe_group".to_string(),
            data: serde_json::json!({
                "agent_id": request.agent_id,
                "group_id": request.group_id,
                "mode": format!("{:?}", mode),
            }),
        })
    ).await;

    Ok(SubscribeGroupResponse {
        success: true,
        group_id: request.group_id,
        mode: request.mode,
    })
}

/// Agent 取消订阅群聊
#[tauri::command]
pub async fn agent_unsubscribe_group(
    agent_id: String,
    group_id: String,
    state: State<'_, AgentRuntimeState>,
) -> Result<bool, String> {
    log::info!("Agent 取消订阅群聊：{} -> {}", agent_id, group_id);

    // TODO: 实现取消订阅逻辑
    // 需要调用 GroupChatBridge 的 unsubscribe 方法

    Ok(true)
}

/// 列出 Agent 的群聊订阅
#[derive(Debug, Serialize)]
pub struct GroupSubscriptionInfo {
    pub group_id: String,
    pub mode: String,
}

#[tauri::command]
pub async fn agent_list_subscriptions(
    _agent_id: String,
    _state: State<'_, AgentRuntimeState>,
) -> Result<Vec<GroupSubscriptionInfo>, String> {
    // TODO: 实现查询逻辑
    Ok(vec![])
}

/// 重新加载媒体工具（配置更新后调用）
#[tauri::command]
pub async fn reload_media_tools(
    state: State<'_, AgentRuntimeState>,
) -> Result<serde_json::Value, String> {
    log::info!("[reload_media_tools] 开始重新加载媒体工具...");
    
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    // 1. 重新加载媒体配置
    let media_config = match crate::agent::media_config::MediaApiConfig::load() {
        Ok(config) => {
            log::info!("[reload_media_tools] 媒体配置加载成功，providers: {:?}", config.providers.keys().collect::<Vec<_>>());
            config
        }
        Err(e) => {
            log::warn!("[reload_media_tools] 媒体配置加载失败: {}, 使用默认配置", e);
            crate::agent::media_config::MediaApiConfig::default()
        }
    };
    
    // 2. 创建新的 ProviderRegistry
    let provider_registry = match crate::agent::providers::ProviderRegistry::new(&media_config) {
        Ok(registry) => {
            log::info!("[reload_media_tools] ProviderRegistry 创建成功，可用 providers: {:?}", registry.available_providers());
            registry
        }
        Err(e) => {
            return Err(format!("创建 ProviderRegistry 失败: {}", e));
        }
    };
    
    // 3. 创建 ArchiveManager
    let archive_manager = match crate::media_archive::MediaArchiveManager::new() {
        Ok(am) => am,
        Err(e) => {
            log::warn!("[reload_media_tools] ArchiveManager 创建失败: {}, 使用默认实例", e);
            return Err(format!("创建 ArchiveManager 失败: {}", e));
        }
    };
    
    // 4. 重新注册媒体工具到 ToolBus
    let provider_count = provider_registry.available_providers().len();
    rt.state.tool_bus.register_media_tools(
        std::sync::Arc::new(provider_registry),
        std::sync::Arc::new(archive_manager),
    );
    
    log::info!("[reload_media_tools] 媒体工具重新注册完成，共 {} 个 Provider", provider_count);
    
    // 5. 返回可用的 Provider 列表
    let available_providers: Vec<String> = rt.state.tool_bus.list_tools().await
        .into_iter()
        .filter(|t| ["generate_image", "generate_video", "generate_audio", "get_video_status"].contains(&t.name.as_str()))
        .map(|t| t.name)
        .collect();
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("媒体工具重新加载成功，共 {} 个 Provider", provider_count),
        "available_tools": available_providers,
        "provider_count": provider_count,
    }))
}
