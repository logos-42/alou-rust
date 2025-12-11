use crate::agent::core::AgentCore;
use crate::agent::session::{ContextEvent, SessionManager};
use crate::agent::stream::{cleanup_session, StreamEvent, StreamPublisher};
use crate::utils::error::AloudError;
use crate::web3::auth::WalletAuth;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::*;

use super::{json_response, json_response_with_status, ErrorResponse};

const MAX_EVENTS_PER_REQUEST: usize = 12;
const MAX_PROMPT_EVENTS: usize = 8;
const MAX_EVENT_DETAIL_CHARS: usize = 120;
const MAX_EVENT_SUMMARY_CHARS: usize = 800;

#[derive(Deserialize)]
struct CreateSessionRequest {
    #[serde(default)]
    wallet_address: Option<String>,
    #[serde(default)]
    chain: Option<String>,
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
}

#[derive(Deserialize, Debug, Clone)]
struct RawContextEvent {
    action: String,
    #[serde(default)]
    detail: Option<Value>,
    #[serde(default)]
    timestamp: Option<i64>,
}

impl RawContextEvent {
    fn to_context_event(&self) -> Option<ContextEvent> {
        if self.action.trim().is_empty() {
            return None;
        }

        let timestamp = self
            .timestamp
            .unwrap_or_else(|| crate::utils::time::now_timestamp() * 1_000);

        Some(ContextEvent {
            action: self.action.clone(),
            detail: self.detail.clone().unwrap_or(Value::Null),
            timestamp,
        })
    }
}

#[derive(Deserialize)]
struct ChatRequest {
    session_id: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<String>,
    #[serde(default)]
    context_events: Vec<RawContextEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_info: Option<Value>, // 包含 name, role_description, custom_prompt, mode 等
}

#[derive(Serialize)]
struct ChatResponse {
    content: String,
    session_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_calls: Vec<crate::agent::core::ToolCallInfo>,
}

