use crate::mcp::executor::{Tool, ToolContext, ToolError, ToolResult};
use serde_json::json;
use std::path::Path;
use tokio::fs;

pub struct FilesystemTool;

impl Tool for FilesystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "当需要读取/写入/编辑文件或验证文件内容时优先使用。支持读取、写入、编辑、列出、复制、移动、删除文件和目录。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": { 
                    "type": "string", 
                    "description": "文件操作类型", 
                    "enum": ["read", "write", "edit", "delete_lines", "delete_block", "list", "copy", "move", "delete", "dir"] 
                },
                "path": { 
                    "type": "string", 
                    "description": "文件或目录路径" 
                },
                "content": { 
                    "type": "string", 
                    "description": "写入的内容（write操作需要）" 
                },
                "create_dirs": { 
                    "type": "boolean", 
                    "description": "是否创建父目录（write操作可选）" 
                },
                "old_text": { 
                    "type": "string", 
                    "description": "要替换的旧文本（edit操作需要）" 
                },
                "new_text": { 
                    "type": "string", 
                    "description": "替换后的新文本（edit操作需要）" 
                },
                "start_line": { 
                    "type": "number", 
                    "description": "起始行号（delete_lines操作需要）" 
                },
                "end_line": { 
                    "type": "number", 
                    "description": "结束行号（delete_lines操作可选）" 
                },
                "block_text": { 
                    "type": "string", 
                    "description": "要删除的文本块（delete_block操作需要）" 
                },
                "all_occurrences": { 
                    "type": "boolean", 
                    "description": "是否删除所有匹配的块（delete_block操作可选）" 
                },
                "recursive": { 
                    "type": "boolean", 
                    "description": "是否递归操作（list/copy/delete操作可选）" 
                },
                "depth": { 
                    "type": "number", 
                    "description": "递归深度限制（list操作可选）" 
                },
                "src": { 
                    "type": "string", 
                    "description": "源路径（copy/move操作需要）" 
                },
                "dest": { 
                    "type": "string", 
                    "description": "目标路径（copy/move操作需要）" 
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'operation' parameter".to_string()))?;

        let path_str = args.get("path").and_then(|v| v.as_str());

        match operation {
            "read" => {
                let path = path_str.ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'path' parameter for read operation".to_string())
                })?;
                
                match fs::read_to_string(path).await {
                    Ok(content) => Ok(ToolResult {
                        success: true,
                        data: json!({
                            "content": content,
                            "path": path,
                            "operation": "read"
                        }),
                        error: None,
                    }),
                    Err(e) => Err(ToolError::ExecutionFailed(format!("Failed to read file: {}", e))),
                }
            }
            "write" => {
                let path = path_str.ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'path' parameter for write operation".to_string())
                })?;
                
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'content' parameter for write operation".to_string()))?;
                
                let create_dirs = args.get("create_dirs").and_then(|v| v.as_bool()).unwrap_or(false);
                
                if create_dirs {
                    if let Some(parent) = Path::new(path).parent() {
                        fs::create_dir_all(parent).await.map_err(|e| {
                            ToolError::ExecutionFailed(format!("Failed to create directories: {}", e))
                        })?;
                    }
                }
                
                match fs::write(path, content).await {
                    Ok(_) => Ok(ToolResult {
                        success: true,
                        data: json!({
                            "path": path,
                            "operation": "write",
                            "bytes_written": content.len()
                        }),
                        error: None,
                    }),
                    Err(e) => Err(ToolError::ExecutionFailed(format!("Failed to write file: {}", e))),
                }
            }
            "list" | "dir" => {
                let path = path_str.unwrap_or(".");
                let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
                
                match self.list_directory(path, recursive).await {
                    Ok(entries) => Ok(ToolResult {
                        success: true,
                        data: json!({
                            "path": path,
                            "operation": "list",
                            "entries": entries
                        }),
                        error: None,
                    }),
                    Err(e) => Err(ToolError::ExecutionFailed(format!("Failed to list directory: {}", e))),
                }
            }
            "delete" => {
                let path = path_str.ok_or_else(|| {
                    ToolError::InvalidArguments("Missing 'path' parameter for delete operation".to_string())
                })?;
                
                let metadata = fs::metadata(path).await.map_err(|e| {
                    ToolError::ExecutionFailed(format!("Failed to get metadata: {}", e))
                })?;
                
                if metadata.is_dir() {
                    fs::remove_dir_all(path).await.map_err(|e| {
                        ToolError::ExecutionFailed(format!("Failed to delete directory: {}", e))
                    })?;
                } else {
                    fs::remove_file(path).await.map_err(|e| {
                        ToolError::ExecutionFailed(format!("Failed to delete file: {}", e))
                    })?;
                }
                
                Ok(ToolResult {
                    success: true,
                    data: json!({
                        "path": path,
                        "operation": "delete",
                        "type": if metadata.is_dir() { "directory" } else { "file" }
                    }),
                    error: None,
                })
            }
            "copy" => {
                let src = args
                    .get("src")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'src' parameter for copy operation".to_string()))?;
                
                let dest = args
                    .get("dest")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'dest' parameter for copy operation".to_string()))?;
                
                fs::copy(src, dest).await.map_err(|e| {
                    ToolError::ExecutionFailed(format!("Failed to copy: {}", e))
                })?;
                
                Ok(ToolResult {
                    success: true,
                    data: json!({
                        "src": src,
                        "dest": dest,
                        "operation": "copy"
                    }),
                    error: None,
                })
            }
            "move" => {
                let src = args
                    .get("src")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'src' parameter for move operation".to_string()))?;
                
                let dest = args
                    .get("dest")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'dest' parameter for move operation".to_string()))?;
                
                fs::rename(src, dest).await.map_err(|e| {
                    ToolError::ExecutionFailed(format!("Failed to move: {}", e))
                })?;
                
                Ok(ToolResult {
                    success: true,
                    data: json!({
                        "src": src,
                        "dest": dest,
                        "operation": "move"
                    }),
                    error: None,
                })
            }
            _ => Err(ToolError::InvalidArguments(format!("Unknown operation: {}", operation))),
        }
    }
}

impl FilesystemTool {
    async fn list_directory(&self, path: &str, recursive: bool) -> Result<Vec<serde_json::Value>, std::io::Error> {
        let mut entries = Vec::new();
        
        let mut dir = fs::read_dir(path).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            let metadata = entry.metadata().await?;
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = if metadata.is_dir() {
                "directory"
            } else if metadata.is_file() {
                "file"
            } else if metadata.is_symlink() {
                "symlink"
            } else {
                "unknown"
            };
            
            entries.push(json!({
                "name": name,
                "type": file_type,
                "size": metadata.len(),
                "modified": metadata.modified().ok().map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
                })
            }));
            
            if recursive && metadata.is_dir() {
                let sub_path = entry.path().to_string_lossy().to_string();
                match self.list_directory(&sub_path, true).await {
                    Ok(sub_entries) => {
                        for mut sub_entry in sub_entries {
                            if let Some(obj) = sub_entry.as_object_mut() {
                                obj.insert("parent".to_string(), json!(name));
                            }
                            entries.push(sub_entry);
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        
        Ok(entries)
    }
}
