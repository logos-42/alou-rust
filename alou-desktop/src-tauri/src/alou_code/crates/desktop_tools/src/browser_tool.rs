//! Browser Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use open;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_browser".to_string();
    let description = "Browser automation and web interaction".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["open", "screenshot", "evaluate"]
            },
            "url": { "type": "string", "format": "uri" },
            "script": { "type": "string" }
        },
        "required": ["operation"]
    });
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("open");

        match operation {
            "open" => {
                let url = input.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                open::that(url)
                    .map_err(|e| format!("Failed to open browser: {}", e))?;

                Ok(serde_json::to_string(&json!({
                    "success": true,
                    "message": format!("Opened URL: {}", url)
                }))?)
            }
            _ => Err(format!("Browser operation {} not yet implemented via adapter", operation))
        }
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
