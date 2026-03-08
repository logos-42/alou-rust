use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolMetadata, ExecutionContext, ToolCategory, ToolPriority, ToolStatus};
use async_trait::async_trait;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct QueryBlockchainTool {
    metadata: ToolMetadata,
}

impl QueryBlockchainTool {
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "query_blockchain".to_string(),
            name: "query_blockchain".to_string(),
            description: "Query blockchain data including account balances, transaction history, contract state, block information, and more.".to_string(),
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

        Self { metadata }
    }

    fn get_supported_methods(&self) -> Value {
        json!({
            "eth_getBalance": {
                "description": "Get account balance",
                "params": ["address", "block"]
            },
            "eth_getTransactionCount": {
                "description": "Get transaction count (nonce)",
                "params": ["address", "block"]
            },
            "eth_getCode": {
                "description": "Get contract code at address",
                "params": ["address", "block"]
            },
            "eth_call": {
                "description": "Execute contract read call",
                "params": ["to", "data", "block"]
            },
            "eth_getLogs": {
                "description": "Get event logs",
                "params": ["address", "topics", "fromBlock", "toBlock"]
            },
            "eth_blockNumber": {
                "description": "Get current block number",
                "params": []
            },
            "eth_getBlockByNumber": {
                "description": "Get block by number",
                "params": ["block", "transactions"]
            },
            "eth_getTransactionByHash": {
                "description": "Get transaction by hash",
                "params": ["hash"]
            },
            "eth_getTransactionReceipt": {
                "description": "Get transaction receipt",
                "params": ["hash"]
            }
        })
    }
}

impl Default for QueryBlockchainTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for QueryBlockchainTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start = std::time::Instant::now();
        
        let method = args.get("method").and_then(|v| v.as_str()).ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'method' field".to_string())
        })?;

        let params = args.get("params").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let chain = args.get("chain").and_then(|v| v.as_str()).unwrap_or("ethereum");

        let result: Result<serde_json::Value, ToolError> = match method {
            "eth_blockNumber" => {
                Ok(json!({
                    "success": true,
                    "method": method,
                    "result": "0x10d4d1e",
                    "note": "This is a mock response. In production, connect to an RPC endpoint."
                }))
            }

            "eth_getBalance" => {
                let _address = params.first().and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("Missing address parameter".to_string())
                })?;

                Ok(json!({
                    "success": true,
                    "method": method,
                    "params": params,
                    "chain": chain,
                    "result": "0x0",
                    "note": "This is a mock response. Connect to RPC for real balance."
                }))
            }

            _ => {
                Ok(json!({
                    "success": true,
                    "method": method,
                    "params": params,
                    "chain": chain,
                    "result": null,
                    "supported_methods": self.get_supported_methods(),
                    "note": "This tool provides RPC method templates. Connect to an actual RPC endpoint for real data."
                }))
            }
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

        if !args.get("method").is_some() {
            return Err(ToolError::InvalidArguments("Missing 'method' field".to_string()));
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Query Blockchain Tool - Query blockchain data

Methods:
- eth_blockNumber: Get current block number
- eth_getBalance: Get account balance
- eth_getTransactionCount: Get transaction count
- eth_getCode: Get contract code
- eth_call: Execute contract read call
- eth_getLogs: Get event logs
- eth_getBlockByNumber: Get block by number
- eth_getTransactionByHash: Get transaction by hash
- eth_getTransactionReceipt: Get transaction receipt

Parameters:
- method: RPC method name (required)
- params: Array of parameters for the method
- chain: Blockchain network (optional, default: ethereum)

Example:
  {"method": "eth_blockNumber"}
  {"method": "eth_getBalance", "params": ["0x...", "latest"], "chain": "ethereum"}"#.to_string()
    }
}
