// LSP SDK integration module
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::fs;
use tauri::{AppHandle, Manager};

use crate::utils::app_data_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct LspRequest {
    pub code: String,
    pub language: String,
    pub file_path: Option<String>,
    pub operation: LspOperation,
    pub position: Option<LspPosition>,
    pub project_root: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LspOperation {
    /// 代码补全
    Completion,
    /// 诊断错误和警告
    Diagnostics,
    /// 定义跳转
    GoToDefinition,
    /// 悬停信息
    Hover,
    /// 引用查找
    FindReferences,
    /// 格式化代码
    Format,
    /// 重命名符号
    Rename,
    /// 符号搜索
    DocumentSymbols,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Serialize)]
pub struct LspResponse {
    pub success: bool,
    pub result: Option<LspResult>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LspResult {
    Completion(CompletionResult),
    Diagnostics(DiagnosticsResult),
    Definition(DefinitionResult),
    Hover(HoverResult),
    References(ReferencesResult),
    Format(FormatResult),
    Rename(RenameResult),
    Symbols(SymbolsResult),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsResult {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: String,
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefinitionResult {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HoverResult {
    pub contents: String,
    pub range: Option<Range>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferencesResult {
    pub references: Vec<Location>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormatResult {
    pub formatted_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameResult {
    pub changes: Vec<FileChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChange {
    pub uri: String,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolsResult {
    pub symbols: Vec<SymbolInformation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: String,
    pub location: Location,
    pub container_name: Option<String>,
}

/// Find Node.js executable
fn find_node_executable() -> Option<PathBuf> {
    let candidates = if cfg!(target_os = "windows") {
        vec!["node", "node.exe"]
    } else {
        vec!["node"]
    };

    for candidate in candidates {
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

/// Get the path to the LSP script
fn get_lsp_script_path(app: &AppHandle) -> Result<PathBuf, String> {
    let script_name = "lsp-agent.js";

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

    if let Ok(resource_dir) = app.path().resource_dir() {
        let script_path = resource_dir.join("scripts").join(script_name);
        if script_path.exists() {
            return Ok(script_path);
        }
    }

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
pub async fn execute_lsp(
    app: AppHandle,
    request: LspRequest,
) -> Result<LspResponse, String> {
    // Find Node.js executable
    let node_exe = find_node_executable()
        .ok_or_else(|| "未找到 Node.js 可执行文件。请确保已安装 Node.js (>=18.0.0)".to_string())?;

    // Get script path
    let script_path = get_lsp_script_path(&app)?;

    // Get app data directory for temporary files
    let app_data_dir = app_data_dir(&app)?;
    let temp_dir = app_data_dir.join("lsp_temp");
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

        return Ok(LspResponse {
            success: false,
            result: None,
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
        return Ok(LspResponse {
            success: false,
            result: None,
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

        return Ok(LspResponse {
            success: false,
            result: None,
            error: Some(error_msg),
        });
    }

    let lsp_result = result.get("result").and_then(|v| {
        serde_json::from_value::<LspResult>(v.clone()).ok()
    });

    Ok(LspResponse {
        success: true,
        result: lsp_result,
        error: None,
    })
}

#[tauri::command]
pub async fn get_supported_languages() -> Result<Vec<String>, String> {
    Ok(vec![
        "javascript".to_string(),
        "typescript".to_string(),
        "python".to_string(),
        "rust".to_string(),
        "go".to_string(),
        "java".to_string(),
        "csharp".to_string(),
        "cpp".to_string(),
        "html".to_string(),
        "css".to_string(),
        "json".to_string(),
        "markdown".to_string(),
        "yaml".to_string(),
        "sql".to_string(),
    ].into_iter().collect())
}
