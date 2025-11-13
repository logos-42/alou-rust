use crate::agent::ai_client::{AiClient, AiMessage, AiTool};
use crate::agent::tools::{BroadcastTool, QueryTool, TransactionTool};
use crate::utils::error::{AloudError, Result};
use crate::web3::tokens::{
    all_tokens, find_token, find_token_by_symbol, normalize_chain_identifier, supported_symbols,
    tokens_for_chain, TokenMetadata,
};
use serde_json::{json, Value};
use std::borrow::Cow;
use worker::console_log;

/// Blockchain Agent that combines AI and blockchain tools
#[allow(dead_code)]
pub struct BlockchainAgent {
    ai_client: AiClient,
    query_tool: QueryTool,
    transaction_tool: TransactionTool,
    broadcast_tool: BroadcastTool,
}

impl BlockchainAgent {
    /// Create new blockchain agent
    #[allow(dead_code)]
    pub fn new(
        provider: &str,
        api_key: String,
        model: Option<String>,
        eth_rpc_url: String,
        sol_rpc_url: String,
    ) -> Result<Self> {
        let ai_client = AiClient::new(provider, api_key, model)?;
        let query_tool = QueryTool::new(eth_rpc_url.clone(), None, sol_rpc_url.clone());
        let transaction_tool = TransactionTool::new(eth_rpc_url.clone(), None, sol_rpc_url.clone());
        let broadcast_tool = BroadcastTool::new(eth_rpc_url, None, sol_rpc_url);

        Ok(Self {
            ai_client,
            query_tool,
            transaction_tool,
            broadcast_tool,
        })
    }

    /// Process user message with blockchain tools
    #[allow(dead_code)]
    pub async fn process_message(&self, user_message: &str) -> Result<String> {
        console_log!("Processing message: {}", user_message);

        let messages = vec![
            AiMessage::text("system", "You are a blockchain assistant. You can help users query balances, build transactions, and broadcast transactions on Ethereum and Solana networks.".to_string()),
            AiMessage::text("user", user_message.to_string()),
        ];

        let tools = self.get_available_tools();

        let response = self.ai_client.send_message(messages, Some(tools)).await?;

        // Process tool calls if any
        if !response.tool_calls.is_empty() {
            let mut results = Vec::new();

            for tool_call in &response.tool_calls {
                console_log!("Executing tool: {}", tool_call.name);
                let result = self
                    .execute_tool(&tool_call.name, &tool_call.arguments)
                    .await?;
                results.push(result);
            }

            // Return combined results
            Ok(format!(
                "{}\n\nTool Results:\n{}",
                response.content,
                results.join("\n")
            ))
        } else {
            Ok(response.content)
        }
    }

