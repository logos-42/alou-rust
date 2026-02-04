//! 工具创建模块
//! 
//! 负责创建新工具文件和定义文件

use crate::tools::{ToolError, ToolResult, ExecutionContext, ToolMetadata, ToolCategory, ToolStatus, ToolPriority};
use crate::tools::tool_parts::definitions::{ToolDefinition, ToolType, ToolUsageRecord};
use crate::tools::tool_parts::executor::DynamicToolExecutor;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use serde_json;
use chrono::Utc;

use crate::tools::tool_parts::executor::DynamicToolResult;

// 实现工具创建相关的功能
impl crate::tools::tool_creation::ToolCreationTool {
    /// 创建新工具文件
    pub async fn create_tool_file(&self, tool_def: &ToolDefinition, target_dir: &Path) -> Result<PathBuf, ToolError> {
        // 确保目标目录存在
        fs::create_dir_all(target_dir).await
            .map_err(|e| ToolError::InternalError(format!("Failed to create target directory: {}", e)))?;

        // 根据工具类型确定文件扩展名
        let extension = match tool_def.tool_type {
            ToolType::Rust => "rs",
            ToolType::Python => "py",
            ToolType::JavaScript => "js",
            ToolType::Shell => "sh",
            ToolType::Custom => "txt", // 默认扩展名
        };

        let filename = format!("{}.{}", tool_def.name, extension);
        let filepath = target_dir.join(&filename);

        // 写入工具内容
        fs::write(&filepath, &tool_def.content)
            .await
            .map_err(|e| ToolError::InternalError(format!("Failed to write tool file: {}", e)))?;

        Ok(filepath)
    }

    /// 创建工具定义文件（JSON格式）
    pub async fn create_tool_definition_file(&self, tool_def: &ToolDefinition, target_dir: &Path) -> Result<PathBuf, ToolError> {
        let def_filename = format!("{}-def.json", tool_def.name);
        let def_filepath = target_dir.join(&def_filename);

        let def_content = serde_json::to_string_pretty(tool_def)
            .map_err(|e| ToolError::InternalError(format!("Failed to serialize tool definition: {}", e)))?;

        fs::write(&def_filepath, def_content)
            .await
            .map_err(|e| ToolError::InternalError(format!("Failed to write tool definition file: {}", e)))?;

        Ok(def_filepath)
    }

    /// 记录工具使用情况到文档
    pub async fn record_tool_usage(&self, record: &ToolUsageRecord, docs_dir: &Path) -> Result<PathBuf, ToolError> {
        // 确保文档目录存在
        fs::create_dir_all(docs_dir).await
            .map_err(|e| ToolError::InternalError(format!("Failed to create docs directory: {}", e)))?;

        // 读取或创建使用记录文件
        let usage_log_path = docs_dir.join("tool_usage_log.json");
        let mut usage_records: Vec<ToolUsageRecord> = if usage_log_path.exists() {
            let content = fs::read_to_string(&usage_log_path).await
                .map_err(|e| ToolError::InternalError(format!("Failed to read usage log: {}", e)))?;

            serde_json::from_str(&content)
                .map_err(|e| ToolError::InternalError(format!("Failed to parse usage log: {}", e)))?
        } else {
            Vec::new()
        };

        // 添加新记录
        usage_records.push(record.clone());

        // 写回文件
        let records_json = serde_json::to_string_pretty(&usage_records)
            .map_err(|e| ToolError::InternalError(format!("Failed to serialize usage records: {}", e)))?;

        fs::write(&usage_log_path, records_json)
            .await
            .map_err(|e| ToolError::InternalError(format!("Failed to write usage log: {}", e)))?;

        Ok(usage_log_path)
    }

    /// 生成工具文档
    pub async fn generate_tool_documentation(&self, tool_def: &ToolDefinition, docs_dir: &Path) -> Result<PathBuf, ToolError> {
        // 确保文档目录存在
        fs::create_dir_all(docs_dir).await
            .map_err(|e| ToolError::InternalError(format!("Failed to create docs directory: {}", e)))?;

        let doc_filename = format!("{}-docs.md", tool_def.name);
        let doc_filepath = docs_dir.join(&doc_filename);

        let mut doc_content = format!("# {}\n\n", tool_def.name);
        doc_content.push_str(&format!("**描述**: {}\n\n", tool_def.description));
        doc_content.push_str(&format!("**类型**: {:?}\n\n", tool_def.tool_type));
        doc_content.push_str(&format!("**作者**: {}\n\n", tool_def.author));
        doc_content.push_str(&format!("**版本**: {}\n\n", tool_def.version));

        if !tool_def.dependencies.is_empty() {
            doc_content.push_str("**依赖项**:\n");
            for dep in &tool_def.dependencies {
                doc_content.push_str(&format!("- {}\n", dep));
            }
            doc_content.push('\n');
        }

        doc_content.push_str("## 参数\n\n");
        if tool_def.parameters.is_empty() {
            doc_content.push_str("*无参数*\n\n");
        } else {
            for param in &tool_def.parameters {
                doc_content.push_str(&format!("### `{}` ({})\n", param.name, param.param_type));
                doc_content.push_str(&format!("- **必需**: {}\n", if param.required { "是" } else { "否" }));
                doc_content.push_str(&format!("- **描述**: {}\n\n", param.description));
            }
        }

        doc_content.push_str("## 示例\n\n```json\n");
        let example_params: HashMap<String, String> = tool_def.parameters.iter()
            .filter(|p| p.required)
            .map(|p| (p.name.clone(), format!("\"<{}>\"", p.name)))
            .collect();
        doc_content.push_str(&serde_json::to_string_pretty(&example_params).unwrap_or_default());
        doc_content.push_str("\n```\n");

        fs::write(&doc_filepath, doc_content)
            .await
            .map_err(|e| ToolError::InternalError(format!("Failed to write documentation: {}", e)))?;

        Ok(doc_filepath)
    }

