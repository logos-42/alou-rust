//! 技能清单定义
//!
//! 统一技能格式，支持多种实现类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 技能清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    /// 技能ID (skill_xxx 格式)
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 技能描述
    pub description: String,
    /// 技能类别
    pub category: SkillCategory,
    /// 版本号
    pub version: String,
    /// 实现类型
    pub implementation: SkillImplementation,
    /// 输入参数模式 (JSON Schema)
    pub input_schema: serde_json::Value,
    /// 输出结果模式 (JSON Schema)
    pub output_schema: serde_json::Value,
    /// 技能来源
    pub source: SkillSource,
    /// 作者信息
    pub author: Option<AuthorInfo>,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 标签
    pub tags: Vec<String>,
    /// 是否启用
    pub enabled: bool,
}

/// 技能类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    /// 文本处理
    TextProcessing,
    /// 代码处理
    CodeProcessing,
    /// 数据处理
    DataProcessing,
    /// 文件处理
    FileProcessing,
    /// 自动化
    Automation,
    /// Agent技能
    Agent,
    /// 工具组合
    ToolChain,
    /// 其他
    Other,
}

impl SkillCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkillCategory::TextProcessing => "text_processing",
            SkillCategory::CodeProcessing => "code_processing",
            SkillCategory::DataProcessing => "data_processing",
            SkillCategory::FileProcessing => "file_processing",
            SkillCategory::Automation => "automation",
            SkillCategory::Agent => "agent",
            SkillCategory::ToolChain => "tool_chain",
            SkillCategory::Other => "other",
        }
    }
}

/// 技能实现类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SkillImplementation {
    /// 内置实现 (Rust代码)
    Builtin {
        /// 处理函数名
        handler: String,
    },
    /// Prompt模板
    PromptTemplate {
        /// 提示词模板
        template: String,
        /// 系统提示词
        system_prompt: Option<String>,
        /// 使用的模型
        model: Option<String>,
    },
    /// 工具链组合
    ToolChain {
        /// 工具ID列表
        tools: Vec<String>,
        /// 执行流程
        flow: ToolChainFlow,
    },
    /// Agent技能
    AgentSkill {
        /// Agent人设/角色定义
        persona: String,
        /// 能力列表
        capabilities: Vec<String>,
        /// 约束条件
        constraints: Vec<String>,
        /// 执行示例
        examples: Vec<SkillExample>,
        /// 使用的模型
        model: Option<String>,
    },
    /// 脚本代码
    Script {
        /// 脚本语言
        language: String,
        /// 脚本代码
        code: String,
    },
}

/// 工具链流程
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChainFlow {
    /// 顺序执行
    Sequential,
    /// 并行执行
    Parallel,
    /// 条件执行
    Conditional {
        condition: String,
        branches: HashMap<String, Vec<String>>,
    },
}

/// 技能示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    /// 示例名称
    pub name: String,
    /// 输入数据
    pub input: serde_json::Value,
    /// 期望输出
    pub output: serde_json::Value,
    /// 示例说明
    pub description: String,
}

/// 技能来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    /// 系统内置
    System,
    /// 用户创建
    UserCreated,
    /// 从市场导入
    Marketplace,
    /// AI生成
    AiGenerated,
}

/// 作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// 作者名称
    pub name: String,
    /// 作者DID
    pub did: Option<String>,
    /// 联系邮箱
    pub email: Option<String>,
}

/// 技能搜索参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSearchParams {
    /// 搜索关键词
    pub query: String,
    /// 类别过滤
    pub category: Option<SkillCategory>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
    /// 来源过滤
    pub source: Option<SkillSource>,
    /// 只显示启用的
    pub enabled_only: bool,
}

/// 技能列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillListResponse {
    /// 技能列表
    pub skills: Vec<SkillManifest>,
    /// 总数
    pub total: usize,
    /// 搜索参数
    pub params: SkillSearchParams,
}

/// 创建技能请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillRequest {
    /// 显示名称
    pub display_name: String,
    /// 描述
    pub description: String,
    /// 类别
    pub category: SkillCategory,
    /// 实现类型
    pub implementation: SkillImplementation,
    /// 输入模式
    pub input_schema: Option<serde_json::Value>,
    /// 输出模式
    pub output_schema: Option<serde_json::Value>,
    /// 标签
    pub tags: Option<Vec<String>>,
}

/// 执行技能请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteSkillRequest {
    /// 技能ID
    pub skill_id: String,
    /// 输入参数
    pub inputs: serde_json::Value,
    /// 会话ID
    pub session_id: Option<String>,
    /// 调试模式
    pub debug_mode: Option<bool>,
}

/// 执行技能响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteSkillResponse {
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub output: serde_json::Value,
    /// 执行时间(毫秒)
    pub execution_time_ms: u64,
    /// 错误信息
    pub error: Option<String>,
    /// 执行ID
    pub execution_id: String,
    /// 技能ID
    pub skill_id: String,
}

