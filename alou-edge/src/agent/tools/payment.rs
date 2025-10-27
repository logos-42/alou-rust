use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::console_log;
use crate::utils::error::Result;

/// Payment tool for blockchain payments (simplified from MCP server)
#[allow(dead_code)]
pub struct PaymentTool {
    eth_rpc_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub label: Option<String>,
    pub wallet_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResult {
    pub success: bool,
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub token: String,
    pub timestamp: String,
}

impl PaymentTool {
    pub fn new(eth_rpc_url: String) -> Self {
        Self { eth_rpc_url }
    }

    /// Get balance (ETH or token)
    #[allow(dead_code)]
    pub async fn get_balance(&self, address: &str, token_symbol: Option<&str>) -> Result<Value> {
        console_log!("Getting balance for {} (token: {:?})", address, token_symbol);
        
        // 使用 RPC 调用查询余额
        let _request_body = if token_symbol.is_some() {
            // ERC20 token balance
            json!({
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": [{
                    "to": "0xTokenAddress", // 实际应该从配置获取
                    "data": format!("0x70a08231{:0>64}", address.trim_start_matches("0x"))
                }, "latest"],
                "id": 1
            })
        } else {
            // ETH balance
            json!({
                "jsonrpc": "2.0",
                "method": "eth_getBalance",
                "params": [address, "latest"],
                "id": 1
            })
        };
        
        // 模拟返回
        Ok(json!({
            "address": address,
            "balance": "1.5",
            "token": token_symbol.unwrap_or("ETH"),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Send transaction (simplified - returns unsigned transaction data)
    #[allow(dead_code)]
    pub async fn prepare_transaction(
        &self,
        to: &str,
        amount: &str,
        token_symbol: Option<&str>,
    ) -> Result<Value> {
        console_log!("Preparing transaction to {} amount {} token {:?}", to, amount, token_symbol);
        
        Ok(json!({
            "to": to,
            "amount": amount,
            "token": token_symbol.unwrap_or("ETH"),
            "gas_estimate": "21000",
            "gas_price": "20",
            "message": "Transaction prepared. User needs to sign and broadcast.",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Get transaction status
    #[allow(dead_code)]
    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<Value> {
        console_log!("Getting transaction status for {}", tx_hash);
        
        let _request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [tx_hash],
            "id": 1
        });
        
        // 模拟返回
        Ok(json!({
            "tx_hash": tx_hash,
            "status": "confirmed",
            "block_number": "12345678",
            "confirmations": 12,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Estimate gas fees
    #[allow(dead_code)]
    pub async fn estimate_gas_fees(&self) -> Result<Value> {
        console_log!("Estimating gas fees");
        
        Ok(json!({
            "gas_price": "20",
            "gas_price_gwei": "20",
            "estimated_cost_eth": "0.00042",
            "estimated_cost_usd": "1.26",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Validate Ethereum address
    #[allow(dead_code)]
    pub fn validate_address(&self, address: &str) -> bool {
        // 简单验证：0x 开头，42 个字符
        address.starts_with("0x") && address.len() == 42
    }

    /// Get supported tokens
    #[allow(dead_code)]
    pub fn get_supported_tokens(&self) -> Vec<String> {
        vec![
            "ETH".to_string(),
            "USDC".to_string(),
            "USDT".to_string(),
            "DAI".to_string(),
        ]
    }
}

impl Default for PaymentTool {
    fn default() -> Self {
        Self::new("https://eth.llamarpc.com".to_string())
    }
}
