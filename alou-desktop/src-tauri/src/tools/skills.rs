//! Skills 系统
//!
//! 提供可复用的技能定义、执行和管理功能

use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 技能定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// 技能ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 技能类别
    pub category: String,
    /// 输入参数模式
    pub input_schema: serde_json::Value,
    /// 输出结果模式
    pub output_schema: serde_json::Value,
    /// 技能实现类型
    pub implementation_type: SkillImplementationType,
    /// 技能代码或配置
    pub implementation: serde_json::Value,
    /// 依赖的其他技能
    pub dependencies: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 版本
    pub version: String,
    /// 作者
    pub author: String,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
}

/// 技能实现类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillImplementationType {
    /// 内置函数
    Builtin,
    /// 脚本代码
    Script,
    /// 工具链式调用
    ToolChain,
    /// AI 生成的代码
    AiGenerated,
    /// 复合技能（多个技能组合）
    Composite,
}

/// 技能执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    /// 执行是否成功
    pub success: bool,
    /// 结果数据
    pub output: serde_json::Value,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 中间步骤
    pub intermediate_steps: Vec<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 性能指标
    pub metrics: HashMap<String, serde_json::Value>,
}

/// 技能执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionContext {
    /// 会话ID
    pub session_id: String,
    /// 技能ID
    pub skill_id: String,
    /// 执行ID
    pub execution_id: String,
    /// 输入参数
    pub inputs: HashMap<String, serde_json::Value>,
    /// 环境变量
    pub environment: HashMap<String, serde_json::Value>,
    /// 超时时间
    pub timeout_seconds: Option<u64>,
    /// 调试模式
    pub debug_mode: bool,
}

/// 技能执行器 trait
#[async_trait]
pub trait SkillExecutor: Send + Sync {
    /// 执行技能
    async fn execute(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String>;

    /// 验证输入参数
    async fn validate_inputs(&self, inputs: &HashMap<String, serde_json::Value>) -> Result<(), String>;

    /// 获取技能帮助信息
    fn help(&self) -> String;
}

/// 内置技能执行器
pub struct BuiltinSkillExecutor {
    skill_definition: SkillDefinition,
}

impl BuiltinSkillExecutor {
    pub fn new(definition: SkillDefinition) -> Self {
        Self {
            skill_definition: definition,
        }
    }

    async fn execute_builtin(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let start_time = std::time::Instant::now();

        match self.skill_definition.id.as_str() {
            "text_summarizer" => self.execute_text_summarizer(context).await,
            "code_formatter" => self.execute_code_formatter(context).await,
            "data_validator" => self.execute_data_validator(context).await,
            "file_processor" => self.execute_file_processor(context).await,
            _ => Err(format!("Unknown builtin skill: {}", self.skill_definition.id)),
        }.map(|mut result| {
            result.execution_time_ms = start_time.elapsed().as_millis() as u64;
            result
        })
    }

    async fn execute_text_summarizer(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let text = context.inputs.get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'text' input")?;

        let max_length = context.inputs.get("max_length")
            .and_then(|v| v.as_u64())
            .unwrap_or(200) as usize;

        // 简单的文本摘要算法
        let summary = if text.len() <= max_length {
            text.to_string()
        } else {
            let mut summary = text.chars().take(max_length).collect::<String>();
            if let Some(last_space) = summary.rfind(' ') {
                summary.truncate(last_space);
            }
            summary.push_str("...");
            summary
        };

        Ok(SkillExecutionResult {
            success: true,
            output: serde_json::json!({
                "summary": summary,
                "original_length": text.len(),
                "summary_length": summary.len()
            }),
            execution_time_ms: 0,
            intermediate_steps: vec![],
            error: None,
            metrics: HashMap::new(),
        })
    }

