//! Agent Creation Module
//! 
//! This module handles agent creation operations:
//! - Creating agents with metadata
//! - Parsing creation commands using AI
//! - Creating agents from natural language commands

use crate::agent::session::SessionManager;
use crate::agent::diap_identity::DiapIdentity;
use crate::utils::error::AloudError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::router::{json_response, json_response_with_status, ErrorResponse};

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

/// Create agent using the internal creation logic
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

/// Internal function to handle agent creation logic
pub(crate) async fn handle_create_agent_internal(
    session_manager: &SessionManager,
    body: CreateAgentRequest,
) -> Result<CreateAgentResult> {
    let session_id = if let Some(sid) = &body.session_id {
        session_manager
            .get_session(sid)
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Session not found: {}", e),
                };
                AloudError::WorkerError(error_response.error.clone())
            })?;
        sid.clone()
    } else {
        session_manager
            .create_session(body.wallet_address.clone(), body.chain.clone())
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Failed to create session: {}", e),
                };
                AloudError::WorkerError(error_response.error.clone())
            })?
    };

    let agent_id = format!("agent_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

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

    if let Err(e) = session_manager.set_agent_metadata(&session_id, agent_info).await {
        return Err(AloudError::AgentError(
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

/// Parse agent creation command using AI
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

    let parsed_info = match crate::agent::creation_parser::parse_creation_command(&body.command).await {
        Ok(info) => info,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to parse creation command: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    let session_id = if let Some(sid) = body.session_id {
        session_manager
            .get_session(&sid)
            .await
            .map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Session not found: {}", e),
                };
                AloudError::WorkerError(error_response.error.clone())
            })?;
        sid
    } else {
        let wallet_address = body.wallet_address.clone().unwrap_or_else(|| "default_user".to_string());
        session_manager
            .create_session(Some(wallet_address), body.chain.clone())
            .await?
    };

    let create_request = serde_json::json!({
        "session_id": session_id,
        "wallet_address": body.wallet_address,
        "chain": body.chain,
        "name": parsed_info.name,
        "role_description": parsed_info.role_description,
        "avatar_cid": null,
        "mcp_config_cid": null,
        "mcp_ports": [],
        "diap_identity": null
    });

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

/// Request structure for updating an agent
#[derive(Deserialize)]
pub(crate) struct UpdateAgentRequest {
    pub session_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub role_description: Option<String>,
    #[serde(default)]
    pub avatar_cid: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub mcp_config_cid: Option<String>,
    #[serde(default)]
    pub mcp_ports: Option<Vec<McpPortConfig>>,
}

/// Update agent metadata
pub(crate) async fn handle_update_agent(
    session_manager: &SessionManager,
    req: &mut Request,
) -> Result<Response> {
    let body: UpdateAgentRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // Verify session exists
    let session = match session_manager.get_session(&body.session_id).await {
        Ok(session) => session,
        Err(_) => {
            let error_response = ErrorResponse {
                error: format!("Session not found: {}", body.session_id),
            };
            return Ok(json_response_with_status(&error_response, 404)?);
        }
    };

    // Get current agent metadata
    let current_metadata = session_manager
        .get_agent_metadata(&body.session_id)
        .await
        .unwrap_or_else(|_| json!({}));

    // Build update
    let mut updates = json!({});
    
    if let Some(name) = body.name {
        updates["name"] = json!(name);
    }
    
    if let Some(role_description) = body.role_description {
        updates["role_description"] = json!(role_description);
    }
    
    if let Some(avatar_cid) = body.avatar_cid {
        updates["avatar_cid"] = json!(avatar_cid);
        // If avatar_url is not provided, generate from CID
        if body.avatar_url.is_none() {
            updates["avatar_url"] = json!(format!("https://gateway.ipfs.io/ipfs/{}", avatar_cid));
        }
    }
    
    if let Some(avatar_url) = body.avatar_url {
        updates["avatar_url"] = json!(avatar_url);
    }
    
    if let Some(mcp_config_cid) = body.mcp_config_cid {
        updates["mcp_config_cid"] = json!(mcp_config_cid);
    }
    
    if let Some(mcp_ports) = body.mcp_ports {
        updates["mcp_ports"] = json!(mcp_ports);
    }

    // Merge with existing metadata
    let mut new_metadata = current_metadata.clone();
    if let Some(obj) = new_metadata.as_object_mut() {
        for (key, value) in updates.as_object().unwrap_or(&serde_json::Map::new()) {
            obj.insert(key.clone(), value.clone());
        }
        obj.insert("updated_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
    }

    // Save updated metadata
    if let Err(e) = session_manager
        .set_agent_metadata(&body.session_id, new_metadata.clone())
        .await
    {
        let error_response = ErrorResponse {
            error: format!("Failed to update agent: {}", e),
        };
        return Ok(json_response_with_status(&error_response, 500)?);
    }

    let response = json!({
        "success": true,
        "message": "Agent updated successfully",
        "session_id": body.session_id,
        "agent_metadata": new_metadata,
    });

    Ok(json_response(&response)?)
}
