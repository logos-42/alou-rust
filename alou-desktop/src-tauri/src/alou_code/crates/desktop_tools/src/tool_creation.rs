//! Tool Creation Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_tool_creation".to_string();
    let description = "Create and register new tools dynamically".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["create", "list", "get", "delete"]
            },
            "tool": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "schema": { "type": "object" },
                    "code": { "type": "string" }
                }
            }
        },
        "required": ["operation"]
    });)
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        Ok(serde_json::to_string(&serde_json::json!({
            "success": true,
            "operation": operation,
            "message": "Tool creation - requires dynamic tool registry"
        }))?)
    });)

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
