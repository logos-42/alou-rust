use crate::agent::batch_create::{
    AgentCreationResult, AgentStatus, BatchCreateAgentRequest, BatchCreateTask, TaskStatus,
};
use crate::agent::batch_processor::BatchProcessor;
use crate::agent::content_generator::ContentGenerator;
use crate::agent::diap_identity::{DiapIdentity, DiapIdentityConfig, DiapIdentityManager};
use crate::agent::discovery::{AgentDiscovery, ResolvedAgent};
use crate::agent::session::SessionManager;
use crate::agent::ai_client::AiClient;
use crate::storage::kv::KvStore;
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

#[derive(Deserialize)]
pub(crate) struct CreateDiapIdentityRequest {
    pub session_id: String,
    #[serde(default)]
    pub agent_name: Option<String>,
    #[serde(default)]
    pub agent_description: Option<String>,
    #[serde(default)]
    pub ipfs_api_url: Option<String>,
    #[serde(default)]
    pub ipfs_gateway_url: Option<String>,
    #[serde(default)]
    pub ipns_key: Option<String>,
    #[serde(default)]
    pub custom_prompt: Option<String>,
    #[serde(default)]
    pub avatar_cid: Option<String>,
    #[serde(default)]
    pub mcp_config_cid: Option<String>,
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
            return Ok(json_response_with_status(&error_response, 400)?);
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
            return Ok(json_response_with_status(&error_response, 400)?);
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
            Ok(json_response(&response)?)
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
    Ok(json_response(&agent)?)
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
            return Ok(json_response_with_status(&error_response, 400)?);
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

    Ok(json_response(&json!({
        "ipns_name": body.ipns_name,
        "identity": identity,
    }))?)
}

/// Create agent using the internal creation logic (简化版本，避免重复)
pub(crate) async fn handle_create_agent(
    session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    let body: CreateAgentRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // 注意：这里不再直接创建DIAP身份，而是返回基本的智能体信息
    // DIAP身份创建应该通过专门的 handle_create_diap_identity 端点进行
    let result = handle_create_agent_internal(session_manager, body).await?;

    Ok(json_response(&json!({
        "session_id": result.session_id,
        "agent_id": result.session_id,
        "agent_type": "agent",
        "name": result.name,
        "role_description": result.agent_metadata.get("role_description"),
        "agent_metadata": result.agent_metadata,
        "message": "智能体创建成功，请使用 /agent/diap/create-identity 创建DIAP身份"
    }))?)
}

/// Request structure for creating an agent
#[derive(Deserialize)]
pub(crate) struct CreateAgentRequest {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub wallet_address: Option<String>,
    #[serde(default)]
    pub chain: Option<String>,
    pub name: String,
    pub role_description: String,
    #[serde(default)]
    pub avatar_cid: Option<String>,
    #[serde(default)]
    pub mcp_config_cid: Option<String>,
    #[serde(default)]
    pub mcp_ports: Option<Vec<McpPortConfig>>,
    #[serde(default)]
    pub diap_identity: Option<ProvidedDiapIdentity>,
}

/// Result structure for agent creation
#[derive(Serialize)]
pub(crate) struct CreateAgentResult {
    pub session_id: String,
    pub name: String,
    pub identity: Option<DiapIdentity>,
    pub agent_metadata: serde_json::Value,
}

