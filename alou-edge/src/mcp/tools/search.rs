use crate::mcp::executor::{Tool, ToolContext, ToolError, ToolResult};
use regex::Regex;
use serde_json::json;
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

pub struct SearchTool;

impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "当需要确认项目结构、定位代码或查找关键文本时优先使用。支持文本搜索、文件模式匹配和高级文件查找。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": { 
                    "type": "string", 
                    "description": "搜索操作类型", 
                    "enum": ["grep", "glob", "find"] 
                },
                "pattern": { 
                    "type": "string", 
                    "description": "搜索模式（grep/glob操作需要）" 
                },
                "directory": { 
                    "type": "string", 
                    "description": "搜索目录" 
                },
                "file_pattern": { 
                    "type": "string", 
                    "description": "文件模式过滤（grep操作可选）" 
                },
                "case_sensitive": { 
                    "type": "boolean", 
                    "description": "是否区分大小写（grep操作可选）" 
                },
                "max_results": { 
                    "type": "number", 
                    "description": "最大结果数（grep/find操作可选）" 
                },
                "recursive": { 
                    "type": "boolean", 
                    "description": "是否递归搜索（glob操作可选）" 
                },
                "options": { 
                    "type": "object", 
                    "description": "高级查找选项（find操作需要）", 
                    "properties": {
                        "file_type": { "type": "string", "enum": ["any", "file", "directory", "symlink"] },
                        "name_pattern": { "type": "string" },
                        "content_pattern": { "type": "string" },
                        "min_size": { "type": "number" },
                        "max_size": { "type": "number" },
                        "max_depth": { "type": "number" },
                        "max_results": { "type": "number" }
                    }
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

        let directory = args.get("directory").and_then(|v| v.as_str()).unwrap_or(".");
        let max_results = args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        match operation {
            "grep" => {
                let pattern = args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'pattern' parameter for grep operation".to_string()))?;
                
                let file_pattern = args.get("file_pattern").and_then(|v| v.as_str());
                let case_sensitive = args.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(true);
                
                let regex = if case_sensitive {
                    Regex::new(pattern).map_err(|e| ToolError::InvalidArguments(format!("Invalid regex pattern: {}", e)))?
                } else {
                    Regex::new(&format!("(?i){}", pattern)).map_err(|e| ToolError::InvalidArguments(format!("Invalid regex pattern: {}", e)))?
                };
                
                let results = self.grep_search(directory, &regex, file_pattern, max_results).await?;
                
                Ok(ToolResult {
                    success: true,
                    data: json!({
                        "operation": "grep",
                        "pattern": pattern,
                        "directory": directory,
                        "matches": results,
                        "total_matches": results.len()
                    }),
                    error: None,
                })
            }
            "glob" => {
                let pattern = args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'pattern' parameter for glob operation".to_string()))?;
                
                let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(true);
                
                let results = self.glob_search(directory, pattern, recursive, max_results).await?;
                
                Ok(ToolResult {
                    success: true,
                    data: json!({
                        "operation": "glob",
                        "pattern": pattern,
                        "directory": directory,
                        "matches": results,
                        "total_matches": results.len()
                    }),
                    error: None,
                })
            }
            "find" => {
                let options = args.get("options").cloned().unwrap_or(json!({}));
                let results = self.find_files(directory, &options, max_results).await?;
                
                Ok(ToolResult {
                    success: true,
                    data: json!({
                        "operation": "find",
                        "directory": directory,
                        "matches": results,
                        "total_matches": results.len()
                    }),
                    error: None,
                })
            }
            _ => Err(ToolError::InvalidArguments(format!("Unknown operation: {}", operation))),
        }
    }
}

impl SearchTool {
    async fn grep_search(
        &self,
        directory: &str,
        regex: &Regex,
        file_pattern: Option<&str>,
        max_results: usize,
    ) -> Result<Vec<serde_json::Value>, ToolError> {
        let mut matches = Vec::new();
        
        for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
            if matches.len() >= max_results {
                break;
            }
            
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            
            // Check file pattern
            if let Some(pattern) = file_pattern {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !file_name.contains(pattern) && !pattern.contains(file_name) {
                    continue;
                }
            }
            
            // Try to read and search file
            if let Ok(content) = fs::read_to_string(path).await {
                let file_path = path.to_string_lossy().to_string();
                let lines: Vec<&str> = content.lines().collect();
                
                for (line_num, line) in lines.iter().enumerate() {
                    if regex.is_match(line) {
                        matches.push(json!({
                            "file": file_path,
                            "line": line_num + 1,
                            "content": line.to_string(),
                            "match": regex.find(line).map(|m| m.as_str().to_string())
                        }));
                        
                        if matches.len() >= max_results {
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(matches)
    }
    
    async fn glob_search(
        &self,
        directory: &str,
        pattern: &str,
        recursive: bool,
        max_results: usize,
    ) -> Result<Vec<serde_json::Value>, ToolError> {
        let mut matches = Vec::new();
        
        let depth = if recursive { usize::MAX } else { 1 };
        
        for entry in WalkDir::new(directory).max_depth(depth).into_iter().filter_map(|e| e.ok()) {
            if matches.len() >= max_results {
                break;
            }
            
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            
            // Simple pattern matching
            let is_match = if pattern.contains("*") {
                // Convert glob pattern to simple check
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.len() == 2 {
                    file_name.starts_with(parts[0]) && file_name.ends_with(parts[1])
                } else {
                    file_name.contains(&pattern.replace("*", ""))
                }
            } else {
                file_name.contains(pattern)
            };
            
            if is_match {
                let metadata = fs::metadata(path).await.ok();
                matches.push(json!({
                    "path": path.to_string_lossy().to_string(),
                    "name": file_name,
                    "type": if path.is_dir() { "directory" } else { "file" },
                    "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0)
                }));
            }
        }
        
        Ok(matches)
    }
    
    async fn find_files(
        &self,
        directory: &str,
        options: &serde_json::Value,
        max_results: usize,
    ) -> Result<Vec<serde_json::Value>, ToolError> {
        let mut matches = Vec::new();
        
        let max_depth = options.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        let file_type = options.get("file_type").and_then(|v| v.as_str()).unwrap_or("any");
        let name_pattern = options.get("name_pattern").and_then(|v| v.as_str());
        let min_size = options.get("min_size").and_then(|v| v.as_u64());
        let max_size = options.get("max_size").and_then(|v| v.as_u64());
        
        for entry in WalkDir::new(directory).max_depth(max_depth).into_iter().filter_map(|e| e.ok()) {
            if matches.len() >= max_results {
                break;
            }
            
            let path = entry.path();
            
            // Check file type
            let metadata = match fs::metadata(path).await {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            let type_match = match file_type {
                "file" => metadata.is_file(),
                "directory" => metadata.is_dir(),
                "symlink" => metadata.is_symlink(),
                _ => true,
            };
            
            if !type_match {
                continue;
            }
            
            // Check name pattern
            if let Some(pattern) = name_pattern {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !file_name.contains(pattern) {
                    continue;
                }
            }
            
            // Check size
            let size = metadata.len();
            if let Some(min) = min_size {
                if size < min {
                    continue;
                }
            }
            if let Some(max) = max_size {
                if size > max {
                    continue;
                }
            }
            
            matches.push(json!({
                "path": path.to_string_lossy().to_string(),
                "type": if metadata.is_dir() { "directory" } else if metadata.is_file() { "file" } else { "symlink" },
                "size": size,
                "modified": metadata.modified().ok().map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
                })
            }));
        }
        
        Ok(matches)
    }
}
