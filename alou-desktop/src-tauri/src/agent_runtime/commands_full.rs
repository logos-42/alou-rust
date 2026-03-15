//! Agent Runtime Tauri Commands - 完整版
//! 
//! 后端群聊集成关键命令

use super::*;
use tauri::{State, AppHandle, Emitter};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Agent Runtime 状态
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
    
    pub fn set_app_handle(&self, app_handle: AppHandle) {
        let mut handle = self.app_handle.blocking_write();
        *handle = Some(app_handle);
    }
    
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

/// 发送用户消息到群聊
#[tauri::command]
pub async fn send_user_message(
    group_id: String,
    sender_id: String,
    sender_name: String,
    content: String,
    state: State<'_, AgentRuntimeState>,
) -> Result<(), String> {
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
    
    rt.state.message_bus.publish(event).await;
    log::info!("用户消息已发送到群聊：{}", group_id);
    Ok(())
}

/// Agent 加入群聊
#[tauri::command]
pub async fn agent_join_group_chat(
    agent_id: String,
    group_id: String,
    state: State<'_, AgentRuntimeState>,
) -> Result<(), String> {
    let runtime = state.runtime.read().await;
    let rt = runtime.as_ref().ok_or("Agent Runtime 未初始化")?;
    
    rt.state.agent_supervisor.join_group_chat(&agent_id, &group_id).await?;
    log::info!("Agent 已加入群聊：{} -> {}", agent_id, group_id);
    Ok(())
}
