//! Broadcast Transaction Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_broadcast_transaction".to_string();
    let description = "Broadcast signed transactions to blockchain".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["broadcast"]
            },
            "chain": { "type": "string" },
            "signed_tx": { "type": "string" }
        },
        "required": ["operation", "chain", "signed_tx"]
    });
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("broadcast");
        let chain = input.get("chain")
            .and_then(|v| v.as_str())
            .unwrap_or("ethereum");

        Ok(serde_json::to_string(&serde_json::json!({
            "success": true,
            "operation": operation,
            "chain": chain,
            "message": "Broadcast transaction - requires RPC endpoint"
        }))?)
    });

    (name, description, schema, permission, executor);
}

pub fn tool_definition() -> ToolDefinition {
    let (name, description, schema, _, _) = tool_spec();
    ToolDefinition {
        name,
        description: Some(description),
        input_schema: schema,
    }
}
