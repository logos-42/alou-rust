//! 工具执行模块
//! 
//! 负责执行动态创建的脚本工具

use crate::tools::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::process::Command;
use chrono::Utc;

/// 动态工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicToolResult {
    /// 是否成功
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
}

/// 动态工具执行器 - 用于执行AI创建的脚本工具（Python、Shell、JavaScript）
pub struct DynamicToolExecutor {
    metadata: ToolMetadata,
}

impl DynamicToolExecutor {
    /// 创建新的动态工具执行器
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "dynamic_tool_executor".to_string(),
                name: "Dynamic Tool Executor".to_string(),
                description: "执行动态创建的脚本工具（Python、Shell、JavaScript）".to_string(),
                category: ToolCategory::Development,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "execute".to_string()],
            },
        }
    }

    /// 执行Python脚本
    pub async fn execute_python(&self, script_path: &Path, args: &HashMap<String, serde_json::Value>) -> Result<DynamicToolResult, ToolError> {
        let start = Utc::now().timestamp_millis();

        // 构建命令行参数
        let mut cmd_args: Vec<String> = vec!["python3".to_string(), script_path.to_str().unwrap_or("").to_string()];
        for (key, value) in args {
            cmd_args.push("--".to_string());
            cmd_args.push(key.clone());
            if let Some(val_str) = value.as_str() {
                cmd_args.push(val_str.to_string());
            } else {
                cmd_args.push(value.to_string());
            }
        }

        let output = Command::new("python3")
            .args(&cmd_args[1..])
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute Python script: {}", e)))?;

        let execution_time_ms = (Utc::now().timestamp_millis() - start) as u64;

        Ok(DynamicToolResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1) as i32,
            execution_time_ms,
        })
    }

    /// 执行Shell脚本
    pub async fn execute_shell(&self, script_path: &Path, args: &HashMap<String, serde_json::Value>) -> Result<DynamicToolResult, ToolError> {
        let start = Utc::now().timestamp_millis();

        // 构建命令行参数
        let mut cmd_args: Vec<String> = vec!["bash".to_string(), script_path.to_str().unwrap_or("").to_string()];
        for (key, value) in args {
            cmd_args.push("--".to_string());
            cmd_args.push(key.clone());
            if let Some(val_str) = value.as_str() {
                cmd_args.push(val_str.to_string());
            } else {
                cmd_args.push(value.to_string());
            }
        }

        let output = Command::new("bash")
            .args(&cmd_args[1..])
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute shell script: {}", e)))?;

        let execution_time_ms = (Utc::now().timestamp_millis() - start) as u64;

        Ok(DynamicToolResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1) as i32,
            execution_time_ms,
        })
    }

    /// 执行JavaScript脚本（使用Node.js）
    pub async fn execute_javascript(&self, script_path: &Path, args: &HashMap<String, serde_json::Value>) -> Result<DynamicToolResult, ToolError> {
        let start = Utc::now().timestamp_millis();

        // 构建命令行参数
        let mut cmd_args: Vec<String> = vec!["node".to_string(), script_path.to_str().unwrap_or("").to_string()];
        for (key, value) in args {
            cmd_args.push("--".to_string());
            cmd_args.push(key.clone());
            if let Some(val_str) = value.as_str() {
                cmd_args.push(val_str.to_string());
            } else {
                cmd_args.push(value.to_string());
            }
        }

        let output = Command::new("node")
            .args(&cmd_args[1..])
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute JavaScript: {}", e)))?;

        let execution_time_ms = (Utc::now().timestamp_millis() - start) as u64;

        Ok(DynamicToolResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1) as i32,
            execution_time_ms,
        })
    }
}

#[async_trait]
impl ToolExecutor for DynamicToolExecutor {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let tool_type = args.get("tool_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_type' field".to_string()))?;

        let script_path = args.get("script_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'script_path' field".to_string()))?;

        let input_params: HashMap<String, serde_json::Value> = args.get("input_params")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let path = Path::new(script_path);

        if !path.exists() {
            return Err(ToolError::InvalidArguments(format!("Script file not found: {}", script_path)));
        }

        let result = match tool_type {
            "Python" | "python" => self.execute_python(path, &input_params).await?,
            "Shell" | "shell" | "bash" => self.execute_shell(path, &input_params).await?,
            "JavaScript" | "javascript" | "node" => self.execute_javascript(path, &input_params).await?,
            _ => return Err(ToolError::InvalidArguments(format!("Unsupported tool type: {}", tool_type))),
        };

        Ok(ToolResult {
            success: result.success,
            data: serde_json::json!(result),
            error: if !result.stderr.is_empty() { Some(result.stderr) } else { None },
            execution_time_ms: result.execution_time_ms,
            output: Some(format!("Tool executed successfully in {}ms", result.execution_time_ms)),
            warnings: vec![],
            context: None,
        })
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }
        if args.get("tool_type").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: tool_type".to_string()));
        }
        if args.get("script_path").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: script_path".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        r#"Dynamic Tool Executor

Execute dynamically created script tools.

Actions:
  Execute script tools (Python, Shell, JavaScript)

Params:
{
  "tool_type": "Python|Shell|JavaScript",
  "script_path": "Path to the script file",
  "input_params": {"key": "value", ...}
}

Examples:

Execute a Python script:
{
  "tool_type": "Python",
  "script_path": "./tools/custom/my_tool.py",
  "input_params": {
    "input_file": "data.txt",
    "output_file": "result.txt"
  }
}

Execute a Shell script:
{
  "tool_type": "Shell",
  "script_path": "./tools/custom/my_script.sh",
  "input_params": {
    "mode": "fast",
    "verbose": "true"
  }
}

Execute a JavaScript script:
{
  "tool_type": "JavaScript",
  "script_path": "./tools/custom/analyzer.js",
  "input_params": {
    "source": "input.json"
  }
}"#.to_string()
    }
}