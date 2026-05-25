//! Filesystem Tool Adapter for alou_code Kernel

use alou_code_api::ToolDefinition;
use alou_code_runtime::PermissionMode;
use serde_json::{json, Value};
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_filesystem".to_string();
    let description = "Desktop filesystem operations: read, write, edit, list, copy, move, delete files and directories".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["read", "write", "edit", "list", "copy", "move", "delete", "dir"]
            },
            "path": { "type": "string" },
            "content": { "type": "string" },
            "old_text": { "type": "string" },
            "new_text": { "type": "string" },
            "src": { "type": "string" },
            "dest": { "type": "string" },
            "recursive": { "type": "boolean", "default": false },
            "create_dirs": { "type": "boolean", "default": false },
            "depth": { "type": "integer" }
        },
        "required": ["operation", "path"]
    });
    let permission = PermissionMode::WorkspaceWrite;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(move |input: &Value| {
        let op = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("read");
        let path = input.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        match op {
            "read" => {
                let content = runtime.block_on(async {
                    fs::read_to_string(path).await
                }).map_err(|e| format!("Failed to read file: {}", e))?;
                let result = json!({
                    "success": true,
                    "content": content,
                    "path": path
                });
                Ok(serde_json::to_string(&result)?)
            }
            "write" => {
                let content = input.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let create_dirs = input.get("create_dirs")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                runtime.block_on(async {
                    if create_dirs {
                        if let Some(parent) = Path::new(path).parent() {
                            fs::create_dir_all(parent).await
                                .map_err(|e| format!("Failed to create directories: {}", e))?;
                        }
                    }
                    fs::write(path, content).await
                }).map_err(|e| format!("Failed to write file: {}", e))?;

                let result = json!({
                    "success": true,
                    "bytes_written": content.len(),
                    "path": path
                });
                Ok(serde_json::to_string(&result)?)
            }
            "edit" => {
                let old_text = input.get("old_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let new_text = input.get("new_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let content = runtime.block_on(async {
                    fs::read_to_string(path).await
                }).map_err(|e| format!("Failed to read file: {}", e))?;

                if !content.contains(old_text) {
                    return Err("Old text not found in file".to_string());
                }

                let new_content = content.replace(old_text, new_text);
                runtime.block_on(async {
                    fs::write(path, new_content).await
                }).map_err(|e| format!("Failed to write file: {}", e))?;

                let result = json!({
                    "success": true,
                    "bytes_replaced": old_text.len(),
                    "path": path
                });
                Ok(serde_json::to_string(&result)?)
            }
            "list" => {
                let recursive = input.get("recursive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let entries = runtime.block_on(async {
                    let mut items = Vec::new();
                    if recursive {
                        let mut walker = WalkDir::new(path).into_iter();
                        while let Some(entry) = walker.next() {
                            if let Ok(entry) = entry {
                                let metadata = entry.metadata().ok();
                                items.push(serde_json::json!({
                                    "path": entry.path().to_string_lossy(),
                                    "name": entry.file_name().to_string_lossy(),
                                    "is_dir": metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                                    "is_file": metadata.as_ref().map(|m| m.is_file()).unwrap_or(false),
                                }));
                            }
                        }
                    } else {
                        let mut entries = fs::read_dir(path).await
                            .map_err(|e| format!("Failed to read directory: {}", e))?;
                        while let Some(entry) = entries.next_entry().await
                            .map_err(|e| format!("Failed to read entry: {}", e))? {
                            let metadata = entry.metadata().await
                                .map_err(|e| format!("Failed to read metadata: {}", e))?;
                            items.push(serde_json::json!({
                                "path": entry.path().to_string_lossy(),
                                "name": entry.file_name().to_string_lossy(),
                                "is_dir": metadata.is_dir(),
                                "is_file": metadata.is_file(),
                            }));
                        }
                    }
                    Ok::<_, String>(items)
                })?;

                let result = json!({
                    "success": true,
                    "path": path,
                    "items": entries,
                    "count": entries.len()
                });
                Ok(serde_json::to_string(&result)?)
            }
            "copy" => {
                let src = input.get("src")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let dest = input.get("dest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let recursive = input.get("recursive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let bytes = runtime.block_on(async {
                    if recursive && Path::new(src).is_dir() {
                        copy_dir_recursive(src, dest).await
                    } else {
                        fs::copy(src, dest).await
                    }
                }).map_err(|e| format!("Failed to copy: {}", e))?;

                let result = json!({
                    "success": true,
                    "bytes_copied": bytes,
                    "src": src,
                    "dest": dest
                });
                Ok(serde_json::to_string(&result)?)
            }
            "move" => {
                let src = input.get("src")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let dest = input.get("dest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                runtime.block_on(async {
                    fs::rename(src, dest).await
                }).map_err(|e| format!("Failed to move: {}", e))?;

                let result = json!({
                    "success": true,
                    "src": src,
                    "dest": dest
                });
                Ok(serde_json::to_string(&result)?)
            }
            "delete" => {
                let recursive = input.get("recursive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                runtime.block_on(async {
                    if Path::new(path).is_dir() {
                        if recursive {
                            fs::remove_dir_all(path).await
                        } else {
                            fs::remove_dir(path).await
                        }
                    } else {
                        fs::remove_file(path).await
                    }
                }).map_err(|e| format!("Failed to delete: {}", e))?;

                let result = json!({
                    "success": true,
                    "path": path
                });
                Ok(serde_json::to_string(&result)?)
            }
            "dir" => {
                let info = runtime.block_on(async {
                    let metadata = fs::metadata(path).await
                        .map_err(|e| format!("Failed to get metadata: {}", e))?;
                    Ok::<_, String>(serde_json::json!({
                        "path": path,
                        "is_dir": metadata.is_dir(),
                        "size": metadata.len(),
                    }))
                })?;

                let result = json!({
                    "success": true,
                    "info": info
                });
                Ok(serde_json::to_string(&result)?)
            }
            _ => Err(format!("Unknown operation: {}", op))
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

fn copy_dir_recursive(src: &str, dest: &str) -> std::io::Result<u64> {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        fs::create_dir_all(dest).await?;
        let mut total_bytes = 0u64;
        let mut dirs = vec![(src.to_string(), dest.to_string())];

        while let Some((src_dir, dest_dir)) = dirs.pop() {
            let mut entries = fs::read_dir(&src_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let src_path = entry.path();
                let dest_path = Path::new(&dest_dir).join(entry.file_name());
                let metadata = entry.metadata().await?;
                if metadata.is_dir() {
                    fs::create_dir_all(&dest_path).await?;
                    dirs.push((src_path.to_str().unwrap().to_string(), dest_path.to_str().unwrap().to_string()));
                } else {
                    total_bytes += fs::copy(&src_path, &dest_path).await?;
                }
            }
        }
        Ok(total_bytes)
    })
}