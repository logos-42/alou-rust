use serde::{Deserialize, Serialize};

/// Agent execution context containing session and user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Session identifier
    pub session_id: String,
    
    /// Optional wallet address of the authenticated user
    pub wallet_address: Option<String>,
    
    /// Optional blockchain type (ethereum, solana, etc.)
    pub chain: Option<String>,
}

impl AgentContext {
    /// Create a new agent context
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            wallet_address: None,
            chain: None,
        }
    }
    
    /// Create a context with wallet information
    #[allow(dead_code)]
    pub fn with_wallet(session_id: String, wallet_address: String, chain: String) -> Self {
        Self {
            session_id,
            wallet_address: Some(wallet_address),
            chain: Some(chain),
        }
    }
}
