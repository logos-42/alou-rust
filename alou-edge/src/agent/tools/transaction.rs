use crate::agent::context::AgentContext;
use crate::mcp::registry::McpTool;
use crate::utils::error::{AloudError, Result};
use crate::web3::tokens::{
    find_token, find_token_by_symbol, normalize_chain_identifier, TokenMetadata,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::{console_log, Fetch, Method, RequestInit};

/// Transaction builder tool
pub struct TransactionTool {
    eth_rpc_url: String,
    eth_testnet_rpc_url: Option<String>,
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

    /// Build Ethereum transaction
    pub async fn build_eth_transaction(
        &self,
        from: &str,
        to: &str,
        value_eth: f64,
        chain_hint: Option<&str>,
    ) -> Result<TransactionData> {
        console_log!(
            "Building ETH transaction from {} to {} value {}",
            from,
            to,
            value_eth
        );

        // Get nonce
        let nonce = self.get_transaction_count(from, chain_hint).await?;

        // Get gas price
        let gas_price = self.get_gas_price(chain_hint).await?;

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
        chain_hint: Option<&str>,
    ) -> Result<TransactionData> {
        console_log!(
            "Building ERC20 transaction from {} to {} amount {}",
            from,
            to,
            amount
        );

        // Get nonce
        let nonce = self.get_transaction_count(from, chain_hint).await?;

        // Get gas price
        let gas_price = self.get_gas_price(chain_hint).await?;

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
        console_log!(
            "Building SOL transaction from {} to {} amount {}",
            from,
            to,
            amount_sol
        );

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

        let rpc_url = self.resolve_eth_rpc(None);
        let response = self.rpc_call(rpc_url, request_body).await?;

        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            Ok(result.to_string())
        } else {
            Err(AloudError::AgentError("Failed to estimate gas".to_string()))
        }
    }

    /// Get transaction count (nonce)
    async fn get_transaction_count(
        &self,
        address: &str,
        chain_hint: Option<&str>,
    ) -> Result<String> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [address, "latest"],
            "id": 1
        });

        let rpc_url = self.resolve_eth_rpc(chain_hint);
        let response = self.rpc_call(rpc_url, request_body).await?;

        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            Ok(result.to_string())
        } else {
            Err(AloudError::AgentError("Failed to get nonce".to_string()))
        }
    }

    /// Get current gas price
    async fn get_gas_price(&self, chain_hint: Option<&str>) -> Result<String> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });

        let rpc_url = self.resolve_eth_rpc(chain_hint);
        let response = self.rpc_call(rpc_url, request_body).await?;

        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            Ok(result.to_string())
        } else {
            Err(AloudError::AgentError(
                "Failed to get gas price".to_string(),
            ))
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
            Err(AloudError::AgentError(
                "Failed to get blockhash".to_string(),
            ))
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

    async fn build_erc20_transfer(
        &self,
        from: &str,
        to: &str,
        args: &Value,
        context_chain_hint: Option<&str>,
    ) -> Result<Value> {
        let chain_arg = args.get("chain").and_then(|v| v.as_str());

        let normalized_chain = chain_arg.map(normalize_chain_identifier);
        let (token_address, metadata): (String, Option<&TokenMetadata>) =
            if let Some(address) = args.get("token_address").and_then(|v| v.as_str()) {
                (
                    address.to_string(),
                    find_token(normalized_chain.as_deref(), address),
                )
            } else if let Some(symbol) = args.get("token_symbol").and_then(|v| v.as_str()) {
                let token = find_token_by_symbol(chain_arg, symbol).ok_or_else(|| {
                    AloudError::InvalidInput(format!(
                        "Unsupported token symbol '{}' on chain {:?}",
                        symbol, chain_arg
                    ))
                })?;
                (token.address.to_string(), Some(token))
            } else {
                return Err(AloudError::InvalidInput(
                    "Missing token_address or token_symbol".to_string(),
                ));
            };

        let decimals = args
            .get("decimals")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .or_else(|| metadata.map(|token| token.decimals))
            .ok_or_else(|| AloudError::InvalidInput("Missing ERC20 token decimals".to_string()))?;

        let amount_value = args
            .get("amount")
            .or_else(|| args.get("value"))
            .ok_or_else(|| {
                AloudError::InvalidInput("Missing transfer amount (amount/value)".to_string())
            })?;

        let amount = parse_decimal_amount(amount_value, decimals)?;

        let metadata_chain = metadata.map(|token| token.chain);
        let effective_chain_hint = metadata_chain.or(chain_arg).or(context_chain_hint);

        let tx_data = self
            .build_erc20_transaction(
                from,
                token_address.as_str(),
                to,
                amount,
                effective_chain_hint,
            )
            .await?;

        let wallet_request = json!({
            "from": tx_data.from.clone(),
            "to": tx_data.to.clone(),
            "value": tx_data.value.clone(),
            "gas": tx_data.gas.clone().unwrap_or_else(|| "0xfde8".to_string()),
            "gasPrice": tx_data.gas_price.clone().unwrap_or_else(|| "0x0".to_string()),
            "nonce": tx_data.nonce.clone().unwrap_or_default(),
            "data": tx_data.data.clone().unwrap_or_else(|| "0x".to_string())
        });

        let symbol = metadata
            .map(|token| token.symbol.to_string())
            .or_else(|| {
                args.get("token_symbol")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "TOKEN".to_string());

        Ok(json!({
            "success": true,
            "chain": "eth",
            "token_address": token_address,
            "token_symbol": symbol,
            "decimals": decimals,
            "raw_amount": amount.to_string(),
            "amount": format_token_amount(amount, decimals),
            "transaction": tx_data,
            "transaction_request": wallet_request.clone(),
            "summary": format!("向 {} 转账 {} {}", to, format_token_amount(amount, decimals), symbol),
            "instruction": {
                "type": "wallet_operation",
                "method": "eth_sendTransaction",
                "params": [wallet_request]
            }
        }))
    }
}

#[async_trait(?Send)]
impl McpTool for TransactionTool {
    fn name(&self) -> &str {
        "build_transaction"
    }

    fn description(&self) -> &str {
        "Build blockchain transactions for Ethereum and Solana networks"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "chain": {
                    "type": "string",
                    "enum": ["eth", "sol"],
                    "description": "Blockchain network (default eth)"
                },
                "from": {
                    "type": "string",
                    "description": "Sender address"
                },
                "to": {
                    "type": "string",
                    "description": "Recipient address"
                },
                "value": {
                    "type": ["number", "string"],
                    "description": "Amount to send for native transfers (ETH or SOL)"
                },
                "amount": {
                    "type": ["number", "string"],
                    "description": "Amount for ERC20 transfers (human readable)"
                },
                "token_address": {
                    "type": "string",
                    "description": "ERC20 token contract address"
                },
                "token_symbol": {
                    "type": "string",
                    "description": "ERC20 token symbol (optional alternative to token_address)"
                },
                "decimals": {
                    "type": "integer",
                    "description": "ERC20 token decimals (optional, inferred if known)"
                }
            },
            "required": ["from", "to"],
            "anyOf": [
                { "required": ["value"] },
                { "required": ["amount"] }
            ],
            "allOf": [
                {
                    "if": { "properties": { "chain": { "const": "sol" } } },
                    "then": { "required": ["value"] }
                },
                {
                    "if": { "properties": { "token_address": { "type": "string" } } },
                    "then": { "required": ["amount"] }
                },
                {
                    "if": { "properties": { "token_symbol": { "type": "string" } } },
                    "then": { "required": ["amount"] }
                }
            ]
        })
    }

    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value> {
        let chain = args.get("chain").and_then(|v| v.as_str()).unwrap_or("eth");

        let from = args
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing from".to_string()))?;

        let to = args
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing to".to_string()))?;

        let chain_hint = context.chain.as_deref();

        match chain {
            "eth" => {
                let wants_erc20 =
                    args.get("token_address").is_some() || args.get("token_symbol").is_some();

                if wants_erc20 {
                    self.build_erc20_transfer(from, to, &args, chain_hint).await
                } else {
                    let value = args.get("value").and_then(|v| v.as_f64()).ok_or_else(|| {
                        AloudError::InvalidInput("Missing value for ETH transfer".to_string())
                    })?;

                    let tx_data = self
                        .build_eth_transaction(from, to, value, chain_hint)
                        .await?;

                    let wallet_request = json!({
                        "from": tx_data.from.clone(),
                        "to": tx_data.to.clone(),
                        "value": tx_data.value.clone(),
                        "gas": tx_data.gas.clone().unwrap_or_else(|| "0x5208".to_string()),
                        "gasPrice": tx_data.gas_price.clone().unwrap_or_else(|| "0x0".to_string()),
                        "nonce": tx_data.nonce.clone().unwrap_or_default(),
                        "data": tx_data.data.clone().unwrap_or_else(|| "0x".to_string())
                    });

                    Ok(json!({
                        "success": true,
                        "chain": "eth",
                        "summary": format!("向 {} 转账 {:.6} ETH", to, value),
                        "transaction": tx_data,
                        "transaction_request": wallet_request.clone(),
                        "instruction": {
                            "type": "wallet_operation",
                            "method": "eth_sendTransaction",
                            "params": [wallet_request]
                        }
                    }))
                }
            }
            "sol" => {
                let value = args.get("value").and_then(|v| v.as_f64()).ok_or_else(|| {
                    AloudError::InvalidInput("Missing value for SOL transfer".to_string())
                })?;

                let tx_data = self.build_sol_transaction(from, to, value).await?;
                let lamports: u64 = tx_data.amount;
                let sol_amount = lamports as f64 / 1e9;

                let instruction = json!({
                    "type": "solana_transfer",
                    "params": {
                        "from": tx_data.from.clone(),
                        "to": tx_data.to.clone(),
                        "lamports": lamports,
                        "recentBlockhash": tx_data.recent_blockhash.clone()
                    }
                });

                Ok(json!({
                    "success": true,
                    "chain": "sol",
                    "summary": format!("向 {} 转账 {:.6} SOL", to, sol_amount),
                    "transaction": tx_data,
                    "instruction": instruction
                }))
            }
            _ => Err(AloudError::InvalidInput(format!(
                "Unsupported chain: {}",
                chain
            ))),
        }
    }
}

