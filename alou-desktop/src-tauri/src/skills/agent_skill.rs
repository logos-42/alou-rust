//! Agent 技能执行器
//!
//! 执行基于AI Agent的技能

use super::executor::{SkillExecutor, SkillExecutionContext, SkillExecutionResult, SkillExecutionStep};
use super::manifest::SkillManifest;
use async_trait::async_trait;
use std::sync::Arc;

/// Agent 技能执行器
pub struct AgentSkillExecutor {
    manifest: SkillManifest,
    llm_client: Arc<dyn LlmClient>,
}

/// LLM 客户端接口
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 发送聊天完成请求
    async fn chat_completion(
        &self,
        system_prompt: &str,
        user_message: &str,
        model: Option<String>,
    ) -> Result<String, String>;
}

/// 简单的 LLM 客户端实现 (基于HTTP API)
pub struct HttpLlmClient {
    api_key: String,
    api_endpoint: String,
    default_model: String,
}

impl HttpLlmClient {
    pub fn new(api_key: String, api_endpoint: String, default_model: String) -> Self {
        Self {
            api_key,
            api_endpoint,
            default_model,
        }
    }
}

#[async_trait]
impl LlmClient for HttpLlmClient {
    async fn chat_completion(
        &self,
        system_prompt: &str,
        user_message: &str,
        model: Option<String>,
    ) -> Result<String, String> {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

        let client = reqwest::Client::new();
        
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|e| format!("Invalid API key: {}", e))?,
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let request_body = serde_json::json!({
            "model": model.unwrap_or_else(|| self.default_model.clone()),
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_message
                }
            ],
            "temperature": 0.7,
            "max_tokens": 4000,
        });

        let response = client
            .post(&self.api_endpoint)
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("API error ({}): {}", status, response_text));
        }

        // 解析响应
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| "Invalid response format".to_string())?;

        Ok(content.to_string())
    }
}

impl AgentSkillExecutor {
    /// 创建新的 Agent 技能执行器
    pub fn new(manifest: SkillManifest, llm_client: Arc<dyn LlmClient>) -> Self {
        Self { manifest, llm_client }
    }

    /// 构建系统提示词
    fn build_system_prompt(&self, persona: &str, capabilities: &[String], constraints: &[String]) -> String {
        let capabilities_text = if capabilities.is_empty() {
            "无特定能力".to_string()
        } else {
            capabilities.join("\n- ")
        };

        let constraints_text = if constraints.is_empty() {
            "无特定约束".to_string()
        } else {
            constraints.join("\n- ")
        };

        format!(
            r#"{}

## 你的能力
- {}

## 约束条件
- {}

## 执行要求
1. 仔细理解用户的输入
2. 使用你的能力完成请求的任务
3. 返回结构化的JSON格式结果
4. 如果无法完成，说明原因

## 输出格式
请始终返回JSON格式的结果:
{{
  "success": true/false,
  "result": "你的执行结果",
  "details": "额外信息（可选）"
}}"#,
            persona,
            capabilities_text,
            constraints_text
        )
    }

    /// 构建用户输入
    fn build_user_input(&self, inputs: &std::collections::HashMap<String, serde_json::Value>) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "inputs": inputs,
            "skill_name": self.manifest.display_name,
            "timestamp": chrono::Utc::now().timestamp(),
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }
}

#[async_trait]
impl SkillExecutor for AgentSkillExecutor {
    async fn execute(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        let start_time = std::time::Instant::now();

        // 提取 Agent Skill 配置
        let (persona, capabilities, constraints, model) = match &self.manifest.implementation {
            super::manifest::SkillImplementation::AgentSkill {
                persona,
                capabilities,
                constraints,
                model,
                ..
            } => (persona.clone(), capabilities.clone(), constraints.clone(), model.clone()),
            _ => return Err("Not an agent skill".to_string()),
        };

        // 构建提示词
        let system_prompt = self.build_system_prompt(&persona, &capabilities, &constraints);
        let user_input = self.build_user_input(&context.inputs);

        // 记录输入步骤
        let mut intermediate_steps = vec![
            SkillExecutionStep {
                name: "prepare_prompt".to_string(),
                step_type: "prompt_building".to_string(),
                input: serde_json::json!({"system_prompt_len": system_prompt.len()}),
                output: serde_json::json!({"user_input_len": user_input.len()}),
                execution_time_ms: 0,
                timestamp: chrono::Utc::now().timestamp(),
            }
        ];

        // 调用 LLM
        let model_for_call = model.clone();
        let llm_result = self.llm_client
            .chat_completion(&system_prompt, &user_input, model_for_call)
            .await;

        match llm_result {
            Ok(response) => {
                let elapsed = start_time.elapsed().as_millis() as u64;

                // 尝试解析为JSON
                let output = match serde_json::from_str::<serde_json::Value>(&response) {
                    Ok(json) => json,
                    Err(_) => {
                        // 如果不是JSON，包装为对象
                        serde_json::json!({
                            "success": true,
                            "result": response,
                            "format": "text"
                        })
                    }
                };

                // 记录LLM调用步骤
                intermediate_steps.push(SkillExecutionStep {
                    name: "llm_call".to_string(),
                    step_type: "llm_inference".to_string(),
                    input: serde_json::json!({"model": model}),
                    output: serde_json::json!({"response_len": response.len()}),
                    execution_time_ms: elapsed,
                    timestamp: chrono::Utc::now().timestamp(),
                });

                Ok(SkillExecutionResult {
                    success: true,
                    output,
                    execution_time_ms: elapsed,
                    intermediate_steps,
                    error: None,
                    metrics: {
                        let mut m = std::collections::HashMap::new();
                        m.insert("prompt_tokens".to_string(), serde_json::json!(system_prompt.len() / 4));
                        m.insert("completion_tokens".to_string(), serde_json::json!(response.len() / 4));
                        m
                    },
                })
            }
            Err(e) => {
                let elapsed = start_time.elapsed().as_millis() as u64;

                intermediate_steps.push(SkillExecutionStep {
                    name: "llm_call_failed".to_string(),
                    step_type: "error".to_string(),
                    input: serde_json::json!({}),
                    output: serde_json::json!({"error": e.clone()}),
                    execution_time_ms: elapsed,
                    timestamp: chrono::Utc::now().timestamp(),
                });

                Ok(SkillExecutionResult {
                    success: false,
                    output: serde_json::Value::Null,
                    execution_time_ms: elapsed,
                    intermediate_steps,
                    error: Some(e),
                    metrics: std::collections::HashMap::new(),
                })
            }
        }
    }
}

/// 创建 Agent Skill 的便捷函数
pub fn create_agent_skill(
    display_name: String,
    description: String,
    persona: String,
    capabilities: Vec<String>,
    constraints: Vec<String>,
    examples: Vec<super::manifest::SkillExample>,
) -> SkillManifest {
    use super::manifest::{SkillManifest, SkillCategory, SkillImplementation, SkillSource};

    let mut manifest = SkillManifest::new(
        display_name,
        description,
        SkillCategory::Agent,
        SkillImplementation::AgentSkill {
            persona,
            capabilities,
            constraints,
            examples,
            model: None,
        },
    );

    manifest.source = SkillSource::UserCreated;
    manifest
}
