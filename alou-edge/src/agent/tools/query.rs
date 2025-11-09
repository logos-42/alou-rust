use crate::agent::context::AgentContext;
use crate::mcp::registry::McpTool;
use crate::utils::error::{AloudError, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use worker::{console_log, Fetch, Method, RequestInit};

/// Query tool for blockchain data
pub struct QueryTool {
    eth_rpc_url: String,
    eth_testnet_rpc_url: Option<String>,
    sol_rpc_url: String,
}

impl QueryTool {
    pub fn new(
        eth_rpc_url: String,
        eth_testnet_rpc_url: Option<String>,
        sol_rpc_url: String,
    ) -> Self {
        Self {
            eth_rpc_url,
            eth_testnet_rpc_url,
            sol_rpc_url,
        }
    }

    /// Query ETH balance
    pub async fn get_eth_balance(&self, address: &str, chain_hint: Option<&str>) -> Result<String> {
        console_log!("Querying ETH balance for {}", address);

        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [address, "latest"],
            "id": 1
        });

        let rpc_url = self.resolve_eth_rpc(chain_hint);
        let response = self.rpc_call(rpc_url, request_body).await?;

        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            // Convert hex to decimal (wei)
            let balance_wei = u128::from_str_radix(result.trim_start_matches("0x"), 16)
                .map_err(|e| AloudError::AgentError(format!("Parse balance error: {}", e)))?;

            // Convert wei to ETH
            let balance_eth = balance_wei as f64 / 1e18;
            Ok(format!("{:.6} ETH", balance_eth))
        } else {
            Err(AloudError::AgentError("Invalid response".to_string()))
        }
    }

    /// Query ERC20 token balance
    pub async fn get_erc20_balance(
        &self,
        token_address: &str,
        wallet_address: &str,
        chain_hint: Option<&str>,
    ) -> Result<String> {
        console_log!(
            "Querying ERC20 balance for token {} wallet {}",
            token_address,
            wallet_address
        );

        // ERC20 balanceOf(address) function signature
        let function_sig = "0x70a08231";
        let padded_address = format!("{:0>64}", wallet_address.trim_start_matches("0x"));
        let data = format!("{}{}", function_sig, padded_address);

        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": [{
                "to": token_address,
                "data": data
            }, "latest"],
            "id": 1
        });

        let rpc_url = self.resolve_eth_rpc(chain_hint);
        let response = self.rpc_call(rpc_url, request_body).await?;

        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            let balance = u128::from_str_radix(result.trim_start_matches("0x"), 16)
                .map_err(|e| AloudError::AgentError(format!("Parse balance error: {}", e)))?;
            Ok(balance.to_string())
        } else {
            Err(AloudError::AgentError("Invalid response".to_string()))
        }
    }

    /// Query SOL balance
    pub async fn get_sol_balance(&self, address: &str) -> Result<String> {
        console_log!("Querying SOL balance for {}", address);

        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "getBalance",
            "params": [address],
            "id": 1
        });

        let response = self.rpc_call(&self.sol_rpc_url, request_body).await?;

        if let Some(value) = response
            .get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_u64())
        {
            let balance_sol = value as f64 / 1e9; // lamports to SOL
            Ok(format!("{:.6} SOL", balance_sol))
        } else {
            Err(AloudError::AgentError("Invalid response".to_string()))
        }
    }

    /// Query transaction history (simplified)
    #[allow(dead_code)]
    pub async fn get_transaction_history(&self, address: &str, chain: &str) -> Result<Vec<Value>> {
        console_log!("Querying transaction history for {} on {}", address, chain);

        match chain.to_lowercase().as_str() {
            "eth" | "ethereum" => self.get_eth_transactions(address, None).await,
            "sol" | "solana" => self.get_sol_transactions(address).await,
            _ => Err(AloudError::InvalidInput(format!(
                "Unsupported chain: {}",
                chain
            ))),
        }
    }

    #[allow(dead_code)]
    async fn get_eth_transactions(
        &self,
        address: &str,
        chain_hint: Option<&str>,
    ) -> Result<Vec<Value>> {
        // Note: eth_getTransactionCount only gives count, not history
        // For full history, you'd need to use a block explorer API like Etherscan
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [address, "latest"],
            "id": 1
        });

        let rpc_url = self.resolve_eth_rpc(chain_hint);
        let response = self.rpc_call(rpc_url, request_body).await?;

        if let Some(count) = response.get("result").and_then(|r| r.as_str()) {
            Ok(vec![json!({
                "transaction_count": count,
                "note": "Use block explorer API for full transaction history"
            })])
        } else {
            Err(AloudError::AgentError("Invalid response".to_string()))
        }
    }

    #[allow(dead_code)]
    async fn get_sol_transactions(&self, address: &str) -> Result<Vec<Value>> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "getSignaturesForAddress",
            "params": [address, {"limit": 10}],
            "id": 1
        });

        let response = self.rpc_call(&self.sol_rpc_url, request_body).await?;

        if let Some(result) = response.get("result").and_then(|r| r.as_array()) {
            Ok(result.clone())
        } else {
            Err(AloudError::AgentError("Invalid response".to_string()))
        }
    }

    /// Generic RPC call helper
    async fn rpc_call(&self, url: &str, body: Value) -> Result<Value> {
        let body_str = serde_json::to_string(&body)
            .map_err(|e| AloudError::AgentError(format!("Serialize error: {}", e)))?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);

        let headers = worker::Headers::new();
        headers
            .set("Content-Type", "application/json")
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

    fn resolve_eth_rpc(&self, chain_hint: Option<&str>) -> &str {
        if let Some(hint) = chain_hint {
            if let Some(testnet_url) = self.eth_testnet_rpc_url.as_deref() {
                let normalized = hint.trim().to_lowercase();
                let is_testnet = normalized.contains("sepolia")
                    || normalized.contains("test")
                    || normalized.contains("devnet")
                    || normalized == "0xaa36a7";

                if is_testnet {
                    return testnet_url;
                }
            }
        }

        &self.eth_rpc_url
    }
}

#[async_trait(?Send)]
impl McpTool for QueryTool {
    fn name(&self) -> &str {
        "query_blockchain"
    }

    fn description(&self) -> &str {
        "Query blockchain data including ETH balance, ERC20 token balance, and Solana balance"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["eth_balance", "erc20_balance", "sol_balance"],
                    "description": "Type of query to perform"
                },
                "address": {
                    "type": "string",
                    "description": "Wallet address to query"
                },
                "token_address": {
                    "type": "string",
                    "description": "ERC20 token contract address (required for erc20_balance)"
                }
            },
            "required": ["action", "address"]
        })
    }

    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing action".to_string()))?;

        let address = args
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing address".to_string()))?;

        let chain_hint = context.chain.as_deref();

        match action {
            "eth_balance" => {
                let balance = self.get_eth_balance(address, chain_hint).await?;
                Ok(json!({ "balance": balance }))
            }
            "erc20_balance" => {
                let token_address = args
                    .get("token_address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing token_address".to_string()))?;

                let balance = self
                    .get_erc20_balance(token_address, address, chain_hint)
                    .await?;
                Ok(json!({ "balance": balance }))
            }
            "sol_balance" => {
                let balance = self.get_sol_balance(address).await?;
                Ok(json!({ "balance": balance }))
            }
            _ => Err(AloudError::InvalidInput(format!(
                "Unknown action: {}",
                action
            ))),
        }
    }
}
