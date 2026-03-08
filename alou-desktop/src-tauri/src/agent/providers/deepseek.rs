//! DeepSeek Provider 实现

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://api.deepseek.com/v1/chat/completions";

pub struct DeepSeekProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        }
    }
}

#[derive(Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<DeepSeekTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<DeepSeekToolCallInMessage>>,
}

#[derive(Serialize)]
struct DeepSeekToolCallInMessage {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: DeepSeekFunctionCallInMessage,
}

#[derive(Serialize)]
struct DeepSeekFunctionCallInMessage {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct DeepSeekTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: DeepSeekFunction,
}

#[derive(Serialize)]
struct DeepSeekFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct DeepSeekResponse {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct DeepSeekResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<DeepSeekToolCall>,
}

#[derive(Deserialize)]
struct DeepSeekToolCall {
    id: String,
    function: DeepSeekFunctionCall,
}

#[derive(Deserialize)]
struct DeepSeekFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait]
impl AiProvider for DeepSeekProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[DeepSeek] 发送请求到 {}", self.base_url);
        log::info!("[DeepSeek] Model: {}, Messages: {}", self.model, messages.len());

        let deepseek_messages: Vec<DeepSeekMessage> = messages
            .into_iter()
            .map(|m| {
                let tool_calls_in_msg = m.tool_calls.and_then(|tcs| {
                    if tcs.is_empty() {
                        None
                    } else {
                        Some(tcs
                            .into_iter()
                            .map(|tc| DeepSeekToolCallInMessage {
                                id: tc.id,
                                call_type: "function".to_string(),
                                function: DeepSeekFunctionCallInMessage {
                                    name: tc.name,
                                    arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                                },
                            })
                            .collect())
                    }
                });

                DeepSeekMessage {
                    role: m.role,
                    content: m.content,
                    tool_call_id: m.tool_call_id,
                    tool_calls: tool_calls_in_msg,
                }
            })
            .collect();

        let deepseek_tools = tools.map(|tools| {
            tools.into_iter()
                .map(|t| DeepSeekTool {
                    tool_type: "function".to_string(),
                    function: DeepSeekFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        let request = DeepSeekRequest {
            model: self.model.clone(),
            messages: deepseek_messages,
            tools: deepseek_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;

        log::debug!("[DeepSeek] Request body: {}", body);

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
        log::info!("[DeepSeek] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[DeepSeek] API error {}: {}", status, error_text);

            return Err(AgentError::AiError(format!(
                "DeepSeek API error {}: {}",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        log::debug!("[DeepSeek] Response: {}", &response_text[..response_text.len().min(500)]);

        let deepseek_response: DeepSeekResponse = serde_json::from_str(&response_text).map_err(|e| {
            AgentError::SerializationError(format!("解析响应失败: {} | 响应: {}", e, response_text))
        })?;

        let choice = deepseek_response
            .choices
            .first()
            .ok_or_else(|| AgentError::AiError("响应中没有 choices".to_string()))?;

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

        log::info!("[DeepSeek] 收到响应, {} 个工具调用", tool_calls.len());

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
        // 流式响应实现（类似 send_message，但使用 SSE）
        // TODO: 实现流式响应
        let stream = futures::stream::empty();
        Ok(Box::pin(stream))
    }
}
