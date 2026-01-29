// DIAP Agent Router - 简化版本（专注于存储功能）
// 
// 新架构：
// 1. 桌面端：创建完整DIAP身份（包含DID、CID、IPNS、ZKP）
// 2. 桌面端：处理所有IPFS操作
// 3. 后端：专注于KV存储和验证

use serde::{Deserialize, Serialize};
use worker;

use crate::{
    agent::session::SessionManager,
    utils::error::AloudError,
    storage::kv::KvStore,
};

/// Error response structure
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Request to save complete DIAP identity
#[derive(Deserialize)]
pub struct SaveDiapIdentityRequest {
    session_id: String,
    diap_identity: CompleteDiapIdentity,
}

/// Complete DIAP identity with ZKP proof
#[derive(Serialize, Deserialize, Clone)]
pub struct CompleteDiapIdentity {
    pub did: String,
    pub did_document: serde_json::Value,
    pub public_key: String,
    pub private_key: String,
    pub ipns_key: String,
    pub cid: String,
    pub ipns: Option<String>,
    pub zkp_proof: Option<serde_json::Value>,
    pub created_by: Option<String>,
}

/// Request to get DIAP identity by session
#[derive(Deserialize)]
pub struct GetDiapIdentityRequest {
    session_id: String,
}

/// Save complete DIAP identity to backend KV storage
pub(crate) async fn handle_save_complete_diap_identity(
    session_manager: &crate::agent::session::SessionManager,
    env: &worker::Env,
    req: &mut worker::Request,
) -> Result<worker::Response, AloudError> {
    let body: SaveDiapIdentityRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
        }
    };

    if body.session_id.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "session_id is required".to_string(),
        };
        return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
    }

    match session_manager.get_session(&body.session_id).await {
        Ok(_) => {
            // 验证DIAP身份数据
            if body.diap_identity.did.trim().is_empty() {
                let error_response = ErrorResponse {
                    error: "did is required".to_string(),
                };
                return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
            }

            if body.diap_identity.cid.trim().is_empty() {
                let error_response = ErrorResponse {
                    error: "cid is required".to_string(),
                };
                return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
            }

            // 保存到KV存储
            match save_diap_identity_to_kv(env, &body.session_id, &body.diap_identity).await {
                Ok(_) => {
                    worker::console_log!("✅ DIAP身份保存成功: {}", body.session_id);
                    
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
                    
                    return Ok(worker::Response::from_json(&response)?);
                }
                Err(e) => {
                    worker::console_log!("❌ DIAP身份保存失败: {}", e);
                    let error_response = ErrorResponse {
                        error: format!("Failed to save DIAP identity: {}", e),
                    };
                    return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
                }
            }
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Session not found or invalid: {}", e),
            };
            return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
        }
    }
}

/// Get DIAP identity by session
pub(crate) async fn handle_get_diap_identity_by_session(
    session_manager: &crate::agent::session::SessionManager,
    env: &worker::Env,
    req: &mut worker::Request,
) -> Result<worker::Response, AloudError> {
    let body: GetDiapIdentityRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
        }
    };

    if body.session_id.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "session_id is required".to_string(),
        };
        return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
    }

    match session_manager.get_session(&body.session_id).await {
        Ok(_) => {
            // 从KV存储获取DIAP身份
            match get_diap_identity_from_kv(env, &body.session_id).await {
                Ok(Some(identity)) => {
                    worker::console_log!("✅ 从KV获取DIAP身份成功: {}", body.session_id);
                    let response = serde_json::json!({
                        "success": true,
                        "identity": identity
                    });
                    return Ok(worker::Response::from_json(&response)?);
                }
                Ok(None) => {
                    worker::console_log!("ℹ️  DIAP身份不存在: {}", body.session_id);
                    let error_response = ErrorResponse {
                        error: "DIAP identity not found".to_string(),
                    };
                    return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
                }
                Err(e) => {
                    worker::console_log!("❌ 从KV获取DIAP身份失败: {}", e);
                    let error_response = ErrorResponse {
                        error: format!("Failed to get DIAP identity: {}", e),
                    };
                    return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
                }
            }
        }
        Err(e) => {
            worker::console_log!("❌ Session not found when getting DIAP identity: {} - {}", body.session_id, e);
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
            return Ok(worker::Response::from_json(&error_response)?);
        }
    }
}

