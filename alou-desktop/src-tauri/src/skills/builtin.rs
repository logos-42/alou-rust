//! 内置技能执行器
//!
//! 执行系统内置的技能

use super::executor::{SkillExecutor, SkillExecutionContext, SkillExecutionResult, SkillExecutionStep};
use super::manifest::SkillManifest;
use async_trait::async_trait;

/// 内置技能执行器
pub struct BuiltinSkillExecutor {
    manifest: SkillManifest,
}

impl BuiltinSkillExecutor {
    pub fn new(manifest: SkillManifest) -> Self {
        Self { manifest }
    }

    /// 执行文本摘要
    async fn execute_text_summarizer(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let text = context.get_string_input("text")
            .ok_or("Missing required input: text")?;
        
        let max_length = context.get_int_input("max_length")
            .unwrap_or(200) as usize;

        let start_time = std::time::Instant::now();

        // 简单的摘要算法
        let summary = if text.len() <= max_length {
            text.to_string()
        } else {
            let mut summary = text.chars().take(max_length).collect::<String>();
            // 尝试在最后一个空格处截断
            if let Some(last_space) = summary.rfind(' ') {
                summary.truncate(last_space);
            }
            summary.push_str("...");
            summary
        };

        let elapsed = start_time.elapsed().as_millis() as u64;

        Ok(SkillExecutionResult {
            success: true,
            output: serde_json::json!({
                "summary": summary,
                "original_length": text.len(),
                "summary_length": summary.len()
            }),
            execution_time_ms: elapsed,
            intermediate_steps: vec![
                SkillExecutionStep {
                    name: "summarize".to_string(),
                    step_type: "text_processing".to_string(),
                    input: serde_json::json!({"text_len": text.len(), "max_length": max_length}),
                    output: serde_json::json!({"summary_len": summary.len()}),
                    execution_time_ms: elapsed,
                    timestamp: chrono::Utc::now().timestamp(),
                }
            ],
            error: None,
            metrics: {
                let mut m = std::collections::HashMap::new();
                m.insert("compression_ratio".to_string(), 
                    serde_json::json!(summary.len() as f64 / text.len() as f64));
                m
            },
        })
    }

