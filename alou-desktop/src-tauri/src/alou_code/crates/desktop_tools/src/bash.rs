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
    let description = "Execute shell commands (Bash, CMD, PowerShell)".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "command": { "type": "string" },
            "shell": {
                "type": "string",
                "enum": ["bash", "cmd", "powershell"],
                "default": "bash"
            },
            "working_dir": { "type": "string" },
            "timeout_secs": { "type": "integer", "minimum": 1 }
        },
        "required": ["command"]
    });
    let permission = PermissionMode::DangerFullAccess;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let command = input.get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let shell = input.get("shell")
            .and_then(|v| v.as_str())
            .unwrap_or("bash");
        let working_dir = input.get("working_dir")
            .and_then(|v| v.as_str());

        let output = if cfg!(windows) {
            match shell {
                "cmd" => {
                    let mut cmd = Command::new("cmd");
                    cmd.arg("/C").arg(command);
                    if let Some(dir) = working_dir {
                        cmd.current_dir(dir);
                    }
                    cmd.output()
                }
                "powershell" => {
                    let mut cmd = Command::new("powershell");
                    cmd.arg("-Command").arg(command);
                    if let Some(dir) = working_dir {
                        cmd.current_dir(dir);
                    }
                    cmd.output()
                }
                _ => {
                    if let Ok(bash_path) = std::process::Command::new("bash.exe")
                        .arg("-c")
                        .arg(command)
                        .current_dir(working_dir.unwrap_or("."))
                        .output()
                    {
                        Ok(bash_path)
                    } else if let Ok(pwsh) = std::process::Command::new("pwsh")
                        .arg("-Command")
                        .arg(command)
                        .current_dir(working_dir.unwrap_or("."))
                        .output()
                    {
                        Ok(pwsh)
                    } else {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "No shell found"
                        ))
                    }
                }
            }
        } else {
            let mut cmd = Command::new("bash");
            cmd.arg("-c").arg(command);
            if let Some(dir) = working_dir {
                cmd.current_dir(dir);
            }
            cmd.output()
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(serde_json::to_string(&serde_json::json!({
                    "success": output.status.success(),
                    "exit_code": exit_code,
                    "stdout": stdout,
                    "stderr": stderr
                }))?);
            }
            Err(e) => Err(format!("Command execution failed: {}", e))
        }
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
