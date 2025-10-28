use async_trait::async_trait;
use serde_json::{json, Value};
use crate::agent::context::AgentContext;
use crate::mcp::registry::McpTool;
use crate::storage::kv::KvStore;
use crate::utils::error::{AloudError, Result};
use worker::js_sys::Math;

/// Agent Wallet Tool
/// Allows the agent to create and manage its own wallets
#[derive(Clone)]
pub struct AgentWalletTool {
    kv: KvStore,
}

impl AgentWalletTool {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }
    
    /// Generate a new wallet for the agent
    async fn create_wallet(&self, session_id: &str, chain: &str) -> Result<Value> {
        // Generate a random wallet address (in production, use proper key generation)
        let random_num = Math::random() * 1e38;
        let wallet_address = format!("0x{:040x}", random_num as u128);
        
        let wallet_data = json!({
            "address": wallet_address,
            "chain": chain,
            "created_at": chrono::Utc::now().timestamp(),
            "balance": "0",
            "transactions": []
        });
        
        // Store wallet in KV
        let key = format!("agent_wallet:{}:{}", session_id, chain);
        self.kv.put(&key, &wallet_data.to_string(), None).await?;
        
        Ok(wallet_data)
    }
    
    /// Get agent's wallet for a specific chain
    async fn get_wallet(&self, session_id: &str, chain: &str) -> Result<Option<Value>> {
        let key = format!("agent_wallet:{}:{}", session_id, chain);
        
        match self.kv.get::<String>(&key).await? {
            Some(data) => {
                let wallet: Value = serde_json::from_str(&data)
                    .map_err(|e| AloudError::CacheError(format!("Failed to parse wallet data: {}", e)))?;
                Ok(Some(wallet))
            }
            None => Ok(None)
        }
    }
    
    /// List all agent wallets for a session
    async fn list_wallets(&self, session_id: &str) -> Result<Vec<Value>> {
        let chains = vec!["ethereum", "base", "polygon"];
        let mut wallets = Vec::new();
        
        for chain in chains {
            if let Some(wallet) = self.get_wallet(session_id, chain).await? {
                wallets.push(wallet);
            }
        }
        
        Ok(wallets)
    }
    
    /// Record a transaction for the agent's wallet
    async fn record_transaction(&self, session_id: &str, chain: &str, tx_data: Value) -> Result<()> {
        let key = format!("agent_wallet:{}:{}", session_id, chain);
        
        if let Some(data) = self.kv.get::<String>(&key).await? {
            let mut wallet: Value = serde_json::from_str(&data)
                .map_err(|e| AloudError::CacheError(format!("Failed to parse wallet data: {}", e)))?;
            
            // Add transaction to history
            if let Some(txs) = wallet.get_mut("transactions").and_then(|v| v.as_array_mut()) {
                txs.push(tx_data);
                
                // Keep only last 100 transactions
                if txs.len() > 100 {
                    txs.drain(0..txs.len() - 100);
                }
            }
            
            // Update wallet
            self.kv.put(&key, &wallet.to_string(), None).await?;
        }
        
        Ok(())
    }
    
    /// Update wallet balance
    async fn update_balance(&self, session_id: &str, chain: &str, balance: &str) -> Result<()> {
        let key = format!("agent_wallet:{}:{}", session_id, chain);
        
        if let Some(data) = self.kv.get::<String>(&key).await? {
            let mut wallet: Value = serde_json::from_str(&data)
                .map_err(|e| AloudError::CacheError(format!("Failed to parse wallet data: {}", e)))?;
            
            wallet["balance"] = json!(balance);
            wallet["last_updated"] = json!(chrono::Utc::now().timestamp());
            
            self.kv.put(&key, &wallet.to_string(), None).await?;
        }
        
        Ok(())
    }
}

#[async_trait(?Send)]
impl McpTool for AgentWalletTool {
    fn name(&self) -> &str {
        "agent_wallet"
    }
    
