//! Agent Runtime Tauri Commands
//! 
//! 提供给前端调用的命令接口
//! 
//! 兼容性设计：
//! - 保留原有前端 unifiedAgentCoordinator 功能
//! - 后端 Agent Runtime 作为补充（后台运行）
//! - 两者通过事件桥接通信

use super::*;
use tauri::{State, AppHandle, Emitter};
use std::sync::Arc;
use tokio::sync::RwLock;

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
        }
    }
}

impl Default for AgentRuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

/// 初始化 Agent Runtime
#[tauri::command]
pub async fn init_agent_runtime(
    state: State<'_, AgentRuntimeState>,
) -> Result<bool, String> {
    let mut runtime = state.runtime.write().await;
    
    if runtime.is_some() {
        log::warn!("Agent Runtime 已初始化");
        return Ok(true);
    }
    
    let rt = AgentRuntime::new().await?;
    rt.start().await?;
    
    *runtime = Some(rt);
    log::info!("Agent Runtime 初始化完成");
    
    Ok(true)
}

/// 注册 Agent
#[tauri::command]
pub async fn register_backend_agent(
    agent: message_bus::AgentInfo,
    config: message_bus::AgentConfig,
    state: State<'_, AgentRuntimeState>,
) -> Result<String, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    let agent_id = agent.id.clone();
    rt.state.agent_supervisor.spawn(agent, config).await;
    
    log::info!("Agent 注册成功：{}", agent_id);
    Ok(agent_id)
}

/// 发送群聊消息
#[tauri::command]
pub async fn send_group_message(
    group_id: String,
    sender_id: String,
    sender_name: String,
    content: String,
    state: State<'_, AgentRuntimeState>,
) -> Result<String, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    let event = message_bus::Event::GroupMessage(message_bus::GroupMessage {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        group_id: group_id.clone(),
        sender_id,
        sender_name,
        content,
        timestamp: chrono::Utc::now().timestamp_millis(),
    });
    
    rt.state.event_router.publish(event).await;
    
    log::info!("群聊消息已发送：{}", group_id);
    Ok("消息已发送".to_string())
}

/// 获取活跃的 Agents
#[tauri::command]
pub async fn get_active_agents(
    state: State<'_, AgentRuntimeState>,
) -> Result<Vec<String>, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    let agents = rt.state.agent_supervisor.list_active_agents().await;
    Ok(agents)
}

/// 停止 Agent
#[tauri::command]
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
#[tauri::command]
pub async fn list_tools(
    state: State<'_, AgentRuntimeState>,
) -> Result<Vec<tool_bus::ToolInfo>, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    Ok(rt.state.tool_bus.list_tools().await)
}

/// 执行工具
#[tauri::command]
pub async fn execute_tool(
    tool_name: String,
    args: serde_json::Value,
    state: State<'_, AgentRuntimeState>,
) -> Result<serde_json::Value, String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    rt.state.tool_bus.execute(&tool_name, args).await
}

/// 获取运行时状态
#[tauri::command]
pub async fn get_runtime_status(
    state: State<'_, AgentRuntimeState>,
) -> Result<serde_json::Value, String> {
    let runtime = state.runtime.read().await;
    
    let status = if runtime.is_some() {
        serde_json::json!({
            "initialized": true,
            "active_agents": runtime.as_ref().unwrap().state.agent_supervisor.list_active_agents().await.len(),
            "message_bus_subscribers": runtime.as_ref().unwrap().state.message_bus.subscriber_count().await,
        })
    } else {
        serde_json::json!({
            "initialized": false,
            "active_agents": 0,
            "message_bus_subscribers": 0,
        })
    };
    
    Ok(status)
}
