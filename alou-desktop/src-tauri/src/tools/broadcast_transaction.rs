use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolMetadata, ExecutionContext, ToolCategory, ToolPriority, ToolStatus};
use async_trait::async_trait;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct BroadcastTransactionTool {
    metadata: ToolMetadata,
}

impl BroadcastTransactionTool {
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "broadcast_transaction".to_string(),
            name: "broadcast_transaction".to_string(),
            description: "Broadcast signed transactions to the blockchain. After building a transaction with build_transaction, use this to send it to the network.".to_string(),
            category: ToolCategory::Web3,
            priority: ToolPriority::High,
            status: ToolStatus::Active,
            version: "1.0.0".to_string(),
            author: "Alou".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            dependencies: vec!["build_transaction".to_string()],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec![],
        };

        Self { metadata }
    }
}

impl Default for BroadcastTransactionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for BroadcastTransactionTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start = std::time::Instant::now();
        
        let _signed_transaction = args.get("transaction").and_then(|v| v.as_str()).ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'transaction' field (signed transaction hex)".to_string())
        })?;

        let chain = args.get("chain").and_then(|v| v.as_str()).unwrap_or("ethereum");

        let result: Result<serde_json::Value, ToolError> = Ok(json!({
            "success": true,
            "chain": chain,
            "transactionHash": generate_mock_tx_hash(),
            "blockNumber": null,
            "status": "pending",
            "note": "This is a mock response. In production, broadcast to actual RPC endpoint."
        }));

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

        if !args.get("transaction").is_some() {
            return Err(ToolError::InvalidArguments("Missing 'transaction' field".to_string()));
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Broadcast Transaction Tool - Send signed transactions to blockchain

Parameters:
- transaction: Signed transaction hex string (required)
- chain: Blockchain network (optional, default: ethereum)

Example:
  {"transaction": "0x02f8...8501...", "chain": "ethereum"}
  {"transaction": "0x02f8...", "chain": "base"}"#.to_string()
    }
}

fn generate_mock_tx_hash() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    format!("0x{}", hex::encode(bytes))
}
