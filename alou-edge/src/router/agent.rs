use crate::agent::diap_identity::{DiapIdentity, DiapIdentityConfig, DiapIdentityManager};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPortConfig {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvidedDiapIdentity {
    pub did: String,
    pub cid: String,
    pub ipns: String,
    #[serde(default)]
    pub public_key: Option<String>,
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

/// Get DIAP identity from IPNS (解析功能)
pub(crate) async fn handle_get_diap_identity(
    _session_manager: &SessionManager,
    env: &Env,
    req: &mut Request,
) -> Result<Response> {
    #[derive(Deserialize)]
    struct GetIdentityRequest {
        ipns_name: String,
    }

    let body: GetIdentityRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    // Get IPFS configuration from environment
    let ipfs_api_url = match env.var("DIAP_IPFS_API_URL") {
        Ok(v) => v.to_string(),
        Err(_) => {
            let error_response = ErrorResponse {
                error: "DIAP_IPFS_API_URL not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let ipfs_gateway_url = match env.var("DIAP_IPFS_GATEWAY_URL") {
        Ok(v) => v.to_string(),
        Err(_) => {
            let error_response = ErrorResponse {
                error: "DIAP_IPFS_GATEWAY_URL not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let timeout_secs = env
        .var("DIAP_IPFS_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.to_string().parse::<u64>().ok())
        .unwrap_or(10);

    // Create DIAP identity manager for resolution only
    let config = DiapIdentityConfig::new(ipfs_api_url, ipfs_gateway_url)
        .with_timeout(timeout_secs);

    let identity_manager = match DiapIdentityManager::new(config) {
        Ok(manager) => manager,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to create identity manager: {}", e),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    // Resolve IPNS to get identity
    let identity = match identity_manager.resolve_identity_from_ipns(&body.ipns_name).await {
        Ok(identity) => identity,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to resolve identity from IPNS: {}", e),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    json_response(&json!({
        "ipns_name": body.ipns_name,
        "identity": identity,
    }))
}

/// Create a new Claude Agent SDK with automatic DIAP identity
pub(crate) async fn handle_create_claude_agent(
    session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    #[derive(Deserialize)]
    struct CreateClaudeAgentRequest {
        #[serde(default)]
        session_id: Option<String>,
        #[serde(default)]
        wallet_address: Option<String>,
        #[serde(default)]
        chain: Option<String>,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        avatar_cid: Option<String>,
        #[serde(default)]
        mcp_config_cid: Option<String>,
        #[serde(default)]
        role_description: Option<String>,
        #[serde(default)]
        mcp_ports: Option<Vec<McpPortConfig>>,
        #[serde(default)]
        diap_identity: Option<ProvidedDiapIdentity>,
    }

    let body: CreateClaudeAgentRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    // Create or use existing session
    let session_id = if let Some(sid) = body.session_id {
        // Verify session exists
        session_manager
            .get_session(&sid)
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Session not found: {}", e),
                };
                worker::Error::RustError(error_response.error.clone())
            })?;
        sid
    } else {
        // Create new session
        session_manager
            .create_session(body.wallet_address.clone(), body.chain.clone())
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Failed to create session: {}", e),
                };
                worker::Error::RustError(error_response.error.clone())
            })?
    };

    let provided_identity = body.diap_identity.clone();
    let mut stored_identity: Option<DiapIdentity> = None;
    if let Some(provided_identity) = provided_identity.clone() {
        let public_key = provided_identity
            .public_key
            .clone()
            .unwrap_or_else(|| format!("pubkey_{}", provided_identity.ipns));
        // Use new() which sets ipns_key to None for provided identities (backward compatible)
        let identity = DiapIdentity::new(
            provided_identity.did.clone(),
            provided_identity.ipns.clone(),
            provided_identity.cid.clone(),
            public_key,
            None,
        );
        if let Err(e) = session_manager
            .set_diap_identity(&session_id, identity.clone())
            .await
        {
            let error_response = ErrorResponse {
                error: format!("Failed to store DIAP identity: {}", e),
            };
            return json_response_with_status(&error_response, 500);
        }
        stored_identity = Some(identity);
    }

    let identity = if let Some(identity) = stored_identity.clone() {
        Some(identity)
    } else {
        session_manager
            .get_diap_identity(&session_id)
            .await
            .ok()
            .flatten()
    };

    let mut agent_metadata = json!({
        "agent_type": "claude_agent_sdk",
        "display_name": body.name.clone().unwrap_or_else(|| "Claude Agent SDK".to_string()),
        "session_id": session_id,
    });

    if let Some(desc) = body.role_description.clone() {
        agent_metadata["role_description"] = json!(desc);
    }
    if let Some(avatar_cid) = body.avatar_cid.clone() {
        agent_metadata["avatar_cid"] = json!(avatar_cid);
    }
    if let Some(config_cid) = body.mcp_config_cid.clone() {
        agent_metadata["mcp_config_cid"] = json!(config_cid);
    }
    if let Some(ports) = body.mcp_ports.clone() {
        if let Ok(value) = serde_json::to_value(ports) {
            agent_metadata["mcp_ports"] = value;
        }
    }
    if let Some(ref identity) = identity {
        agent_metadata["did"] = json!(identity.did);
        agent_metadata["cid"] = json!(identity.cid);
        agent_metadata["ipns"] = json!(identity.ipns);
    }
    if let Some(provided) = provided_identity {
        if let Ok(value) = serde_json::to_value(provided) {
            agent_metadata["diap_identity"] = value;
        }
    }

    if let Err(e) = session_manager
        .set_agent_metadata(&session_id, agent_metadata.clone())
        .await
    {
        console_warn!(
            "Failed to store agent metadata for session {}: {}",
            session_id,
            e
        );
    }

    // Build response with error handling
    let response_json = json!({
        "session_id": session_id,
        "agent_type": "claude_agent_sdk",
        "name": body.name.unwrap_or_else(|| "Claude Agent SDK".to_string()),
        "diap_identity": identity,
        "agent_metadata": agent_metadata,
    });

    json_response(&response_json)
}

/// Get DIAP identity for a session
pub(crate) async fn handle_get_diap_identity_by_session(
    session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    #[derive(Deserialize)]
    struct GetIdentityRequest {
        session_id: String,
    }

    let body: GetIdentityRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    // First check if session exists
    let session_result = session_manager.get_session(&body.session_id).await;
    match session_result {
        Ok(_) => {
            // Session exists, try to get identity
            match session_manager.get_diap_identity(&body.session_id).await {
                Ok(Some(identity)) => json_response(&json!({
                    "session_id": body.session_id,
                    "identity": identity,
                })),
                Ok(None) => {
                    let error_response = ErrorResponse {
                        error: "DIAP identity not found for this session".to_string(),
                    };
                    json_response_with_status(&error_response, 404)
                }
                Err(e) => {
                    console_error!("Failed to get DIAP identity for session {}: {}", body.session_id, e);
                    let error_response = ErrorResponse {
                        error: format!("Failed to get DIAP identity: {}", e),
                    };
                    json_response_with_status(&error_response, 500)
                }
            }
        }
        Err(e) => {
            // Session doesn't exist
            console_warn!("Session not found when getting DIAP identity: {} - {}", body.session_id, e);
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}

/// Register agent to DIAP network on-chain (returns encoded transaction)
pub(crate) async fn handle_register_agent_onchain(
    _session_manager: &SessionManager,
    env: &Env,
    req: &mut Request,
) -> Result<Response> {
    use crate::router::diap::common::resolve_environment;
    use crate::web3::clients::DiapAgentNetworkClient;
    use crate::agent::diap_identity::{DiapIdentity, DiapIdentityConfig, DiapIdentityManager};

    #[derive(Deserialize)]
    struct RegisterRequest {
        ipns: String,
        did: String,
        cid: String,
        public_key: String,
        network: String,
        stake_amount: String,
        #[serde(default)]
        use_aa: bool,
        #[serde(default)]
        salt: Option<u64>,
    }

    let body: RegisterRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    // Create identity object from request
    let identity = DiapIdentity::new(
        body.did.clone(),
        body.ipns.clone(),
        body.cid.clone(),
        body.public_key.clone(),
        None,
    );

    // Optional: Validate identity using DIAP SDK
    let ipfs_api_url = env.var("DIAP_IPFS_API_URL").ok().map(|v| v.to_string());
    let ipfs_gateway_url = env.var("DIAP_IPFS_GATEWAY_URL").ok().map(|v| v.to_string());
    
    if let (Some(api_url), Some(gateway_url)) = (ipfs_api_url, ipfs_gateway_url) {
        let timeout_secs = env
            .var("DIAP_IPFS_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.to_string().parse::<u64>().ok())
            .unwrap_or(10);
        
        let config = DiapIdentityConfig::new(api_url, gateway_url)
            .with_timeout(timeout_secs);
        
        if let Ok(manager) = DiapIdentityManager::new(config) {
            if let Err(e) = manager.validate_identity(&identity) {
                let error_response = ErrorResponse {
                    error: format!("Identity validation failed: {}", e),
                };
                return json_response_with_status(&error_response, 400);
            }
        }
    }

    // Resolve contract environment
    let contract_env = match resolve_environment(env, &body.network) {
        Ok((_, env)) => env,
        Err(msg) => {
            let error_response = ErrorResponse { error: msg };
            return json_response_with_status(&error_response, 400);
        }
    };

    let client = DiapAgentNetworkClient::new(&contract_env);

    // Generate encoded call for registration
    let encoded_call = if body.use_aa {
        client.register_agent_with_aa_call(
            &identity.ipns, // Use IPNS as identifier
            &identity.public_key,
            &body.stake_amount,
            body.salt.unwrap_or(0),
        )
    } else {
        client.register_agent_call(
            &identity.ipns, // Use IPNS as identifier
            &identity.public_key,
            &body.stake_amount,
        )
    };

    match encoded_call {
        Ok(encoded) => {
            // Get registration fee and min stake amount for reference
            let registration_fee = client.registration_fee().await.ok();
            let min_stake = client.min_stake_amount().await.ok();

            json_response(&json!({
                "encoded_call": encoded,
                "network": body.network,
                "registration_fee": registration_fee,
                "min_stake_amount": min_stake,
                "stake_amount": body.stake_amount,
                "use_aa": body.use_aa,
                "identity": {
                    "ipns": identity.ipns,
                    "did": identity.did,
                    "cid": identity.cid,
                    "public_key": identity.public_key,
                },
            }))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to encode registration call: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}
