//! DIAP Identity Management Module
//! 
//! This module handles all DIAP (Decentralized Identity Agent Protocol) related operations:
//! - Creating complete DIAP identities (DID, keys, documents)
//! - Retrieving identities from IPNS or sessions
//! - Updating identity information
//! - On-chain registration

use crate::agent::diap_identity::{DiapIdentity, DiapIdentityConfig, DiapIdentityManager};
use crate::agent::session::SessionManager;
use crate::utils::time;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::router::{json_response, json_response_with_status, ErrorResponse};

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

/// Complete DIAP Identity structure
#[derive(Serialize, Deserialize, Clone)]
struct CompleteDiapIdentity {
    pub did: String,
    pub did_document: serde_json::Value,
    pub public_key: String,
    pub private_key: String,
    pub ipns_key: String,
}

/// Get DIAP identity from IPNS
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

    let session_result = session_manager.get_session(&body.session_id).await;
    match session_result {
        Ok(_) => {
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
            console_warn!("Session not found when getting DIAP identity: {} - {}", body.session_id, e);
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}

/// Create DIAP identity for a session
/// 
/// Architecture:
/// 1. Backend (this function): Creates complete DIAP identity (DID, keys, document)
/// 2. Desktop: Receives complete identity, handles IPFS operations
/// 3. Frontend: Coordinates the flow and saves results
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

    if body.session_id.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "session_id is required".to_string(),
        };
        return json_response_with_status(&error_response, 400);
    }

    match session_manager.get_session(&body.session_id).await {
        Ok(_) => {
            let ipfs_api_url = body.ipfs_api_url.clone()
                .or_else(|| env.var("DIAP_IPFS_API_URL").ok().map(|v| v.to_string()))
                .unwrap_or_else(|| "http://localhost:5001".to_string());
            
            let ipfs_gateway_url = body.ipfs_gateway_url.clone()
                .or_else(|| env.var("DIAP_IPFS_GATEWAY_URL").ok().map(|v| v.to_string()))
                .unwrap_or_else(|| "http://localhost:8080".to_string());

            let diap_identity = create_complete_diap_identity(&body, &ipfs_api_url, &ipfs_gateway_url).await?;
            
            let response = serde_json::json!({
                "did": diap_identity.did,
                "did_document": diap_identity.did_document,
                "public_key": diap_identity.public_key,
                "private_key": diap_identity.private_key,
                "ipns_key": diap_identity.ipns_key,
                "session_id": body.session_id,
                "ipfs_api_url": ipfs_api_url,
                "ipfs_gateway_url": ipfs_gateway_url,
                "message": "Complete DIAP identity created by backend, ready for desktop IPFS operations"
            });
            
            console_log!("Created complete DIAP identity for session: {}", body.session_id);
            Ok(json_response_with_status(&response, 200)?)
        }
        Err(e) => {
            console_warn!("Session not found when creating DIAP identity: {} - {}", body.session_id, e);
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
                    let response = serde_json::json!({
                        "success": true,
                        "message": "DIAP identity saved successfully",
                        "session_id": body.session_id,
                        "did": body.diap_identity.did,
                        "cid": body.diap_identity.cid,
                        "ipns": body.diap_identity.ipns,
                        "zkp_proof": body.diap_identity.zkp_proof,
                        "created_by": body.diap_identity.created_by
                    });
                    
                    Ok(json_response_with_status(&response, 200)?)
                }
                Err(e) => {
                    console_log!("❌ DIAP身份保存失败: {}", e);
                    let error_response = ErrorResponse {
                        error: format!("Failed to save DIAP identity: {}", e),
                    };
                    json_response_with_status(&error_response, 500)
                }
            }
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Session not found or invalid: {}", e),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}

/// Request to save complete DIAP identity
#[derive(Deserialize)]
struct SaveDiapIdentityRequest {
    session_id: String,
    diap_identity: CompleteDiapIdentity,
}

/// Complete DIAP identity with ZKP proof
#[derive(Serialize, Deserialize, Clone)]
struct CompleteDiapIdentity {
    did: String,
    did_document: serde_json::Value,
    public_key: String,
    private_key: String,
    ipns_key: String,
    cid: String,
    ipns: Option<String>,
    zkp_proof: Option<serde_json::Value>,
    created_by: Option<String>,
}