pub(crate) async fn handle_create_session(
    session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    let body: CreateSessionRequest = match req.json().await {
        Ok(body) => body,
        Err(_) => CreateSessionRequest {
            wallet_address: None,
            chain: None,
        },
    };

    console_log!(
        "Creating session with wallet_address: {:?}, chain: {:?}",
        body.wallet_address,
        body.chain
    );

    match session_manager
        .create_session(body.wallet_address, body.chain)
        .await
    {
        Ok(session_id) => {
            console_log!("Session created: {}", session_id);
            let response = CreateSessionResponse { session_id };
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

pub(crate) async fn handle_get_session(
    session_manager: &SessionManager,
    session_id: &str,
) -> Result<Response> {
    match session_manager.get_session(session_id).await {
        Ok(session) => json_response(&session),
        Err(AloudError::InvalidInput(_)) => {
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", session_id),
            };
            json_response_with_status(&error_response, 404)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

pub(crate) async fn handle_delete_session(
    session_manager: &SessionManager,
    session_id: &str,
) -> Result<Response> {
    match session_manager.clear_session(session_id).await {
        Ok(_) => {
            cleanup_session(session_id).await;
            Response::ok("Session deleted")
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

pub(crate) async fn handle_agent_chat(
    session_manager: &SessionManager,
    agent_core: Option<&AgentCore>,
    wallet_auth: Option<&WalletAuth>,
    req: &mut Request,
) -> Result<Response> {
    let agent_core = match agent_core {
        Some(core) => core,
        None => {
            let error_response = ErrorResponse {
                error: "Agent not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let body: ChatRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    let stream_publisher = StreamPublisher::new(&body.session_id);
    stream_publisher
        .publish(
            StreamEvent::new(&body.session_id, "conversation.received")
                .with_label("收到会话请求")
                .with_payload(json!({
                    "message_preview": body.message.chars().take(140).collect::<String>(),
                })),
        )
        .await;

    console_log!(
        "Chat request - session_id: {}, wallet_address from body: {:?}",
        body.session_id,
        body.wallet_address
    );

    let mut incoming_events: Vec<ContextEvent> = body
        .context_events
        .iter()
        .filter_map(|raw| raw.to_context_event())
        .collect();

    if incoming_events.len() > MAX_EVENTS_PER_REQUEST {
        incoming_events = incoming_events
            .into_iter()
            .rev()
            .take(MAX_EVENTS_PER_REQUEST)
            .collect::<Vec<_>>();
        incoming_events.reverse();
    }

    if !incoming_events.is_empty() {
        if let Err(e) = session_manager
            .append_context_events(&body.session_id, &incoming_events)
            .await
        {
            console_log!(
                "Failed to append context events for session {}: {}",
                body.session_id,
                e
            );
        }
    }

    let session = match session_manager.get_session(&body.session_id).await {
        Ok(session) => Some(session),
        Err(e) => {
            console_log!(
                "Failed to load session {} while handling chat: {}",
                body.session_id,
                e
            );
            None
        }
    };

    let wallet_from_body = body.wallet_address.clone();
    let wallet_from_token = wallet_auth.and_then(|auth| {
        super::wallet::extract_wallet_from_token(auth, req)
            .map_err(|e| {
                console_log!(
                    "Failed to extract wallet from token for session {}: {}",
                    body.session_id,
                    e
                );
            })
            .ok()
    });

    let mut wallet_address = wallet_from_body.or(wallet_from_token);
    console_log!("Wallet address after body/jwt: {:?}", wallet_address);

    if wallet_address.is_none() {
        if let Some(session) = session.as_ref() {
            console_log!("Wallet from session: {:?}", session.wallet_address);
            wallet_address = session.wallet_address.clone();
        } else {
            console_log!("No wallet found anywhere");
        }
    } else if let Some(addr) = wallet_address.as_ref() {
        console_log!("Using wallet from body/jwt: {}", addr);
    }

    let chain = body
        .chain
        .clone()
        .or_else(|| session.as_ref().and_then(|s| s.chain.clone()));

    if let Some(ref new_chain) = body.chain {
        if let Err(e) = session_manager
            .update_session_chain(&body.session_id, Some(new_chain.clone()))
            .await
        {
            console_log!(
                "Failed to update session chain for {}: {}",
                body.session_id,
                e
            );
        }
    }

    let combined_events = if let Some(session) = session.as_ref() {
        if !session.recent_events.is_empty() {
            session.recent_events.clone()
        } else {
            incoming_events.clone()
        }
    } else {
        incoming_events.clone()
    };

    let (events_for_prompt, event_summary) = summarize_context_events(&combined_events);

    if let Some(summary) = event_summary.as_ref() {
        console_log!(
            "Context summary for session {} ({} events): {}",
            body.session_id,
            events_for_prompt.len(),
            summary
        );
    }

    console_log!(
        "Final wallet_address being passed to agent: {:?}",
        wallet_address
    );

    // 如果提供了 agent_info，更新 agent_metadata
    if let Some(ref agent_info) = body.agent_info {
        if let Some(ref agent_id) = body.agent_id {
            // 构建完整的 metadata，包含 mode
            let mut metadata = agent_info.clone();
            // 确保 agent_id 也在 metadata 中
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("agent_id".to_string(), json!(agent_id));
            }
            if let Err(e) = session_manager
                .set_agent_metadata(&body.session_id, metadata)
                .await
            {
                console_log!(
                    "Failed to set agent metadata for session {}: {}",
                    body.session_id,
                    e
                );
            } else {
                console_log!(
                    "Agent metadata updated for session {} with mode: {:?}",
                    body.session_id,
                    agent_info.get("mode")
                );
            }
        }
    }

    match agent_core
        .handle_message(
            &body.session_id,
            &body.message,
            wallet_address,
            chain.clone(),
            events_for_prompt,
            event_summary.clone(),
            Some(stream_publisher.clone()),
        )
        .await
    {
        Ok(response) => {
            let chat_response = ChatResponse {
                content: response.content.clone(),
                session_id: response.session_id,
                tool_calls: response.tool_calls.clone(),
            };
            stream_publisher
                .publish(
                    StreamEvent::new(&body.session_id, "conversation.completed")
                        .with_label("代理响应完成")
                        .with_payload(json!({
                            "content": response.content,
                            "tool_calls": response.tool_calls,
                        }))
                        .mark_final(),
                )
                .await;
            json_response(&chat_response)
        }
        Err(e) => {
            let error_msg = e.to_string();
            console_log!(
                "Agent chat error for session {}: {}",
                body.session_id,
                error_msg
            );
            stream_publisher
                .publish(
                    StreamEvent::new(&body.session_id, "conversation.error")
                        .with_label("代理执行失败")
                        .with_payload(json!({
                            "message": error_msg.clone(),
                        }))
                        .mark_final(),
                )
                .await;
            let error_response = ErrorResponse {
                error: format!("Agent execution failed: {}", error_msg),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

fn summarize_context_events(events: &[ContextEvent]) -> (Vec<ContextEvent>, Option<String>) {
    if events.is_empty() {
        return (Vec::new(), None);
    }

    let start = events.len().saturating_sub(MAX_PROMPT_EVENTS);
    let raw_selected = &events[start..];

    let mut summary_lines = Vec::new();
    let mut total_chars = 0;
    let mut sanitized_events = Vec::with_capacity(raw_selected.len());

    for event in raw_selected {
        let sanitized = sanitize_event(event);
        let mut line = format!("- {}", sanitized.action.trim());

        if let Value::String(detail_str) = &sanitized.detail {
            if !detail_str.is_empty() {
                line.push_str(": ");
                line.push_str(detail_str);
            }
        }

        let line_len = line.len();
        if total_chars + line_len > MAX_EVENT_SUMMARY_CHARS {
            break;
        }

        summary_lines.push(line);
        total_chars += line_len;
        sanitized_events.push(sanitized);
    }

    let summary = if summary_lines.is_empty() {
        None
    } else {
        Some(summary_lines.join("\n"))
    };

    (sanitized_events, summary)
}

fn sanitize_event(event: &ContextEvent) -> ContextEvent {
    let mut detail_value = event.detail.clone();
    let mut snippet = String::new();

    if !detail_value.is_null() {
        let raw_detail = match &detail_value {
            Value::String(s) => s.clone(),
            _ => serde_json::to_string(&detail_value).unwrap_or_else(|_| "".to_string()),
        };

        snippet = raw_detail.trim().to_string();
        if snippet.len() > MAX_EVENT_DETAIL_CHARS {
            snippet.truncate(MAX_EVENT_DETAIL_CHARS);
            snippet.push('…');
        }
    }

    if snippet.is_empty() {
        detail_value = Value::Null;
    } else {
        detail_value = Value::String(snippet);
    }

    ContextEvent {
        action: event.action.clone(),
        detail: detail_value,
        timestamp: event.timestamp,
    }
}
