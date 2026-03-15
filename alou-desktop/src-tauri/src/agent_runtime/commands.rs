//! Agent Runtime Tauri Commands

use std::sync::Arc;
use tauri::{State, AppHandle};
use serde::{Deserialize, Serialize};
use crate::agent_runtime::{AgentRuntimeManager, agent_registry::AgentInfo};

/// Agent Runtime 状态（Tauri 状态管理）
pub struct AgentRuntimeState {
    pub manager: Arc<AgentRuntimeManager>,
}

impl AgentRuntimeState {
    pub fn new() -> Self {
        // 返回一个空的状态，实际初始化在 init_agent_runtime 中完成
        Self {
            manager: Arc::new(
                AgentRuntimeManager::new(
                    Arc::new(crate::tools::ToolRegistry::new()),
                    Arc::new(crate::bridges::BridgeManager::new(
                        crate::bridges::BridgeConfig::default()
                    )),
                ).unwrap()
            ),
        }
    }
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

/// 发送消息请求
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub agent_id: String,
    pub content: String,
    pub group_id: Option<String>,
}

/// 初始化 AgentRuntime
#[tauri::command]
pub async fn init_agent_runtime(
    app_handle: AppHandle,
) -> Result<bool, String> {
    log::info!("初始化 AgentRuntime...");
    
    // 从 app state 获取资源
    let tool_registry = app_handle.state::<Arc<crate::tools::ToolRegistry>>();
    let bridge_manager = app_handle.state::<Arc<crate::bridges::BridgeManager>>();
    
    // 创建 AgentRuntimeManager
    let manager = AgentRuntimeManager::new(
        tool_registry.inner().clone(),
        bridge_manager.inner().clone(),
    ).await?;
    
    // 设置到 app state
    app_handle.manage(AgentRuntimeState {
        manager: Arc::new(manager),
    });
    
    log::info!("AgentRuntime 初始化完成");
    Ok(true)
}

/// 注册后端 Agent（兼容旧接口）
#[tauri::command]
pub async fn register_backend_agent(
    request: CreateAgentRequest,
    state: State<'_, AgentRuntimeState>,
) -> Result<AgentInfoResponse, String> {
    create_agent(request, state).await
}

/// 发送群聊消息（兼容旧接口）
#[tauri::command]
pub async fn send_group_message(
    request: SendMessageRequest,
    state: State<'_, AgentRuntimeState>,
) -> Result<bool, String> {
    send_to_agent(SendMessageRequest {
        agent_id: request.agent_id,
        content: request.content,
        group_id: request.group_id,
    }, state).await
}

/// 获取活跃 Agent 列表（兼容旧接口）
#[tauri::command]
pub async fn get_active_agents(
    state: State<'_, AgentRuntimeState>,
) -> Result<Vec<AgentInfoResponse>, String> {
    list_agents(state).await
}

/// 获取群聊历史（兼容旧接口）
#[tauri::command]
pub async fn get_group_history(
    _group_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    // TODO: 实现群聊历史查询
    Ok(vec![])
}

/// 提交任务（兼容旧接口）
#[tauri::command]
pub async fn submit_task(
    _task: serde_json::Value,
) -> Result<serde_json::Value, String> {
    // TODO: 实现任务提交
    Ok(serde_json::json!({"success": true}))
}

/// 执行工具（兼容旧接口）
#[tauri::command]
pub async fn execute_tool(
    _name: String,
    _args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    // TODO: 使用 ToolFacade 执行工具
    Ok(serde_json::json!({"success": true}))
}

/// 获取运行时状态（兼容旧接口）
#[tauri::command]
pub async fn get_runtime_status(
    state: State<'_, AgentRuntimeState>,
) -> Result<serde_json::Value, String> {
    get_agent_runtime_status(state).await
}

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

/// 列出群聊订阅响应
#[derive(Debug, Serialize)]
pub struct ListSubscriptionsResponse {
    pub subscriptions: Vec<GroupSubscriptionInfo>,
}

/// Agent 订阅群聊
#[tauri::command]
pub async fn agent_subscribe_group(
    request: SubscribeGroupRequest,
    state: State<'_, AgentRuntimeState>,
) -> Result<SubscribeGroupResponse, String> {
    log::info!("Agent 订阅群聊：{} -> {} ({})", request.agent_id, request.group_id, request.mode);
    
    // 转换 mode 字符串
    let mode = match request.mode.to_lowercase().as_str() {
        "pubsub" => crate::agent_runtime::agent_registry::GroupChatMode::PubSub,
        "iroh" => crate::agent_runtime::agent_registry::GroupChatMode::Iroh,
        "memory" => crate::agent_runtime::agent_registry::GroupChatMode::Memory,
        _ => return Err(format!("不支持的群聊模式：{}", request.mode)),
    };
    
    // 订阅群聊
    state.manager.state().message_bus.publish(
        crate::agent_runtime::message_bus::Event::System(crate::agent_runtime::message_bus::SystemEvent {
            event_type: "subscribe_group".to_string(),
            data: serde_json::json!({
                "agent_id": request.agent_id,
                "group_id": request.group_id,
                "mode": request.mode,
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
    
    Ok(true)
}

/// 列出 Agent 的群聊订阅
#[tauri::command]
pub async fn agent_list_subscriptions(
    agent_id: String,
    state: State<'_, AgentRuntimeState>,
) -> Result<ListSubscriptionsResponse, String> {
    log::info!("列出 Agent 群聊订阅：{}", agent_id);
    
    // TODO: 实现查询逻辑
    
    Ok(ListSubscriptionsResponse {
        subscriptions: vec![],
    })
}
