// Claude Agent SDK integration module
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::fs;
use tauri::{AppHandle, Manager};

use crate::utils::app_data_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeAgentQueryRequest {
    pub api_key: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub history: Option<Vec<ChatMessage>>,
    pub agent_info: Option<AgentInfo>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    /// API provider: "deepseek" or "claude" (default: "deepseek")
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentInfo {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ClaudeAgentQueryResponse {
    pub success: bool,
    pub response: Option<String>,
    pub tool_calls: Option<Vec<serde_json::Value>>,
    pub usage: Option<Usage>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Find Node.js executable
fn find_node_executable() -> Option<PathBuf> {
    // Try common paths and commands
    let candidates = if cfg!(target_os = "windows") {
        vec!["node", "node.exe"]
    } else {
        vec!["node"]
    };

    for candidate in candidates {
        // Try to run `node --version` to verify it exists
        if Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok()
        {
            return Some(PathBuf::from(candidate));
        }
    }

    None
}

/// Get the path to the agent script (deepseek-agent.js or claude-agent.js)
fn get_script_path(app: &AppHandle, provider: &str) -> Result<PathBuf, String> {
    let script_name = if provider == "deepseek" {
        "deepseek-agent.js"
    } else {
        "claude-agent.js"
    };

    // In development, use the scripts directory relative to CARGO_MANIFEST_DIR
    #[cfg(debug_assertions)]
    {
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("无法获取父目录")?
            .join("scripts")
            .join(script_name);
        
        if dev_path.exists() {
            return Ok(dev_path);
        }
    }

    // In production, try to find it in the resource directory
    if let Ok(resource_dir) = app.path().resource_dir() {
        let script_path = resource_dir.join("scripts").join(script_name);
        if script_path.exists() {
            return Ok(script_path);
        }
    }

    // Fallback: try relative to app data directory
    let app_data_dir = app_data_dir(app)?;
    let script_path = app_data_dir
        .parent()
        .ok_or("无法获取应用数据目录的父目录")?
        .join("scripts")
        .join(script_name);

    if script_path.exists() {
        Ok(script_path)
    } else {
        Err(format!(
            "找不到 {} 脚本。请确保脚本位于: {:?}",
            script_name, script_path
        ))
    }
}

#[tauri::command]
pub async fn query_claude_agent(
    app: AppHandle,
    request: ClaudeAgentQueryRequest,
) -> Result<ClaudeAgentQueryResponse, String> {
    // Validate API key
    if request.api_key.is_empty() {
        return Ok(ClaudeAgentQueryResponse {
            success: false,
            response: None,
            tool_calls: None,
            usage: None,
            error: Some("API key 不能为空".to_string()),
        });
    }

    // Determine provider (default to "deepseek")
    let provider = request.provider.as_deref().unwrap_or("deepseek");
    if provider != "deepseek" && provider != "claude" {
        return Ok(ClaudeAgentQueryResponse {
            success: false,
            response: None,
            tool_calls: None,
            usage: None,
            error: Some(format!("不支持的 provider: {}. 支持: deepseek, claude", provider)),
        });
    }

    // Find Node.js executable
    let node_exe = find_node_executable()
        .ok_or_else(|| "未找到 Node.js 可执行文件。请确保已安装 Node.js (>=18.0.0)".to_string())?;

    // Get script path based on provider
    let script_path = get_script_path(&app, provider)?;

    // Get app data directory for temporary files
    let app_data_dir = app_data_dir(&app)?;
    let temp_dir = app_data_dir.join("claude_agent_temp");
    fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    // Create temporary input file
    let input_file = NamedTempFile::new_in(&temp_dir)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    let input_path = input_file.path().to_path_buf();

    // Write request data to input file
    let request_json = serde_json::to_string_pretty(&request)
        .map_err(|e| format!("序列化请求失败: {}", e))?;
    fs::write(&input_path, request_json)
        .await
        .map_err(|e| format!("写入请求文件失败: {}", e))?;

    // Prepare command
    let mut cmd = Command::new(&node_exe);
    cmd.arg(&script_path)
        .arg(&input_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    // On Windows, hide the console window
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    // Execute command
    let output = cmd
        .output()
        .map_err(|e| format!("执行 Node.js 脚本失败: {}", e))?;

    // Clean up input file immediately
    let _ = fs::remove_file(&input_path).await;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Try to read error file if it exists
        let error_file = input_path
            .to_string_lossy()
            .to_string()
            .replace(".json", ".error.json");
        let error_msg = if let Ok(error_content) = fs::read_to_string(&error_file).await {
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_content) {
                error_json
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&stderr)
                    .to_string()
            } else {
                stderr.to_string()
            }
        } else {
            stderr.to_string()
        };

        // Clean up error file
        let _ = fs::remove_file(&error_file).await;

        return Ok(ClaudeAgentQueryResponse {
            success: false,
            response: None,
            tool_calls: None,
            usage: None,
            error: Some(format!("Node.js 脚本执行失败: {}", error_msg)),
        });
    }

    // Read result file path from stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result_path_str = stdout
        .lines()
        .next()
        .ok_or_else(|| "未从 stdout 读取到结果文件路径".to_string())?
        .trim();

    if result_path_str.is_empty() {
        return Ok(ClaudeAgentQueryResponse {
            success: false,
            response: None,
            tool_calls: None,
            usage: None,
            error: Some("结果文件路径为空".to_string()),
        });
    }

    let result_path = PathBuf::from(result_path_str);

    // Read result file
    let result_content = fs::read_to_string(&result_path)
        .await
        .map_err(|e| format!("读取结果文件失败: {}", e))?;

    // Parse result
    let result: serde_json::Value = serde_json::from_str(&result_content)
        .map_err(|e| format!("解析结果 JSON 失败: {}", e))?;

    // Clean up result file
    let _ = fs::remove_file(&result_path).await;

    // Extract response data
    let success = result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !success {
        let error_msg = result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误")
            .to_string();

        return Ok(ClaudeAgentQueryResponse {
            success: false,
            response: None,
            tool_calls: None,
            usage: None,
            error: Some(error_msg),
        });
    }

    let response = result
        .get("response")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let tool_calls = result
        .get("toolCalls")
        .and_then(|v| v.as_array())
        .map(|arr| arr.clone());

    let usage = result.get("usage").and_then(|v| {
        serde_json::from_value::<Usage>(v.clone()).ok()
    });

    Ok(ClaudeAgentQueryResponse {
        success: true,
        response,
        tool_calls,
        usage,
        error: None,
    })
}

