//! Prompt 模板执行器
//!
//! 执行基于Prompt模板的技能

use super::executor::{SkillExecutor, SkillExecutionContext, SkillExecutionResult, SkillExecutionStep};
use super::manifest::{SkillManifest, SkillImplementation};
use async_trait::async_trait;

/// Prompt 模板执行器
pub struct PromptTemplateExecutor {
    manifest: SkillManifest,
}

impl PromptTemplateExecutor {
    pub fn new(manifest: SkillManifest) -> Self {
        Self { manifest }
    }

    /// 渲染模板
    fn render_template(&self, template: &str, inputs: &std::collections::HashMap<String, serde_json::Value>) -> String {
        let mut result = template.to_string();
        
        // 替换 {{variable}} 格式的变量
        for (key, value) in inputs {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        result
    }
}

#[async_trait]
impl SkillExecutor for PromptTemplateExecutor {
    async fn execute(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let start_time = std::time::Instant::now();

        // 提取 Prompt 配置
        let (template, system_prompt) = match &self.manifest.implementation {
            SkillImplementation::PromptTemplate { template, system_prompt, .. } => {
                (template.clone(), system_prompt.clone())
            }
            _ => return Err("Not a prompt template skill".to_string()),
        };

        // 验证输入
        self.validate_inputs(&self.manifest, &context.inputs)?;

        // 渲染模板
        let rendered = self.render_template(&template, &context.inputs);

        let elapsed = start_time.elapsed().as_millis() as u64;

        // 构建结果
        let output = serde_json::json!({
            "rendered_prompt": rendered,
            "system_prompt": system_prompt,
            "template_variables": context.inputs.keys().collect::<Vec<_>>(),
        });

        Ok(SkillExecutionResult {
            success: true,
            output,
            execution_time_ms: elapsed,
            intermediate_steps: vec![
                SkillExecutionStep {
                    name: "render_template".to_string(),
                    step_type: "template_rendering".to_string(),
                    input: serde_json::json!({"template_len": template.len()}),
                    output: serde_json::json!({"rendered_len": rendered.len()}),
                    execution_time_ms: elapsed,
                    timestamp: chrono::Utc::now().timestamp(),
                }
            ],
            error: None,
            metrics: {
                let mut m = std::collections::HashMap::new();
                m.insert("template_variables".to_string(), 
                    serde_json::json!(context.inputs.len()));
                m
            },
        })
    }
}