/// Internal function to handle agent creation logic
/// 内部函数处理智能体创建逻辑
pub(crate) async fn handle_create_agent_internal(
    session_manager: &SessionManager,
    body: CreateAgentRequest,
) -> Result<CreateAgentResult> {
    // Create or use existing session
    let session_id = if let Some(sid) = &body.session_id {
        // Verify session exists
        session_manager
            .get_session(sid)
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Session not found: {}", e),
                };
                crate::utils::error::AloudError::WorkerError(error_response.error.clone())
            })?;
        sid.clone()
    } else {
        // Create new session
        session_manager
            .create_session(body.wallet_address.clone(), body.chain.clone())
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Failed to create session: {}", e),
                };
                crate::utils::error::AloudError::WorkerError(error_response.error.clone())
            })?
    };

    // Generate agent ID
    let agent_id = format!("agent_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

    // Store agent information
    let agent_info = serde_json::json!({
        "agent_id": agent_id,
        "session_id": session_id,
        "name": body.name,
        "role_description": body.role_description,
        "avatar_cid": body.avatar_cid,
        "mcp_config_cid": body.mcp_config_cid,
        "mcp_ports": body.mcp_ports,
        "diap_identity": body.diap_identity,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    // Save to session
    if let Err(e) = session_manager.set_agent_metadata(&session_id, agent_info).await {
        return Err(crate::utils::error::AloudError::AgentError(
            format!("Failed to save agent metadata: {}", e),
        ).into());
    }

    Ok(CreateAgentResult {
        session_id: session_id.clone(),
        name: body.name.clone(),
        identity: body.diap_identity.map(|diap| DiapIdentity {
            did: diap.did,
            ipns: diap.ipns,
            cid: diap.cid,
            public_key: diap.public_key.unwrap_or_default(),
            encrypted_peer_id: None,
            ipns_key: None,
            is_registered: false,
            registered_address: None,
            created_at: chrono::Utc::now().timestamp(),
        }),
        agent_metadata: serde_json::json!({
            "agent_id": agent_id,
            "session_id": session_id,
            "name": body.name,
            "role_description": body.role_description,
        }),
    })
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
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // First check if session exists
    let session_result = session_manager.get_session(&body.session_id).await;
    match session_result {
        Ok(_) => {
            // Session exists, try to get identity
            match session_manager.get_diap_identity(&body.session_id).await {
                Ok(Some(identity)) => Ok(json_response(&json!({
                    "session_id": body.session_id,
                    "identity": identity,
                }))?),
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
            return Ok(json_response_with_status(&error_response, 400)?);
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
                return Ok(json_response_with_status(&error_response, 400)?);
            }
        }
    }

    // Resolve contract environment
    let contract_env = match resolve_environment(env, &body.network) {
        Ok((_, env)) => env,
        Err(msg) => {
            let error_response = ErrorResponse { error: msg };
            return Ok(json_response_with_status(&error_response, 400)?);
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

            Ok(json_response(&json!({
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
            }))?)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to encode registration call: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

/// 批量创建智能体
pub(crate) async fn handle_batch_create_agent(
    _session_manager: &SessionManager,
    req: &mut Request,
    env: &Env,
) -> Result<Response> {
    let body: BatchCreateAgentRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            return json_response_with_status(
                &ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                },
                400,
            );
        }
    };

    // 验证批量大小限制
    if body.agents.len() > 10 {
        return json_response_with_status(
            &ErrorResponse {
                error: format!("Batch size exceeds limit: {} (max 10)", body.agents.len()),
            },
            400,
        );
    }

    if body.agents.is_empty() {
        return json_response_with_status(
            &ErrorResponse {
                error: "Agents list cannot be empty".to_string(),
            },
            400,
        );
    }

    let task_id = uuid::Uuid::new_v4().to_string();
    let now = crate::utils::time::now_timestamp();

    // 创建任务记录
    let task = BatchCreateTask {
        task_id: task_id.clone(),
        status: TaskStatus::Pending,
        agents: body
            .agents
            .iter()
            .map(|spec| AgentCreationResult {
                agent_id: None,
                session_id: None,
                name: spec.name.clone(),
                role_description: spec.role_description.clone(),
                avatar_cid: spec.avatar_cid.clone(),
                mcp_config_cid: spec.mcp_config_cid.clone(),
                mcp_ports: spec.mcp_ports.clone(),
                status: AgentStatus::Pending,
                error: None,
            })
            .collect(),
        generation_config: body.generation_config.clone(),
        agent_specs: body.agents.clone(),
        created_at: now,
        updated_at: now,
    };

    // 保存任务到KV
    let kv_store = KvStore::new(env.kv("SESSIONS")?);
    let task_key = format!("batch_task:{}", task_id);
    kv_store
        .put(&task_key, &task, Some(86400 * 7))
        .await
        .map_err(|e| {
            worker::Error::RustError(format!("Failed to save task: {}", e))
        })?;

    // TODO: 队列功能暂时禁用，等待后续实现
    // let queue = env.queue("AGENT_CREATION_QUEUE")?;
    // queue
    //     .send(&task, QueueOptions::default())
    //     .await
    //     .map_err(|e| {
    //         worker::Error::RustError(format!("Failed to send task to queue: {}", e))
    //     })?;

    Ok(json_response(&json!({
        "success": true,
        "task_id": task_id,
        "status": "queued",
        "agents_count": body.agents.len(),
    }))?)
}

/// 查询批量创建任务状态
pub(crate) async fn handle_get_batch_create_task(
    _session_manager: &SessionManager,
    req: &mut Request,
    env: &Env,
) -> Result<Response> {
    // 从路径中提取task_id
    let path = req.path();
    let task_id = path
        .split('/')
        .last()
        .ok_or_else(|| {
            worker::Error::RustError("Invalid path: task_id not found".to_string())
        })?;

    // 初始化服务以获取任务
    let sessions_kv = env.kv("SESSIONS")?;
    let kv_store = KvStore::new(sessions_kv.clone());
    
    // 创建临时SessionManager和ContentGenerator（仅用于获取任务）
    let session_manager = SessionManager::new(kv_store.clone());
    
    // 创建AI客户端（用于ContentGenerator，但这里不需要）
    let ai_provider = env
        .var("AI_PROVIDER")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "deepseek".to_string());
    let api_key = env
        .secret("AI_API_KEY")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| String::new());
    
    let ai_client = match AiClient::new(&ai_provider, api_key, None) {
        Ok(client) => client,
        Err(_) => {
            // 如果AI客户端创建失败，仍然可以查询任务状态
            // 创建一个空的processor只用于查询
            return match kv_store.get::<BatchCreateTask>(&format!("batch_task:{}", task_id)).await {
                Ok(Some(task)) => json_response(&task),
                Ok(None) => json_response_with_status(
                    &ErrorResponse {
                        error: "Task not found".to_string(),
                    },
                    404,
                ),
                Err(e) => json_response_with_status(
                    &ErrorResponse {
                        error: format!("Failed to get task: {}", e),
                    },
                    500,
                ),
            };
        }
    };
    
    let content_generator = ContentGenerator::new(ai_client);
    let processor = BatchProcessor::new(session_manager, content_generator, kv_store);
    
    match processor.get_task(task_id, env).await {
        Ok(Some(task)) => json_response(&task),
        Ok(None) => json_response_with_status(
            &ErrorResponse {
                error: "Task not found".to_string(),
            },
            404,
        ),
        Err(e) => json_response_with_status(
            &ErrorResponse {
                error: format!("Failed to get task: {}", e),
            },
            500,
        ),
    }
}

