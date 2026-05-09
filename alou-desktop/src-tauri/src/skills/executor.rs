//! 技能执行器
//!
//! 提供统一的技能执行接口

use super::manifest::{SkillManifest, SkillImplementation};
use async_trait::async_trait;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 技能执行上下文
#[derive(Debug, Clone)]
pub struct SkillExecutionContext {
    /// 会话ID
    pub session_id: String,
    /// 执行ID
    pub execution_id: String,
    /// 技能ID
    pub skill_id: String,
    /// 输入参数
    pub inputs: HashMap<String, serde_json::Value>,
    /// 环境变量
    pub environment: HashMap<String, String>,
    /// 超时时间(秒)
    pub timeout_seconds: Option<u64>,
    /// 调试模式
    pub debug_mode: bool,
}

impl SkillExecutionContext {
    /// 创建新的执行上下文
    pub fn new(session_id: String, skill_id: String) -> Self {
        Self {
            session_id,
            execution_id: format!("exec_{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            skill_id,
            inputs: HashMap::new(),
            environment: std::env::vars().collect(),
            timeout_seconds: Some(60),
            debug_mode: false,
        }
    }

    /// 设置输入参数
    pub fn with_inputs(mut self, inputs: HashMap<String, serde_json::Value>) -> Self {
        self.inputs = inputs;
        self
    }

    /// 获取输入参数
    pub fn get_input(&self, key: &str) -> Option<&serde_json::Value> {
        self.inputs.get(key)
    }

    /// 获取字符串输入
    pub fn get_string_input(&self, key: &str) -> Option<String> {
        self.inputs.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    /// 获取整数输入
    pub fn get_int_input(&self, key: &str) -> Option<i64> {
        self.inputs.get(key).and_then(|v| v.as_i64())
    }

    /// 获取布尔输入
    pub fn get_bool_input(&self, key: &str) -> Option<bool> {
        self.inputs.get(key).and_then(|v| v.as_bool())
    }
}

/// 技能执行结果
#[derive(Debug, Clone)]
pub struct SkillExecutionResult {
    /// 是否成功
    pub success: bool,
    /// 输出数据
    pub output: serde_json::Value,
    /// 执行时间(毫秒)
    pub execution_time_ms: u64,
    /// 中间步骤记录
    pub intermediate_steps: Vec<SkillExecutionStep>,
    /// 错误信息
    pub error: Option<String>,
    /// 性能指标
    pub metrics: HashMap<String, serde_json::Value>,
}

impl SkillExecutionResult {
    /// 创建成功结果
    pub fn success(output: serde_json::Value) -> Self {
        Self {
            success: true,
            output,
            execution_time_ms: 0,
            intermediate_steps: vec![],
            error: None,
            metrics: HashMap::new(),
        }
    }

    /// 创建失败结果
    pub fn error(error_message: String) -> Self {
        Self {
            success: false,
            output: serde_json::Value::Null,
            execution_time_ms: 0,
            intermediate_steps: vec![],
            error: Some(error_message),
            metrics: HashMap::new(),
        }
    }

    /// 添加执行时间
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = ms;
        self
    }

    /// 添加中间步骤
    pub fn with_step(mut self, step: SkillExecutionStep) -> Self {
        self.intermediate_steps.push(step);
        self
    }

    /// 转换为JSON
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "success": self.success,
            "output": self.output,
            "execution_time_ms": self.execution_time_ms,
            "intermediate_steps": self.intermediate_steps,
            "error": self.error,
            "metrics": self.metrics,
        })
    }
}

/// 执行步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionStep {
    /// 步骤名称
    pub name: String,
    /// 步骤类型
    pub step_type: String,
    /// 输入
    pub input: serde_json::Value,
    /// 输出
    pub output: serde_json::Value,
    /// 执行时间(毫秒)
    pub execution_time_ms: u64,
    /// 时间戳
    pub timestamp: i64,
}

/// 技能执行器 trait
#[async_trait]
pub trait SkillExecutor: Send + Sync {
    /// 执行技能
    async fn execute(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String>;

    /// 验证输入参数
    fn validate_inputs(&self, manifest: &SkillManifest, inputs: &HashMap<String, serde_json::Value>) -> Result<(), String> {
        // 检查必需参数
        if let Some(required) = manifest.input_schema.get("required").and_then(|r| r.as_array()) {
            for req in required {
                if let Some(key) = req.as_str() {
                    if !inputs.contains_key(key) {
                        return Err(format!("Missing required input: {}", key));
                    }
                }
            }
        }

        // 检查参数类型
        if let Some(properties) = manifest.input_schema.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in inputs {
                if let Some(prop_def) = properties.get(key) {
                    if let Some(expected_type) = prop_def.get("type").and_then(|t| t.as_str()) {
                        let actual_type = match value {
                            serde_json::Value::String(_) => "string",
                            serde_json::Value::Number(_) => "number",
                            serde_json::Value::Bool(_) => "boolean",
                            serde_json::Value::Array(_) => "array",
                            serde_json::Value::Object(_) => "object",
                            serde_json::Value::Null => "null",
                        };

                        if expected_type != actual_type && actual_type != "null" {
                            return Err(format!(
                                "Invalid type for '{}': expected {}, got {}",
                                key, expected_type, actual_type
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// 执行器工厂
pub struct SkillExecutorFactory;

impl SkillExecutorFactory {
    /// 根据技能类型创建对应的执行器
    pub fn create(manifest: &SkillManifest) -> Result<Box<dyn SkillExecutor>, String> {
        match &manifest.implementation {
            SkillImplementation::Builtin { .. } => {
                Ok(Box::new(super::builtin::BuiltinSkillExecutor::new(manifest.clone())))
            }
            SkillImplementation::AgentSkill { .. } => {
                // Agent技能需要LLM客户端，这里返回占位
                Err("Agent skill executor requires LLM client".to_string())
            }
            SkillImplementation::PromptTemplate { .. } => {
                Ok(Box::new(super::prompt::PromptTemplateExecutor::new(manifest.clone())))
            }
            SkillImplementation::ToolChain { .. } => {
                Ok(Box::new(super::toolchain::ToolChainExecutor::new(manifest.clone())))
            }
            SkillImplementation::Script { language, .. } => {
                Ok(Box::new(super::script::ScriptExecutor::new(manifest.clone(), language.clone())))
            }
        }
    }
}
