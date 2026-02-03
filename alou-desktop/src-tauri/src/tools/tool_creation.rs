//! 工具创建、记录和智能体工具跟踪工具
//!
//! 允许智能体：
//! 1. 创建新工具并在特定目录中存放
//! 2. 记录工具使用情况到文档
//! 3. 跟踪智能体自己使用的工具
//! 4. 自动生成工具使用文档
//! 5. 动态执行已创建的工具（Python、Shell、JavaScript）
//! 6. 注册动态工具到工具注册表

use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::process::Command;
use chrono::Utc;

/// 智能体工具注册表 - 跟踪每个智能体使用的工具

/// 动态工具执行器 - 用于执行AI创建的脚本工具
pub struct DynamicToolExecutor {
    metadata: ToolMetadata,
}

/// 动态工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicToolResult {
    /// 是否成功
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
}

impl DynamicToolExecutor {
    /// 创建新的动态工具执行器
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "dynamic_tool_executor".to_string(),
                name: "Dynamic Tool Executor".to_string(),
                description: "执行动态创建的脚本工具（Python、Shell、JavaScript）".to_string(),
                category: ToolCategory::Development,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "execute".to_string()],
            },
        }
    }

    /// 执行Python脚本
    async fn execute_python(&self, script_path: &Path, args: &HashMap<String, serde_json::Value>) -> Result<DynamicToolResult, ToolError> {
        let start = Utc::now().timestamp_millis();
        
        // 构建命令行参数
        let mut cmd_args: Vec<String> = vec!["python3".to_string(), script_path.to_str().unwrap_or("").to_string()];
        for (key, value) in args {
            cmd_args.push("--".to_string());
            cmd_args.push(key.clone());
            if let Some(val_str) = value.as_str() {
                cmd_args.push(val_str.to_string());
            } else {
                cmd_args.push(value.to_string());
            }
        }
        
        let output = Command::new("python3")
            .args(&cmd_args[1..])
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute Python script: {}", e)))?;
        
        let execution_time_ms = (Utc::now().timestamp_millis() - start) as u64;
        
        Ok(DynamicToolResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1) as i32,
            execution_time_ms,
        })
    }

    /// 执行Shell脚本
    async fn execute_shell(&self, script_path: &Path, args: &HashMap<String, serde_json::Value>) -> Result<DynamicToolResult, ToolError> {
        let start = Utc::now().timestamp_millis();
        
        // 构建命令行参数
        let mut cmd_args: Vec<String> = vec!["bash".to_string(), script_path.to_str().unwrap_or("").to_string()];
        for (key, value) in args {
            cmd_args.push("--".to_string());
            cmd_args.push(key.clone());
            if let Some(val_str) = value.as_str() {
                cmd_args.push(val_str.to_string());
            } else {
                cmd_args.push(value.to_string());
            }
        }
        
        let output = Command::new("bash")
            .args(&cmd_args[1..])
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute shell script: {}", e)))?;
        
        let execution_time_ms = (Utc::now().timestamp_millis() - start) as u64;
        
        Ok(DynamicToolResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1) as i32,
            execution_time_ms,
        })
    }

    /// 执行JavaScript脚本（使用Node.js）
    async fn execute_javascript(&self, script_path: &Path, args: &HashMap<String, serde_json::Value>) -> Result<DynamicToolResult, ToolError> {
        let start = Utc::now().timestamp_millis();
        
        // 构建命令行参数
        let mut cmd_args: Vec<String> = vec!["node".to_string(), script_path.to_str().unwrap_or("").to_string()];
        for (key, value) in args {
            cmd_args.push("--".to_string());
            cmd_args.push(key.clone());
            if let Some(val_str) = value.as_str() {
                cmd_args.push(val_str.to_string());
            } else {
                cmd_args.push(value.to_string());
            }
        }
        
        let output = Command::new("node")
            .args(&cmd_args[1..])
            .output()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute JavaScript: {}", e)))?;
        
        let execution_time_ms = (Utc::now().timestamp_millis() - start) as u64;
        
        Ok(DynamicToolResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1) as i32,
            execution_time_ms,
        })
    }
}