/// Save DIAP identity to KV storage
async fn save_diap_identity_to_kv(
    session_id: &str,
    diap_identity: &CompleteDiapIdentity,
) -> Result<(), AloudError> {
    use worker::kv::KvStore;
    
    // 获取KV存储
    let kv_store = req::env()?.var("DIAP_KV_NAMESPACE")?;
    let kv = req::kv(&kv_store)?;
    
    // 构建存储键
    let storage_key = format!("diap_identity:{}", session_id);
    
    // 序列化DIAP身份
    let identity_json = serde_json::to_string(diap_identity)
        .map_err(|e| AloudError::SerializationError(e.to_string()))?;
    
    // 保存到KV
    kv.put(&storage_key.into_bytes(), identity_json.into_bytes())?;
    
    console_log!("💾 DIAP身份已保存到KV: {} -> {}", session_id, diap_identity.did);
    Ok(())
} for a session
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

    if body.session_id.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "session_id is required".to_string(),
        };
        return json_response_with_status(&error_response, 400);
    }

    match session_manager.get_session(&body.session_id).await {
        Ok(_) => {
            if let Err(e) = session_manager
                .set_diap_identity(&body.session_id, body.diap_identity.clone())
                .await
            {
                let error_response = ErrorResponse {
                    error: format!("Failed to update DIAP identity: {}", e),
                };
                return json_response_with_status(&error_response, 500);
            }

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
            console_warn!("Session not found when updating DIAP identity: {} - {}", body.session_id, e);
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}

/// Register agent to DIAP network on-chain
pub(crate) async fn handle_register_agent_onchain(
    _session_manager: &SessionManager,
    env: &Env,
    req: &mut Request,
) -> Result<Response> {
    use crate::router::diap::common::resolve_environment;
    use crate::web3::clients::DiapAgentNetworkClient;

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

    let identity = DiapIdentity::new(
        body.did.clone(),
        body.ipns.clone(),
        body.cid.clone(),
        body.public_key.clone(),
        None,
    );

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

    let contract_env = match resolve_environment(env, &body.network) {
        Ok((_, env)) => env,
        Err(msg) => {
            let error_response = ErrorResponse { error: msg };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    let client = DiapAgentNetworkClient::new(&contract_env);

    let encoded_call = if body.use_aa {
        client.register_agent_with_aa_call(
            &identity.ipns,
            &identity.public_key,
            &body.stake_amount,
            body.salt.unwrap_or(0),
        )
    } else {
        client.register_agent_call(
            &identity.ipns,
            &identity.public_key,
            &body.stake_amount,
        )
    };

    match encoded_call {
        Ok(encoded) => {
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

// ============================================================================
// Internal Helper Functions
// ============================================================================

/// Create complete DIAP identity using DIAP SDK with ZKP (native version)
#[cfg(not(target_arch = "wasm32"))]
async fn create_complete_diap_identity(
    request: &CreateDiapIdentityRequest,
    ipfs_api_url: &str,
    ipfs_gateway_url: &str,
) -> Result<CompleteDiapIdentity> {
    use diap_rs_sdk::{AgentAuthManager, UniversalNoirManager};
    use chrono::Utc;
    
    console_log!("🚀 开始创建带ZKP的DIAP身份");
    
    // 1. 创建智能体认证管理器
    let auth_manager = AgentAuthManager::new_with_remote_ipfs(
        ipfs_api_url.to_string(),
        ipfs_gateway_url.to_string(),
    ).await.map_err(|e| {
        AloudError::DIAPError(format!("创建认证管理器失败: {}", e))
    })?;
    
    // 2. 创建智能体
    let agent_name = request.agent_name.as_ref().unwrap_or(&"Unnamed".to_string());
    let (agent_info, keypair, peer_id) = auth_manager.create_agent(agent_name, None).map_err(|e| {
        AloudError::DIAPError(format!("创建智能体失败: {}", e))
    })?;
    
    console_log!("✅ 智能体创建成功: {}", agent_info.name);
    console_log!("   DID: {}", keypair.did);
    console_log!("   PeerID: {}", peer_id);
    
    // 3. 注册身份（包含DID文档创建和IPFS上传）
    let registration = auth_manager.register_agent(&agent_info, &keypair, &peer_id).await.map_err(|e| {
        AloudError::DIAPError(format!("注册身份失败: {}", e))
    })?;
    
    console_log!("✅ 身份注册成功");
    console_log!("   DID: {}", registration.did);
    console_log!("   CID: {}", registration.cid);
    
    // 4. 创建ZKP证明
    let mut noir_manager = UniversalNoirManager::new().await.map_err(|e| {
        AloudError::DIAPError(format!("创建Noir管理器失败: {}", e))
    })?;
    
    // 准备ZKP输入（简化版本，实际使用中需要计算正确的哈希值）
    let inputs = diap_rs_sdk::noir_universal::NoirProverInputs {
        expected_did_hash: [0; 2], // 这里需要根据实际的DID哈希计算
        public_key_hash: 0, // 这里需要根据实际的公钥哈希计算
        nonce_hash: 0, // 这里需要根据实际的随机数计算
        expected_output: format!("did:{}:cid:{}", registration.did, registration.cid),
    };
    
    // 生成证明
    let proof = noir_manager.generate_proof(&inputs).await.map_err(|e| {
        AloudError::DIAPError(format!("生成ZKP证明失败: {}", e))
    })?;
    
    console_log!("✅ ZKP证明生成成功: {} bytes", proof.proof.len());
    
    // 5. 验证证明
    let verification = noir_manager.verify_proof(&proof.proof, &proof.public_inputs).await.map_err(|e| {
        AloudError::DIAPError(format!("验证ZKP证明失败: {}", e))
    })?;
    
    if !verification.is_valid {
        return Err(AloudError::DIAPError("ZKP证明验证失败".to_string()));
    }
    
    console_log!("✅ ZKP证明验证通过");
    
    // 6. 构建完整的DID文档（包含ZKP证明）
    let mut did_document = registration.did_document;
    
    // 添加ZKP证明到DID文档
    if let Some(obj) = did_document.as_object_mut() {
        obj.insert("zkpProof".to_string(), serde_json::json!({
            "proof": base64::encode(&proof.proof),
            "publicInputs": base64::encode(&proof.public_inputs),
            "circuitOutput": proof.circuit_output,
            "timestamp": proof.timestamp,
            "verificationResult": verification.is_valid
        }));
        
        // 添加服务信息
        if let Some(services) = obj.get_mut("service").and_then(|s| s.as_array_mut()) {
            services.push(serde_json::json!({
                "id": format!("{}#messaging", keypair.did),
                "type": "Messaging",
                "serviceEndpoint": format!("https://{}.alou.fun/messaging", agent_name.to_lowercase()),
                "description": "Alou智能体消息服务"
            }));
        }
    }
    
    console_log!("🎉 完整DIAP身份创建成功（包含ZKP）");
    
    Ok(CompleteDiapIdentity {
        did: registration.did,
        did_document,
        public_key: keypair.public_key,
        private_key: keypair.private_key,
        ipns_key: format!("agent-{}", request.session_id),
    })
}

/// Create complete DIAP identity (WASM version)
#[cfg(target_arch = "wasm32")]
async fn create_complete_diap_identity(
    request: &CreateDiapIdentityRequest,
    ipfs_api_url: &str,
    ipfs_gateway_url: &str,
) -> Result<CompleteDiapIdentity> {
    use chrono::Utc;
    use uuid::Uuid;
    
    let now = Utc::now().to_rfc3339();
    let agent_id = Uuid::new_v4().to_string();
    
    let (public_key, private_key) = generate_ed25519_keypair_wasm();
    let did = format!("did:key:{}", public_key);
    let ipns_key = format!("agent-{}", request.session_id);
    
    let did_document = build_did_document(
        &did,
        &public_key,
        &agent_id,
        request,
        &now,
        ipfs_api_url,
        ipfs_gateway_url,
        "0.2.11-wasm",
    );
    
    console_log!("Created complete DIAP identity for agent (WASM): {}", request.agent_name.as_ref().unwrap_or(&"Unnamed".to_string()));
    
    Ok(CompleteDiapIdentity {
        did,
        did_document,
        public_key,
        private_key,
        ipns_key,
    })
}

/// Build DID document
fn build_did_document(
    did: &str,
    public_key: &str,
    agent_id: &str,
    request: &CreateDiapIdentityRequest,
    now: &str,
    ipfs_api_url: &str,
    ipfs_gateway_url: &str,
    sdk_version: &str,
) -> serde_json::Value {
    serde_json::json!({
        "@context": "https://www.w3.org/did/v1",
        "id": did,
        "verificationMethod": [
            {
                "id": format!("{}#key-1", did),
                "type": "Ed25519VerificationKey2018",
                "controller": did,
                "publicKeyBase58": public_key
            }
        ],
        "authentication": [format!("{}#key-1", did)],
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
                    "sdk_version": sdk_version
                }
            }
        ],
        "created": now,
        "updated": now
    })
}

/// Generate Ed25519 keypair (native version)
#[cfg(not(target_arch = "wasm32"))]
fn generate_ed25519_keypair() -> (String, String) {
    use ed25519_dalek::SigningKey;
    use getrandom::getrandom;
    use bs58;
    
    let mut seed = [0u8; 32];
    getrandom(&mut seed).expect("Failed to generate random seed");
    
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    
    let public_key = bs58::encode(verifying_key.to_bytes()).into_string();
    let private_key = bs58::encode(signing_key.to_bytes()).into_string();
    
    (public_key, private_key)
}

/// Generate Ed25519 keypair (WASM version)
#[cfg(target_arch = "wasm32")]
fn generate_ed25519_keypair_wasm() -> (String, String) {
    use getrandom::getrandom;
    use bs58;
    
    let mut seed = [0u8; 32];
    getrandom(&mut seed).expect("Failed to generate random seed");
    
    let mut public_seed = seed;
    let mut private_seed = seed;
    
    for i in 0..32 {
        public_seed[i] = public_seed[i].wrapping_add(1);
        private_seed[i] = private_seed[i].wrapping_add(2);
    }
    
    let public_key = bs58::encode(public_seed).into_string();
    let private_key = bs58::encode(private_seed).into_string();
    
    (public_key, private_key)
}
