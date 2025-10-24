use serde_json::{json, Value};
use worker::console_log;
use crate::agent::ai_client::{AiClient, AiMessage, AiTool};
use crate::agent::tools::{QueryTool, TransactionTool, BroadcastTool};
use crate::utils::error::{AloudError, Result};

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
        let query_tool = QueryTool::new(eth_rpc_url.clone(), sol_rpc_url.clone());
        let transaction_tool = TransactionTool::new(eth_rpc_url.clone(), sol_rpc_url.clone());
        let broadcast_tool = BroadcastTool::new(eth_rpc_url, sol_rpc_url);
        
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
            AiMessage {
                role: "system".to_string(),
                content: "You are a blockchain assistant. You can help users query balances, build transactions, and broadcast transactions on Ethereum and Solana networks.".to_string(),
            },
            AiMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ];
        
        let tools = self.get_available_tools();
        
        let response = self.ai_client.send_message(messages, Some(tools)).await?;
        
        // Process tool calls if any
        if !response.tool_calls.is_empty() {
            let mut results = Vec::new();
            
            for tool_call in &response.tool_calls {
                console_log!("Executing tool: {}", tool_call.name);
                let result = self.execute_tool(&tool_call.name, &tool_call.arguments).await?;
                results.push(result);
            }
            
            // Return combined results
            Ok(format!("{}\n\nTool Results:\n{}", response.content, results.join("\n")))
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
                        "wallet_address": {
                            "type": "string",
                            "description": "Wallet address to check"
                        }
                    },
                    "required": ["token_address", "wallet_address"]
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
                
                let balance = self.query_tool.get_eth_balance(address).await?;
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
                let token_address = arguments
                    .get("token_address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing token_address".to_string()))?;
                
                let wallet_address = arguments
                    .get("wallet_address")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing wallet_address".to_string()))?;
                
                let balance = self.query_tool.get_erc20_balance(token_address, wallet_address).await?;
                Ok(format!("Token Balance: {}", balance))
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
                
                let tx_data = self.transaction_tool.build_eth_transaction(from, to, value_eth).await?;
                Ok(format!("Transaction built: {:?}", tx_data))
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
                
                let confirmed = self.broadcast_tool.is_transaction_confirmed(tx_hash, chain).await?;
                Ok(format!("Transaction confirmed: {}", confirmed))
            }
            _ => Err(AloudError::InvalidInput(format!("Unknown tool: {}", tool_name))),
        }
    }
}