#[async_trait]
impl ToolExecutor for DynamicToolExecutor {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let tool_type = args.get("tool_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_type' field".to_string()))?;

        let script_path = args.get("script_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'script_path' field".to_string()))?;

        let input_params: HashMap<String, serde_json::Value> = args.get("input_params")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let path = Path::new(script_path);
        
        if !path.exists() {
            return Err(ToolError::InvalidArguments(format!("Script file not found: {}", script_path)));
        }

        let result = match tool_type {
            "Python" | "python" => self.execute_python(path, &input_params).await?,
            "Shell" | "shell" | "bash" => self.execute_shell(path, &input_params).await?,
            "JavaScript" | "javascript" | "node" => self.execute_javascript(path, &input_params).await?,
            _ => return Err(ToolError::InvalidArguments(format!("Unsupported tool type: {}", tool_type))),
        };

        Ok(ToolResult {
            success: result.success,
            data: serde_json::json!(result),
            error: if !result.stderr.is_empty() { Some(result.stderr) } else { None },
            execution_time_ms: result.execution_time_ms,
            output: Some(format!("Tool executed successfully in {}ms", result.execution_time_ms)),
            warnings: vec![],
            context: None,
        })
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }
        if args.get("tool_type").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: tool_type".to_string()));
        }
        if args.get("script_path").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: script_path".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        r#"Dynamic Tool Executor

Execute dynamically created script tools.

Actions:
  Execute script tools (Python, Shell, JavaScript)

Params:
{
  "tool_type": "Python|Shell|JavaScript",
  "script_path": "Path to the script file",
  "input_params": {"key": "value", ...}
}

Examples:

Execute a Python script:
{
  "tool_type": "Python",
  "script_path": "./tools/custom/my_tool.py",
  "input_params": {
    "input_file": "data.txt",
    "output_file": "result.txt"
  }
}

Execute a Shell script:
{
  "tool_type": "Shell",
  "script_path": "./tools/custom/my_script.sh",
  "input_params": {
    "mode": "fast",
    "verbose": "true"
  }
}

Execute a JavaScript script:
{
  "tool_type": "JavaScript",
  "script_path": "./tools/custom/analyzer.js",
  "input_params": {
    "source": "input.json"
  }
}"#.to_string()
    }
}

/// 工具创建和记录工具
pub struct ToolCreationTool {
    metadata: ToolMetadata,
    /// 智能体工具注册表
    agent_tool_registry: Arc<RwLock<HashMap<String, AgentToolRegistry>>>,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 工具类型 (Rust, Python, JavaScript, Shell)
    pub tool_type: ToolType,
    /// 工具代码内容
    pub content: String,
    /// 参数定义
    pub parameters: Vec<ParameterDef>,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 作者
    pub author: String,
    /// 版本
    pub version: String,
    /// 依赖项
    pub dependencies: Vec<String>,
}

/// 工具类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Rust,
    Python,
    JavaScript,
    Shell,
    Custom,
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    /// 参数名
    pub name: String,
    /// 参数类型
    pub param_type: String,
    /// 是否必需
    pub required: bool,
    /// 描述
    pub description: String,
}

/// 工具使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageRecord {
    /// 记录ID
    pub id: String,
    /// 工具名称
    pub tool_name: String,
    /// 使用者（智能体ID或用户ID）
    pub user: String,
    /// 使用时间
    pub timestamp: i64,
    /// 输入参数
    pub input_params: HashMap<String, serde_json::Value>,
    /// 执行结果
    pub result: String,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
}

/// 智能体工具使用记录 - 专门用于跟踪智能体自己使用的工具
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolUsageRecord {
    /// 记录ID
    pub id: String,
    /// 智能体ID
    pub agent_id: String,
    /// 工具名称
    pub tool_name: String,
    /// 调用时间
    pub timestamp: i64,
    /// 目的/用途描述
    pub purpose: String,
    /// 输入参数
    pub input_params: HashMap<String, serde_json::Value>,
    /// 执行结果
    pub result: String,
    /// 是否成功
    pub success: bool,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
}

