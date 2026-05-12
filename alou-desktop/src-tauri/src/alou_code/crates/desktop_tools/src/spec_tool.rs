//! Spec Tool Adapter for alou_code Kernel

use alou_code_runtime::permissions::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_spec".to_string();
    let description = "Spec documentation management".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["create", "get", "update", "list", "delete"]
            },
            "spec": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "title": { "type": "string" },
                    "content": { "type": "string" }
                }
            }
        },
        "required": ["operation"]
    });
    let permission = PermissionMode::WorkspaceWrite;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        Ok(serde_json::to_string(&json!({
            "success": true,
            "operation": operation,
            "message": "Spec tool - requires spec storage integration"
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
