use async_trait::async_trait;
use serde_json::{json, Value};
use crate::agent::context::AgentContext;
use crate::mcp::registry::McpTool;
use crate::storage::kv::KvStore;
use crate::utils::error::{AloudError, Result};
use crate::utils::crypto::ChainType;
use crate::web3::auth::WalletAuth;

/// Wallet authentication MCP tool
/// Handles nonce generation, signature verification, and JWT token creation
#[allow(dead_code)]
pub struct WalletAuthTool {
    wallet_auth: WalletAuth,
}

#[allow(dead_code)]
impl WalletAuthTool {
    pub fn new(kv: KvStore, jwt_secret: String) -> Self {
        let wallet_auth = WalletAuth::new(kv, jwt_secret);
        Self { wallet_auth }
    }
    
    /// Generate a nonce for wallet authentication
    pub async fn generate_nonce(&self, address: &str) -> Result<String> {
        self.wallet_auth.generate_nonce_for_address(address).await
    }
    
    /// Verify signature and create session token
    pub async fn verify_signature(
        &self,
        address: &str,
        signature: &str,
        message: &str,
        chain: &str,
    ) -> Result<String> {
        let chain_type = ChainType::from_str(chain)?;
        self.wallet_auth.verify_and_create_token(address, signature, message, chain_type).await
    }
    
    /// Create a session token for authenticated wallet
    pub fn create_session(&self, wallet_address: &str, chain: &str) -> Result<String> {
        let chain_type = ChainType::from_str(chain)?;
        self.wallet_auth.create_token(wallet_address, chain_type)
    }
}

#[async_trait(?Send)]
impl McpTool for WalletAuthTool {
    fn name(&self) -> &str {
        "wallet_auth"
    }
    
    fn description(&self) -> &str {
        "Authenticate wallet using signature verification. Supports generating nonce, verifying signatures, and creating JWT tokens for Ethereum and Solana wallets."
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["generate_nonce", "verify_signature", "create_session"],
                    "description": "Action to perform: generate_nonce, verify_signature, or create_session"
                },
                "address": {
                    "type": "string",
                    "description": "Wallet address (required for all actions)"
                },
                "signature": {
                    "type": "string",
                    "description": "Wallet signature (required for verify_signature)"
                },
                "message": {
                    "type": "string",
                    "description": "Signed message (required for verify_signature)"
                },
                "chain": {
                    "type": "string",
                    "enum": ["ethereum", "solana"],
                    "description": "Blockchain type (required for verify_signature and create_session)"
                }
            },
            "required": ["action", "address"]
        })
    }
    
    async fn execute(&self, args: Value, _context: &AgentContext) -> Result<Value> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'action' field".to_string()))?;
        
        let address = args.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'address' field".to_string()))?;
        
        match action {
            "generate_nonce" => {
                let nonce = self.generate_nonce(address).await?;
                Ok(json!({
                    "success": true,
                    "nonce": nonce,
                    "message": format!("Sign this message to authenticate: {}", nonce)
                }))
            }
            
            "verify_signature" => {
                let signature = args.get("signature")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'signature' field".to_string()))?;
                
                let message = args.get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'message' field".to_string()))?;
                
                let chain = args.get("chain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'chain' field".to_string()))?;
                
                let token = self.verify_signature(address, signature, message, chain).await?;
                
                Ok(json!({
                    "success": true,
                    "token": token,
                    "wallet_address": address,
                    "chain": chain
                }))
            }
            
            "create_session" => {
                let chain = args.get("chain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'chain' field".to_string()))?;
                
                let token = self.create_session(address, chain)?;
                
                Ok(json!({
                    "success": true,
                    "token": token,
                    "wallet_address": address,
                    "chain": chain
                }))
            }
            
            _ => Err(AloudError::InvalidToolArgs(format!("Unknown action: {}", action)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_wallet_auth_tool_schema() {
        let kv = KvStore::new(worker::kv::KvStore::default());
        let tool = WalletAuthTool::new(kv, "test_secret".to_string());
        
        assert_eq!(tool.name(), "wallet_auth");
        assert!(!tool.description().is_empty());
        
        let schema = tool.input_schema();
        assert!(schema.get("properties").is_some());
    }
}