/// 智能体工具注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolRegistry {
    /// 智能体ID
    pub agent_id: String,
    /// 智能体名称
    pub agent_name: String,
    /// 已使用的工具列表
    pub used_tools: Vec<String>,
    /// 工具使用记录
    pub usage_records: Vec<AgentToolUsageRecord>,
    /// 注册时间
    pub registered_at: i64,
    /// 最后更新
    pub last_updated: i64,
}

impl ToolCreationTool {
    /// 创建新的工具创建和记录工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "tool_creation".to_string(),
                name: "Tool Creation and Documentation Tool".to_string(),
                description: "创建新工具并记录到文档，支持多种语言；跟踪智能体使用的工具，自动生成文档".to_string(),
                category: ToolCategory::Development,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "2.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "execute".to_string()],
            },
            agent_tool_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新工具文件
    async fn create_tool_file(&self, tool_def: &ToolDefinition, target_dir: &Path) -> Result<PathBuf, ToolError> {
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
    async fn create_tool_definition_file(&self, tool_def: &ToolDefinition, target_dir: &Path) -> Result<PathBuf, ToolError> {
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
    async fn record_tool_usage(&self, record: &ToolUsageRecord, docs_dir: &Path) -> Result<PathBuf, ToolError> {
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
    async fn generate_tool_documentation(&self, tool_def: &ToolDefinition, docs_dir: &Path) -> Result<PathBuf, ToolError> {
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
    async fn create_tool(&self, tool_def: ToolDefinition, target_dir: &str, docs_dir: &str) -> Result<HashMap<String, String>, ToolError> {
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
    async fn create_and_execute_tool(&self, tool_def: ToolDefinition, target_dir: &str, docs_dir: &str) -> Result<HashMap<String, serde_json::Value>, ToolError> {
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
    async fn log_tool_usage(&self, 
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

    // ==================== 智能体工具跟踪功能 ====================

    /// 注册智能体（初始化智能体的工具跟踪）
    pub async fn register_agent(&self, agent_id: &str, agent_name: &str) -> Result<(), ToolError> {
        let mut registry = self.agent_tool_registry.write().await;
        
        if registry.contains_key(agent_id) {
            return Ok(()); // 已注册
        }
        
        let agent_registry = AgentToolRegistry {
            agent_id: agent_id.to_string(),
            agent_name: agent_name.to_string(),
            used_tools: Vec::new(),
            usage_records: Vec::new(),
            registered_at: Utc::now().timestamp(),
            last_updated: Utc::now().timestamp(),
        };
        
        registry.insert(agent_id.to_string(), agent_registry);
        Ok(())
    }

    /// 智能体记录自己使用的工具
    pub async fn agent_log_tool_usage(&self,
                                   agent_id: &str,
                                   tool_name: &str,
                                   purpose: &str,
                                   input_params: HashMap<String, serde_json::Value>,
                                   result: &str,
                                   success: bool,
                                   execution_time_ms: u64) -> Result<String, ToolError> {
        let mut registry = self.agent_tool_registry.write().await;
        
        if let Some(agent_registry) = registry.get_mut(agent_id) {
            let record = AgentToolUsageRecord {
                id: format!("agent_usage_{}", uuid::Uuid::new_v4()),
                agent_id: agent_id.to_string(),
                tool_name: tool_name.to_string(),
                timestamp: Utc::now().timestamp(),
                purpose: purpose.to_string(),
                input_params,
                result: result.to_string(),
                success,
                execution_time_ms,
            };
            
            // 如果是新工具，添加到已使用工具列表
            if !agent_registry.used_tools.contains(&tool_name.to_string()) {
                agent_registry.used_tools.push(tool_name.to_string());
            }
            
            agent_registry.usage_records.push(record.clone());
            agent_registry.last_updated = Utc::now().timestamp();
            
            Ok(record.id)
        } else {
            Err(ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))
        }
    }

    /// 获取智能体使用的所有工具
    pub async fn get_agent_used_tools(&self, agent_id: &str) -> Result<Vec<String>, ToolError> {
        let registry = self.agent_tool_registry.read().await;
        
        if let Some(agent_registry) = registry.get(agent_id) {
            Ok(agent_registry.used_tools.clone())
        } else {
            Err(ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))
        }
    }

    /// 获取智能体的工具使用记录
    async fn get_agent_usage_records(&self, agent_id: &str, limit: Option<usize>) -> Result<Vec<AgentToolUsageRecord>, ToolError> {
        let registry = self.agent_tool_registry.read().await;
        
        if let Some(agent_registry) = registry.get(agent_id) {
            let mut records = agent_registry.usage_records.clone();
            
            if let Some(limit_val) = limit {
                records = records.into_iter().rev().take(limit_val).collect();
                records.reverse();
            }
            
            Ok(records)
        } else {
            Err(ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))
        }
    }

    /// 生成智能体工具使用报告（Markdown格式）
    #[allow(dead_code)]
    async fn generate_agent_tool_report(&self, agent_id: &str, docs_dir: &str) -> Result<String, ToolError> {
        let registry = self.agent_tool_registry.read().await;
        
        let agent_registry = registry.get(agent_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))?;
        
        let docs_path = Path::new(docs_dir);
        fs::create_dir_all(docs_path).await
            .map_err(|e| ToolError::InternalError(format!("Failed to create docs directory: {}", e)))?;
        
        let mut report = format!("# 智能体工具使用报告\n\n");
        report.push_str(&format!("**智能体ID**: {}\n\n", agent_registry.agent_id));
        report.push_str(&format!("**智能体名称**: {}\n\n", agent_registry.agent_name));
        report.push_str(&format!("**注册时间**: {}\n\n", chrono::DateTime::from_timestamp(agent_registry.registered_at, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "未知".to_string())));
        report.push_str(&format!("**最后更新**: {}\n\n", chrono::DateTime::from_timestamp(agent_registry.last_updated, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "未知".to_string())));
        report.push_str(&format!("**已使用工具数量**: {}\n\n", agent_registry.used_tools.len()));
        report.push_str(&format!("**工具调用总次数**: {}\n\n", agent_registry.usage_records.len()));
        
        report.push_str("## 已使用的工具列表\n\n");
        if agent_registry.used_tools.is_empty() {
            report.push_str("*暂无工具使用记录*\n\n");
        } else {
            for (index, tool) in agent_registry.used_tools.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", index + 1, tool));
            }
            report.push('\n');
        }
        
        report.push_str("## 工具使用详细记录\n\n");
        if agent_registry.usage_records.is_empty() {
            report.push_str("*暂无使用记录*\n\n");
        } else {
            for record in &agent_registry.usage_records {
                report.push_str(&format!("### {}\n\n", record.tool_name));
                report.push_str(&format!("- **时间**: {}\n", chrono::DateTime::from_timestamp(record.timestamp, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "未知".to_string())));
                report.push_str(&format!("- **目的**: {}\n", record.purpose));
                report.push_str(&format!("- **结果**: {}\n", record.result));
                report.push_str(&format!("- **耗时**: {}ms\n", record.execution_time_ms));
                report.push_str(&format!("- **成功**: {}\n\n", if record.success { "是" } else { "否" }));
            }
        }
        
        let filename = format!("agent_{}_tool_report.md", agent_id);
        let filepath = docs_path.join(&filename);
        
        fs::write(&filepath, report).await
            .map_err(|e| ToolError::InternalError(format!("Failed to write agent tool report: {}", e)))?;
        
        Ok(filepath.to_string_lossy().to_string())
    }

    /// 导出智能体工具使用数据为JSON
    #[allow(dead_code)]
    async fn export_agent_tool_data(&self, agent_id: &str, docs_dir: &str) -> Result<String, ToolError> {
        let registry = self.agent_tool_registry.read().await;
        
        let agent_registry = registry.get(agent_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))?;
        
        let docs_path = Path::new(docs_dir);
        fs::create_dir_all(docs_path).await
            .map_err(|e| ToolError::InternalError(format!("Failed to create docs directory: {}", e)))?;
        
        let data = serde_json::to_string_pretty(agent_registry)
            .map_err(|e| ToolError::InternalError(format!("Failed to serialize agent tool data: {}", e)))?;
        
        let filename = format!("agent_{}_tool_data.json", agent_id);
        let filepath = docs_path.join(&filename);
        
        fs::write(&filepath, data).await
            .map_err(|e| ToolError::InternalError(format!("Failed to export agent tool data: {}", e)))?;
        
        Ok(filepath.to_string_lossy().to_string())
    }

    /// 发现指定目录下的所有工具
    #[allow(dead_code)]
    async fn discover_tools(&self, dir: &str) -> Result<Vec<ToolDefinition>, ToolError> {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut tools = Vec::new();
        let mut entries = fs::read_dir(dir_path).await
            .map_err(|e| ToolError::InternalError(format!("Failed to read directory: {}", e)))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| ToolError::InternalError(format!("Failed to read directory entry: {}", e)))? {
            
            let path = entry.path();
            if path.is_file() && path.to_string_lossy().ends_with("-def.json") {
                let content = fs::read_to_string(&path).await
                    .map_err(|e| ToolError::InternalError(format!("Failed to read tool definition: {}", e)))?;
                
                let tool_def: ToolDefinition = serde_json::from_str(&content)
                    .map_err(|e| ToolError::InternalError(format!("Failed to parse tool definition: {}", e)))?;
                
                tools.push(tool_def);
            }
        }
        
        Ok(tools)
    }

    /// 列出所有已注册的智能体
    #[allow(dead_code)]
    async fn list_registered_agents(&self) -> Result<Vec<String>, ToolError> {
        let registry = self.agent_tool_registry.read().await;
        Ok(registry.keys().cloned().collect())
    }
}

