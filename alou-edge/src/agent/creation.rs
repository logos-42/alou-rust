use crate::agent::session::SessionManager;
use crate::agent::ai_client::AiClient;
use crate::agent::spec::TaskSpec;
use crate::agent::spec_validator::SpecValidator;
use crate::utils::error::{Result, AloudError};
use serde::Serialize;
use serde_json::Value;
use worker::*;

/// Create Claude agent from parsed information
pub async fn create_claude_agent_from_parsed(
    session_manager: &SessionManager,
    env: &Env,
    request: Value,
) -> Result<ClaudeAgentResult> {
    // Extract fields from request
    let session_id = request.get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "session_id is required".to_string())?;

    let wallet_address = request.get("wallet_address")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let chain = request.get("chain")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let name = request.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Claude Agent SDK")
        .to_string();

    let role_description = request.get("roleDescription")
        .and_then(|v| v.as_str())
        .unwrap_or("一个去中心化的智能体")
        .to_string();

    // Create or verify session
    let _session = if let Ok(s) = session_manager.get_session(session_id).await {
        s
    } else {
        return Err(AloudError::AgentError("Session not found".to_string()));
    };

    // Check for task specification in request
    let task_spec = request.get("task_spec").and_then(|v| serde_json::from_value::<TaskSpec>(v.clone()).ok());
    
    // Validate task specification if provided
    let validation_result = if let Some(ref spec) = task_spec {
        let validator = SpecValidator;
        let result = validator.validate(spec);
        
        if !result.is_valid {
            console_log!("Task specification validation failed: {:?}", result.errors);
        }
        
        Some(result)
    } else {
        None
    };
    
    // Create agent metadata
    let mut metadata = serde_json::json!({
        "agent_type": "claude_agent_sdk",
        "display_name": name,
        "session_id": session_id,
        "role_description": role_description,
    });
    
    // Add validation result to metadata if available
    if let Some(ref validation) = validation_result {
        metadata["spec_validation"] = serde_json::to_value(validation).unwrap_or(Value::Null);
    }
    
    // Add task spec to metadata if available
    if let Some(ref spec) = task_spec {
        metadata["task_spec"] = serde_json::to_value(spec).unwrap_or(Value::Null);
    }

    if let Some(wallet) = &wallet_address {
        metadata["wallet_address"] = Value::String(wallet.clone());
    }

    if let Some(c) = &chain {
        metadata["chain"] = Value::String(c.clone());
    }

    // Store metadata
    if let Err(e) = session_manager.set_agent_metadata(session_id, metadata.clone()).await {
        return Err(AloudError::DatabaseError(format!("Failed to store agent metadata: {}", e)));
    }

    // Create simple AI response
    let api_key = env.secret("AI_API_KEY")
        .map_err(|_| "AI_API_KEY not configured".to_string())?
        .to_string();

    let _ai_client = AiClient::new("deepseek", api_key, None)
        .map_err(|e| AloudError::InternalError(format!("Failed to create AI client: {}", e)))?;

    Ok(ClaudeAgentResult {
        agent_id: session_id.to_string(),
        session_id: session_id.to_string(),
        name,
        agent_metadata: metadata,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaudeAgentResult {
    pub agent_id: String,
    pub session_id: String,
    pub name: String,
    pub agent_metadata: Value,
}