    /// Get available blockchain tools
    #[allow(dead_code)]
    fn get_available_tools(&self) -> Vec<AiTool> {
        vec![
            AiTool {
                name: "get_eth_balance".to_string(),
                description: "Query Ethereum balance for an address".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "string",
                            "description": "Ethereum address (0x...)"
                        }
                    },
                    "required": ["address"]
                }),
            },
            AiTool {
                name: "get_sol_balance".to_string(),
                description: "Query Solana balance for an address".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "address": {
                            "type": "string",
                            "description": "Solana address"
                        }
                    },
                    "required": ["address"]
                }),
            },
            AiTool {
                name: "get_erc20_balance".to_string(),
                description: "Query ERC20 token balance".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "token_address": {
                            "type": "string",
                            "description": "ERC20 token contract address"
                        },
                        "token_symbol": {
                            "type": "string",
                            "description": "ERC20 token symbol (optional if token_address is provided)"
                        },
                        "chain": {
                            "type": "string",
                            "description": "Chain identifier (e.g. eth_sepolia, base_sepolia) used when resolving symbols"
                        },
                        "wallet_address": {
                            "type": "string",
                            "description": "Wallet address to check"
                        }
                    },
                    "required": ["wallet_address"],
                    "anyOf": [
                        { "required": ["token_address"] },
                        { "required": ["token_symbol"] }
                    ]
                }),
            },
            AiTool {
                name: "list_supported_tokens".to_string(),
                description: "List supported stablecoins / ERC20 tokens for a given chain".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "chain": {
                            "type": "string",
                            "description": "Chain identifier (optional). If omitted, all supported tokens are returned."
                        }
                    }
                }),
            },
            AiTool {
                name: "build_eth_transaction".to_string(),
                description: "Build an Ethereum transaction (not signed)".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "from": {
                            "type": "string",
                            "description": "Sender address"
                        },
                        "to": {
                            "type": "string",
                            "description": "Recipient address"
                        },
                        "value_eth": {
                            "type": "number",
                            "description": "Amount in ETH"
                        }
                    },
                    "required": ["from", "to", "value_eth"]
                }),
            },
            AiTool {
                name: "build_erc20_transaction".to_string(),
                description: "Build an ERC20 transfer transaction (not signed)".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "from": {
                            "type": "string",
                            "description": "Sender address"
                        },
                        "to": {
                            "type": "string",
                            "description": "Recipient address"
                        },
                        "amount": {
                            "type": ["number", "string"],
                            "description": "Transfer amount in human-readable units (e.g. 25.5)"
                        },
                        "token_address": {
                            "type": "string",
                            "description": "ERC20 token contract address (required if token_symbol not provided)"
                        },
                        "token_symbol": {
                            "type": "string",
                            "description": "ERC20 token symbol (optional alternative to token_address)"
                        },
                        "chain": {
                            "type": "string",
                            "description": "Chain identifier (optional)"
                        }
                    },
                    "required": ["from", "to"],
                    "anyOf": [
                        { "required": ["token_address"] },
                        { "required": ["token_symbol"] }
                    ],
                    "oneOf": [
                        { "required": ["amount"] }
                    ]
                }),
            },
            AiTool {
                name: "get_transaction_status".to_string(),
                description: "Check transaction status".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "tx_hash": {
                            "type": "string",
                            "description": "Transaction hash"
                        },
                        "chain": {
                            "type": "string",
                            "description": "Chain name (eth or sol)"
                        }
                    },
                    "required": ["tx_hash", "chain"]
                }),
            },
        ]
    }

    /// Execute a tool call
    #[allow(dead_code)]
    async fn execute_tool(&self, tool_name: &str, arguments: &Value) -> Result<String> {
        match tool_name {
            "get_eth_balance" => {
                let address = arguments
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing address".to_string()))?;

                let balance = self.query_tool.get_eth_balance(address, None).await?;
                Ok(format!("ETH Balance: {}", balance))
            }
            "get_sol_balance" => {
                let address = arguments
                    .get("address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing address".to_string()))?;

                let balance = self.query_tool.get_sol_balance(address).await?;
                Ok(format!("SOL Balance: {}", balance))
            }
            "get_erc20_balance" => {
                let chain_hint = arguments.get("chain").and_then(|v| v.as_str());

                let normalized_chain = chain_hint.map(normalize_chain_identifier);
                let (token_address_cow, metadata): (Cow<'_, str>, Option<&TokenMetadata>) =
                    if let Some(address) = arguments
                        .get("token_address")
                        .and_then(|v| v.as_str())
                    {
                        (
                            Cow::Owned(address.to_string()),
                            find_token(normalized_chain.as_deref(), address),
                        )
                    } else if let Some(symbol) = arguments
                        .get("token_symbol")
                        .and_then(|v| v.as_str())
                    {
                        let token = find_token_by_symbol(chain_hint, symbol).ok_or_else(|| {
                            AloudError::InvalidInput(format!(
                                "Unsupported token symbol '{}' for chain {:?}",
                                symbol, chain_hint
                            ))
                        })?;
                        (Cow::Borrowed(token.address), Some(token))
                    } else {
                        return Err(AloudError::InvalidInput(
                            "Missing token_address or token_symbol".to_string(),
                        ));
                    };

                let wallet_address = arguments
                    .get("wallet_address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AloudError::InvalidInput("Missing wallet_address".to_string())
                    })?;

                let chain_hint_effective: Option<&str> = match metadata {
                    Some(token) => Some(token.chain),
                    None => chain_hint,
                };

                let balance = self
                    .query_tool
                    .get_erc20_balance(token_address_cow.as_ref(), wallet_address, chain_hint_effective)
                    .await?;

                let symbol = balance
                    .symbol
                    .clone()
                    .or_else(|| metadata.as_ref().map(|token| token.symbol.to_string()))
                    .unwrap_or_else(|| "TOKEN".to_string());
                let display_balance = balance
                    .normalized_balance
                    .clone()
                    .unwrap_or_else(|| balance.balance.clone());

                Ok(format!(
                    "{} Balance: {}",
                    symbol,
                    display_balance
                ))
            }
            "list_supported_tokens" => {
                let chain = arguments.get("chain").and_then(|v| v.as_str());
                let tokens: Vec<_> = chain
                    .map(tokens_for_chain)
                    .unwrap_or_else(|| all_tokens().iter().collect());

                if tokens.is_empty() {
                    return Ok("当前链暂未配置稳定币".to_string());
                }

                let descriptions: Vec<String> = tokens
                    .into_iter()
                    .map(|token| {
                        format!(
                            "{} ({}) - 合约地址 {}{}",
                            token.symbol,
                            token.name,
                            token.address,
                            if token.is_testnet { " [testnet]" } else { "" }
                        )
                    })
                    .collect();

                let symbol_list = supported_symbols(chain)
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ");

                Ok(format!(
                    "可用稳定币：{}\n详情：{}",
                    symbol_list,
                    descriptions.join("; ")
                ))
            }
            "build_eth_transaction" => {
                let from = arguments
                    .get("from")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing from".to_string()))?;

                let to = arguments
                    .get("to")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing to".to_string()))?;

                let value_eth = arguments
                    .get("value_eth")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| AloudError::InvalidInput("Missing value_eth".to_string()))?;

                let tx_data = self
                    .transaction_tool
                    .build_eth_transaction(from, to, value_eth, None)
                    .await?;
                Ok(format!("Transaction built: {:?}", tx_data))
            }
            "build_erc20_transaction" => {
                let from = arguments
                    .get("from")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing from".to_string()))?;

                let to = arguments
                    .get("to")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing to".to_string()))?;

                let chain_hint = arguments.get("chain").and_then(|v| v.as_str());

                let (token_address, metadata) = if let Some(address) = arguments
                    .get("token_address")
                    .and_then(|v| v.as_str())
                {
                    let normalized = chain_hint.map(normalize_chain_identifier);
                    let metadata = find_token(normalized.as_deref(), address);
                    (address.to_string(), metadata)
                } else if let Some(symbol) = arguments
                    .get("token_symbol")
                    .and_then(|v| v.as_str())
                {
                    let token = find_token_by_symbol(chain_hint, symbol).ok_or_else(|| {
                        AloudError::InvalidInput(format!(
                            "Unsupported token symbol '{}' on chain {:?}",
                            symbol, chain_hint
                        ))
                    })?;
                    (token.address.to_string(), Some(token))
                } else {
                    return Err(AloudError::InvalidInput(
                        "Missing token_address or token_symbol".to_string(),
                    ));
                };

                let decimals = arguments
                    .get("decimals")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u8)
                    .or_else(|| metadata.map(|token| token.decimals))
                    .ok_or_else(|| {
                        AloudError::InvalidInput("Missing ERC20 token decimals".to_string())
                    })?;

                let amount_value = arguments
                    .get("amount")
                    .or_else(|| arguments.get("value"))
                    .ok_or_else(|| {
                        AloudError::InvalidInput("Missing transfer amount (amount/value)".to_string())
                    })?;

                let amount = parse_amount_to_u128(amount_value, decimals)?;

                let chain_for_hint = metadata
                    .map(|token| token.chain)
                    .or(chain_hint);

                let tx_data = self
                    .transaction_tool
                    .build_erc20_transaction(
                        from,
                        token_address.as_str(),
                        to,
                        amount,
                        chain_for_hint,
                    )
                    .await?;

                let symbol = metadata
                    .map(|token| token.symbol.to_string())
                    .or_else(|| {
                        arguments
                            .get("token_symbol")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| "TOKEN".to_string());

                let human_amount = format_token_amount(amount, decimals);

                Ok(format!(
                    "ERC20 Transaction built: 向 {} 转账 {} {} (nonce {:?})",
                    to,
                    human_amount,
                    symbol,
                    tx_data.nonce
                ))
            }
            "get_transaction_status" => {
                let tx_hash = arguments
                    .get("tx_hash")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing tx_hash".to_string()))?;

                let chain = arguments
                    .get("chain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing chain".to_string()))?;

                let confirmed = self
                    .broadcast_tool
                    .is_transaction_confirmed(tx_hash, chain)
                    .await?;
                Ok(format!("Transaction confirmed: {}", confirmed))
            }
            _ => Err(AloudError::InvalidInput(format!(
                "Unknown tool: {}",
                tool_name
            ))),
        }
    }
}

fn parse_amount_to_u128(value: &Value, decimals: u8) -> Result<u128> {
    let amount_str = if let Some(str_value) = value.as_str() {
        str_value.trim()
    } else if let Some(num) = value.as_f64() {
        return decimal_to_u128(&format!("{:.prec$}", num, prec = decimals as usize), decimals);
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

    let mut remainder_str = format!(
        "{:0>width$}",
        remainder,
        width = decimals as usize
    );

    while remainder_str.ends_with('0') {
        remainder_str.pop();
    }

    format!("{}.{}", whole, remainder_str)
}