#[async_trait]
impl ToolExecutor for ToolCreationTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "create_tool" => {
                // 解析工具定义
                let tool_def_value = args.get("tool_definition")
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_definition' field".to_string()))?;

                let tool_def: ToolDefinition = serde_json::from_value(tool_def_value.clone())
                    .map_err(|e| ToolError::InvalidArguments(format!("Invalid tool definition: {}", e)))?;

                let target_dir = args.get("target_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./tools/custom"); // 默认目录

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/tools"); // 默认文档目录

                let result = self.create_tool(tool_def, target_dir, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(result),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Created tool '{}' and documentation", result.get("tool_name").unwrap_or(&"unknown".to_string()))),
                    warnings: vec![],
                    context: None,
                })
            }

            "log_usage" => {
                let tool_name = args.get("tool_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_name' field".to_string()))?
                    .to_string();

                let user = args.get("user")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'user' field".to_string()))?
                    .to_string();

                let input_params: HashMap<String, serde_json::Value> = args.get("input_params")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let result = args.get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let execution_time_ms = args.get("execution_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/tools"); // 默认文档目录

                let log_path = self.log_tool_usage(&tool_name, &user, input_params, &result, execution_time_ms, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "tool_name": tool_name,
                        "user": user,
                        "log_path": log_path
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Logged usage of tool '{}' by user '{}'", tool_name, user)),
                    warnings: vec![],
                    context: None,
                })
            }

            "create_and_log" => {
                // 结合创建工具和记录使用的操作
                let tool_def_value = args.get("tool_definition")
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_definition' field".to_string()))?;

                let tool_def: ToolDefinition = serde_json::from_value(tool_def_value.clone())
                    .map_err(|e| ToolError::InvalidArguments(format!("Invalid tool definition: {}", e)))?;

                let target_dir = args.get("target_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./tools/custom");

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/tools");

                let creation_result = self.create_tool(tool_def.clone(), target_dir, docs_dir).await?;

                // 记录创建操作
                let input_params = HashMap::from([
                    ("tool_name".to_string(), serde_json::Value::String(tool_def.name.clone())),
                    ("tool_type".to_string(), serde_json::Value::String(format!("{:?}", tool_def.tool_type))),
                ]);

                let _log_result = self.log_tool_usage(
                    &tool_def.name,
                    "system",
                    input_params,
                    "Tool created successfully",
                    0,
                    docs_dir
                ).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(creation_result),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Created and logged tool '{}'", tool_def.name)),
                    warnings: vec![],
                    context: None,
                })
            }

            // ==================== 动态工具执行操作 ====================

            "execute_tool" => {
                let tool_type = args.get("tool_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_type' field".to_string()))?
                    .to_string();

                let script_path = args.get("script_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'script_path' field".to_string()))?
                    .to_string();

                let input_params: HashMap<String, serde_json::Value> = args.get("input_params")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let executor = DynamicToolExecutor::new();
                let result = executor.execute(
                    serde_json::json!({
                        "tool_type": tool_type,
                        "script_path": script_path,
                        "input_params": input_params
                    }),
                    _context
                ).await?;

                Ok(ToolResult {
                    success: result.success,
                    data: result.data,
                    error: result.error,
                    execution_time_ms: result.execution_time_ms,
                    output: result.output,
                    warnings: vec![],
                    context: None,
                })
            }

            "create_and_execute" => {
                // 创建工具并立即执行
                let tool_def_value = args.get("tool_definition")
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_definition' field".to_string()))?;

                let tool_def: ToolDefinition = serde_json::from_value(tool_def_value.clone())
                    .map_err(|e| ToolError::InvalidArguments(format!("Invalid tool definition: {}", e)))?;

                let target_dir = args.get("target_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./tools/custom");

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/tools");

                let result = self.create_and_execute_tool(tool_def, target_dir, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(result),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Created and executed tool")),
                    warnings: vec![],
                    context: None,
                })
            }

            // ==================== 智能体工具跟踪操作 ====================

            "register_agent" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let agent_name = args.get("agent_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&agent_id)
                    .to_string();

                self.register_agent(&agent_id, &agent_name).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "agent_name": agent_name
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Registered agent '{}' ({})", agent_id, agent_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "agent_log_tool" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let tool_name = args.get("tool_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_name' field".to_string()))?
                    .to_string();

                let purpose = args.get("purpose")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let input_params: HashMap<String, serde_json::Value> = args.get("input_params")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let result = args.get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let success = args.get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let execution_time_ms = args.get("execution_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let record_id = self.agent_log_tool_usage(
                    &agent_id, &tool_name, &purpose, input_params, &result, success, execution_time_ms
                ).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "tool_name": tool_name,
                        "record_id": record_id
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Agent '{}' logged tool usage: {}", agent_id, tool_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "get_agent_used_tools" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let tools = self.get_agent_used_tools(&agent_id).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "used_tools": tools,
                        "count": tools.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Agent '{}' has used {} tools", agent_id, tools.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "get_agent_usage_records" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);

                let records = self.get_agent_usage_records(&agent_id, limit).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "usage_records": records,
                        "count": records.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} usage records for agent '{}'", records.len(), agent_id)),
                    warnings: vec![],
                    context: None,
                })
            }

            "generate_agent_tool_report" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/agents");

                let report_path = self.generate_agent_tool_report(&agent_id, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "report_path": report_path
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Generated tool report for agent '{}' at {}", agent_id, report_path)),
                    warnings: vec![],
                    context: None,
                })
            }

            "export_agent_tool_data" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/agents");

                let export_path = self.export_agent_tool_data(&agent_id, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "export_path": export_path
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Exported agent '{}' tool data to {}", agent_id, export_path)),
                    warnings: vec![],
                    context: None,
                })
            }

            "discover_tools" => {
                let dir = args.get("dir")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'dir' field".to_string()))?
                    .to_string();

                let tools = self.discover_tools(&dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "directory": dir,
                        "tools": tools,
                        "count": tools.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Discovered {} tools in directory '{}'", tools.len(), dir)),
                    warnings: vec![],
                    context: None,
                })
            }

            "list_registered_agents" => {
                let agents = self.list_registered_agents().await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agents": agents,
                        "count": agents.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} registered agents", agents.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            _ => Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }
        if args.get("action").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: action".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        r##"Tool Creation and Documentation Tool (Enhanced)

