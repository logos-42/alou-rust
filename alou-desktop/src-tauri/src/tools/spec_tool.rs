//! Spec 工具 - 规格文档管理
//!
//! 封装 Node.js spec-agent.js 脚本调用，提供统一的 ToolExecutor 接口
//! 支持创建、获取、列出、验证、更新、删除、导出、导入规格文档

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

/// Spec 工具
pub struct SpecTool {
    metadata: ToolMetadata,
    script_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecType {
    Product,
    Technical,
    Design,
    Api,
    UserStory,
    Tasks,
    Structure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecOperation {
    Create,
    Update,
    Get,
    Delete,
    List,
    Validate,
    GenerateFromTemplate,
    Export,
    Import,
}

impl SpecTool {
    /// 创建新的 Spec 工具
    pub fn new() -> Self {
        // 获取脚本路径（相对于 Cargo 清单文件）
        let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .join("scripts")
            .join("spec-agent.js");

        Self {
            metadata: ToolMetadata {
                id: "spec".to_string(),
                name: "Spec Tool".to_string(),
                description: "管理规格文档（创建、获取、列出、验证、更新、删除、导出、导入）".to_string(),
                category: ToolCategory::Development,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec!["nodejs".to_string()],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
                tags: vec![],
            },
            script_path,
        }
    }

    /// 调用 Node.js 脚本执行 Spec 操作
    async fn call_node_script(&self, request: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        let _start_time = std::time::Instant::now();

        // 创建临时目录
        let temp_dir = std::env::temp_dir().join("alou_spec");
        fs::create_dir_all(&temp_dir)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("创建临时目录失败：{}", e)))?;

        // 生成唯一文件名
        let request_id = Uuid::new_v4().to_string();
        let request_file = temp_dir.join(format!("request_{}.json", request_id));

        // 写入请求文件
        fs::write(&request_file, request.to_string())
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("写入请求文件失败：{}", e)))?;

        // 调用 Node.js 脚本（带超时）
        let timeout_duration = Duration::from_secs(30);
        let output = timeout(timeout_duration, async {
            let mut cmd = Command::new("node");
            cmd.arg(&self.script_path)
                .arg(&request_file)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            // Windows 下隐藏控制台窗口
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }

            cmd.output().await
        })
        .await
        .map_err(|_| ToolError::Timeout("Spec 工具执行超时（30 秒）".to_string()))?
        .map_err(|e| ToolError::ExecutionFailed(format!("执行 Node.js 脚本失败：{}", e)))?;

        // 检查执行结果
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // 尝试读取错误文件
            let error_file = temp_dir.join(format!("error_{}.json", request_id));
            let error_msg = if fs::try_exists(&error_file).await.unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&error_file).await {
                    if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&content) {
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
                }
            } else {
                stderr.to_string()
            };

            // 清理错误文件
            let _ = fs::remove_file(&error_file).await;
            let _ = fs::remove_file(&request_file).await;

            return Err(ToolError::ExecutionFailed(format!("Spec 脚本执行失败：{}", error_msg)));
        }

        // 读取 stdout 获取结果文件路径
        let stdout = String::from_utf8_lossy(&output.stdout);
        let result_path_str = stdout
            .lines()
            .next()
            .ok_or_else(|| ToolError::ExecutionFailed("未从脚本输出中读取到结果文件路径".to_string()))?
            .trim();

        let result_file = PathBuf::from(result_path_str);

        // 读取结果文件
        let result_content = fs::read_to_string(&result_file)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("读取结果文件失败：{}", e)))?;

        // 解析结果
        let result: serde_json::Value = serde_json::from_str(&result_content)
            .map_err(|e| ToolError::ExecutionFailed(format!("解析结果 JSON 失败：{}", e)))?;

        // 清理临时文件
        let _ = fs::remove_file(&request_file).await;
        let _ = fs::remove_file(&result_file).await;

        Ok(result)
    }

    /// 获取脚本路径（用于调试）
    #[allow(dead_code)]
    fn get_script_path(&self) -> &PathBuf {
        &self.script_path
    }
}