    /// 创建新工具
    pub async fn create_tool(&self, tool_def: ToolDefinition, target_dir: &str, docs_dir: &str) -> Result<HashMap<String, String>, ToolError> {
        let target_path = Path::new(target_dir);
        let docs_path = Path::new(docs_dir);

        // 创建工具文件
        let tool_file_path = self.create_tool_file(&tool_def, target_path).await?;

        // 创建工具定义文件
        let def_file_path = self.create_tool_definition_file(&tool_def, target_path).await?;

        // 生成文档
        let doc_file_path = self.generate_tool_documentation(&tool_def, docs_path).await?;

        let mut result = HashMap::new();
        result.insert("tool_file".to_string(), tool_file_path.to_string_lossy().to_string());
        result.insert("definition_file".to_string(), def_file_path.to_string_lossy().to_string());
        result.insert("documentation_file".to_string(), doc_file_path.to_string_lossy().to_string());
        result.insert("tool_name".to_string(), tool_def.name.clone());
        result.insert("tool_type".to_string(), format!("{:?}", tool_def.tool_type));

        Ok(result)
    }

    /// 创建并执行工具（一条龙服务）
    pub async fn create_and_execute_tool(&self, tool_def: ToolDefinition, target_dir: &str, docs_dir: &str) -> Result<HashMap<String, serde_json::Value>, ToolError> {
        // 先创建工具
        let creation_result = self.create_tool(tool_def.clone(), target_dir, docs_dir).await?;

        // 获取工具文件路径
        let tool_file_path = creation_result.get("tool_file").unwrap().to_string();
        let path = Path::new(&tool_file_path);

        // 执行工具
        let exec_result = match tool_def.tool_type {
            ToolType::Python => {
                let executor = DynamicToolExecutor::new();
                executor.execute(serde_json::json!({
                    "tool_type": "Python",
                    "script_path": tool_file_path,
                    "input_params": {}
                }), &ExecutionContext {
                    session_id: "".to_string(),
                    user_id: None,
                    working_directory: None,
                    environment: HashMap::new(),
                    timeout_seconds: Some(60),
                    permissions: vec![],
                    timestamp: Utc::now().timestamp(),
                }).await
            },
            ToolType::Shell => {
                let executor = DynamicToolExecutor::new();
                executor.execute(serde_json::json!({
                    "tool_type": "Shell",
                    "script_path": tool_file_path,
                    "input_params": {}
                }), &ExecutionContext {
                    session_id: "".to_string(),
                    user_id: None,
                    working_directory: None,
                    environment: HashMap::new(),
                    timeout_seconds: Some(60),
                    permissions: vec![],
                    timestamp: Utc::now().timestamp(),
                }).await
            },
            ToolType::JavaScript => {
                let executor = DynamicToolExecutor::new();
                executor.execute(serde_json::json!({
                    "tool_type": "JavaScript",
                    "script_path": tool_file_path,
                    "input_params": {}
                }), &ExecutionContext {
                    session_id: "".to_string(),
                    user_id: None,
                    working_directory: None,
                    environment: HashMap::new(),
                    timeout_seconds: Some(60),
                    permissions: vec![],
                    timestamp: Utc::now().timestamp(),
                }).await
            },
            _ => Err(ToolError::ExecutionFailed("Unsupported tool type for execution".to_string())),
        }?;

        let mut result = HashMap::new();
        for (k, v) in creation_result {
            result.insert(k, serde_json::Value::String(v));
        }
        result.insert("execution_result".to_string(), serde_json::json!({
            "success": exec_result.success,
            "output": exec_result.output,
            "execution_time_ms": exec_result.execution_time_ms,
            "data": exec_result.data,
        }));

        Ok(result)
    }

    /// 记录工具使用
    pub async fn log_tool_usage(&self,
                           tool_name: &str,
                           user: &str,
                           input_params: HashMap<String, serde_json::Value>,
                           result: &str,
                           execution_time_ms: u64,
                           docs_dir: &str) -> Result<String, ToolError> {
        let record = ToolUsageRecord {
            id: format!("usage_{}", uuid::Uuid::new_v4()),
            tool_name: tool_name.to_string(),
            user: user.to_string(),
            timestamp: Utc::now().timestamp(),
            input_params,
            result: result.to_string(),
            execution_time_ms,
        };

        let docs_path = Path::new(docs_dir);
        let log_path = self.record_tool_usage(&record, docs_path).await?;

        Ok(log_path.to_string_lossy().to_string())
    }
}