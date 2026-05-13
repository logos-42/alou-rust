//! Query Blockchain Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_query_blockchain".to_string();
    let description = "Blockchain query operations for multiple chains".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["get_balance", "get_transaction", "get_block", "get_logs"]
            },
            "chain": { "type": "string" },
            "address": { "type": "string" },
            "tx_hash": { "type": "string" },
            "block_number": { "type": "integer" }
        },
        "required": ["operation", "chain"]
    });
    let permission = PermissionMode::ReadOnly;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("get_balance");
        let chain = input.get("chain")
            .and_then(|v| v.as_str())
            .unwrap_or("ethereum");

        Ok(serde_json::to_string(&json!({
            "success": true,
            "operation": operation,
            "chain": chain,
            "message": "Blockchain query - requires RPC endpoint configuration"
        }).map_err(|e| e.to_string())?)
    });

    (name, description, schema, permission, executor)
}

pub fn tool_definition() -> ToolDefinition {
    let (name, description, schema, _, _) = tool_spec();
    ToolDefinition {
        name,
        description: Some(description),
        input_schema: schema,
    }
}
