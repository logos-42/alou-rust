use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::{console_log, Fetch, Method, RequestInit};
use crate::utils::error::{AloudError, Result};

/// Transaction builder tool
pub struct TransactionTool {
    eth_rpc_url: String,
    sol_rpc_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionData {
    pub from: String,
    pub to: String,
    pub value: String,
    pub data: Option<String>,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub nonce: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolanaTransactionData {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub recent_blockhash: String,
}

impl TransactionTool {
    pub fn new(eth_rpc_url: String, sol_rpc_url: String) -> Self {
        Self {
            eth_rpc_url,
            sol_rpc_url,
        }
    }

    /// Build Ethereum transaction
    pub async fn build_eth_transaction(
        &self,
        from: &str,
        to: &str,
        value_eth: f64,
    ) -> Result<TransactionData> {
        console_log!("Building ETH transaction from {} to {} value {}", from, to, value_eth);
        
        // Get nonce
        let nonce = self.get_transaction_count(from).await?;
        
        // Get gas price
        let gas_price = self.get_gas_price().await?;
        
        // Convert ETH to wei
        let value_wei = (value_eth * 1e18) as u128;
        let value_hex = format!("0x{:x}", value_wei);
        
        // Estimate gas (21000 for simple transfer)
        let gas = "0x5208"; // 21000 in hex
        
        Ok(TransactionData {
            from: from.to_string(),
            to: to.to_string(),
            value: value_hex,
            data: None,
            gas: Some(gas.to_string()),
            gas_price: Some(gas_price),
            nonce: Some(nonce),
        })
    }

    /// Build ERC20 transfer transaction
    #[allow(dead_code)]
    pub async fn build_erc20_transaction(
        &self,
        from: &str,
        token_address: &str,
        to: &str,
        amount: u128,
    ) -> Result<TransactionData> {
        console_log!("Building ERC20 transaction from {} to {} amount {}", from, to, amount);
        
        // Get nonce
        let nonce = self.get_transaction_count(from).await?;
        
        // Get gas price
        let gas_price = self.get_gas_price().await?;
        
        // Build transfer(address,uint256) call data
        let function_sig = "0xa9059cbb"; // transfer function signature
        let padded_to = format!("{:0>64}", to.trim_start_matches("0x"));
        let padded_amount = format!("{:0>64x}", amount);
        let data = format!("{}{}{}", function_sig, padded_to, padded_amount);
        
        // Estimate gas for ERC20 transfer (typically ~65000)
        let gas = "0xfde8"; // 65000 in hex
        
        Ok(TransactionData {
            from: from.to_string(),
            to: token_address.to_string(),
            value: "0x0".to_string(),
            data: Some(data),
            gas: Some(gas.to_string()),
            gas_price: Some(gas_price),
            nonce: Some(nonce),
        })
    }

    /// Build Solana transaction
    pub async fn build_sol_transaction(
        &self,
        from: &str,
        to: &str,
        amount_sol: f64,
    ) -> Result<SolanaTransactionData> {
        console_log!("Building SOL transaction from {} to {} amount {}", from, to, amount_sol);
        
        // Get recent blockhash
        let blockhash = self.get_recent_blockhash().await?;
        
        // Convert SOL to lamports
        let amount_lamports = (amount_sol * 1e9) as u64;
        
        Ok(SolanaTransactionData {
            from: from.to_string(),
            to: to.to_string(),
            amount: amount_lamports,
            recent_blockhash: blockhash,
        })
    }

    /// Estimate gas for custom transaction
    #[allow(dead_code)]
    pub async fn estimate_gas(&self, tx_data: &TransactionData) -> Result<String> {
        console_log!("Estimating gas for transaction");
        
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_estimateGas",
            "params": [{
                "from": tx_data.from,
                "to": tx_data.to,
                "value": tx_data.value,
                "data": tx_data.data
            }],
            "id": 1
        });

        let response = self.rpc_call(&self.eth_rpc_url, request_body).await?;
        
        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            Ok(result.to_string())
        } else {
            Err(AloudError::AgentError("Failed to estimate gas".to_string()))
        }
    }

    /// Get transaction count (nonce)
    async fn get_transaction_count(&self, address: &str) -> Result<String> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [address, "latest"],
            "id": 1
        });

        let response = self.rpc_call(&self.eth_rpc_url, request_body).await?;
        
        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            Ok(result.to_string())
        } else {
            Err(AloudError::AgentError("Failed to get nonce".to_string()))
        }
    }

    /// Get current gas price
    async fn get_gas_price(&self) -> Result<String> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });

        let response = self.rpc_call(&self.eth_rpc_url, request_body).await?;
        
        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            Ok(result.to_string())
        } else {
            Err(AloudError::AgentError("Failed to get gas price".to_string()))
        }
    }

    /// Get recent blockhash for Solana
    async fn get_recent_blockhash(&self) -> Result<String> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "getRecentBlockhash",
            "params": [],
            "id": 1
        });

        let response = self.rpc_call(&self.sol_rpc_url, request_body).await?;
        
        if let Some(blockhash) = response
            .get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.get("blockhash"))
            .and_then(|b| b.as_str())
        {
            Ok(blockhash.to_string())
        } else {
            Err(AloudError::AgentError("Failed to get blockhash".to_string()))
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