/// Save DIAP identity to KV storage
async fn save_diap_identity_to_kv(
    env: &worker::Env,
    session_id: &str,
    diap_identity: &CompleteDiapIdentity,
) -> Result<(), AloudError> {
    use worker::kv::KvStore;
    
    // 获取KV存储
    let kv = env.kv("DIAP_KV_NAMESPACE")
        .map_err(|e| AloudError::WorkerError(format!("KV store access failed: {:?}", e)))?;
    
    // 构建存储键
    let storage_key = format!("diap_identity:{}", session_id);
    
    // 序列化DIAP身份
    let identity_json = serde_json::to_string(diap_identity)
        .map_err(|e| AloudError::InvalidInput(e.to_string()))?;
    
    // 保存到KV
    kv.put(&storage_key, identity_json)
        .map_err(|e| AloudError::WorkerError(format!("KV put failed: {:?}", e)))?;
    
    worker::console_log!("💾 DIAP身份已保存到KV: {} -> {}", session_id, diap_identity.did);
    Ok(())
}

/// Get DIAP identity from KV storage
async fn get_diap_identity_from_kv(
    env: &worker::Env,
    session_id: &str,
) -> Result<Option<CompleteDiapIdentity>, AloudError> {
    // 获取KV存储
    let kv = env.kv("DIAP_KV_NAMESPACE")
        .map_err(|e| AloudError::WorkerError(format!("KV store access failed: {:?}", e)))?;
    
    // 构建存储键
    let storage_key = format!("diap_identity:{}", session_id);
    
    // 从KV获取
    let result = kv.get(&storage_key).text().await
        .map_err(|e| AloudError::WorkerError(format!("KV get failed: {:?}", e)))?;
    
    match result {
        Some(text) => {
            let identity: CompleteDiapIdentity = serde_json::from_str(&text)
                .map_err(|e| AloudError::InvalidInput(format!("Failed to deserialize DIAP identity: {}", e)))?;
            worker::console_log!("💾 从KV获取DIAP身份: {} -> {}", session_id, identity.did);
            Ok(Some(identity))
        }
        None => {
            worker::console_log!("💾 KV中未找到DIAP身份: {}", session_id);
            Ok(None)
        }
    }
}

/// Create basic DIAP identity on backend
pub(crate) async fn handle_create_diap_identity(
    session_manager: &crate::agent::session::SessionManager,
    env: &worker::Env,
    req: &mut worker::Request,
) -> Result<worker::Response, AloudError> {
    use serde_json::json;
    
    // 解析请求体
    let request_body: serde_json::Value = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
        }
    };

    let session_id = request_body.get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    if session_id.is_empty() {
        let error_response = ErrorResponse {
            error: "session_id is required".to_string(),
        };
        return Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?);
    }

    // 验证会话
    match session_manager.get_session(session_id).await {
        Ok(_) => {
            // 在新架构中，基础身份创建已移至桌面端
            // 这里只返回一个响应，指示应该使用桌面端创建
            let response = json!({
                "success": false,
                "error": "Basic identity creation is deprecated. Please use desktop端的 createCompleteDiapIdentity instead.",
                "message": "请使用桌面端的完整DIAP身份创建流程"
            });
            
            Ok(worker::Response::from_json(&response)?)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", e),
            };
            Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?)
        }
    }
}

/// Register agent to DIAP network on-chain
pub(crate) async fn handle_register_agent_onchain(
    _session_manager: &crate::agent::session::SessionManager,
    env: &worker::Env,
    _req: &mut worker::Request,
) -> Result<worker::Response, AloudError> {
    // TODO: Implement on-chain registration
    let error_response = ErrorResponse {
        error: "On-chain registration not yet implemented".to_string(),
    };
    Ok(worker::Response::from_bytes(serde_json::to_vec(&error_response).unwrap().into())?)
}
