use serde::{Deserialize, Serialize};

use crate::agent::session::ContextEvent;

/// Agent execution context containing session and user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Session identifier
    pub session_id: String,

    /// Optional wallet address of the authenticated user
    pub wallet_address: Option<String>,

    /// Optional blockchain type (ethereum, solana, etc.)
    pub chain: Option<String>,

    /// Recent context events captured from the frontend
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_events: Vec<ContextEvent>,

    /// Preformatted recent event summary for prompt usage
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_summary: Option<String>,
}

impl AgentContext {
    /// Create a new agent context
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            wallet_address: None,
            chain: None,
            recent_events: Vec::new(),
            event_summary: None,
        }
    }

    /// Create a context with wallet information
    #[allow(dead_code)]
    pub fn with_wallet(session_id: String, wallet_address: String, chain: String) -> Self {
        Self {
            session_id,
            wallet_address: Some(wallet_address),
            chain: Some(chain),
            recent_events: Vec::new(),
            event_summary: None,
        }
    }
}
