//! 文件系统工具
//!
//! 提供文件和目录操作功能：读取、写入、编辑、列出、复制、移动、删除等

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 文件系统工具
pub struct FileSystemTool {
    metadata: ToolMetadata,
}

impl FileSystemTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "filesystem".to_string(),
                name: "File System Tool".to_string(),
                description: "Comprehensive file system operations: read, write, edit, list, copy, move, delete".to_string(),
                category: ToolCategory::FileSystem,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "0.1.2".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["file_read".to_string(), "file_write".to_string()],
            },
        }
    }

    /// 读取文件
    async fn read_file(&self, path: &str) -> Result<String, ToolError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;
        Ok(content)
    }

    /// 写入文件
    async fn write_file(&self, path: &str, content: &str, create_dirs: bool) -> Result<(), ToolError> {
        if create_dirs {
            let parent = Path::new(path).parent()
                .ok_or_else(|| ToolError::ExecutionFailed("Invalid path".to_string()))?;
            fs::create_dir_all(parent).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create directories: {}", e)))?;
        }

        fs::write(path, content)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// 编辑文件（精确替换）
    async fn edit_file(&self, path: &str, old_text: &str, new_text: &str) -> Result<usize, ToolError> {
        let content = self.read_file(path).await?;

        if !content.contains(old_text) {
            return Err(ToolError::ExecutionFailed("Old text not found in file".to_string()));
        }

        let new_content = content.replace(old_text, new_text);
        fs::write(path, new_content).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;

        Ok(old_text.len())
    }


    /// 按行删除
    async fn delete_lines(&self, path: &str, start_line: usize, end_line: Option<usize>) -> Result<usize, ToolError> {
        let content = self.read_file(path).await?;
        let lines: Vec<&str> = content.lines().collect();

        if start_line == 0 {
            return Err(ToolError::ExecutionFailed("Start line must be greater than 0".to_string()));
        }

        let end_line = end_line.unwrap_or(start_line);
        if end_line < start_line {
            return Err(ToolError::ExecutionFailed("End line must be greater than or equal to start line".to_string()));
        }

        if start_line > lines.len() {
            return Err(ToolError::ExecutionFailed(format!("Start line {} exceeds file length {}", start_line, lines.len())));
        }

        let actual_end_line = end_line.min(lines.len());
        let deleted_lines = actual_end_line - start_line + 1;

        let new_content = lines.into_iter()
            .enumerate()
            .filter(|(i, _)| *i < start_line - 1 || *i > actual_end_line - 1)
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(path, new_content).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;

        Ok(deleted_lines)
    }

    /// 按块删除
    async fn delete_block(&self, path: &str, block_text: &str, all_occurrences: bool) -> Result<usize, ToolError> {
        let content = self.read_file(path).await?;

        if !content.contains(block_text) {
            return Err(ToolError::ExecutionFailed(format!("Block text '{}' not found", block_text)));
        }

        let (new_content, deleted_count) = if all_occurrences {
            // 删除所有匹配的块
            let new_content = content.replace(block_text, "");
            let deleted_count = content.matches(block_text).count();
            (new_content, deleted_count)
        } else {
            // 只删除第一个匹配的块
            if let Some(pos) = content.find(block_text) {
                let new_content = format!("{}{}",
                    &content[..pos],
                    &content[pos + block_text.len()..]
                );
                (new_content, 1)
            } else {
                (content, 0)
            }
        };

        fs::write(path, new_content).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;

        Ok(deleted_count)
    }

    /// 列出目录内容
    async fn list_directory(&self, path: &str, recursive: bool, depth: Option<usize>) -> Result<Vec<FileInfo>, ToolError> {
        let mut files = Vec::new();

        if recursive {
            self.list_recursive(path, depth.unwrap_or(usize::MAX), 0, &mut files).await?;
        } else {
            let mut entries = fs::read_dir(path)
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))? {
                let metadata = entry.metadata().await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read metadata: {}", e)))?;

                files.push(FileInfo {
                    path: entry.path().to_string_lossy().to_string(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_dir: metadata.is_dir(),
                    is_file: metadata.is_file(),
                    size: metadata.len(),
                    modified: metadata.modified()
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .map(|d| d.as_secs()),
                });
            }
        }

        Ok(files)
    }

    /// 递归列出目录
    fn list_recursive<'a>(&'a self, path: &'a str, max_depth: usize, current_depth: usize, files: &'a mut Vec<FileInfo>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ToolError>> + Send + '_>> {
        Box::pin(async move {
            if current_depth >= max_depth {
                return Ok(());
            }

            let mut entries = fs::read_dir(path)
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))? {
                let metadata = entry.metadata().await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read metadata: {}", e)))?;

                let path_str = entry.path().to_string_lossy().to_string();
                files.push(FileInfo {
                    path: path_str.clone(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_dir: metadata.is_dir(),
                    is_file: metadata.is_file(),
                    size: metadata.len(),
                    modified: metadata.modified()
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .map(|d| d.as_secs()),
                });

                if metadata.is_dir() {
                    self.list_recursive(&path_str, max_depth, current_depth + 1, files).await?;
                }
            }

            Ok(())
        })
    }

    /// 复制文件/目录
    async fn copy_item(&self, src: &str, dest: &str, recursive: bool) -> Result<u64, ToolError> {
        if recursive && Path::new(src).is_dir() {
            self.copy_recursive(src, dest).await
        } else {
            let bytes = fs::copy(src, dest).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to copy: {}", e)))?;
            Ok(bytes)
        }
    }

    /// 递归复制目录
    fn copy_recursive<'a>(&'a self, src: &'a str, dest: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64, ToolError>> + Send + '_>> {
        Box::pin(async move {
            fs::create_dir_all(dest).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create directory: {}", e)))?;

            let mut entries = fs::read_dir(src)
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;

            let mut total_bytes = 0u64;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))? {
                let src_path = entry.path();
                let dest_path = Path::new(dest).join(entry.file_name());

                let metadata = entry.metadata().await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read metadata: {}", e)))?;

                if metadata.is_dir() {
                    total_bytes += self.copy_recursive(
                        src_path.to_str().unwrap(),
                        dest_path.to_str().unwrap()
                    ).await?;
                } else {
                    let bytes = fs::copy(&src_path, &dest_path).await
                        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to copy file: {}", e)))?;
                    total_bytes += bytes;
                }
            }

            Ok(total_bytes)
        })
    }

    /// 移动文件/目录
    async fn move_item(&self, src: &str, dest: &str) -> Result<(), ToolError> {
        fs::rename(src, dest).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to move: {}", e)))?;
        Ok(())
    }

    /// 删除文件/目录
    async fn delete_item(&self, path: &str, recursive: bool) -> Result<(), ToolError> {
        if Path::new(path).is_dir() {
            if recursive {
                fs::remove_dir_all(path).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to remove directory: {}", e)))?;
            } else {
                fs::remove_dir(path).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to remove directory: {}", e)))?;
            }
        } else {
            fs::remove_file(path).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to remove file: {}", e)))?;
        }
        Ok(())
    }

    /// 获取目录信息
    async fn get_dir_info(&self, path: &str) -> Result<DirInfo, ToolError> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get metadata: {}", e)))?;

        let mut file_count = 0usize;
        let mut dir_count = 0usize;
        let mut total_size = 0u64;

        if metadata.is_dir() {
            let mut entries = fs::read_dir(path).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))? {
                let entry_metadata = entry.metadata().await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read metadata: {}", e)))?;

                if entry_metadata.is_dir() {
                    dir_count += 1;
                } else {
                    file_count += 1;
                    total_size += entry_metadata.len();
                }
            }
        }

        Ok(DirInfo {
            path: path.to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            file_count,
            dir_count,
            total_size,
            modified: metadata.modified()
                .ok()
                .and_then(|t| t.elapsed().ok())
                .map(|d| d.as_secs()),
        })
    }
}