    async fn execute_code_formatter(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let code = context.inputs.get("code")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'code' input")?;

        let language = context.inputs.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        // 简单的代码格式化（实际应该调用相应语言的格式化工具）
        let formatted_code = match language {
            "json" => {
                serde_json::from_str::<serde_json::Value>(code)
                    .map_err(|e| format!("Invalid JSON: {}", e))?
                    .to_string()
            }
            "rust" => {
                // 简单的 Rust 格式化
                code.lines()
                    .map(|line| line.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            _ => code.to_string(),
        };

        Ok(SkillExecutionResult {
            success: true,
            output: serde_json::json!({
                "formatted_code": formatted_code,
                "language": language,
                "original_length": code.len(),
                "formatted_length": formatted_code.len()
            }),
            execution_time_ms: 0,
            intermediate_steps: vec![],
            error: None,
            metrics: HashMap::new(),
        })
    }

    async fn execute_data_validator(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let data = context.inputs.get("data")
            .ok_or("Missing 'data' input")?;

        let schema = context.inputs.get("schema")
            .ok_or("Missing 'schema' input")?;

        // 简单的模式验证
        let is_valid = match schema.get("type").and_then(|v| v.as_str()) {
            Some("string") => data.is_string(),
            Some("number") => data.is_number(),
            Some("boolean") => data.is_boolean(),
            Some("object") => data.is_object(),
            Some("array") => data.is_array(),
            _ => false,
        };

        let errors = if !is_valid {
            vec![format!("Data does not match schema type: {:?}", schema.get("type"))]
        } else {
            vec![]
        };

        Ok(SkillExecutionResult {
            success: is_valid,
            output: serde_json::json!({
                "valid": is_valid,
                "errors": errors,
                "schema": schema,
                "data": data
            }),
            execution_time_ms: 0,
            intermediate_steps: vec![],
            error: if !is_valid { Some("Validation failed".to_string()) } else { None },
            metrics: HashMap::new(),
        })
    }

    async fn execute_file_processor(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let file_path = context.inputs.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'file_path' input")?;

        let operation = context.inputs.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("read");

        match operation {
            "read" => {
                let content = tokio::fs::read_to_string(file_path).await
                    .map_err(|e| format!("Failed to read file: {}", e))?;

                Ok(SkillExecutionResult {
                    success: true,
                    output: serde_json::json!({
                        "operation": "read",
                        "file_path": file_path,
                        "content": content,
                        "size": content.len()
                    }),
                    execution_time_ms: 0,
                    intermediate_steps: vec![],
                    error: None,
                    metrics: HashMap::new(),
                })
            }
            "analyze" => {
                let content = tokio::fs::read_to_string(file_path).await
                    .map_err(|e| format!("Failed to read file: {}", e))?;

                let lines = content.lines().count();
                let words = content.split_whitespace().count();
                let chars = content.chars().count();

                Ok(SkillExecutionResult {
                    success: true,
                    output: serde_json::json!({
                        "operation": "analyze",
                        "file_path": file_path,
                        "lines": lines,
                        "words": words,
                        "characters": chars,
                        "size_bytes": content.len()
                    }),
                    execution_time_ms: 0,
                    intermediate_steps: vec![],
                    error: None,
                    metrics: HashMap::new(),
                })
            }
            _ => Err(format!("Unknown file operation: {}", operation)),
        }
    }
}

#[async_trait]
impl SkillExecutor for BuiltinSkillExecutor {
    async fn execute(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        match self.skill_definition.implementation_type {
            SkillImplementationType::Builtin => self.execute_builtin(context).await,
            _ => Err("Unsupported implementation type".to_string()),
        }
    }

    async fn validate_inputs(&self, inputs: &HashMap<String, serde_json::Value>) -> Result<(), String> {
        // 简单的输入验证
        if inputs.is_empty() {
            return Err("Inputs cannot be empty".to_string());
        }
        Ok(())
    }

    fn help(&self) -> String {
        format!("{}: {}", self.skill_definition.name, self.skill_definition.description)
    }
}

/// Skills 工具
pub struct SkillsTool {
    metadata: ToolMetadata,
    /// 技能定义存储
    skills: Arc<Mutex<HashMap<String, SkillDefinition>>>,
    /// 技能执行器存储
    executors: Arc<Mutex<HashMap<String, Box<dyn SkillExecutor>>>>,
}

impl SkillsTool {
    /// 创建新的 Skills 工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "skills".to_string(),
                name: "Skills Management Tool".to_string(),
                description: "管理和执行可复用的技能，支持技能的创建、修改和执行".to_string(),
                category: ToolCategory::Skills,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "execute".to_string()],
            },
            skills: Arc::new(Mutex::new(HashMap::new())),
            executors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 初始化内置技能
    pub async fn initialize_builtin_skills(&self) -> Result<(), ToolError> {
        let builtin_skills = vec![
            ("text_summarizer", "文本摘要技能", "对长文本进行智能摘要", "text_processing"),
            ("code_formatter", "代码格式化技能", "对代码进行格式化和美化", "code_processing"),
            ("data_validator", "数据验证技能", "验证数据是否符合指定模式", "data_processing"),
            ("file_processor", "文件处理技能", "读取和分析文件内容", "file_processing"),
        ];

        for (id, name, description, category) in builtin_skills {
            let skill_def = SkillDefinition {
                id: id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                category: category.to_string(),
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
                implementation_type: SkillImplementationType::Builtin,
                implementation: serde_json::json!({}),
                dependencies: vec![],
                tags: vec![category.to_string()],
                version: "1.0.0".to_string(),
                author: "system".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
            };

            let executor = Box::new(BuiltinSkillExecutor::new(skill_def.clone()));

            let mut skills = self.skills.lock().await;
            let mut executors = self.executors.lock().await;

            skills.insert(id.to_string(), skill_def);
            executors.insert(id.to_string(), executor);
        }

        Ok(())
    }

