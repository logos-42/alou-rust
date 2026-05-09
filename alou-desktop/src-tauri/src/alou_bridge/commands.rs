use serde_json::{Value, json};
use std::sync::Arc;
use tauri::State;
use crate::alou_bridge::AlouBridge;

#[tauri::command]
pub async fn create_unified_session(
    bridge: State<'_, Arc<AlouBridge>>,
    diap_identity: Value,
) -> Result<String, String> {
    bridge.create_session(diap_identity).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_unified_session(
    bridge: State<'_, Arc<AlouBridge>>,
    session_id: String,
) -> Result<Option<Value>, String> {
    let session = bridge.get_session(&session_id).await;
    Ok(session.map(|s| json!({
        "id": s.id,
        "diap_identity": s.diap_identity,
        "alou_session_id": s.alou_session_id,
        "created_at": s.created_at.to_rfc3339(),
        "last_active": s.last_active.to_rfc3339(),
        "state": format!("{:?}", s.state),
    })))
}

#[tauri::command]
pub async fn send_alou_message(
    bridge: State<'_, Arc<AlouBridge>>,
    session_id: String,
    message: String,
) -> Result<Value, String> {
    bridge.send_message(&session_id, message).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_alou_tool(
    bridge: State<'_, Arc<AlouBridge>>,
    session_id: String,
    tool_name: String,
    input: Value,
) -> Result<Value, String> {
    bridge.execute_tool(&session_id, tool_name, input).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_alou_tool_executors(
    bridge: State<'_, Arc<AlouBridge>>,
) -> Result<Vec<String>, String> {
    Ok(bridge.list_tool_executors().await)
}

#[tauri::command]
pub async fn health_check_alou_bridge(
    bridge: State<'_, Arc<AlouBridge>>,
) -> Result<Value, String> {
    Ok(json!({
        "status": "ok",
        "bridge_version": "0.4.0",
        "features": ["unified_session", "diap_binding", "tool_execution"],
    }))
}
