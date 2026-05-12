//! Polymarket Tool Adapter for alou_code Kernel

use alou_code_runtime::permissions::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_polymarket".to_string();
    let description = "Polymarket prediction market operations".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["list_markets", "get_orderbook", "place_order", "get_orders"]
            },
            "token_id": { "type": "string" },
            "side": { "type": "string", "enum": ["BUY", "SELL"] },
            "price": { "type": "number" },
            "size": { "type": "number" }
        },
        "required": ["operation"]
    });
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("list_markets");

        Ok(serde_json::to_string(&json!({
            "success": true,
            "operation": operation,
            "message": "Polymarket - requires API key configuration"
        }))?)
    });

    (name, description, schema, permission, executor)
}

pub fn tool_definition() -> alou_code_api::types::ToolDefinition {
    let (name, description, schema, _, _) = tool_spec();
    alou_code_api::types::ToolDefinition {
        name,
        description: Some(description),
        input_schema: schema,
    }
}