Create new tools, document usage, and track agent tool usage.

=== Core Actions ===

  - create_tool: Create a new tool with definition and documentation
    params: {
      "tool_definition": {...},
      "target_dir": "Directory to store the tool (default: ./tools/custom)",
      "docs_dir": "Directory to store documentation (default: ./docs/tools)"
    }

  - log_usage: Log tool usage to documentation
    params: {
      "tool_name": "Name of the tool used",
      "user": "User who used the tool",
      "input_params": {...},
      "result": "Execution result",
      "execution_time_ms": Execution time in milliseconds,
      "docs_dir": "Directory to store logs (default: ./docs/tools)"
    }

  - create_and_log: Create a tool and log its creation
    params: Same as create_tool

  - execute_tool: Execute a dynamically created tool
    params: {
      "tool_type": "Python|Shell|JavaScript",
      "script_path": "Path to the script file",
      "input_params": {"key": "value", ...}
    }

  - create_and_execute: Create a tool and execute it immediately
    params: {
      "tool_definition": {...},
      "target_dir": "Directory to store the tool",
      "docs_dir": "Directory to store documentation"
    }

=== Agent Tool Tracking Actions ===

  - register_agent: Register an agent for tool tracking
    params: {
      "agent_id": "Unique agent identifier",
      "agent_name": "Human-readable agent name (optional, defaults to agent_id)"
    }

  - agent_log_tool: Agent records its own tool usage
    params: {
      "agent_id": "Agent identifier",
      "tool_name": "Name of the tool used",
      "purpose": "Purpose/description of using this tool",
      "input_params": {...},
      "result": "Execution result",
      "success": true|false,
      "execution_time_ms": Execution time in milliseconds
    }

  - get_agent_used_tools: Get list of tools used by an agent
    params: {
      "agent_id": "Agent identifier"
    }

  - get_agent_usage_records: Get detailed usage records for an agent
    params: {
      "agent_id": "Agent identifier",
      "limit": Optional number of records to return
    }

  - generate_agent_tool_report: Generate Markdown report of agent tool usage
    params: {
      "agent_id": "Agent identifier",
      "docs_dir": "Output directory (default: ./docs/agents)"
    }

  - export_agent_tool_data: Export agent tool data as JSON
    params: {
      "agent_id": "Agent identifier",
      "docs_dir": "Output directory (default: ./docs/agents)"
    }

  - discover_tools: Discover all tools in a directory
    params: {
      "dir": "Directory to scan for tools"
    }

  - list_registered_agents: List all registered agents
    params: {}