    fn description(&self) -> &str {
        "Manage agent's own wallets. The agent can create wallets, check balances, and record transactions. This allows the agent to have its own on-chain identity and manage assets autonomously."
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "create_wallet",
                        "get_wallet",
                        "list_wallets",
                        "record_transaction",
                        "update_balance"
                    ],
                    "description": "Action to perform: create_wallet (create new wallet), get_wallet (get wallet info), list_wallets (list all wallets), record_transaction (record a transaction), update_balance (update wallet balance)"
                },
                "chain": {
                    "type": "string",
                    "enum": ["ethereum", "base", "polygon"],
                    "description": "Blockchain to operate on (required for create_wallet, get_wallet, record_transaction, update_balance)"
                },
                "transaction": {
                    "type": "object",
                    "description": "Transaction data for record_transaction action",
                    "properties": {
                        "hash": { "type": "string" },
                        "from": { "type": "string" },
                        "to": { "type": "string" },
                        "value": { "type": "string" },
                        "type": { "type": "string" },
                        "status": { "type": "string" }
                    }
                },
                "balance": {
                    "type": "string",
                    "description": "New balance value for update_balance action"
                }
            },
            "required": ["action"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'action' field".to_string()))?;
        
        let session_id = &context.session_id;
        
        match action {
            "create_wallet" => {
                let chain = args.get("chain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'chain' field".to_string()))?;
                
                // Check if wallet already exists
                if let Some(existing) = self.get_wallet(session_id, chain).await? {
                    return Ok(json!({
                        "success": true,
                        "wallet": existing,
                        "message": format!("Wallet already exists for {} chain", chain),
                        "is_new": false
                    }));
                }
                
                let wallet = self.create_wallet(session_id, chain).await?;
                
                Ok(json!({
                    "success": true,
                    "wallet": wallet,
                    "message": format!("Created new wallet for {} chain", chain),
                    "is_new": true
                }))
            }
            
            "get_wallet" => {
                let chain = args.get("chain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'chain' field".to_string()))?;
                
                match self.get_wallet(session_id, chain).await? {
                    Some(wallet) => Ok(json!({
                        "success": true,
                        "wallet": wallet,
                        "message": format!("Retrieved wallet for {} chain", chain)
                    })),
                    None => Ok(json!({
                        "success": false,
                        "message": format!("No wallet found for {} chain. Use create_wallet to create one.", chain)
                    }))
                }
            }
            
            "list_wallets" => {
                let wallets = self.list_wallets(session_id).await?;
                
                Ok(json!({
                    "success": true,
                    "wallets": wallets,
                    "count": wallets.len(),
                    "message": format!("Found {} wallet(s)", wallets.len())
                }))
            }
            
            "record_transaction" => {
                let chain = args.get("chain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'chain' field".to_string()))?;
                
                let transaction = args.get("transaction")
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'transaction' field".to_string()))?
                    .clone();
                
                self.record_transaction(session_id, chain, transaction).await?;
                
                Ok(json!({
                    "success": true,
                    "message": "Transaction recorded successfully"
                }))
            }
            
            "update_balance" => {
                let chain = args.get("chain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'chain' field".to_string()))?;
                
                let balance = args.get("balance")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidToolArgs("Missing 'balance' field".to_string()))?;
                
                self.update_balance(session_id, chain, balance).await?;
                
                Ok(json!({
                    "success": true,
                    "message": format!("Balance updated to {}", balance)
                }))
            }
            
            _ => Err(AloudError::InvalidToolArgs(format!("Unknown action: {}", action)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_wallet_tool_schema() {
        let kv = KvStore::new(worker::kv::KvStore::default());
        let tool = AgentWalletTool::new(kv);
        
        assert_eq!(tool.name(), "agent_wallet");
        assert!(!tool.description().is_empty());
        
        let schema = tool.input_schema();
        assert!(schema.get("properties").is_some());
    }
}