impl SkillManifest {
    /// 创建新的技能清单
    pub fn new(
        display_name: String,
        description: String,
        category: SkillCategory,
        implementation: SkillImplementation,
    ) -> Self {
        let id = format!("skill_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let now = chrono::Utc::now().timestamp();

        // 根据实现类型生成默认的输入输出模式
        let (input_schema, output_schema) = Self::generate_schemas(&implementation);

        Self {
            id,
            display_name,
            description,
            category,
            version: "1.0.0".to_string(),
            implementation,
            input_schema,
            output_schema,
            source: SkillSource::UserCreated,
            author: None,
            created_at: now,
            updated_at: now,
            tags: vec![],
            enabled: true,
        }
    }

    /// 生成默认的输入输出模式
    fn generate_schemas(implementation: &SkillImplementation) -> (serde_json::Value, serde_json::Value) {
        match implementation {
            SkillImplementation::Builtin { handler: _ } => {
                (serde_json::json!({"type": "object"}), serde_json::json!({"type": "object"}))
            }
            SkillImplementation::PromptTemplate { template, .. } => {
                // 从模板中提取变量
                let vars = extract_template_vars(template);
                let properties: HashMap<String, serde_json::Value> = vars.iter()
                    .map(|v| (v.clone(), serde_json::json!({"type": "string"})))
                    .collect();
                
                let input = serde_json::json!({
                    "type": "object",
                    "properties": properties,
                    "required": vars
                });
                let output = serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "string"}
                    }
                });
                (input, output)
            }
            SkillImplementation::AgentSkill { .. } => {
                (serde_json::json!({"type": "object"}), serde_json::json!({"type": "object"}))
            }
            _ => (serde_json::json!({"type": "object"}), serde_json::json!({"type": "object"})),
        }
    }

    /// 更新技能
    pub fn update(&mut self, updates: SkillUpdate) {
        if let Some(name) = updates.display_name {
            self.display_name = name;
        }
        if let Some(desc) = updates.description {
            self.description = desc;
        }
        if let Some(imp) = updates.implementation {
            self.implementation = imp;
            // 重新生成模式
            let (input, output) = Self::generate_schemas(&self.implementation);
            if updates.input_schema.is_none() {
                self.input_schema = input;
            }
            if updates.output_schema.is_none() {
                self.output_schema = output;
            }
        }
        if let Some(input) = updates.input_schema {
            self.input_schema = input;
        }
        if let Some(output) = updates.output_schema {
            self.output_schema = output;
        }
        if let Some(tags) = updates.tags {
            self.tags = tags;
        }
        if let Some(enabled) = updates.enabled {
            self.enabled = enabled;
        }
        
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// 计算搜索相关性分数
    pub fn relevance_score(&self, query: &str) -> u32 {
        let query_lower = query.to_lowercase();
        let mut score = 0u32;

        // 名称匹配 (最高权重)
        if self.display_name.to_lowercase().contains(&query_lower) {
            score += 100;
        }

        // 描述匹配
        if self.description.to_lowercase().contains(&query_lower) {
            score += 50;
        }

        // 标签匹配
        for tag in &self.tags {
            if tag.to_lowercase().contains(&query_lower) {
                score += 30;
            }
        }

        // ID匹配
        if self.id.to_lowercase().contains(&query_lower) {
            score += 20;
        }

        score
    }
}

/// 技能更新参数
#[derive(Debug, Clone, Default)]
pub struct SkillUpdate {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub implementation: Option<SkillImplementation>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

/// 从模板中提取变量 (格式: {{variable}})
fn extract_template_vars(template: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut chars = template.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // 跳过第二个 {
            let mut var_name = String::new();
            while let Some(c) = chars.next() {
                if c == '}' && chars.peek() == Some(&'}') {
                    chars.next(); // 跳过第二个 }
                    if !var_name.is_empty() && !vars.contains(&var_name) {
                        vars.push(var_name);
                    }
                    break;
                }
                var_name.push(c);
            }
        }
    }
    
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_template_vars() {
        let template = "Hello {{name}}, your age is {{age}}";
        let vars = extract_template_vars(template);
        assert_eq!(vars, vec!["name", "age"]);
    }

    #[test]
    fn test_relevance_score() {
        let skill = SkillManifest {
            id: "skill_123".to_string(),
            display_name: "Text Summarizer".to_string(),
            description: "Summarize long text".to_string(),
            category: SkillCategory::TextProcessing,
            version: "1.0.0".to_string(),
            implementation: SkillImplementation::Builtin { handler: "summarize".to_string() },
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            source: SkillSource::System,
            author: None,
            created_at: 0,
            updated_at: 0,
            tags: vec!["text".to_string(), "ai".to_string()],
            enabled: true,
        };

        assert!(skill.relevance_score("text") > 0);
        assert!(skill.relevance_score("summarizer") > 0);
        assert_eq!(skill.relevance_score("xyz"), 0);
    }
}
