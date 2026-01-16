//! 搜索工具
//!
//! 提供文本搜索、文件模式匹配、高级文件查找功能

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use glob::Pattern;
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// 搜索工具
pub struct SearchTool {
    metadata: ToolMetadata,
}

impl SearchTool {
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "search".to_string(),
                name: "Search Tool".to_string(),
                description: "Text search, file pattern matching, and advanced file finding".to_string(),
                category: ToolCategory::Search,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["file_read".to_string()],
            },
        }
    }

    /// Grep - 文本搜索
    async fn grep(&self, pattern: &str, directory: &str, file_pattern: Option<&str>, case_sensitive: bool, max_results: Option<usize>) -> Result<Vec<GrepMatch>, ToolError> {
        let regex = Regex::new(pattern)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid regex pattern: {}", e)))?;

        let mut matches = Vec::new();

        let mut walker = WalkDir::new(directory)
            .follow_links(true)
            .into_iter();

        while let Some(entry) = walker.next() {
            let entry = entry.map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))?;

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();

            // 文件模式过滤
            if let Some(pattern) = file_pattern {
                if let Ok(glob_pattern) = Pattern::new(pattern) {
                    if !glob_pattern.matches_path(path) {
                        continue;
                    }
                }
            }

            // 读取文件内容
            let content = match File::open(path).await {
                Ok(mut file) => {
                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer).await
                        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;
                    buffer
                }
                Err(_) => continue, // 跳过无法读取的文件
            };

            // 搜索匹配
            for (line_num, line) in content.lines().enumerate() {
                let search_line = if case_sensitive { line } else { &line.to_lowercase() };
                let search_pattern = if case_sensitive { pattern } else { &pattern.to_lowercase() };

                if search_line.contains(search_pattern) {
                    matches.push(GrepMatch {
                        path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line: line.to_string(),
                        match_start: line.find(search_pattern).unwrap_or(0),
                        match_length: search_pattern.len(),
                    });

                    if let Some(max) = max_results {
                        if matches.len() >= max {
                            return Ok(matches);
                        }
                    }
                }
            }
        }

        Ok(matches)
    }

    /// Glob - 文件名模式匹配
    async fn glob(&self, pattern: &str, directory: &str, recursive: bool) -> Result<Vec<String>, ToolError> {
        let glob_pattern = Pattern::new(pattern)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid glob pattern: {}", e)))?;

        let mut matches = Vec::new();

        if recursive {
            let walker = WalkDir::new(directory).follow_links(true);
            for entry in walker {
                let entry = entry.map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))?;
                let path = entry.path();

                if glob_pattern.matches_path(path) {
                    matches.push(path.to_string_lossy().to_string());
                }
            }
        } else {
            let mut entries = tokio::fs::read_dir(directory).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))? {
                let path = entry.path();
                if glob_pattern.matches_path(&path) {
                    matches.push(path.to_string_lossy().to_string());
                }
            }
        }

        Ok(matches)
    }

    /// Find - 高级文件查找
    async fn find(&self, directory: &str, options: FindOptions) -> Result<Vec<FileInfo>, ToolError> {
        let mut files = Vec::new();

        let mut walker = WalkDir::new(directory)
            .follow_links(true)
            .into_iter();

        while let Some(entry) = walker.next() {
            let entry = entry.map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))?;

            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            // 类型过滤
            if options.file_type != FileType::Any {
                let is_match = match options.file_type {
                    FileType::File => metadata.is_file(),
                    FileType::Directory => metadata.is_dir(),
                    FileType::Any => true,
                    FileType::Symlink => metadata.file_type().is_symlink(),
                };
                if !is_match {
                    continue;
                }
            }

            // 名称模式过滤
            if let Some(pattern) = &options.name_pattern {
                if !Pattern::new(pattern).map_err(|e| ToolError::InvalidArguments(format!("Invalid pattern: {}", e)))?
                    .matches_path(path) {
                    continue;
                }
            }

            // 大小过滤
            if let Some(min_size) = options.min_size {
                if metadata.len() < min_size {
                    continue;
                }
            }
            if let Some(max_size) = options.max_size {
                if metadata.len() > max_size {
                    continue;
                }
            }

            // 内容过滤（仅文件）
            if let Some(content_pattern) = &options.content_pattern {
                if !metadata.is_file() {
                    continue;
                }

                let content = match File::open(path).await {
                    Ok(mut file) => {
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer).await.map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;
                        buffer
                    }
                    Err(_) => continue,
                };

                if !content.contains(content_pattern) {
                    continue;
                }
            }

            // 深度限制
            if let Some(max_depth) = options.max_depth {
                if entry.depth() > max_depth {
                    continue;
                }
            }

            files.push(FileInfo {
                path: path.to_string_lossy().to_string(),
                name: path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                size: metadata.len(),
                is_dir: metadata.is_dir(),
                is_file: metadata.is_file(),
                depth: entry.depth(),
                modified: metadata.modified()
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .map(|d| d.as_secs()),
            });

            // 结果数量限制
            if let Some(max_results) = options.max_results {
                if files.len() >= max_results {
                    break;
                }
            }
        }

        Ok(files)
    }
}

