//! GLM (智谱 AI) Provider 实现
//! 
//! 智谱 AI 是中国领先的 AI 公司，提供 GLM 系列大模型
//! 官网：https://www.zhipuai.cn/
//! 文档：https://open.bigmodel.cn/dev/api

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://open.bigmodel.cn/api/paas/v4/chat/completions";

pub struct GlmProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl GlmProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        }
    }
}

#[derive(Serialize)]
struct GlmRequest {
    model: String,
    messages: Vec<GlmMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GlmTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct GlmMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<GlmToolCallInMessage>>,
}

#[derive(Serialize)]
struct GlmToolCallInMessage {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: GlmFunctionCallInMessage,
}

#[derive(Serialize)]
struct GlmFunctionCallInMessage {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct GlmTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: GlmFunction,
}

#[derive(Serialize)]
struct GlmFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct GlmResponse {
    choices: Vec<GlmChoice>,
}

#[derive(Deserialize)]
struct GlmChoice {
    message: GlmResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct GlmResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<GlmToolCall>,
}

#[derive(Deserialize)]
struct GlmToolCall {
    id: String,
    function: GlmFunctionCall,
}

#[derive(Deserialize)]
struct GlmFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait]
impl AiProvider for GlmProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[GLM] 发送请求到 {}", self.base_url);
        log::info!("[GLM] Model: {}, Messages: {}", self.model, messages.len());

        let client = Client::new();

        let glm_messages: Vec<GlmMessage> = messages
            .into_iter()
            .map(|m| {
                let tool_calls_in_msg = m.tool_calls.and_then(|tcs| {
                    if tcs.is_empty() {
                        None
                    } else {
                        Some(tcs.into_iter()
                            .map(|tc| GlmToolCallInMessage {
                                id: tc.id,
                                call_type: "function".to_string(),
                                function: GlmFunctionCallInMessage {
                                    name: tc.name,
                                    arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                                },
                            })
                            .collect())
                    }
                });

                GlmMessage {
                    role: m.role,
                    content: m.content,
                    tool_call_id: m.tool_call_id,
                    tool_calls: tool_calls_in_msg,
                }
            })
            .collect();

        let glm_tools = tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| GlmTool {
                    tool_type: "function".to_string(),
                    function: GlmFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        let request = GlmRequest {
            model: self.model.clone(),
            messages: glm_messages,
            tools: glm_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::AgentError(format!("Serialize error: {}", e)))?;

        if self.api_key.is_empty() {
            return Err(AgentError::AgentError(
                "GLM API key is empty. Please configure your API key.".to_string()
            ));
        }

        let response = client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AgentError::AgentError(format!("Request failed: {}", e)))?;

        let status = response.status();
        log::info!("[GLM] 响应状态码：{}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[GLM] API 错误：{}", error_text);
            return Err(AgentError::AgentError(
                format!("GLM API error ({}): {}", status, error_text)
            ));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::AgentError(format!("Read response failed: {}", e)))?;

        let glm_response: GlmResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::AgentError(format!("Parse error: {} | Response: {}", e, response_text)))?;

        let choice = glm_response
            .choices
            .first()
            .ok_or_else(|| AgentError::AgentError("No choices in response".to_string()))?;

        let content = choice.message.content.clone().unwrap_or_default();

        let tool_calls: Vec<AiToolCall> = choice
            .message
            .tool_calls
            .iter()
            .filter_map(|tc| {
                let arguments = serde_json::from_str(&tc.function.arguments).ok()?;
                Some(AiToolCall {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments,
                })
            })
            .collect();

        log::info!("[GLM] 响应收到，{} tool calls", tool_calls.len());

        Ok(AiResponse {
            content,
            tool_calls,
            finish_reason: choice.finish_reason.clone(),
            usage: None,
        })
    }

    async fn send_message_stream(
        &self,
        _messages: Vec<AiMessage>,
        _tools: Option<Vec<AiTool>>,
    ) -> Result<BoxStream<'static, std::result::Result<AiStreamEvent, AgentError>>> {
        Err(AgentError::AgentError("GLM streaming not yet implemented".to_string()))
    }
}
