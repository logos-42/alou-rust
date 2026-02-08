use crate::mcp::executor::{Tool, ToolContext, ToolError, ToolResult};
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub struct BashTool;

impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "当任务需要执行命令或获得真实终端结果时优先使用。用于运行命令、脚本、Git操作（Bash/CMD/PowerShell/Python/Node）。返回的输出必须作为后续判断依据。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": { 
                    "type": "string", 
                    "description": "操作类型", 
                    "enum": ["execute"], 
                    "default": "execute" 
                },
                "shell": { 
                    "type": "string", 
                    "description": "Shell类型", 
                    "enum": ["bash", "cmd", "powershell", "python", "node"], 
                    "default": "bash" 
                },
                "command": { 
                    "type": "string", 
                    "description": "要执行的命令" 
                },
                "working_dir": { 
                    "type": "string", 
                    "description": "工作目录（可选）" 
                },
                "environment": { 
                    "type": "array", 
                    "description": "环境变量（可选）", 
                    "items": { 
                        "type": "array", 
                        "items": { "type": "string" } 
                    } 
                },
                "timeout_seconds": { 
                    "type": "number", 
                    "description": "超时时间（秒，可选）" 
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'command' parameter".to_string()))?;

        let shell = args
            .get("shell")
            .and_then(|v| v.as_str())
            .unwrap_or("bash");

        let working_dir = args.get("working_dir").and_then(|v| v.as_str());
        let timeout_secs = args
            .get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        // 构建命令
        let (program, args_vec) = match shell {
            "bash" => ("bash", vec!["-c", command]),
            "cmd" => ("cmd", vec!["/c", command]),
            "powershell" => ("powershell", vec!["-Command", command]),
            "python" => ("python", vec!["-c", command]),
            "node" => ("node", vec!["-e", command]),
            _ => ("bash", vec!["-c", command]),
        };

        let mut cmd = Command::new(program);
        cmd.args(&args_vec)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // 设置环境变量
        if let Some(env_vars) = args.get("environment").and_then(|v| v.as_array()) {
            for var_pair in env_vars {
                if let Some(pair) = var_pair.as_array() {
                    if pair.len() == 2 {
                        let key = pair[0].as_str().unwrap_or("");
                        let value = pair[1].as_str().unwrap_or("");
                        cmd.env(key, value);
                    }
                }
            }
        }

        // 执行命令并设置超时
        let result = timeout(Duration::from_secs(timeout_secs), cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(ToolResult {
                    success: output.status.success(),
                    data: json!({
                        "stdout": stdout.to_string(),
                        "stderr": stderr.to_string(),
                        "exit_code": exit_code,
                        "command": command,
                        "shell": shell
                    }),
                    error: if output.status.success() {
                        None
                    } else {
                        Some(format!("Command failed with exit code {}: {}", exit_code, stderr))
                    },
                })
            }
            Ok(Err(e)) => Err(ToolError::ExecutionFailed(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => Err(ToolError::ExecutionFailed(
                "Command execution timed out".to_string()
            )),
        }
    }
}