#[async_trait]
impl ToolExecutor for FileSystemTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let operation: FileOperation = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        match operation {
            FileOperation::Read { path } => {
                let content = self.read_file(&path).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "content": content,
                        "path": path,
                        "operation": "read"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Read {} bytes", content.len())),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::Write { path, content, create_dirs } => {
                self.write_file(&path, &content, create_dirs).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "size": content.len(),
                        "operation": "write"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Wrote {} bytes to {}", content.len(), path)),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::Edit { path, old_text, new_text } => {
                let bytes_replaced = self.edit_file(&path, &old_text, &new_text).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "bytes_replaced": bytes_replaced,
                        "operation": "edit"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Replaced {} bytes in {}", bytes_replaced, path)),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::DeleteLines { path, start_line, end_line } => {
                let lines_deleted = self.delete_lines(&path, start_line, end_line).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "lines_deleted": lines_deleted,
                        "start_line": start_line,
                        "end_line": end_line,
                        "operation": "delete_lines"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Deleted {} lines from {} (lines {}-{})", lines_deleted, path, start_line, end_line.unwrap_or(start_line))),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::DeleteBlock { path, block_text, all_occurrences } => {
                let blocks_deleted = self.delete_block(&path, &block_text, all_occurrences).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "blocks_deleted": blocks_deleted,
                        "all_occurrences": all_occurrences,
                        "operation": "delete_block"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Deleted {} block(s) from {}", blocks_deleted, path)),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::List { path, recursive, depth } => {
                let files = self.list_directory(&path, recursive, depth).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "files": files,
                        "count": files.len(),
                        "operation": "list"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Listed {} items", files.len())),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::Copy { src, dest, recursive } => {
                let bytes_copied = self.copy_item(&src, &dest, recursive).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "src": src,
                        "dest": dest,
                        "bytes": bytes_copied,
                        "operation": "copy"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Copied {} bytes from {} to {}", bytes_copied, src, dest)),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::Move { src, dest } => {
                self.move_item(&src, &dest).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "src": src,
                        "dest": dest,
                        "operation": "move"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Moved {} to {}", src, dest)),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::Delete { path, recursive } => {
                self.delete_item(&path, recursive).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "operation": "delete"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Deleted {}", path)),
                    warnings: vec![],
                    context: None,
                })
            },

            FileOperation::Dir { path } => {
                let info = self.get_dir_info(&path).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "info": info,
                        "operation": "dir"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Directory info for {}", path)),
                    warnings: vec![],
                    context: None,
                })
            },
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if serde_json::from_value::<FileOperation>(args.clone()).is_ok() {
            Ok(())
        } else {
            Err(ToolError::InvalidArguments("Invalid file system operation arguments".to_string()))
        }
    }

    fn help(&self) -> String {
        r#"File System Tool - Comprehensive file and directory operations

Available operations:
- read: Read file contents
- write: Write content to a file (create if not exists)
- edit: Edit file by exact text replacement
- delete_lines: Delete specific lines from a file
- delete_block: Delete text blocks from a file
- list: List directory contents
- copy: Copy file/directory
- move: Move/rename file/directory
- delete: Delete file/directory
- dir: Get directory information

Example usage:
{
  "operation": "read",
  "path": "/path/to/file.txt"
}

{
  "operation": "write",
  "path": "/path/to/file.txt",
  "content": "Hello, World!",
  "create_dirs": true
}

{
  "operation": "delete_lines",
  "path": "/path/to/file.txt",
  "start_line": 5,
  "end_line": 10
}

{
  "operation": "delete_block",
  "path": "/path/to/file.txt",
  "block_text": "function oldFunction() {\n  // old code\n}",
  "all_occurrences": false
}

{
  "operation": "list",
  "path": "/path/to/dir",
  "recursive": true,
  "depth": 2
}"#.to_string()
    }
}

