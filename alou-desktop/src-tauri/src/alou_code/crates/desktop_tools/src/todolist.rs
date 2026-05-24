//! TodoList Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_todolist".to_string();
    let description = "Task and todo list management".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["add", "list", "update", "delete"]
            },
            "task": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "status": { "type": "string", "enum": ["pending", "in_progress", "completed"] },
                    "priority": { "type": "string", "enum": ["low", "medium", "high"] }
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

        Ok(serde_json::to_string(&serde_json::json!({
            "success": true,
            "operation": operation,
            "message": "TodoList tool adapter - requires session storage integration"
        });
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