#[async_trait]
impl ToolExecutor for SpecTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();

        // 验证必需字段
        let operation = args.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("必须提供 'operation' 字段".to_string()))?;

        // 构建 Spec 请求
        let request = json!({
            "spec_type": args.get("spec_type").unwrap_or(&json!(null)),
            "operation": operation,
            "spec_data": args.get("spec_data").unwrap_or(&json!(null)),
            "spec_id": args.get("spec_id").unwrap_or(&json!(null)),
            "template_id": args.get("template_id").unwrap_or(&json!(null)),
            "output_format": args.get("output_format").unwrap_or(&json!(null)),
        });

        // 调用 Node.js 脚本
        let result = self.call_node_script(request).await?;

        let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);

        if !success {
            let error = result.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("未知错误")
                .to_string();
            return Err(ToolError::ExecutionFailed(error));
        }

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // 构建返回数据
        let data = json!({
            "operation": operation,
            "spec": result.get("spec"),
            "specs": result.get("specs"),
            "validation_result": result.get("validation_result"),
            "export_result": result.get("export_result"),
        });

        Ok(ToolResult {
            success: true,
            data,
            error: None,
            execution_time_ms,
            output: Some(format!("Spec 操作 '{}' 执行成功", operation)),
            warnings: vec![],
            context: None,
        })
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        let operation = args.get("operation")
            .and_then(|v| v.as_str());

        if operation.is_none() {
            return Err(ToolError::InvalidArguments("必须提供 'operation' 字段".to_string()));
        }

        let op = operation.unwrap();
        
        // 验证操作类型
        match op {
            "create" | "update" | "get" | "delete" | "list" | "validate" | "generate_from_template" | "export" | "import" => {
                // 验证 spec_type（除了 list 操作）
                if op != "list" {
                    let spec_type = args.get("spec_type").and_then(|v| v.as_str());
                    if spec_type.is_none() && op != "get" && op != "delete" && op != "export" {
                        // get/delete/export 只需要 spec_id
                    } else if spec_type.is_none() {
                        return Err(ToolError::InvalidArguments("必须提供 'spec_type' 字段".to_string()));
                    }
                }

                // 验证特定操作所需的字段
                match op {
                    "create" => {
                        if args.get("spec_data").is_none() {
                            return Err(ToolError::InvalidArguments("create 操作必须提供 'spec_data' 字段".to_string()));
                        }
                    }
                    "update" => {
                        if args.get("spec_id").is_none() {
                            return Err(ToolError::InvalidArguments("update 操作必须提供 'spec_id' 字段".to_string()));
                        }
                        if args.get("spec_data").is_none() {
                            return Err(ToolError::InvalidArguments("update 操作必须提供 'spec_data' 字段".to_string()));
                        }
                    }
                    "get" | "delete" => {
                        if args.get("spec_id").is_none() {
                            return Err(ToolError::InvalidArguments(format!("{} 操作必须提供 'spec_id' 字段", op)));
                        }
                    }
                    "export" => {
                        if args.get("spec_id").is_none() {
                            return Err(ToolError::InvalidArguments("export 操作必须提供 'spec_id' 字段".to_string()));
                        }
                        if args.get("output_format").is_none() {
                            return Err(ToolError::InvalidArguments("export 操作必须提供 'output_format' 字段".to_string()));
                        }
                    }
                    "validate" => {
                        if args.get("spec_id").is_none() && args.get("spec_data").is_none() {
                            return Err(ToolError::InvalidArguments("validate 操作必须提供 'spec_id' 或 'spec_data' 字段".to_string()));
                        }
                    }
                    _ => {}
                }

                Ok(())
            }
            _ => Err(ToolError::InvalidArguments(format!("不支持的操作类型：{}", op))),
        }
    }

    fn help(&self) -> String {
        r#"Spec 工具 - 规格文档管理

支持的操作：
  create                创建新的规格文档
  update                更新规格文档
  get                   获取规格文档
  delete                删除规格文档
  list                  列出所有规格文档
  validate              验证规格文档
  generate_from_template 从模板生成规格
  export                导出规格文档
  import                导入规格文档

规格类型：
  product               产品需求规格
  technical             技术规格
  design                设计规格
  api                   API 规格
  user_story            用户故事
  tasks                 任务规格
  structure             结构规格

使用示例：
  // 创建产品规格
  {
    "operation": "create",
    "spec_type": "product",
    "spec_data": {
      "title": "产品需求文档",
      "content": "..."
    },
    "template_id": "product"
  }

  // 获取规格
  {
    "operation": "get",
    "spec_type": "product",
    "spec_id": "abc123"
  }

  // 列出所有产品规格
  {
    "operation": "list",
    "spec_type": "product"
  }

  // 验证规格
  {
    "operation": "validate",
    "spec_type": "product",
    "spec_id": "abc123"
  }

  // 导出规格为 Markdown
  {
    "operation": "export",
    "spec_type": "product",
    "spec_id": "abc123",
    "output_format": "md"
  }
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_tool_creation() {
        let tool = SpecTool::new();
        assert_eq!(tool.metadata().id, "spec");
        assert_eq!(tool.metadata().name, "Spec Tool");
    }

    #[tokio::test]
    async fn test_validate_args_create() {
        let tool = SpecTool::new();
        let args = json!({
            "operation": "create",
            "spec_type": "product",
            "spec_data": {
                "title": "Test Spec"
            }
        });
        assert!(tool.validate_args(&args).await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_args_missing_operation() {
        let tool = SpecTool::new();
        let args = json!({
            "spec_type": "product"
        });
        assert!(tool.validate_args(&args).await.is_err());
    }

    #[tokio::test]
    async fn test_validate_args_invalid_operation() {
        let tool = SpecTool::new();
        let args = json!({
            "operation": "invalid_op"
        });
        assert!(tool.validate_args(&args).await.is_err());
    }
}
