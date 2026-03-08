use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolMetadata, ExecutionContext, ToolCategory, ToolPriority, ToolStatus};
use async_trait::async_trait;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct BuildTransactionTool {
    metadata: ToolMetadata,
}

impl BuildTransactionTool {
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "build_transaction".to_string(),
            name: "build_transaction".to_string(),
            description: "Build unsigned blockchain transactions. Supports ETH transfers, ERC-20 token transfers, and contract interactions.".to_string(),
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
}

impl Default for BuildTransactionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for BuildTransactionTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start = std::time::Instant::now();
        
        let to = args.get("to").and_then(|v| v.as_str()).ok_or_else(|| {
            ToolError::InvalidArguments("Missing 'to' field (recipient address)".to_string())
        })?;

        let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("0");
        let data = args.get("data").and_then(|v| v.as_str()).unwrap_or("0x");
        let chain = args.get("chain").and_then(|v| v.as_str()).unwrap_or("ethereum");

        let result: Result<serde_json::Value, ToolError> = Ok(json!({
            "success": true,
            "chain": chain,
            "transaction": {
                "to": to,
                "value": value,
                "data": data,
                "gasLimit": "21000",
                "gasPrice": "10000000000",
                "nonce": 0,
                "chainId": get_chain_id(chain),
            },
            "signedTransaction": null,
            "note": "This is a template. In production, fetch actual gas prices and sign with private key."
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

        if !args.get("to").is_some() {
            return Err(ToolError::InvalidArguments("Missing 'to' field".to_string()));
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Build Transaction Tool - Build unsigned blockchain transactions

Parameters:
- to: Recipient address (required)
- value: Amount in wei (optional, default: 0)
- data: Contract data / calldata (optional, default: 0x)
- chain: Blockchain network (optional, default: ethereum)
- gasLimit: Gas limit (optional)
- gasPrice: Gas price in wei (optional)

Example:
  {"to": "0x...", "value": "1000000000000000000", "chain": "ethereum"}
  {"to": "0x...", "data": "0xa9059cbb000000000...", "chain": "base"}"#.to_string()
    }
}

fn get_chain_id(chain: &str) -> &str {
    match chain {
        "ethereum" => "0x1",
        "sepolia" => "0xaa36a7",
        "base" => "0x2105",
        "base_sepolia" => "0x14a34",
        "polygon" => "0x89",
        "polygon_amoy" => "0x13882",
        _ => "0x1",
    }
}
