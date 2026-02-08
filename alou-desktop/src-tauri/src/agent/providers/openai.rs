//! OpenAI Provider 实现

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        }
    }
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAiTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAiTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAiFunction,
}

#[derive(Serialize)]
struct OpenAiFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<OpenAiToolCall>,
}

#[derive(Deserialize)]
struct OpenAiToolCall {
    id: String,
    function: OpenAiFunctionCall,
}

#[derive(Deserialize)]
struct OpenAiFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[OpenAI] 发送请求到 {}", self.base_url);
        log::info!("[OpenAI] Model: {}, Messages: {}", self.model, messages.len());

        let openai_messages: Vec<OpenAiMessage> = messages
            .into_iter()
            .map(|m| OpenAiMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let openai_tools = tools.map(|tools| {
            tools.into_iter()
                .map(|t| OpenAiTool {
                    tool_type: "function".to_string(),
                    function: OpenAiFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: openai_messages,
            tools: openai_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;

        log::debug!("[OpenAI] Request body: {}", body);

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
        log::info!("[OpenAI] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[OpenAI] API error {}: {}", status, error_text);

            return Err(AgentError::AiError(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        log::debug!("[OpenAI] Response: {}", &response_text[..response_text.len().min(500)]);

        let openai_response: OpenAiResponse = serde_json::from_str(&response_text).map_err(|e| {
            AgentError::SerializationError(format!("解析响应失败: {} | 响应: {}", e, response_text))
        })?;

        let choice = openai_response
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

        log::info!("[OpenAI] 收到响应, {} 个工具调用", tool_calls.len());

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
        // 流式响应实现
        // TODO: 实现 SSE 流式响应
        let stream = futures::stream::empty();
        Ok(Box::pin(stream))
    }
}