#[async_trait]
impl ToolExecutor for SearchTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let operation: SearchOperation = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        match operation {
            SearchOperation::Grep {
                pattern,
                directory,
                file_pattern,
                case_sensitive,
                max_results,
            } => {
                let matches = self.grep(&pattern, &directory, file_pattern.as_deref(), case_sensitive, max_results).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "matches": matches,
                        "count": matches.len(),
                        "pattern": pattern,
                        "directory": directory,
                        "operation": "grep"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} matches", matches.len())),
                    warnings: vec![],
                    context: None,
                })
            },

            SearchOperation::Glob {
                pattern,
                directory,
                recursive,
            } => {
                let matches = self.glob(&pattern, &directory, recursive).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "matches": matches,
                        "count": matches.len(),
                        "pattern": pattern,
                        "directory": directory,
                        "operation": "glob"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} files", matches.len())),
                    warnings: vec![],
                    context: None,
                })
            },

            SearchOperation::Find { directory, options } => {
                let files = self.find(&directory, options).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "files": files,
                        "count": files.len(),
                        "directory": directory,
                        "operation": "find"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} items", files.len())),
                    warnings: vec![],
                    context: None,
                })
            },
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if serde_json::from_value::<SearchOperation>(args.clone()).is_ok() {
            Ok(())
        } else {
            Err(ToolError::InvalidArguments("Invalid search operation arguments".to_string()))
        }
    }

    fn help(&self) -> String {
        r#"Search Tool - Text search, file pattern matching, and advanced file finding

Available operations:
- grep: Search for text in files
- glob: Match files by name pattern
- find: Advanced file search with filters

Example usage:
{
  "operation": "grep",
  "pattern": "TODO",
  "directory": "/path/to/search",
  "file_pattern": "*.rs",
  "case_sensitive": false,
  "max_results": 100
}

{
  "operation": "glob",
  "pattern": "*.rs",
  "directory": "/path/to/search",
  "recursive": true
}

{
  "operation": "find",
  "directory": "/path/to/search",
  "options": {
    "file_type": "file",
    "name_pattern": "*.rs",
    "min_size": 1024,
    "max_depth": 3,
    "max_results": 50
  }
}

Supported glob patterns:
- *.rs - All Rust files
- test_*.* - Files starting with "test_"
- **/target/** - Files in "target" directories (recursive)
- src/**/*.rs - Rust files in src directory (recursive)"#.to_string()
    }
}

/// 搜索操作
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation")]
pub enum SearchOperation {
    /// 文本搜索
    Grep {
        pattern: String,
        directory: String,
        #[serde(default)]
        file_pattern: Option<String>,
        #[serde(default)]
        case_sensitive: bool,
        #[serde(default)]
        max_results: Option<usize>,
    },
    /// 文件模式匹配
    Glob {
        pattern: String,
        directory: String,
        #[serde(default)]
        recursive: bool,
    },
    /// 高级查找
    Find {
        directory: String,
        options: FindOptions,
    },
}

/// Grep 匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepMatch {
    /// 文件路径
    pub path: String,
    /// 行号
    pub line_number: usize,
    /// 行内容
    pub line: String,
    /// 匹配开始位置
    pub match_start: usize,
    /// 匹配长度
    pub match_length: usize,
}

/// Find 选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindOptions {
    /// 文件类型
    #[serde(default)]
    pub file_type: FileType,
    /// 名称模式
    #[serde(default)]
    pub name_pattern: Option<String>,
    /// 内容模式
    #[serde(default)]
    pub content_pattern: Option<String>,
    /// 最小大小
    #[serde(default)]
    pub min_size: Option<u64>,
    /// 最大大小
    #[serde(default)]
    pub max_size: Option<u64>,
    /// 最大深度
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// 最大结果数
    #[serde(default)]
    pub max_results: Option<usize>,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            file_type: FileType::Any,
            name_pattern: None,
            content_pattern: None,
            min_size: None,
            max_size: None,
            max_depth: None,
            max_results: None,
        }
    }
}

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Any,
    File,
    Directory,
    Symlink,
}

impl Default for FileType {
    fn default() -> Self {
        FileType::Any
    }
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 路径
    pub path: String,
    /// 文件名
    pub name: String,
    /// 大小
    pub size: u64,
    /// 是否为目录
    pub is_dir: bool,
    /// 是否为文件
    pub is_file: bool,
    /// 深度
    pub depth: usize,
    /// 修改时间
    pub modified: Option<u64>,
}

/// 搜索模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPattern {
    /// 模式字符串
    pub pattern: String,
    /// 是否为正则表达式
    #[serde(default)]
    pub is_regex: bool,
    /// 是否区分大小写
    #[serde(default)]
    pub case_sensitive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_glob_search() {
        let tool = SearchTool::new();

        let context = ExecutionContext {
            session_id: "test".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(10),
            permissions: vec!["file_read".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let args = serde_json::json!({
            "operation": "glob",
            "pattern": "*.rs",
            "directory": "src",
            "recursive": true
        });

        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());
    }
}
