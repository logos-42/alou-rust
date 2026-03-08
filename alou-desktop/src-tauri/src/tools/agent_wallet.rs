use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolMetadata, ExecutionContext, ToolCategory, ToolPriority, ToolStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

const WALLET_KV_PREFIX: &str = "agent_wallet";

#[derive(Clone)]
pub struct AgentWalletTool {
    metadata: ToolMetadata,
    wallets: Arc<RwLock<HashMap<String, WalletData>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub address: String,
    pub chain: String,
    pub created_at: i64,
    pub balance: String,
    pub transactions: Vec<Value>,
}

impl AgentWalletTool {
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "agent_wallet".to_string(),
            name: "agent_wallet".to_string(),
            description: "Manage agent's own wallets. The agent can create wallets, check balances, and record transactions.".to_string(),
            category: ToolCategory::Web3,
            priority: ToolPriority::High,
            status: ToolStatus::Active,
            version: "1.0.0".to_string(),
            author: "Alou".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            dependencies: vec![],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec![],
        };

        Self {
            metadata,
            wallets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn generate_wallet_address(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
        format!("0x{}", hex::encode(random_bytes))
    }

    async fn create_wallet(&self, session_id: &str, chain: &str) -> Result<WalletData, ToolError> {
        let key = format!("{}:{}:{}", WALLET_KV_PREFIX, session_id, chain);
        
        let wallets = self.wallets.read().await;
        if let Some(existing) = wallets.get(&key) {
            return Ok(existing.clone());
        }
        drop(wallets);

        let wallet = WalletData {
            address: self.generate_wallet_address(),
            chain: chain.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            balance: "0".to_string(),
            transactions: vec![],
        };

        let mut wallets = self.wallets.write().await;
        wallets.insert(key, wallet.clone());

        Ok(wallet)
    }

    async fn get_wallet(&self, session_id: &str, chain: &str) -> Result<Option<WalletData>, ToolError> {
        let key = format!("{}:{}:{}", WALLET_KV_PREFIX, session_id, chain);
        let wallets = self.wallets.read().await;
        Ok(wallets.get(&key).cloned())
    }

    async fn list_wallets(&self, session_id: &str) -> Result<Vec<WalletData>, ToolError> {
        let chains = vec!["ethereum", "base", "polygon"];
        let mut result = vec![];
        
        for chain in chains {
            if let Ok(Some(wallet)) = self.get_wallet(session_id, chain).await {
                result.push(wallet);
            }
        }
        
        Ok(result)
    }

    async fn record_transaction(
        &self,
        session_id: &str,
        chain: &str,
        tx_data: Value,
    ) -> Result<(), ToolError> {
        let key = format!("{}:{}:{}", WALLET_KV_PREFIX, session_id, chain);
        
        let mut wallets = self.wallets.write().await;
        if let Some(wallet) = wallets.get_mut(&key) {
            wallet.transactions.push(tx_data);
            if wallet.transactions.len() > 100 {
                wallet.transactions.drain(0..wallet.transactions.len() - 100);
            }
        }

        Ok(())
    }

    async fn update_balance(&self, session_id: &str, chain: &str, balance: &str) -> Result<(), ToolError> {
        let key = format!("{}:{}:{}", WALLET_KV_PREFIX, session_id, chain);
        
        let mut wallets = self.wallets.write().await;
        if let Some(wallet) = wallets.get_mut(&key) {
            wallet.balance = balance.to_string();
        }

        Ok(())
    }
}

impl Default for AgentWalletTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for AgentWalletTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: Value,
        context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start = std::time::Instant::now();
        
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        let session_id = &context.session_id;

        let result = match action {
            "create_wallet" => {
                let chain = args.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'chain' field".to_string())
                })?;

                let wallet = self.create_wallet(session_id, chain).await?;

                Ok(json!({
                    "success": true,
                    "wallet": wallet,
                    "message": format!("Created new wallet for {} chain", chain),
                    "is_new": true
                }))
            }

            "get_wallet" => {
                let chain = args.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'chain' field".to_string())
                })?;

                match self.get_wallet(session_id, chain).await? {
                    Some(wallet) => Ok(json!({
                        "success": true,
                        "wallet": wallet,
                        "message": format!("Retrieved wallet for {} chain", chain)
                    })),
                    None => Ok(json!({
                        "success": false,
                        "message": format!("No wallet found for {} chain", chain)
                    })),
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
                let chain = args.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'chain' field".to_string())
                })?;

                let transaction = args.get("transaction").ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'transaction' field".to_string())
                })?.clone();

                self.record_transaction(session_id, chain, transaction).await?;

                Ok(json!({
                    "success": true,
                    "message": "Transaction recorded successfully"
                }))
            }

            "update_balance" => {
                let chain = args.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'chain' field".to_string())
                })?;

                let balance = args.get("balance").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'balance' field".to_string())
                })?;

                self.update_balance(session_id, chain, balance).await?;

                Ok(json!({
                    "success": true,
                    "message": format!("Balance updated to {}", balance)
                }))
            }

            _ => Err(ToolError::InvalidArguments(format!(
                "Unknown action: {}",
                action
            ))),
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(data) => Ok(ToolResult {
                success: true,
                data,
                error: None,
                execution_time_ms,
                output: None,
                warnings: vec![],
                context: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                data: json!({}),
                error: Some(e.to_string()),
                execution_time_ms,
                output: None,
                warnings: vec![],
                context: None,
            }),
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Args must be an object".to_string()));
        }

        if let Some(action) = args.get("action").and_then(|v| v.as_str()) {
            match action {
                "create_wallet" | "get_wallet" | "record_transaction" | "update_balance" => {
                    if !args.get("chain").is_some() {
                        return Err(ToolError::InvalidArguments("Missing 'chain' field".to_string()));
                    }
                }
                "list_wallets" => {}
                _ => return Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
            }
        } else {
            return Err(ToolError::InvalidArguments("Missing 'action' field".to_string()));
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Agent Wallet Tool - Manage agent's own wallets

Actions:
- create_wallet: Create a new wallet for a specific chain
- get_wallet: Get wallet information for a specific chain  
- list_wallets: List all wallets for the session
- record_transaction: Record a transaction to wallet history
- update_balance: Update wallet balance

Parameters:
- action: The action to perform (required)
- chain: The blockchain chain (required for create_wallet, get_wallet, record_transaction, update_balance)
- transaction: Transaction data (required for record_transaction)
- balance: New balance value (required for update_balance)

Example:
  {"action": "create_wallet", "chain": "ethereum"}"#.to_string()
    }
}
