//! Git Helper Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_git".to_string();
    let description = "Git operations: status, commit, log, branch management".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["status", "commit", "log", "branch", "checkout", "pull", "push"]
            },
            "message": { "type": "string" },
            "branch": { "type": "string" },
            "path": { "type": "string" }
        },
        "required": ["operation"]
    });
    let permission = PermissionMode::WorkspaceWrite;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("status");
        let path = input.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let output = match operation {
            "status" => std::process::Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(path)
                .output(),
            "log" => std::process::Command::new("git")
                .args(["log", "--oneline", "-10"])
                .current_dir(path)
                .output(),
            "branch" => std::process::Command::new("git")
                .args(["branch", "-a"])
                .current_dir(path)
                .output(),
            "commit" => {
                let msg = input.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                std::process::Command::new("git")
                    .args(["commit", "-m", msg])
                    .current_dir(path)
                    .output()
            }
            "checkout" => {
                let branch = input.get("branch")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                std::process::Command::new("git")
                    .args(["checkout", branch])
                    .current_dir(path)
                    .output()
            }
            "pull" => std::process::Command::new("git")
                .args(["pull"])
                .current_dir(path)
                .output(),
            "push" => std::process::Command::new("git")
                .args(["push"])
                .current_dir(path)
                .output(),
            _ => return Err(format!("Unknown operation: {}", operation))
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Ok(serde_json::to_string(&serde_json::json!({
                    "success": output.status.success(),
                    "exit_code": output.status.code().unwrap_or(-1),
                    "stdout": stdout,
                    "stderr": stderr
                });
            }
            Err(e) => Err(format!("Git command failed: {}", e))
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