fn parse_decimal_amount(value: &Value, decimals: u8) -> Result<u128> {
    let amount_str = if let Some(str_value) = value.as_str() {
        str_value.trim()
    } else if let Some(num) = value.as_f64() {
        return decimal_to_u128(
            &format!("{:.prec$}", num, prec = decimals as usize),
            decimals,
        );
    } else if let Some(num) = value.as_u64() {
        return decimal_to_u128(&num.to_string(), decimals);
    } else if let Some(num) = value.as_i64() {
        if num < 0 {
            return Err(AloudError::InvalidInput(
                "Amount cannot be negative".to_string(),
            ));
        }
        return decimal_to_u128(&num.to_string(), decimals);
    } else {
        return Err(AloudError::InvalidInput(
            "Unsupported amount format".to_string(),
        ));
    };

    decimal_to_u128(amount_str, decimals)
}

fn decimal_to_u128(amount: &str, decimals: u8) -> Result<u128> {
    let trimmed = amount.trim();
    if trimmed.is_empty() {
        return Err(AloudError::InvalidInput(
            "Amount cannot be empty".to_string(),
        ));
    }

    if trimmed.starts_with('-') {
        return Err(AloudError::InvalidInput(
            "Amount cannot be negative".to_string(),
        ));
    }

    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() > 2 {
        return Err(AloudError::InvalidInput(
            "Invalid decimal format for amount".to_string(),
        ));
    }

    let whole_part = parts[0];
    let fractional_part = if parts.len() == 2 { parts[1] } else { "" };

    if fractional_part.len() > decimals as usize {
        return Err(AloudError::InvalidInput(format!(
            "Amount has more than {} decimal places",
            decimals
        )));
    }

    let scaling_factor = 10u128.pow(decimals as u32);
    let whole = if whole_part.is_empty() {
        0u128
    } else {
        whole_part
            .parse::<u128>()
            .map_err(|_| AloudError::InvalidInput("Invalid whole number".to_string()))?
    };

    let mut fractional = if fractional_part.is_empty() {
        0u128
    } else {
        fractional_part
            .parse::<u128>()
            .map_err(|_| AloudError::InvalidInput("Invalid fractional number".to_string()))?
    };

    let fractional_digits = fractional_part.len();
    if fractional_digits > 0 {
        let padding = decimals as usize - fractional_digits;
        fractional *= 10u128.pow(padding as u32);
    }

    whole
        .checked_mul(scaling_factor)
        .and_then(|whole_scaled| whole_scaled.checked_add(fractional))
        .ok_or_else(|| AloudError::InvalidInput("Amount overflow".to_string()))
}

fn format_token_amount(value: u128, decimals: u8) -> String {
    if decimals == 0 {
        return value.to_string();
    }

    let scaling_factor = 10u128
        .checked_pow(decimals as u32)
        .unwrap_or_else(|| 10u128.pow(18));

    let whole = value / scaling_factor;
    let remainder = value % scaling_factor;

    if remainder == 0 {
        return whole.to_string();
    }

    let mut remainder_str = format!("{:0>width$}", remainder, width = decimals as usize);

    while remainder_str.ends_with('0') {
        remainder_str.pop();
    }

    format!("{}.{}", whole, remainder_str)
}
