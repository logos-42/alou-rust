use crate::types::*;
use std::collections::HashMap;
use async_trait::async_trait;
use serde_json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// 这些类型现在在 types.rs 中定义

/// 基础工具调用实现
pub struct BaseToolInvocation {
    pub name: String,
    pub params: HashMap<String, serde_json::Value>,
}

impl BaseToolInvocation {
    pub fn new(params: HashMap<String, serde_json::Value>) -> Self {
        Self {
            name: String::new(),
            params,
        }
    }
}

#[async_trait]
impl ToolInvocation for BaseToolInvocation {
    fn name(&self) -> &str {
        &self.name
    }

    fn params(&self) -> &HashMap<String, serde_json::Value> {
        &self.params
    }

    async fn should_confirm_execute(&self, _abort_signal: &CancellationToken) -> Result<Option<ToolCallConfirmationDetails>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(ToolCallConfirmationDetails {
            tool_name: self.name.clone(),
            params: self.params.clone(),
        }))
    }

    async fn execute(&self) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ToolResultContent {
            content: String::new(),
            mime_type: None,
            llm_content: None,
            return_display: None,
        })
    }

    fn get_description(&self) -> &str {
        ""
    }
}

/// 基础声明式工具
pub struct BaseDeclarativeTool {
    pub name: String,
    pub description: String,
    pub display_name: String,
    pub kind: Kind,
    pub parameter_schema: serde_json::Value,
    pub schema: serde_json::Value,
    pub is_output_markdown: bool,
    pub can_update_output: bool,
    pub params: HashMap<String, serde_json::Value>,
}

impl BaseDeclarativeTool {
    pub fn new(
        name: String,
        display_name: String,
        description: String,
        kind: Kind,
        parameter_schema: serde_json::Value,
        is_output_markdown: bool,
        can_update_output: bool,
    ) -> Self {
        Self {
            name,
            display_name,
            description,
            kind,
            parameter_schema: parameter_schema.clone(),
            schema: parameter_schema,
            is_output_markdown,
            can_update_output,
            params: HashMap::new(),
        }
    }
}

#[async_trait]
impl Tool for BaseDeclarativeTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn kind(&self) -> Kind {
        self.kind.clone()
    }

    fn parameter_schema(&self) -> &serde_json::Value {
        &self.parameter_schema
    }

    fn is_output_markdown(&self) -> bool {
        self.is_output_markdown
    }

    fn can_update_output(&self) -> bool {
        self.can_update_output
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn should_confirm_execute(&self, _abort_signal: &CancellationToken) -> Result<Option<ToolCallConfirmationDetails>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(ToolCallConfirmationDetails {
            tool_name: self.name.clone(),
            params: HashMap::new(), // 这里没有具体的参数
        }))
    }

    async fn execute(&self, _params: HashMap<String, serde_json::Value>) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ToolResultContent {
            content: String::new(),
            mime_type: None,
            llm_content: None,
            return_display: None,
        })
    }

    async fn build_and_execute(&self, params: HashMap<String, serde_json::Value>, _abort_signal: Option<&CancellationToken>) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>> {
        let invocation = self.create_invocation(params);
        invocation.execute().await
    }
}

impl BaseDeclarativeTool {
    fn create_invocation(&self, params: HashMap<String, serde_json::Value>) -> BaseToolInvocation {
        BaseToolInvocation::new(params)
    }
}

/// 编辑工具
pub struct EditTool;

impl EditTool {
    pub const NAME: &'static str = "edit";
}

/// 全局工具
pub struct GlobTool;

impl GlobTool {
    pub const NAME: &'static str = "glob";
}

/// Grep工具
pub struct GrepTool;

impl GrepTool {
    pub const NAME: &'static str = "grep";
}

/// 读取文件工具
pub struct ReadFileTool;

impl ReadFileTool {
    pub const NAME: &'static str = "read_file";
}

/// 读取多个文件工具
pub struct ReadManyFilesTool;

impl ReadManyFilesTool {
    pub const NAME: &'static str = "read_many_files";
}

/// Shell工具
pub struct ShellTool;

impl ShellTool {
    pub const NAME: &'static str = "run_shell_command";
}

/// 写入文件工具
pub struct WriteFileTool;

