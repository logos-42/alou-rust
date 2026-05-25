//! Group Coordinator Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_group_coordinator".to_string();
    let description = "Multi-agent collaboration and group coordination".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["create_group", "join_group", "leave_group", "send_message", "get_messages"]
            },
            "group_id": { "type": "string" },
            "message": { "type": "string" },
            "agent_id": { "type": "string" }
        },
        "required": ["operation"]
    })
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("create_group");

        Ok(serde_json::to_string(&serde_json::json!({
            "success": true,
            "operation": operation,
            "message": "Group coordinator - requires message bus integration"
        }))?)
    })

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