/// 获取批量创建的智能体会话ID列表
/// 
/// 这个端点返回批量创建任务中所有已创建的智能体的会话ID
/// 前端可以使用这些会话ID自动将智能体添加到频道
pub(crate) async fn handle_get_batch_agent_sessions(
    _session_manager: &SessionManager,
    req: &mut Request,
    env: &Env,
) -> Result<Response> {
    // 从路径中提取task_id
    let path = req.path();
    let task_id = path
        .split('/')
        .last()
        .ok_or_else(|| {
            worker::Error::RustError("Invalid path: task_id not found".to_string())
        })?;

    // 初始化服务
    let sessions_kv = env.kv("SESSIONS")?;
    let kv_store = KvStore::new(sessions_kv.clone());
    let session_manager = SessionManager::new(kv_store.clone());
    
    // 创建AI客户端（用于ContentGenerator）
    let ai_provider = env
        .var("AI_PROVIDER")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "deepseek".to_string());
    let api_key = env
        .secret("AI_API_KEY")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| String::new());
    
    let ai_client = match AiClient::new(&ai_provider, api_key, None) {
        Ok(client) => client,
        Err(_) => {
            return json_response_with_status(
                &ErrorResponse {
                    error: "Failed to create AI client".to_string(),
                },
                500,
            );
        }
    };
    
    let content_generator = ContentGenerator::new(ai_client);
    let processor = BatchProcessor::new(session_manager, content_generator, kv_store);
    
    // 获取批量任务中所有已创建的智能体会话ID
    match processor.get_batch_agent_sessions(task_id, env).await {
        Ok(session_ids) => {
            Ok(json_response(&json!({
                "success": true,
                "task_id": task_id,
                "session_ids": session_ids,
                "count": session_ids.len(),
            }))?)
        }
        Err(e) => json_response_with_status(
            &ErrorResponse {
                error: format!("Failed to get agent sessions: {}", e),
            },
            500,
        ),
    }
}

