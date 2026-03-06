use crate::agent::context::AgentContext;
use crate::agent::session::SessionManager;
use crate::mcp::registry::McpTool;
use crate::utils::error::{AloudError, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Agent Avatar Tool
/// Allows the agent to update its own avatar/profile image
#[derive(Clone)]
pub struct AgentAvatarTool {
    session_manager: SessionManager,
}

impl AgentAvatarTool {
    pub fn new(session_manager: SessionManager) -> Self {
        Self { session_manager }
    }

    /// Update agent's avatar with a URL or IPFS CID
    async fn update_avatar(
        &self,
        session_id: &str,
        avatar_url: Option<&str>,
        avatar_cid: Option<&str>,
    ) -> Result<Value> {
        // Get current session/agent metadata
        let session = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|e| AloudError::AgentError(format!("Session not found: {}", e)))?;

        // Get current agent metadata
        let agent_metadata = self
            .session_manager
            .get_agent_metadata(session_id)
            .await
            .unwrap_or_else(|_| json!({}));

        // Build the update
        let mut updates = json!({});
        
        if let Some(url) = avatar_url {
            updates["avatar_url"] = json!(url);
        }
        
        if let Some(cid) = avatar_cid {
            updates["avatar_cid"] = json!(cid);
            // If no URL provided but CID is provided, build the IPFS URL
            if avatar_url.is_none() {
                updates["avatar_url"] = json!(format!("https://gateway.ipfs.io/ipfs/{}", cid));
            }
        }

        // Merge with existing metadata
        let mut new_metadata = agent_metadata.clone();
        if let Some(obj) = new_metadata.as_object_mut() {
            for (key, value) in updates.as_object().unwrap_or(&serde_json::Map::new()) {
                obj.insert(key.clone(), value.clone());
            }
        }

        // Save updated metadata
        self.session_manager
            .set_agent_metadata(session_id, new_metadata.clone())
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to update avatar: {}", e)))?;

        Ok(json!({
            "success": true,
            "message": "Avatar updated successfully",
            "session_id": session_id,
            "avatar_url": new_metadata.get("avatar_url").cloned(),
            "avatar_cid": new_metadata.get("avatar_cid").cloned(),
        }))
    }

    /// Get agent's current avatar info
    async fn get_avatar(&self, session_id: &str) -> Result<Value> {
        let agent_metadata = self
            .session_manager
            .get_agent_metadata(session_id)
            .await
            .unwrap_or_else(|_| json!({}));

        let avatar_url = agent_metadata
            .get("avatar_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let avatar_cid = agent_metadata
            .get("avatar_cid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(json!({
            "success": true,
            "session_id": session_id,
            "has_avatar": avatar_url.is_some() || avatar_cid.is_some(),
            "avatar_url": avatar_url,
            "avatar_cid": avatar_cid,
        }))
    }

    /// Clear agent's avatar (reset to default)
    async fn clear_avatar(&self, session_id: &str) -> Result<Value> {
        let agent_metadata = self
            .session_manager
            .get_agent_metadata(session_id)
            .await
            .unwrap_or_else(|_| json!({}));

        let mut new_metadata = agent_metadata.clone();
        if let Some(obj) = new_metadata.as_object_mut() {
            obj.remove("avatar_url");
            obj.remove("avatar_cid");
        }

        self.session_manager
            .set_agent_metadata(session_id, new_metadata)
            .await
            .map_err(|e| AloudError::AgentError(format!("Failed to clear avatar: {}", e)))?;

        Ok(json!({
            "success": true,
            "message": "Avatar cleared successfully",
            "session_id": session_id,
        }))
    }
}

#[async_trait(?Send)]
impl McpTool for AgentAvatarTool {
    fn name(&self) -> &str {
        "agent_avatar"
    }

    fn description(&self) -> &str {
        "Manage agent's own avatar/profile image. The agent can update its avatar using a URL, IPFS CID, or clear it. This allows the agent to customize its visual identity."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "update_avatar",
                        "get_avatar",
                        "clear_avatar"
                    ],
                    "description": "Action to perform: update_avatar (set a new avatar), get_avatar (get current avatar info), clear_avatar (remove avatar and use default)"
                },
                "avatar_url": {
                    "type": "string",
                    "description": "URL of the avatar image (http/https/data URL). Required for update_avatar action if avatar_cid is not provided."
                },
                "avatar_cid": {
                    "type": "string",
                    "description": "IPFS CID of the avatar image. If provided without avatar_url, the URL will be automatically generated."
                },
                "session_id": {
                    "type": "string",
                    "description": "Session ID of the agent. If not provided, will use the context session_id."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: Value, context: &AgentContext) -> Result<Value> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing 'action' parameter".to_string()))?;

        let session_id = params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or(&context.session_id);

        match action {
            "update_avatar" => {
                let avatar_url = params.get("avatar_url").and_then(|v| v.as_str());
                let avatar_cid = params.get("avatar_cid").and_then(|v| v.as_str());

                if avatar_url.is_none() && avatar_cid.is_none() {
                    return Err(AloudError::InvalidInput(
                        "Either 'avatar_url' or 'avatar_cid' must be provided for update_avatar".to_string()
                    ));
                }

                self.update_avatar(session_id, avatar_url, avatar_cid).await
            }
            "get_avatar" => self.get_avatar(session_id).await,
            "clear_avatar" => self.clear_avatar(session_id).await,
            _ => Err(AloudError::InvalidInput(format!(
                "Unknown action: {}. Valid actions are: update_avatar, get_avatar, clear_avatar",
                action
            ))),
        }
    }
}