impl WriteFileTool {
    pub const NAME: &'static str = "write_file";
}

/// Todo写入工具
pub struct TodoWriteTool;

impl TodoWriteTool {
    pub const NAME: &'static str = "todo_write";
}

/// 内存工具
pub struct MemoryTool;

impl MemoryTool {
    pub const NAME: &'static str = "save_memory";
    pub const GEMINI_CONFIG_DIR: &'static str = ".gemini";
}

/// Git工具
pub struct GitUtils;

impl GitUtils {
    /// 检查是否为Git仓库
    pub fn is_git_repository(path: &str) -> bool {
        std::path::Path::new(path).join(".git").exists()
    }
}

// 这些trait已经在types.rs中定义了，这里不需要重复定义

/// 声明式工具类型
pub type AnyDeclarativeTool = BaseDeclarativeTool;

/// 工具结果构建器
pub struct ToolResultBuilder {
    content: String,
    mime_type: Option<String>,
    llm_content: Option<serde_json::Value>,
    return_display: Option<String>,
}

impl ToolResultBuilder {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            mime_type: None,
            llm_content: None,
            return_display: None,
        }
    }

    pub fn content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    pub fn llm_content(mut self, llm_content: serde_json::Value) -> Self {
        self.llm_content = Some(llm_content);
        self
    }

    pub fn return_display(mut self, return_display: String) -> Self {
        self.return_display = Some(return_display);
        self
    }

    pub fn build(self) -> ToolResultContent {
        ToolResultContent {
            content: self.content,
            mime_type: self.mime_type,
            llm_content: self.llm_content,
            return_display: self.return_display,
        }
    }
}

impl Default for ToolResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具工厂
pub struct ToolFactory;

impl ToolFactory {
    /// 创建基础工具
    pub fn create_base_tool(
        name: String,
        display_name: String,
        description: String,
        parameter_schema: serde_json::Value,
    ) -> BaseDeclarativeTool {
        BaseDeclarativeTool::new(
            name,
            display_name,
            description,
            Kind::Other,
            parameter_schema,
            true,  // is_output_markdown
            false, // can_update_output
        )
    }

    /// 创建工具调用
    pub fn create_tool_invocation(
        name: String,
        params: HashMap<String, serde_json::Value>,
    ) -> BaseToolInvocation {
        let mut invocation = BaseToolInvocation::new(params);
        invocation.name = name;
        invocation
    }
}

/// 工具验证器
pub struct ToolValidator;

impl ToolValidator {
    /// 验证工具参数
    pub fn validate_params(
        params: &HashMap<String, serde_json::Value>,
        schema: &serde_json::Value,
    ) -> Result<(), String> {
        // 这里可以实现参数验证逻辑
        // 暂时返回Ok
        Ok(())
    }

    /// 验证工具名称
    pub fn validate_tool_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("工具名称不能为空".to_string());
        }
        
        if name.len() > 63 {
            return Err("工具名称长度不能超过63个字符".to_string());
        }
        
        // 检查是否包含无效字符
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
            return Err("工具名称包含无效字符".to_string());
        }
        
        Ok(())
    }
}

/// 工具工具函数
pub struct ToolUtils;

impl ToolUtils {
    /// 生成有效的工具名称
    pub fn generate_valid_name(name: &str) -> String {
        let mut valid_name = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        // 如果长度超过63个字符，截断中间部分
        if valid_name.len() > 63 {
            let start = &valid_name[..28];
            let end = &valid_name[valid_name.len() - 32..];
            valid_name = format!("{}___{}", start, end);
        }

        valid_name
    }

    /// 合并工具参数
    pub fn merge_params(
        base: HashMap<String, serde_json::Value>,
        override_params: HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut merged = base;
        for (key, value) in override_params {
            merged.insert(key, value);
        }
        merged
    }

    /// 序列化工具参数
    pub fn serialize_params(params: &HashMap<String, serde_json::Value>) -> Result<String, serde_json::Error> {
        serde_json::to_string(params)
    }

    /// 反序列化工具参数
    pub fn deserialize_params(json: &str) -> Result<HashMap<String, serde_json::Value>, serde_json::Error> {
        serde_json::from_str(json)
    }
}
