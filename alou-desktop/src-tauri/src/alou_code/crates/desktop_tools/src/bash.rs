//! Bash Tool Adapter for alou_code Kernel

use alou_code_api::ToolDefinition;
use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};
use std::process::Command;

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_bash".to_string();
    let description = "Execute bash commands on the desktop system".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "command": { "type": "string" },
            "working_dir": { "type": "string" },
            "timeout": { "type": "integer", "default": 30 }
        },
        "required": ["command"]
    });
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(move |input: &Value| {
        let command = input.get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let working_dir = input.get("working_dir")
            .and_then(|v| v.as_str());
        let timeout = input.get("timeout")
            .and_then(|v| v.as_i64())
            .unwrap_or(30) as u64;

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(command);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(command);
            c
        };

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        let result = json!({
            "success": output.status.success(),
            "exit_code": output.status.code().unwrap_or(-1),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr)
        });
        Ok(serde_json::to_string(&result)?)
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