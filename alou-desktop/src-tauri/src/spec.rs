// Spec SDK integration module
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::fs;
use tauri::{AppHandle, Manager};

use crate::utils::app_data_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct SpecRequest {
    pub spec_type: SpecType,
    pub operation: SpecOperation,
    pub spec_data: Option<serde_json::Value>,
    pub spec_id: Option<String>,
    pub template_id: Option<String>,
    pub output_format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SpecType {
    /// 产品需求规格
    Product,
    /// 技术规格
    Technical,
    /// 设计规格
    Design,
    /// API规格
    Api,
    /// 用户故事
    UserStory,
    /// 任务规格
    Tasks,
    /// 结构规格
    Structure,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SpecOperation {
    /// 创建新规格文档
    Create,
    /// 更新规格文档
    Update,
    /// 获取规格文档
    Get,
    /// 删除规格文档
    Delete,
    /// 列出所有规格
    List,
    /// 验证规格
    Validate,
    /// 从模板生成
    GenerateFromTemplate,
    /// 导出规格
    Export,
    /// 导入规格
    Import,
}

#[derive(Debug, Serialize)]
pub struct SpecResponse {
    pub success: bool,
    pub spec: Option<SpecDocument>,
    pub specs: Option<Vec<SpecDocument>>,
    pub validation_result: Option<ValidationResult>,
    pub export_result: Option<ExportResult>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecDocument {
    pub id: String,
    pub spec_type: String,
    pub title: String,
    pub content: String,
    pub metadata: SpecMetadata,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecMetadata {
    pub version: String,
    pub status: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub template_id: Option<String>,
    pub related_specs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub output_path: String,
    pub format: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpecTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub spec_type: String,
    pub template_content: String,
    pub required_fields: Vec<String>,
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

/// Get the path to the Spec script
fn get_spec_script_path(app: &AppHandle) -> Result<PathBuf, String> {
    let script_name = "spec-agent.js";

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
pub async fn execute_spec(
    app: AppHandle,
    request: SpecRequest,
) -> Result<SpecResponse, String> {
    // Find Node.js executable
    let node_exe = find_node_executable()
        .ok_or_else(|| "未找到 Node.js 可执行文件。请确保已安装 Node.js (>=18.0.0)".to_string())?;

    // Get script path
    let script_path = get_spec_script_path(&app)?;

    // Get app data directory for temporary files
    let app_data_dir = app_data_dir(&app)?;
    let temp_dir = app_data_dir.join("spec_temp");
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

        return Ok(SpecResponse {
            success: false,
            spec: None,
            specs: None,
            validation_result: None,
            export_result: None,
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
        return Ok(SpecResponse {
            success: false,
            spec: None,
            specs: None,
            validation_result: None,
            export_result: None,
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

        return Ok(SpecResponse {
            success: false,
            spec: None,
            specs: None,
            validation_result: None,
            export_result: None,
            error: Some(error_msg),
        });
    }

    let spec = result.get("spec").and_then(|v| {
        serde_json::from_value::<SpecDocument>(v.clone()).ok()
    });

    let specs = result.get("specs").and_then(|v| {
        serde_json::from_value::<Vec<SpecDocument>>(v.clone()).ok()
    });

    let validation_result = result.get("validation_result").and_then(|v| {
        serde_json::from_value::<ValidationResult>(v.clone()).ok()
    });

    let export_result = result.get("export_result").and_then(|v| {
        serde_json::from_value::<ExportResult>(v.clone()).ok()
    });

    Ok(SpecResponse {
        success: true,
        spec,
        specs,
        validation_result,
        export_result,
        error: None,
    })
}

#[tauri::command]
pub async fn get_spec_templates() -> Result<Vec<SpecTemplate>, String> {
    // Read templates from the .spec-workflow directory
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("无法获取父目录")?
        .join(".spec-workflow")
        .join("templates");

    if !templates_dir.exists() {
        return Ok(vec![]);
    }

    let mut templates = vec![];

    let entries = fs::read_dir(&templates_dir)
        .await
        .map_err(|e| format!("读取模板目录失败: {}", e))?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let file_name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let template_id = file_name.replace("-template", "");

            templates.push(SpecTemplate {
                id: template_id.clone(),
                name: format!("{} Template", template_id),
                description: format!("{} specification template", template_id),
                spec_type: template_id,
                template_content: String::new(), // Will be loaded on demand
                required_fields: vec![],
            });
        }
    }

    Ok(templates)
}

#[tauri::command]
pub async fn load_template_content(template_id: String) -> Result<String, String> {
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("无法获取父目录")?
        .join(".spec-workflow")
        .join("templates");

    let template_file = templates_dir.join(format!("{}-template.md", template_id));

    if !template_file.exists() {
        return Err(format!("模板文件不存在: {:?}", template_file));
    }

    let content = fs::read_to_string(&template_file)
        .await
        .map_err(|e| format!("读取模板文件失败: {}", e))?;

    Ok(content)
}
