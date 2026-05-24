//! Search Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use glob;
use serde_json::{json, Value};
use regex::Regex;
use walkdir::WalkDir;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_search".to_string();
    let description = "Text search and file pattern matching".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["grep", "glob"]
            },
            "pattern": { "type": "string" },
            "path": { "type": "string" },
            "file_pattern": { "type": "string" },
            "case_sensitive": { "type": "boolean", "default": true },
            "max_results": { "type": "integer", "minimum": 1 }
        },
        "required": ["operation", "path"]
    });
    let permission = PermissionMode::ReadOnly;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("grep");
        let path = input.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let pattern = input.get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let file_pattern = input.get("file_pattern")
            .and_then(|v| v.as_str());
        let case_sensitive = input.get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let max_results = input.get("max_results")
            .and_then(|v| v.as_i64())
            .map(|v| v as usize);

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        match operation {
            "grep" => {
                let regex = Regex::new(pattern)
                    .map_err(|e| format!("Invalid regex: {}", e))?;

                let matches = runtime.block_on(async {
                    let mut results = Vec::new();
                    let walker = WalkDir::new(path)
                        .follow_links(true)
                        .into_iter();

                    for entry in walker.filter_map(|e| e.ok()) {
                        if !entry.file_type().is_file() {
                            continue;
                        }

                        if let Some(fp) = file_pattern {
                            if let Ok(glob_pattern) = glob::Pattern::new(fp) {
                                if !glob_pattern.matches_path(entry.path()) {
                                    continue;
                                }
                            }
                        }

                        let content = match File::open(entry.path()).await {
                            Ok(mut f) => {
                                let mut buf = String::new();
                                if f.read_to_string(&mut buf).await.is_ok() {
                                    buf
                                } else {
                                    continue;
                                }
                            }
                            Err(_) => continue,
                        };

                        for (line_num, line) in content.lines().enumerate() {
                            let search_line = if case_sensitive { line } else { &line.to_lowercase() };
                            let search_pattern = if case_sensitive { pattern } else { &pattern.to_lowercase() };

                            if search_line.contains(search_pattern) {
                                results.push(serde_json::json!({
                                    "path": entry.path().to_string_lossy(),
                                    "line_number": line_num + 1,
                                    "line": line,
                                }));

                                if let Some(max) = max_results {
                                    if results.len() >= max {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Ok::<_, String>(results)
                })?;

                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "matches": matches,
                    "count": matches.len()
                });
            }
            "glob" => {
                let matches = runtime.block_on(async {
                    let mut results = Vec::new();
                    let walker = WalkDir::new(path)
                        .follow_links(true)
                        .into_iter();

                    let pattern = glob::Pattern::new(pattern)
                        .map_err(|e| format!("Invalid glob pattern: {}", e))?;

                    for entry in walker.filter_map(|e| e.ok()) {
                        if pattern.matches_path(entry.path()) {
                            results.push(serde_json::json!({
                                "path": entry.path().to_string_lossy(),
                                "is_dir": entry.file_type().is_dir(),
                            }));
                        }
                    }
                    Ok::<_, String>(results)
                })?;

                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "matches": matches,
                    "count": matches.len()
                });
            }
            _ => Err(format!("Unknown operation: {}", operation))
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