    /// 创建自定义技能
    async fn create_skill(&self, definition: SkillDefinition) -> Result<(), ToolError> {
        let skill_id = definition.id.clone();

        // 检查技能ID是否已存在
        let mut skills = self.skills.lock().await;
        if skills.contains_key(&skill_id) {
            return Err(ToolError::InvalidArguments(format!("Skill '{}' already exists", skill_id)));
        }

        // 创建执行器（暂时只支持内置类型）
        let executor: Box<dyn SkillExecutor> = match definition.implementation_type {
            SkillImplementationType::Builtin => {
                Box::new(BuiltinSkillExecutor::new(definition.clone()))
            }
            _ => return Err(ToolError::InvalidArguments("Unsupported implementation type".to_string())),
        };

        skills.insert(skill_id.clone(), definition);

        let mut executors = self.executors.lock().await;
        executors.insert(skill_id, executor);

        Ok(())
    }

    /// 执行技能
    async fn execute_skill(&self, skill_id: &str, inputs: HashMap<String, serde_json::Value>, context: &ExecutionContext) -> Result<SkillExecutionResult, ToolError> {
        let executors = self.executors.lock().await;
        let executor = executors.get(skill_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Skill '{}' not found", skill_id)))?;

        // 验证输入
        executor.validate_inputs(&inputs).await
            .map_err(|e| ToolError::InvalidArguments(e))?;

        // 创建执行上下文
        let execution_context = SkillExecutionContext {
            session_id: context.session_id.clone(),
            skill_id: skill_id.to_string(),
            execution_id: format!("exec_{}_{}", skill_id, uuid::Uuid::new_v4().to_string()),
            inputs,
            environment: context.environment.iter()
                .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                .collect(),
            timeout_seconds: context.timeout_seconds,
            debug_mode: false,
        };

        // 执行技能
        let result = executor.execute(&execution_context).await
            .map_err(|e| ToolError::ExecutionFailed(e))?;

        Ok(result)
    }

    /// 获取技能列表
    async fn list_skills(&self, category_filter: Option<String>) -> Vec<SkillDefinition> {
        let skills = self.skills.lock().await;
        if let Some(category) = category_filter {
            skills.values()
                .filter(|skill| skill.category == category)
                .cloned()
                .collect()
        } else {
            skills.values().cloned().collect()
        }
    }

    /// 更新技能
    async fn update_skill(&self, skill_id: &str, updates: serde_json::Value) -> Result<(), ToolError> {
        let mut skills = self.skills.lock().await;
        let skill = skills.get_mut(skill_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Skill '{}' not found", skill_id)))?;

        // 更新字段
        if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
            skill.name = name.to_string();
        }
        if let Some(description) = updates.get("description").and_then(|v| v.as_str()) {
            skill.description = description.to_string();
        }
        if let Some(version) = updates.get("version").and_then(|v| v.as_str()) {
            skill.version = version.to_string();
        }

        skill.updated_at = chrono::Utc::now().timestamp();

        Ok(())
    }

    /// 删除技能
    async fn delete_skill(&self, skill_id: &str) -> Result<(), ToolError> {
        let mut skills = self.skills.lock().await;
        let mut executors = self.executors.lock().await;

        if skills.remove(skill_id).is_none() {
            return Err(ToolError::InvalidArguments(format!("Skill '{}' not found", skill_id)));
        }

        executors.remove(skill_id);
        Ok(())
    }
}

#[async_trait]
impl ToolExecutor for SkillsTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "initialize" => {
                self.initialize_builtin_skills().await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({"message": "Builtin skills initialized"}),
                    error: None,
                    execution_time_ms: 0,
                    output: Some("Successfully initialized builtin skills".to_string()),
                    warnings: vec![],
                    context: None,
                })
            }

            "create_skill" => {
                let definition: SkillDefinition = serde_json::from_value(
                    args.get("definition")
                        .ok_or_else(|| ToolError::InvalidArguments("Missing 'definition' field".to_string()))?
                        .clone()
                ).map_err(|e| ToolError::InvalidArguments(format!("Invalid skill definition: {}", e)))?;

                self.create_skill(definition.clone()).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "skill": definition
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Successfully created skill '{}'", definition.name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "execute_skill" => {
                let skill_id = args.get("skill_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'skill_id' field".to_string()))?;

                let inputs: HashMap<String, serde_json::Value> = args.get("inputs")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let result = self.execute_skill(skill_id, inputs, context).await?;

                Ok(ToolResult {
                    success: result.success,
                    data: serde_json::json!({
                        "skill_id": skill_id,
                        "result": result
                    }),
                    error: result.error,
                    execution_time_ms: result.execution_time_ms,
                    output: Some(format!("Skill '{}' executed", skill_id)),
                    warnings: vec![],
                    context: None,
                })
            }

            "list_skills" => {
                let category_filter = args.get("category")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let skills = self.list_skills(category_filter.clone()).await;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "skills": skills,
                        "category_filter": category_filter
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} skills", skills.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "update_skill" => {
                let skill_id = args.get("skill_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'skill_id' field".to_string()))?;

                let updates = args.get("updates")
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'updates' field".to_string()))?
                    .clone();

                self.update_skill(skill_id, updates).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "skill_id": skill_id,
                        "message": "Skill updated successfully"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Successfully updated skill '{}'", skill_id)),
                    warnings: vec![],
                    context: None,
                })
            }

            "delete_skill" => {
                let skill_id = args.get("skill_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'skill_id' field".to_string()))?;

                self.delete_skill(skill_id).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "skill_id": skill_id,
                        "message": "Skill deleted successfully"
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Successfully deleted skill '{}'", skill_id)),
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
        r#"Skills Management Tool

Create, manage, and execute reusable skills.

Actions:
  - initialize: Initialize builtin skills
  - create_skill: Create a new custom skill
  - execute_skill: Execute a skill with inputs
  - list_skills: List all skills (optionally filtered by category)
  - update_skill: Update an existing skill
  - delete_skill: Delete a skill

Examples:

Initialize builtin skills:
{
  "action": "initialize"
}

Execute a builtin skill:
{
  "action": "execute_skill",
  "skill_id": "text_summarizer",
  "inputs": {
    "text": "This is a long text that needs to be summarized...",
    "max_length": 100
  }
}

Create a custom skill:
{
  "action": "create_skill",
  "definition": {
    "id": "custom_analyzer",
    "name": "Custom Data Analyzer",
    "description": "Analyzes custom data formats",
    "category": "data_processing",
    "implementation_type": "builtin"
  }
}

List skills by category:
{
  "action": "list_skills",
  "category": "text_processing"
}"#
        .to_string()
    }
}