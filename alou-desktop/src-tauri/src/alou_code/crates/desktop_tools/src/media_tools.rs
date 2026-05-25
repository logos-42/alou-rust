//! Media Tools Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_media".to_string();
    let description = "Media generation and processing: image, audio, video".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["generate_image", "generate_audio", "generate_video", "get_status"]
            },
            "prompt": { "type": "string" },
            "task_id": { "type": "string" }
        },
        "required": ["operation"]
    });
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("generate_image");

        Ok(serde_json::to_string(&serde_json::json!({
            "success": true,
            "operation": operation,
            "message": "Media tools - requires media API configuration"
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