Examples:

Register an agent:
{
  "action": "register_agent",
  "agent_id": "researcher-agent-001",
  "agent_name": "Research Agent"
}

Agent logs tool usage:
{
  "action": "agent_log_tool",
  "agent_id": "researcher-agent-001",
  "tool_name": "web_search",
  "purpose": "Search for information about AI trends",
  "input_params": {
    "query": "AI development trends 2024"
  },
  "result": "Found 15 relevant articles",
  "success": true,
  "execution_time_ms": 150
}

Generate agent tool usage report:
{
  "action": "generate_agent_tool_report",
  "agent_id": "researcher-agent-001",
  "docs_dir": "./docs/agent-reports"
}

Discover available tools:
{
  "action": "discover_tools",
  "dir": "./tools/custom"
}

=== Dynamic Tool Execution Examples ===

Execute a created Python tool:
{
  "action": "execute_tool",
  "tool_type": "Python",
  "script_path": "./tools/custom/my_python_tool.py",
  "input_params": {
    "input": "data.csv",
    "format": "json"
  }
}

Create and execute a tool in one step:
{
  "action": "create_and_execute",
  "tool_definition": {
    "name": "quick_calculator",
    "description": "A simple calculator tool",
    "tool_type": "Python",
    "content": "#!/usr/bin/env python3\nimport sys\na = int(sys.argv[1])\nb = int(sys.argv[2])\nprint(str(a) + str(b) + str(a + b))",
    "parameters": [
      {
        "name": "a",
        "param_type": "number",
        "required": true,
        "description": "First number"
      },
      {
        "name": "b",
        "param_type": "number",
        "required": true,
        "description": "Second number"
      }
    ],
    "author": "AI Assistant",
    "version": "1.0.0",
    "dependencies": []
  },
  "target_dir": "./tools/custom",
  "docs_dir": "./docs/tools"
}"##.to_string()
    }
}