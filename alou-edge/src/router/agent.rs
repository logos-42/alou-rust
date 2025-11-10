use crate::agent::discovery::{AgentDiscovery, ResolvedAgent};
use crate::agent::session::SessionManager;
use crate::utils::error::AloudError;
use serde::{Deserialize, Serialize};
use worker::*;

use super::{json_response, json_response_with_status, ErrorResponse};

#[derive(Deserialize)]
pub(crate) struct ResolveAgentRequest {
    pub target: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SearchAgentRequest {
    pub query: String,
}

#[derive(Serialize)]
struct SearchAgentResponse {
    pub agents: Vec<ResolvedAgent>,
    pub count: usize,
}

pub(crate) async fn handle_resolve_agent(
    discovery: Option<&AgentDiscovery>,
    session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    let discovery = match discovery {
        Some(d) => d,
        None => {
            let error_response = ErrorResponse {
                error: "Agent discovery is not configured".to_string(),
            };
            return json_response_with_status(&error_response, 503);
        }
    };

    let body: ResolveAgentRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    if body.target.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "target is required".to_string(),
        };
        return json_response_with_status(&error_response, 400);
    }

    match discovery.resolve_agent(&body.target).await {
        Ok(agent) => {
            if let Some(session_id) = body.session_id {
                if let Ok(metadata) = serde_json::to_value(&agent) {
                    if let Err(e) = session_manager
                        .set_agent_metadata(&session_id, metadata)
                        .await
                    {
                        console_warn!(
                            "Failed to store agent metadata for session {}: {}",
                            session_id,
                            e
                        );
                    }
                }
            }
            json_response(&agent)
        }
        Err(AloudError::InvalidInput(message)) => {
            let error_response = ErrorResponse { error: message };
            json_response_with_status(&error_response, 400)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

pub(crate) async fn handle_search_agents(
    discovery: Option<&AgentDiscovery>,
    req: &mut Request,
) -> Result<Response> {
    let discovery = match discovery {
        Some(d) => d,
        None => {
            let error_response = ErrorResponse {
                error: "Agent discovery is not configured".to_string(),
            };
            return json_response_with_status(&error_response, 503);
        }
    };

    let body: SearchAgentRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    if body.query.trim().is_empty() {
        return json_response(&SearchAgentResponse {
            agents: Vec::new(),
            count: 0,
        });
    }

    match discovery.search_agents(&body.query).await {
        Ok(agents) => {
            let response = SearchAgentResponse {
                count: agents.len(),
                agents,
            };
            json_response(&response)
        }
        Err(AloudError::InvalidInput(message)) => {
            let error_response = ErrorResponse { error: message };
            json_response_with_status(&error_response, 400)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}