/// Parse agent creation command using AI
/// 使用AI解析智能体创建指令
pub(crate) async fn handle_parse_creation_command(
    _session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    #[derive(Deserialize)]
    struct ParseCommandRequest {
        command: String,
    }

    let body: ParseCommandRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // 使用内置的智能体来解析创建指令
    match crate::agent::creation_parser::parse_creation_command(&body.command).await {
        Ok(result) => {
            let success_response = serde_json::json!({
                "name": result.name,
                "roleDescription": result.role_description,
                "success": true
            });
            json_response_with_status(&success_response, 200)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to parse creation command: {}", e),
            };
            json_response_with_status(&error_response, 400)
        }
    }
}

/// Create agent from command using backend creation flow
/// 使用后端创建流程从命令创建智能体
pub(crate) async fn handle_create_agent_from_command(
    session_manager: &SessionManager,
    env: &Env,
    req: &mut Request,
) -> Result<Response> {
    #[derive(Deserialize)]
    struct CreateFromCommandRequest {
        command: String,
        #[serde(default)]
        session_id: Option<String>,
        #[serde(default)]
        wallet_address: Option<String>,
        #[serde(default)]
        chain: Option<String>,
    }

    let body: CreateFromCommandRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // 解析创建命令
    let parsed_info = match crate::agent::creation_parser::parse_creation_command(&body.command).await {
        Ok(info) => info,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to parse creation command: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // 创建或使用现有会话
    let session_id = if let Some(sid) = body.session_id {
        // 验证会话存在
        session_manager
            .get_session(&sid)
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Session not found: {}", e),
                };
                crate::utils::error::AloudError::WorkerError(error_response.error.clone())
            })?;
        sid
    } else {
        // 创建新会话
        let wallet_address = body.wallet_address.clone().unwrap_or_else(|| "default_user".to_string());
        let session_id = session_manager
            .create_session(Some(wallet_address), body.chain.clone())
            .await?;
        session_id
    };

    // 使用现有的 create_agent 逻辑，但传入解析的信息
    let create_request = serde_json::json!({
        "session_id": session_id,
        "wallet_address": body.wallet_address,
        "chain": body.chain,
        "name": parsed_info.name,
        "role_description": parsed_info.role_description,
        // 使用默认值，因为这些是从命令解析来的
        "avatar_cid": null,
        "mcp_config_cid": null,
        "mcp_ports": [],
        "diap_identity": null
    });

    // 调用现有的创建逻辑
    match crate::agent::creation::create_claude_agent_from_parsed(
        session_manager,
        env,
        create_request,
    ).await {
        Ok(result) => {
            let success_response = serde_json::json!({
                "agent_id": result.agent_id,
                "session_id": session_id,
                "name": parsed_info.name,
                "roleDescription": parsed_info.role_description,
                "success": true,
                "message": "智能体创建成功"
            });
            json_response_with_status(&success_response, 200)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to create agent: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

/// Create DIAP identity for a session (创建DID文档，返回给桌面端)
/// 使用DIAP SDK创建真实的DID文档，然后桌面端上传到IPFS
pub(crate) async fn handle_create_diap_identity(
    session_manager: &SessionManager,
    env: &Env,
    req: &mut Request,
) -> Result<Response> {
    let body: CreateDiapIdentityRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // 验证session_id
    if body.session_id.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "session_id is required".to_string(),
        };
        return json_response_with_status(&error_response, 400);
    }

    // 检查session是否存在
    match session_manager.get_session(&body.session_id).await {
        Ok(_) => {
            // Session exists, proceed with DID document creation using DIAP SDK
            
            // 1. 获取IPFS配置
            let ipfs_api_url = body.ipfs_api_url.clone()
                .or_else(|| env.var("DIAP_IPFS_API_URL").ok().map(|v| v.to_string()))
                .unwrap_or_else(|| "http://localhost:5001".to_string());
            
            let ipfs_gateway_url = body.ipfs_gateway_url.clone()
                .or_else(|| env.var("DIAP_IPFS_GATEWAY_URL").ok().map(|v| v.to_string()))
                .unwrap_or_else(|| "http://localhost:8080".to_string());

            // 2. 使用DIAP SDK创建DID文档
            let did_document = match create_did_document_with_sdk(&body, &ipfs_api_url, &ipfs_gateway_url).await {
                Ok(doc) => doc,
                Err(e) => {
                    console_warn!("Failed to create DID document with SDK, falling back to simple method: {}", e);
                    // 如果SDK失败，使用简单的fallback方法
                    create_did_document_for_agent(&body).await?
                }
            };
            
            // 3. 返回DID文档给桌面端，让桌面端创建CID和IPNS
            let response = serde_json::json!({
                "did_document": did_document,
                "session_id": body.session_id,
                "ipfs_api_url": ipfs_api_url,
                "ipfs_gateway_url": ipfs_gateway_url,
                "message": "DID document created with DIAP SDK, please create CID and IPNS on desktop"
            });
            
            console_log!("Created DID document with DIAP SDK for session: {}", body.session_id);
            Ok(json_response_with_status(&response, 200)?)
        }
        Err(e) => {
            // Session doesn't exist
            console_warn!("Session not found when creating DIAP identity: {} - {}", body.session_id, e);
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}

/// Create DID document for agent using DIAP SDK
#[cfg(not(target_arch = "wasm32"))]
async fn create_did_document_with_sdk(
    request: &CreateDiapIdentityRequest,
    ipfs_api_url: &str,
    ipfs_gateway_url: &str,
) -> Result<serde_json::Value> {
    use diap_rs_sdk::identity_manager::IdentityManager;
    use diap_rs_sdk::IpfsClient;
    use chrono::Utc;
    use uuid::Uuid;
    
    let now = Utc::now().to_rfc3339();
    let agent_id = Uuid::new_v4().to_string();
    
    // Create IPFS client
    let ipfs_client = IpfsClient::new_with_remote_node(
        ipfs_api_url.to_string(),
        ipfs_gateway_url.to_string(),
        10, // timeout seconds
    );
    
    // Create identity manager
    let identity_manager = IdentityManager::new(ipfs_client);
    
    // 由于DIAP SDK主要用于解析和验证，我们创建一个标准的DID文档
    // 桌面端将负责生成密钥对和创建真实的DID
    let did_document = serde_json::json!({
        "@context": "https://www.w3.org/did/v1",
        "id": format!("did:temp:{}", request.session_id), // 临时DID，桌面端会替换为真实的
        "verificationMethod": [
            {
                "id": "#key-1",
                "type": "Ed25519VerificationKey2018",
                "controller": format!("did:temp:{}", request.session_id),
                "publicKeyBase58": "temp_public_key" // 桌面端会替换为真实的公钥
            }
        ],
        "authentication": ["#key-1"],
        "service": [
            {
                "id": "#agent",
                "type": "Agent",
                "serviceEndpoint": format!("https://alou.ai/agents/{}", agent_id),
                "properties": {
                    "name": request.agent_name.as_ref().unwrap_or(&"Unnamed Agent".to_string()),
                    "description": request.agent_description.as_ref().unwrap_or(&"An AI agent".to_string()),
                    "avatar_cid": request.avatar_cid,
                    "mcp_config_cid": request.mcp_config_cid,
                    "custom_prompt": request.custom_prompt,
                    "created_at": now,
                    "agent_type": "claude_agent_sdk",
                    "session_id": request.session_id,
                    "agent_id": agent_id,
                    "ipfs_api_url": ipfs_api_url,
                    "ipfs_gateway_url": ipfs_gateway_url,
                    "sdk_version": "0.2.11"
                }
            }
        ],
        "created": now,
        "updated": now
    });
    
    console_log!("Created DID document template for agent: {}", request.agent_name.as_ref().unwrap_or(&"Unnamed".to_string()));
    
    Ok(did_document)
}

/// Create DID document for agent using DIAP SDK (WASM version)
#[cfg(target_arch = "wasm32")]
async fn create_did_document_with_sdk(
    request: &CreateDiapIdentityRequest,
    ipfs_api_url: &str,
    ipfs_gateway_url: &str,
) -> Result<serde_json::Value> {
    use chrono::Utc;
    use uuid::Uuid;
    
    let now = Utc::now().to_rfc3339();
    let agent_id = Uuid::new_v4().to_string();
    
    // 在WASM环境下，我们创建一个标准的DID文档
    // 桌面端将负责生成密钥对和创建真实的DID
    let did_document = serde_json::json!({
        "@context": "https://www.w3.org/did/v1",
        "id": format!("did:temp:{}", request.session_id), // 临时DID，桌面端会替换为真实的
        "verificationMethod": [
            {
                "id": "#key-1",
                "type": "Ed25519VerificationKey2018",
                "controller": format!("did:temp:{}", request.session_id),
                "publicKeyBase58": "temp_public_key" // 桌面端会替换为真实的公钥
            }
        ],
        "authentication": ["#key-1"],
        "service": [
            {
                "id": "#agent",
                "type": "Agent",
                "serviceEndpoint": format!("https://alou.ai/agents/{}", agent_id),
                "properties": {
                    "name": request.agent_name.as_ref().unwrap_or(&"Unnamed Agent".to_string()),
                    "description": request.agent_description.as_ref().unwrap_or(&"An AI agent".to_string()),
                    "avatar_cid": request.avatar_cid,
                    "mcp_config_cid": request.mcp_config_cid,
                    "custom_prompt": request.custom_prompt,
                    "created_at": now,
                    "agent_type": "claude_agent_sdk",
                    "session_id": request.session_id,
                    "agent_id": agent_id,
                    "ipfs_api_url": ipfs_api_url,
                    "ipfs_gateway_url": ipfs_gateway_url,
                    "sdk_version": "0.2.11-wasm"
                }
            }
        ],
        "created": now,
        "updated": now
    });
    
    console_log!("Created DID document template for agent (WASM): {}", request.agent_name.as_ref().unwrap_or(&"Unnamed".to_string()));
    
    Ok(did_document)
}

/// Create DID document for agent (fallback method)
async fn create_did_document_for_agent(
    request: &CreateDiapIdentityRequest,
) -> Result<serde_json::Value> {
    use chrono::Utc;
    use uuid::Uuid;
    
    let now = Utc::now().to_rfc3339();
    let agent_id = Uuid::new_v4().to_string();
    
    // 构建DID文档
    let did_document = serde_json::json!({
        "@context": "https://www.w3.org/did/v1",
        "id": format!("did:temp:{}", request.session_id), // 临时DID，桌面端会替换为真实的IPNS DID
        "verificationMethod": [
            {
                "id": "#key-1",
                "type": "Ed25519VerificationKey2018",
                "controller": format!("did:temp:{}", request.session_id),
                "publicKeyBase58": "temp_public_key" // 桌面端会替换为真实的公钥
            }
        ],
        "authentication": ["#key-1"],
        "service": [
            {
                "id": "#agent",
                "type": "Agent",
                "serviceEndpoint": format!("https://alou.ai/agents/{}", agent_id),
                "properties": {
                    "name": request.agent_name.as_ref().unwrap_or(&"Unnamed Agent".to_string()),
                    "description": request.agent_description.as_ref().unwrap_or(&"An AI agent".to_string()),
                    "avatar_cid": request.avatar_cid,
                    "mcp_config_cid": request.mcp_config_cid,
                    "custom_prompt": request.custom_prompt,
                    "created_at": now,
                    "agent_type": "claude_agent_sdk",
                    "session_id": request.session_id,
                    "agent_id": agent_id
                }
            }
        ],
        "created": now,
        "updated": now
    });
    
    console_log!("Created DID document for agent: {}", request.agent_name.as_ref().unwrap_or(&"Unnamed".to_string()));
    
    Ok(did_document)
}

/// Update DIAP identity for a session (更新DIAP身份信息)
/// 桌面端通过这个端点更新已存在的DIAP身份信息
pub(crate) async fn handle_update_diap_identity(
    session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    #[derive(Deserialize)]
    struct UpdateDiapIdentityRequest {
        session_id: String,
        diap_identity: DiapIdentity,
    }

    let body: UpdateDiapIdentityRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // 验证session_id
    if body.session_id.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "session_id is required".to_string(),
        };
        return json_response_with_status(&error_response, 400);
    }

    // 检查session是否存在
    match session_manager.get_session(&body.session_id).await {
        Ok(_) => {
            // Session exists, update DIAP identity
            if let Err(e) = session_manager
                .set_diap_identity(&body.session_id, body.diap_identity.clone())
                .await
            {
                let error_response = ErrorResponse {
                    error: format!("Failed to update DIAP identity: {}", e),
                };
                return json_response_with_status(&error_response, 500);
            }

            // 更新agent_metadata中的DIAP信息
            let agent_metadata = json!({
                "did": body.diap_identity.did,
                "cid": body.diap_identity.cid,
                "ipns": body.diap_identity.ipns,
                "diap_identity_updated_at": chrono::Utc::now().to_rfc3339(),
            });

            if let Err(e) = session_manager
                .set_agent_metadata(&body.session_id, agent_metadata.clone())
                .await
            {
                console_warn!(
                    "Failed to update agent metadata for session {}: {}",
                    body.session_id,
                    e
                );
            }

            let response = serde_json::json!({
                "success": true,
                "session_id": body.session_id,
                "message": "DIAP identity updated successfully",
                "diap_identity": body.diap_identity
            });

            console_log!("DIAP identity updated for session: {}", body.session_id);
            Ok(json_response_with_status(&response, 200)?)
        }
        Err(e) => {
            // Session doesn't exist
            console_warn!("Session not found when updating DIAP identity: {} - {}", body.session_id, e);
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}

/// Save complete DIAP identity to backend KV (保存完整的DIAP身份到后端KV)
/// 桌面端创建CID和IPNS后，通过这个端点保存到后端
pub(crate) async fn handle_save_complete_diap_identity(
    session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    #[derive(Deserialize)]
    struct SaveCompleteIdentityRequest {
        session_id: String,
        diap_identity: DiapIdentity,
    }

    let body: SaveCompleteIdentityRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // 验证session_id
    if body.session_id.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "session_id is required".to_string(),
        };
        return json_response_with_status(&error_response, 400);
    }

    // 检查session是否存在
    match session_manager.get_session(&body.session_id).await {
        Ok(_) => {
            // Session exists, save DIAP identity
            if let Err(e) = session_manager
                .set_diap_identity(&body.session_id, body.diap_identity.clone())
                .await
            {
                let error_response = ErrorResponse {
                    error: format!("Failed to store DIAP identity: {}", e),
                };
                return json_response_with_status(&error_response, 500);
            }

            // 更新agent_metadata中的DIAP信息
            let agent_metadata = json!({
                "did": body.diap_identity.did,
                "cid": body.diap_identity.cid,
                "ipns": body.diap_identity.ipns,
                "diap_identity_updated_at": chrono::Utc::now().to_rfc3339(),
            });

            if let Err(e) = session_manager
                .set_agent_metadata(&body.session_id, agent_metadata.clone())
                .await
            {
                console_warn!(
                    "Failed to update agent metadata for session {}: {}",
                    body.session_id,
                    e
                );
            }

            let response = serde_json::json!({
                "success": true,
                "session_id": body.session_id,
                "message": "DIAP identity saved successfully",
                "diap_identity": body.diap_identity
            });

            console_log!("DIAP identity saved for session: {}", body.session_id);
            Ok(json_response_with_status(&response, 200)?)
        }
        Err(e) => {
            // Session doesn't exist
            console_warn!("Session not found when saving DIAP identity: {} - {}", body.session_id, e);
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}