    /// 执行代码格式化
    async fn execute_code_formatter(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let code = context.get_string_input("code")
            .ok_or("Missing required input: code")?;
        
        let language = context.get_string_input("language")
            .unwrap_or_else(|| "text".to_string());

        let start_time = std::time::Instant::now();

        // 简单的格式化
        let formatted = match language.as_str() {
            "json" => {
                match serde_json::from_str::<serde_json::Value>(&code) {
                    Ok(v) => serde_json::to_string_pretty(&v)
                        .map_err(|e| format!("JSON format error: {}", e))?,
                    Err(e) => return Err(format!("Invalid JSON: {}", e)),
                }
            }
            "rust" => {
                // 简单的 Rust 格式化：去除多余空行，统一缩进
                code.lines()
                    .map(|line| line.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            _ => {
                // 通用格式化：去除行尾空格
                code.lines()
                    .map(|line| line.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };

        let elapsed = start_time.elapsed().as_millis() as u64;

        Ok(SkillExecutionResult {
            success: true,
            output: serde_json::json!({
                "formatted_code": formatted,
                "language": language,
                "original_length": code.len(),
                "formatted_length": formatted.len()
            }),
            execution_time_ms: elapsed,
            intermediate_steps: vec![
                SkillExecutionStep {
                    name: "format".to_string(),
                    step_type: "code_processing".to_string(),
                    input: serde_json::json!({"language": language, "code_len": code.len()}),
                    output: serde_json::json!({"formatted_len": formatted.len()}),
                    execution_time_ms: elapsed,
                    timestamp: chrono::Utc::now().timestamp(),
                }
            ],
            error: None,
            metrics: std::collections::HashMap::new(),
        })
    }

    /// 执行数据验证
    async fn execute_data_validator(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let data = context.get_input("data")
            .ok_or("Missing required input: data")?;
        
        let schema = context.get_input("schema")
            .ok_or("Missing required input: schema")?;

        let start_time = std::time::Instant::now();

        // 简单的类型验证
        let schema_type = schema.get("type").and_then(|t| t.as_str());
        let mut errors = Vec::new();
        let mut is_valid = true;

        if let Some(expected_type) = schema_type {
            let actual_type = match data {
                serde_json::Value::String(_) => "string",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
                serde_json::Value::Null => "null",
            };

            if expected_type != actual_type {
                is_valid = false;
                errors.push(format!(
                    "Type mismatch: expected {}, got {}",
                    expected_type, actual_type
                ));
            }
        }

        // 验证必需字段
        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            if let Some(obj) = data.as_object() {
                for req in required {
                    if let Some(key) = req.as_str() {
                        if !obj.contains_key(key) {
                            is_valid = false;
                            errors.push(format!("Missing required field: {}", key));
                        }
                    }
                }
            }
        }

        let elapsed = start_time.elapsed().as_millis() as u64;

        Ok(SkillExecutionResult {
            success: true,
            output: serde_json::json!({
                "valid": is_valid,
                "errors": errors,
                "data_type": schema_type
            }),
            execution_time_ms: elapsed,
            intermediate_steps: vec![
                SkillExecutionStep {
                    name: "validate".to_string(),
                    step_type: "data_validation".to_string(),
                    input: serde_json::json!({"schema": schema}),
                    output: serde_json::json!({"valid": is_valid, "error_count": errors.len()}),
                    execution_time_ms: elapsed,
                    timestamp: chrono::Utc::now().timestamp(),
                }
            ],
            error: if is_valid { None } else { Some("Validation failed".to_string()) },
            metrics: std::collections::HashMap::new(),
        })
    }

    /// 执行文件分析
    async fn execute_file_analyzer(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let file_path = context.get_string_input("file_path")
            .ok_or("Missing required input: file_path")?;
        
        let operation = context.get_string_input("operation")
            .unwrap_or_else(|| "read".to_string());

        let start_time = std::time::Instant::now();

        match operation.as_str() {
            "read" => {
                let content = tokio::fs::read_to_string(&file_path).await
                    .map_err(|e| format!("Failed to read file: {}", e))?;

                let elapsed = start_time.elapsed().as_millis() as u64;

                Ok(SkillExecutionResult {
                    success: true,
                    output: serde_json::json!({
                        "content": content,
                        "file_path": file_path,
                        "size": content.len()
                    }),
                    execution_time_ms: elapsed,
                    intermediate_steps: vec![
                        SkillExecutionStep {
                            name: "read_file".to_string(),
                            step_type: "file_io".to_string(),
                            input: serde_json::json!({"file_path": file_path}),
                            output: serde_json::json!({"size": content.len()}),
                            execution_time_ms: elapsed,
                            timestamp: chrono::Utc::now().timestamp(),
                        }
                    ],
                    error: None,
                    metrics: std::collections::HashMap::new(),
                })
            }
            "analyze" => {
                let content = tokio::fs::read_to_string(&file_path).await
                    .map_err(|e| format!("Failed to read file: {}", e))?;

                let lines = content.lines().count();
                let words = content.split_whitespace().count();
                let chars = content.chars().count();

                let elapsed = start_time.elapsed().as_millis() as u64;

                Ok(SkillExecutionResult {
                    success: true,
                    output: serde_json::json!({
                        "file_path": file_path,
                        "lines": lines,
                        "words": words,
                        "characters": chars,
                        "size_bytes": content.len(),
                        "analysis": {
                            "avg_words_per_line": if lines > 0 { words as f64 / lines as f64 } else { 0.0 },
                            "avg_chars_per_word": if words > 0 { chars as f64 / words as f64 } else { 0.0 },
                        }
                    }),
                    execution_time_ms: elapsed,
                    intermediate_steps: vec![
                        SkillExecutionStep {
                            name: "analyze_file".to_string(),
                            step_type: "file_analysis".to_string(),
                            input: serde_json::json!({"file_path": file_path}),
                            output: serde_json::json!({"lines": lines, "words": words}),
                            execution_time_ms: elapsed,
                            timestamp: chrono::Utc::now().timestamp(),
                        }
                    ],
                    error: None,
                    metrics: std::collections::HashMap::new(),
                })
            }
            _ => Err(format!("Unknown operation: {}", operation)),
        }
    }
}

#[async_trait]
impl SkillExecutor for BuiltinSkillExecutor {
    async fn execute(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        // 验证输入
        self.validate_inputs(&self.manifest, &context.inputs)?;

        // 根据技能ID执行对应的处理
        match self.manifest.id.as_str() {
            "skill_text_summarizer" => self.execute_text_summarizer(context).await,
            "skill_code_formatter" => self.execute_code_formatter(context).await,
            "skill_data_validator" => self.execute_data_validator(context).await,
            "skill_file_analyzer" => self.execute_file_analyzer(context).await,
            _ => Err(format!("Unknown builtin skill: {}", self.manifest.id)),
        }
    }
}
