use crate::agent::discovery::{AgentDiscovery, ResolvedAgent};
use crate::agent::session::SessionManager;
use crate::utils::error::AloudError;
use crate::utils::time;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
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

    let agent_result = if let Some(discovery) = discovery {
        discovery
            .resolve_agent(&body.target)
            .await
            .map_err(|err| match err {
                AloudError::InvalidInput(message) => {
                    (StatusCode::BAD_REQUEST, ErrorResponse { error: message })
                }
                other => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: other.to_string(),
                    },
                ),
            })
    } else {
        fallback_resolve_agent(&body.target).ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorResponse {
                error: "Agent discovery is not configured".to_string(),
            },
        ))
    };

    match agent_result {
        Ok(agent) => store_and_respond_agent(session_manager, body.session_id, agent).await,
        Err((status, error_response)) => json_response_with_status(&error_response, status.into()),
    }
}

pub(crate) async fn handle_search_agents(
    discovery: Option<&AgentDiscovery>,
    req: &mut Request,
) -> Result<Response> {
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

    let agents_result = if let Some(discovery) = discovery {
        discovery
            .search_agents(&body.query)
            .await
            .map_err(|err| match err {
                AloudError::InvalidInput(message) => {
                    (StatusCode::BAD_REQUEST, ErrorResponse { error: message })
                }
                other => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: other.to_string(),
                    },
                ),
            })
    } else {
        Ok(fallback_search_agents(&body.query))
    };

    match agents_result {
        Ok(agents) => {
            let response = SearchAgentResponse {
                count: agents.len(),
                agents,
            };
            json_response(&response)
        }
        Err((status, error_response)) => json_response_with_status(&error_response, status.into()),
    }
}

async fn store_and_respond_agent(
    session_manager: &SessionManager,
    session_id: Option<String>,
    agent: ResolvedAgent,
) -> Result<Response> {
    if let Some(session_id) = session_id {
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

fn fallback_resolve_agent(target: &str) -> Option<ResolvedAgent> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return None;
    }

    let fallback = build_fallback_agent(trimmed);
    if fallback.is_none() {
        console_warn!(
            "Fallback agent discovery could not parse identifier: {}",
            trimmed
        );
    }
    fallback
}

fn fallback_search_agents(query: &str) -> Vec<ResolvedAgent> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    match build_fallback_agent(trimmed) {
        Some(agent) => vec![agent],
        None => Vec::new(),
    }
}

fn build_fallback_agent(identifier: &str) -> Option<ResolvedAgent> {
    if identifier.trim().is_empty() {
        return None;
    }

    let normalized = identifier.trim();
    let (ipns, cid_hint, resolved_path_hint) = if normalized.starts_with("/ipns/")
        || normalized.starts_with("ipns://")
        || normalized.contains(".ipns")
        || normalized.starts_with("k51")
    {
        let ipns_value = normalize_ipns_identifier(normalized);
        let cid = format!("bafy{}", simple_hash_fragment(&ipns_value));
        let resolved_path = format!("/ipfs/{}", cid);
        (Some(ipns_value), Some(cid), Some(resolved_path))
    } else if normalized.starts_with("bafy") || normalized.starts_with("Qm") {
        (
            None,
            Some(normalized.to_string()),
            Some(format!("/ipfs/{}", normalized)),
        )
    } else {
        (
            None,
            Some(format!("bafy{}", simple_hash_fragment(normalized))),
            None,
        )
    };

    let cid = cid_hint?;
    let did = if normalized.starts_with("did:") {
        normalized.to_string()
    } else {
        format!("did:alou:{}", simple_hash_fragment(&cid))
    };

    let doc = json!({
        "id": did,
        "alsoKnownAs": [normalized],
        "service": [{
            "id": format!("{}#agent", did),
            "type": "AgentEndpoint",
            "serviceEndpoint": format!("https://agents.alou.local/{}", simple_slug(normalized)),
        }],
        "metadata": {
            "source": "fallback",
            "identifier": normalized,
            "generatedAt": time::now_rfc3339(),
        }
    });

    Some(ResolvedAgent {
        ipns,
        cid,
        did,
        did_document: doc,
        encrypted_peer_id: None,
        resolved_path: resolved_path_hint,
    })
}

fn simple_hash_fragment(input: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn normalize_ipns_identifier(value: &str) -> String {
    if value.starts_with("/ipns/") {
        value.to_string()
    } else if value.starts_with("ipns://") {
        format!("/ipns/{}", value.trim_start_matches("ipns://"))
    } else if value.contains(".ipns") {
        format!("/ipns/{}", value)
    } else {
        format!("/ipns/{}", value)
    }
}

fn simple_slug(value: &str) -> String {
    let mut slug = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').chars().take(48).collect()
}
