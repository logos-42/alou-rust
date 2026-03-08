//! Claude Provider 实现

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct ClaudeProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        }
    }
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ClaudeTool>>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: ClaudeMessageContent,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ClaudeMessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Serialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<Value>,
}

#[derive(Serialize)]
struct ClaudeTool {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ResponseContentBlock>,
    stop_reason: String,
}

#[derive(Deserialize)]
struct ResponseContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<Value>,
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[Claude] 发送请求到 {}", self.base_url);

        let claude_messages: Vec<ClaudeMessage> = messages
            .into_iter()
            .map(|m| ClaudeMessage {
                role: m.role,
                content: ClaudeMessageContent::Text(m.content),
            })
            .collect();

        let claude_tools = tools.map(|tools| {
            tools.into_iter()
                .map(|t| ClaudeTool {
                    name: t.name,
                    description: t.description,
                    input_schema: t.parameters,
                })
                .collect()
        });

        let request = ClaudeRequest {
            model: self.model.clone(),
            messages: claude_messages,
            tools: claude_tools,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;

        log::debug!("[Claude] Request body: {}", body);

        let client = Client::new();
        let response = client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .body(body)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        let status = response.status();
        log::info!("[Claude] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[Claude] API error {}: {}", status, error_text);

            return Err(AgentError::AiError(format!(
                "Claude API error {}: {}",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        log::debug!("[Claude] Response: {}", &response_text[..response_text.len().min(500)]);

        let claude_response: ClaudeResponse = serde_json::from_str(&response_text).map_err(|e| {
            AgentError::SerializationError(format!("解析响应失败: {} | 响应: {}", e, response_text))
        })?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for block in &claude_response.content {
            match block.block_type.as_str() {
                "text" => {
                    content.push_str(block.text.as_deref().unwrap_or(""));
                }
                "tool_use" => {
                    if let Some(id) = &block.id {
                        if let Some(name) = &block.name {
                            if let Some(input) = &block.input {
                                tool_calls.push(AiToolCall {
                                    id: id.clone(),
                                    name: name.clone(),
                                    arguments: input.clone(),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        log::info!("[Claude] 收到响应, {} 个工具调用", tool_calls.len());

        Ok(AiResponse {
            content,
            tool_calls,
            finish_reason: claude_response.stop_reason.clone(),
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
