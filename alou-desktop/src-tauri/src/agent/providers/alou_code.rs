//! AlouCode Provider
//!
//! Provides integration with alou-code's conversation runtime
//! using the Anthropic API-compatible interface.
//!
//! Alou-code uses ANTHROPIC_API_KEY with ANTHROPIC_BASE_URL set to
//! https://api.deepseek.com for DeepSeek models.

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent, TokenUsage};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://api.deepseek.com/v1/chat/completions";

pub struct AlouCodeProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl AlouCodeProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        let final_base_url = base_url
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_API_URL.to_string());

        Self {
            api_key,
            model,
            base_url: final_base_url,
        }
    }

    pub fn with_deepseek(api_key: String, model: Option<String>) -> Self {
        Self::new(
            api_key,
            model.unwrap_or_else(|| "deepseek-chat".to_string()),
            Some("https://api.deepseek.com".to_string()),
        )
    }

    pub fn with_anthropic(api_key: String, model: Option<String>) -> Self {
        Self::new(
            api_key,
            model.unwrap_or_else(|| "claude-opus-4-6".to_string()),
            Some("https://api.anthropic.com/v1/messages".to_string()),
        )
    }
}

#[derive(Serialize)]
struct AlouCodeRequest {
    model: String,
    messages: Vec<AlouCodeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AlouCodeTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct AlouCodeMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<AlouCodeToolCallInMessage>>,
}

#[derive(Serialize)]
struct AlouCodeToolCallInMessage {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: AlouCodeFunctionCallInMessage,
}

#[derive(Serialize)]
struct AlouCodeFunctionCallInMessage {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct AlouCodeTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: AlouCodeFunction,
}

#[derive(Serialize)]
struct AlouCodeFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct AlouCodeResponse {
    choices: Vec<AlouCodeChoice>,
}

#[derive(Deserialize)]
struct AlouCodeChoice {
    message: AlouCodeResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct AlouCodeResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<AlouCodeToolCall>,
}

#[derive(Deserialize)]
struct AlouCodeToolCall {
    id: String,
    function: AlouCodeFunctionCall,
}

#[derive(Deserialize)]
struct AlouCodeFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait]
impl AiProvider for AlouCodeProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[AlouCode] Sending request to {}", self.base_url);
        log::info!("[AlouCode] Model: {}", self.model);

        let aloucode_messages: Vec<AlouCodeMessage> = messages
            .into_iter()
            .map(|m| {
                let tool_calls_in_msg = m.tool_calls.and_then(|tcs| {
                    if tcs.is_empty() {
                        None
                    } else {
                        Some(tcs
                            .into_iter()
                            .map(|tc| AlouCodeToolCallInMessage {
                                id: tc.id,
                                call_type: "function".to_string(),
                                function: AlouCodeFunctionCallInMessage {
                                    name: tc.name,
                                    arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                                },
                            })
                            .collect())
                    }
                });

                AlouCodeMessage {
                    role: m.role,
                    content: m.content,
                    tool_call_id: m.tool_call_id,
                    tool_calls: tool_calls_in_msg,
                }
            })
            .collect();

        let aloucode_tools = tools.map(|tools| {
            tools.into_iter()
                .map(|t| AlouCodeTool {
                    tool_type: "function".to_string(),
                    function: AlouCodeFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        let request = AlouCodeRequest {
            model: self.model.clone(),
            messages: aloucode_messages,
            tools: aloucode_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;

        let client = Client::new();
        let response = client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .body(body)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        let status = response.status();
        log::info!("[AlouCode] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[AlouCode] API error {}: {}", status, error_text);

            return Err(AgentError::AiError(format!(
                "AlouCode API error {}: {}",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        let aloucode_response: AlouCodeResponse = serde_json::from_str(&response_text).map_err(|e| {
            AgentError::SerializationError(format!("Failed to parse response: {} | Response: {}", e, response_text))
        })?;

        let choice = aloucode_response
            .choices
            .first()
            .ok_or_else(|| AgentError::AiError("Response has no choices".to_string()))?;

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

        log::info!("[AlouCode] Received response, {} tool calls", tool_calls.len());

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
        let stream = futures::stream::empty();
        Ok(Box::pin(stream))
    }
}
