//! OpenRouter Provider 实现
//! 
//! OpenRouter 是一个统一的 API 网关，支持访问多种 LLM 模型
//! 官网：https://openrouter.ai/
//! 文档：https://openrouter.ai/docs

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        }
    }
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenRouterTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenRouterToolCallInMessage>>,
}

#[derive(Serialize)]
struct OpenRouterToolCallInMessage {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenRouterFunctionCallInMessage,
}

#[derive(Serialize)]
struct OpenRouterFunctionCallInMessage {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct OpenRouterTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenRouterFunction,
}

#[derive(Serialize)]
struct OpenRouterFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct OpenRouterResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<OpenRouterToolCall>,
}

#[derive(Deserialize)]
struct OpenRouterToolCall {
    id: String,
    function: OpenRouterFunctionCall,
}

#[derive(Deserialize)]
struct OpenRouterFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait]
impl AiProvider for OpenRouterProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[OpenRouter] 发送请求到 {}", self.base_url);
        log::info!("[OpenRouter] Model: {}, Messages: {}", self.model, messages.len());

        let client = Client::new();

        let openrouter_messages: Vec<OpenRouterMessage> = messages
            .into_iter()
            .map(|m| {
                let tool_calls_in_msg = m.tool_calls.and_then(|tcs| {
                    if tcs.is_empty() {
                        None
                    } else {
                        Some(tcs.into_iter()
                            .map(|tc| OpenRouterToolCallInMessage {
                                id: tc.id,
                                call_type: "function".to_string(),
                                function: OpenRouterFunctionCallInMessage {
                                    name: tc.name,
                                    arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                                },
                            })
                            .collect())
                    }
                });

                OpenRouterMessage {
                    role: m.role,
                    content: m.content,
                    tool_call_id: m.tool_call_id,
                    tool_calls: tool_calls_in_msg,
                }
            })
            .collect();

        let openrouter_tools = tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| OpenRouterTool {
                    tool_type: "function".to_string(),
                    function: OpenRouterFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages: openrouter_messages,
            tools: openrouter_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::AgentError(format!("Serialize error: {}", e)))?;

        if self.api_key.is_empty() {
            return Err(AgentError::AgentError(
                "OpenRouter API key is empty. Please configure your API key.".to_string()
            ));
        }

        let response = client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/logos-42/alou-rust")
            .header("X-Title", "Alou AI Agent")
            .body(body)
            .send()
            .await
            .map_err(|e| AgentError::AgentError(format!("Request failed: {}", e)))?;

        let status = response.status();
        log::info!("[OpenRouter] 响应状态码：{}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[OpenRouter] API 错误：{}", error_text);
            return Err(AgentError::AgentError(
                format!("OpenRouter API error ({}): {}", status, error_text)
            ));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::AgentError(format!("Read response failed: {}", e)))?;

        let openrouter_response: OpenRouterResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::AgentError(format!("Parse error: {} | Response: {}", e, response_text)))?;

        let choice = openrouter_response
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

        log::info!("[OpenRouter] 响应收到，{} tool calls", tool_calls.len());

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
        Err(AgentError::AgentError("OpenRouter streaming not yet implemented".to_string()))
    }
}