/// 文件操作
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation")]
pub enum FileOperation {
    /// 读取文件
    Read { path: String },
    /// 写入文件
    Write {
        path: String,
        content: String,
        #[serde(default)]
        create_dirs: bool,
    },
    /// 编辑文件
    Edit {
        path: String,
        old_text: String,
        new_text: String,
    },
    /// 按行删除
    DeleteLines {
        path: String,
        /// 起始行号（1-based）
        start_line: usize,
        /// 结束行号（包含，可选，为None时只删除start_line行）
        end_line: Option<usize>,
    },
    /// 按文本块删除
    DeleteBlock {
        path: String,
        /// 要删除的文本块
        block_text: String,
        /// 是否删除所有匹配的块（默认false，只删除第一个）
        #[serde(default)]
        all_occurrences: bool,
    },
    /// 列出目录
    List {
        path: String,
        #[serde(default)]
        recursive: bool,
        #[serde(default)]
        depth: Option<usize>,
    },
    /// 复制
    Copy {
        src: String,
        dest: String,
        #[serde(default)]
        recursive: bool,
    },
    /// 移动
    Move {
        src: String,
        dest: String,
    },
    /// 删除
    Delete {
        path: String,
        #[serde(default)]
        recursive: bool,
    },
    /// 目录信息
    Dir { path: String },
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 完整路径
    pub path: String,
    /// 文件名
    pub name: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 是否为文件
    pub is_file: bool,
    /// 大小（字节）
    pub size: u64,
    /// 修改时间（秒）
    pub modified: Option<u64>,
}

/// 目录信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirInfo {
    /// 路径
    pub path: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 大小
    pub size: u64,
    /// 文件数
    pub file_count: usize,
    /// 目录数
    pub dir_count: usize,
    /// 总大小
    pub total_size: u64,
    /// 修改时间
    pub modified: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filesystem_operations() {
        let tool = FileSystemTool::new();

        // 创建临时目录
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // 写入文件
        let args = serde_json::json!({
            "operation": "write",
            "path": test_file.to_str().unwrap(),
            "content": "Hello, World!"
        });

        let context = ExecutionContext {
            session_id: "test".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(10),
            permissions: vec!["file_read".to_string(), "file_write".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);

        // 读取文件
        let args = serde_json::json!({
            "operation": "read",
            "path": test_file.to_str().unwrap()
        });

        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.data["content"], "Hello, World!");

        // 编辑文件
        let args = serde_json::json!({
            "operation": "edit",
            "path": test_file.to_str().unwrap(),
            "old_text": "World",
            "new_text": "Rust"
        });

        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());

        // 测试按行删除
        let args = serde_json::json!({
            "operation": "delete_lines",
            "path": test_file.to_str().unwrap(),
            "start_line": 1,
            "end_line": 1
        });

        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());

        // 测试按文本块删除
        let args = serde_json::json!({
            "operation": "delete_block",
            "path": test_file.to_str().unwrap(),
            "block_text": "Hello",
            "all_occurrences": false
        });

        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());

        // 列出目录
        let args = serde_json::json!({
            "operation": "list",
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());
    }
}