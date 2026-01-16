//! 终端工具
//!
//! 提供Bash/CMD/PowerShell命令执行功能

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// 终端工具
pub struct BashTool {
    metadata: ToolMetadata,
}

impl BashTool {
    /// 创建新的终端工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "bash".to_string(),
                name: "Terminal Tool".to_string(),
                description: "Execute shell commands (Bash, CMD, PowerShell, Python, Node.js)".to_string(),
                category: ToolCategory::Terminal,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["execute".to_string()],
            },
        }
    }

    /// 执行命令
    async fn execute_command(&self, shell: &Shell, command: &str, working_dir: Option<&str>, env_vars: &[(String, String)]) -> Result<CommandResult, ToolError> {
        let start_time = std::time::Instant::now();

        let mut cmd = match shell {
            Shell::Bash => {
                if cfg!(windows) {
                    // Windows: 使用 git bash 或 wsl bash
                    if let Ok(bash_path) = which::which("bash.exe") {
                        let mut cmd = Command::new(&bash_path);
                        cmd.arg("-c").arg(command);
                        cmd
                    } else {
                        return Err(ToolError::ToolUnavailable("Bash not found on Windows. Install Git Bash or WSL.".to_string()));
                    }
                } else {
                    let mut cmd = Command::new("bash");
                    cmd.arg("-c").arg(command);
                    cmd
                }
            }
            Shell::Cmd => {
                if !cfg!(windows) {
                    return Err(ToolError::ToolUnavailable("CMD is only available on Windows".to_string()));
                }
                let mut cmd = Command::new("cmd");
                cmd.arg("/C").arg(command);
                cmd
            }
            Shell::PowerShell => {
                if cfg!(windows) {
                    let mut cmd = Command::new("powershell");
                    cmd.arg("-Command").arg(command);
                    cmd
                } else if let Ok(pwsh) = which::which("pwsh") {
                    let mut cmd = Command::new(pwsh);
                    cmd.arg("-Command").arg(command);
                    cmd
                } else {
                    return Err(ToolError::ToolUnavailable("PowerShell not found. Install PowerShell Core.".to_string()));
                }
            }
            Shell::Python => {
                let python_cmd = if cfg!(windows) { "python.exe" } else { "python3" };
                let python_path = which::which(python_cmd)
                    .or_else(|_| which::which("python"))
                    .map_err(|_| ToolError::ToolUnavailable("Python not found".to_string()))?;

                let mut cmd = Command::new(python_path);
                cmd.arg("-c").arg(command);
                cmd
            }
            Shell::Node => {
                let node_cmd = if cfg!(windows) { "node.exe" } else { "node" };
                let node_path = which::which(node_cmd)
                    .map_err(|_| ToolError::ToolUnavailable("Node.js not found".to_string()))?;

                let mut cmd = Command::new(node_path);
                cmd.arg("-e").arg(command);
                cmd
            }
        };

        // 设置工作目录
        if let Some(dir) = working_dir {
            let path = PathBuf::from(dir);
            if path.exists() && path.is_dir() {
                cmd.current_dir(path);
            }
        }

        // 设置环境变量
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // 执行命令
        let output = cmd.output().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute command: {}", e)))?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        Ok(CommandResult {
            success,
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(-1),
            execution_time_ms,
        })
    }
}

#[async_trait]
impl ToolExecutor for BashTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let operation: BashOperation = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        match operation {
            BashOperation::Execute { shell, command, working_dir, environment, timeout_seconds } => {
                self.execute_command_internal(shell, &command, working_dir.as_deref(), &environment, timeout_seconds).await
            }
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if serde_json::from_value::<BashOperation>(args.clone()).is_ok() {
            Ok(())
        } else {
            Err(ToolError::InvalidArguments("Invalid bash operation arguments".to_string()))
        }
    }

    fn help(&self) -> String {
        r#"Terminal Tool - Execute shell commands

Available shells:
- bash: Bash shell (Linux/macOS, or Git Bash/WSL on Windows)
- cmd: Windows Command Prompt
- powershell: PowerShell
- python: Python interpreter
- node: Node.js runtime

Example usage:
{
  "shell": "bash",
  "command": "ls -la",
  "working_dir": "/path/to/dir",
  "environment": [["VAR", "value"]],
  "timeout_seconds": 30
}"#.to_string()
    }
}

impl BashTool {
    async fn execute_command_internal(
        &self,
        shell: Shell,
        command: &str,
        working_dir: Option<&str>,
        environment: &[(String, String)],
        timeout_seconds: Option<u64>,
    ) -> Result<ToolResult, ToolError> {
        let timeout_duration = timeout_seconds.unwrap_or(30);

        let result = timeout(
            Duration::from_secs(timeout_duration),
            self.execute_command(&shell, command, working_dir, environment)
        ).await;

        let cmd_result = match result {
            Ok(r) => r,
            Err(_) => return Err(ToolError::Timeout(format!("Command timed out after {} seconds", timeout_duration))),
        }?;

        Ok(ToolResult {
            success: cmd_result.success,
            data: serde_json::json!({
                "stdout": cmd_result.stdout,
                "stderr": cmd_result.stderr,
                "exit_code": cmd_result.exit_code,
                "shell": format!("{:?}", shell),
                "command": command,
                "working_dir": working_dir,
            }),
            error: if cmd_result.success { None } else { Some(format!("Command failed with exit code {}", cmd_result.exit_code)) },
            execution_time_ms: cmd_result.execution_time_ms,
            output: Some(format!(
                "Command executed in {}ms. Exit code: {}",
                cmd_result.execution_time_ms,
                cmd_result.exit_code
            )),
            warnings: vec![],
            context: None,
        })
    }
}

/// Shell 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shell {
    Bash,
    Cmd,
    PowerShell,
    Python,
    Node,
}

/// Bash 操作
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation")]
pub enum BashOperation {
    /// 执行命令
    Execute {
        shell: Shell,
        command: String,
        #[serde(default)]
        working_dir: Option<String>,
        #[serde(default)]
        environment: Vec<(String, String)>,
        #[serde(default)]
        timeout_seconds: Option<u64>,
    },
}

/// 命令执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// 是否成功
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_simple_command() {
        let tool = BashTool::new();
        let context = ExecutionContext {
            session_id: "test".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(10),
            permissions: vec!["execute".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let shell = if cfg!(windows) { Shell::Cmd } else { Shell::Bash };
        let command = if cfg!(windows) { "echo test" } else { "echo 'test'" };

        let args = serde_json::json!({
            "operation": "execute",
            "shell": shell,
            "command": command
        });

        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert!(result.data["stdout"].as_str().unwrap().contains("test"));
    }

    #[tokio::test]
    async fn test_execute_with_timeout() {
        let tool = BashTool::new();
        let context = ExecutionContext {
            session_id: "test".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(10),
            permissions: vec!["execute".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let shell = if cfg!(windows) { Shell::Cmd } else { Shell::Bash };
        let command = if cfg!(windows) { "timeout 2" } else { "sleep 2" };

        let args = serde_json::json!({
            "operation": "execute",
            "shell": shell,
            "command": command,
            "timeout_seconds": 1
        });

        let result = tool.execute(args, &context).await;
        assert!(result.is_err());
        matches!(result.unwrap_err(), ToolError::Timeout(_));
    }
}
