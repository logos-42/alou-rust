use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::{console_log, Fetch, Method, RequestInit};
use crate::utils::error::{AloudError, Result};

/// Broadcast tool for sending signed transactions
pub struct BroadcastTool {
    eth_rpc_url: String,
    sol_rpc_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub tx_hash: String,
    pub status: String,
    pub block_number: Option<String>,
}

impl BroadcastTool {
    pub fn new(eth_rpc_url: String, sol_rpc_url: String) -> Self {
        Self {
            eth_rpc_url,
            sol_rpc_url,
        }
    }

    /// Broadcast Ethereum transaction
    pub async fn broadcast_eth_transaction(&self, signed_tx: &str) -> Result<String> {
        console_log!("Broadcasting ETH transaction");
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [signed_tx],
            "id": 1
        });

        let response = self.rpc_call(&self.eth_rpc_url, request_body).await?;
        
        if let Some(tx_hash) = response.get("result").and_then(|r| r.as_str()) {
            console_log!("Transaction broadcasted: {}", tx_hash);
            Ok(tx_hash.to_string())
        } else if let Some(error) = response.get("error") {
            Err(AloudError::AgentError(format!("Broadcast error: {}", error)))
        } else {
            Err(AloudError::AgentError("Failed to broadcast transaction".to_string()))
        }
    }

    /// Broadcast Solana transaction
    pub async fn broadcast_sol_transaction(&self, signed_tx: &str) -> Result<String> {
        console_log!("Broadcasting SOL transaction");
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "sendTransaction",
            "params": [signed_tx, {"encoding": "base64"}],
            "id": 1
        });

        let response = self.rpc_call(&self.sol_rpc_url, request_body).await?;
        
        if let Some(tx_hash) = response.get("result").and_then(|r| r.as_str()) {
            console_log!("Transaction broadcasted: {}", tx_hash);
            Ok(tx_hash.to_string())
        } else if let Some(error) = response.get("error") {
            Err(AloudError::AgentError(format!("Broadcast error: {}", error)))
        } else {
            Err(AloudError::AgentError("Failed to broadcast transaction".to_string()))
        }
    }

    /// Get Ethereum transaction receipt
    pub async fn get_eth_transaction_receipt(&self, tx_hash: &str) -> Result<TransactionReceipt> {
        console_log!("Getting ETH transaction receipt for {}", tx_hash);
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [tx_hash],
            "id": 1
        });

        let response = self.rpc_call(&self.eth_rpc_url, request_body).await?;
        
        if let Some(result) = response.get("result") {
            let status = result
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            
            let block_number = result
                .get("blockNumber")
                .and_then(|b| b.as_str())
                .map(|s| s.to_string());
            
            Ok(TransactionReceipt {
                tx_hash: tx_hash.to_string(),
                status: if status == "0x1" { "success".to_string() } else { "failed".to_string() },
                block_number,
            })
        } else {
            Err(AloudError::AgentError("Transaction not found or pending".to_string()))
        }
    }

    /// Get Solana transaction status
    pub async fn get_sol_transaction_status(&self, tx_hash: &str) -> Result<TransactionReceipt> {
        console_log!("Getting SOL transaction status for {}", tx_hash);
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "getSignatureStatuses",
            "params": [[tx_hash]],
            "id": 1
        });

        let response = self.rpc_call(&self.sol_rpc_url, request_body).await?;
        
        if let Some(result) = response
            .get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
        {
            if result.is_null() {
                return Err(AloudError::AgentError("Transaction not found".to_string()));
            }
            
            let status = if result.get("err").is_some() {
                "failed"
            } else {
                "success"
            };
            
            let slot = result
                .get("slot")
                .and_then(|s| s.as_u64())
                .map(|s| s.to_string());
            
            Ok(TransactionReceipt {
                tx_hash: tx_hash.to_string(),
                status: status.to_string(),
                block_number: slot,
            })
        } else {
            Err(AloudError::AgentError("Transaction not found or pending".to_string()))
        }
    }

    /// Check if transaction is confirmed
    #[allow(dead_code)]
    pub async fn is_transaction_confirmed(&self, tx_hash: &str, chain: &str) -> Result<bool> {
        match chain.to_lowercase().as_str() {
            "eth" | "ethereum" => {
                match self.get_eth_transaction_receipt(tx_hash).await {
                    Ok(receipt) => Ok(receipt.block_number.is_some()),
                    Err(_) => Ok(false),
                }
            }
            "sol" | "solana" => {
                match self.get_sol_transaction_status(tx_hash).await {
                    Ok(receipt) => Ok(receipt.status == "success"),
                    Err(_) => Ok(false),
                }
            }
            _ => Err(AloudError::InvalidInput(format!("Unsupported chain: {}", chain))),
        }
    }

    /// Generic RPC call helper
    async fn rpc_call(&self, url: &str, body: Value) -> Result<Value> {
        let body_str = serde_json::to_string(&body)
            .map_err(|e| AloudError::AgentError(format!("Serialize error: {}", e)))?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        
        let headers = worker::Headers::new();
        headers.set("Content-Type", "application/json")
            .map_err(|e| AloudError::AgentError(e.to_string()))?;
        init.with_headers(headers);
        init.with_body(Some(body_str.into()));

        let request = worker::Request::new_with_init(url, &init)
            .map_err(|e| AloudError::AgentError(e.to_string()))?;

        let mut response = Fetch::Request(request)
            .send()
            .await
            .map_err(|e| AloudError::AgentError(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(e.to_string()))?;

        serde_json::from_str(&response_text)
            .map_err(|e| AloudError::AgentError(format!("Parse response error: {}", e)))
    }
}
