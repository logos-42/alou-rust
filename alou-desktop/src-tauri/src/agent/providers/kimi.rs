//! Kimi Provider 实现

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://api.moonshot.cn/v1/chat/completions";

pub struct KimiProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl KimiProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        }
    }
}

#[derive(Serialize)]
struct KimiRequest {
    model: String,
    messages: Vec<KimiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<KimiTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct KimiMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct KimiTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: KimiFunction,
}

#[derive(Serialize)]
struct KimiFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct KimiResponse {
    choices: Vec<KimiChoice>,
}

#[derive(Deserialize)]
struct KimiChoice {
    message: KimiResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct KimiResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<KimiToolCall>,
}

#[derive(Deserialize)]
struct KimiToolCall {
    id: String,
    function: KimiFunctionCall,
}

#[derive(Deserialize)]
struct KimiFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait]
impl AiProvider for KimiProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[Kimi] 发送请求到 {}", self.base_url);

        let kimi_messages: Vec<KimiMessage> = messages
            .into_iter()
            .map(|m| KimiMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let kimi_tools = tools.map(|tools| {
            tools.into_iter()
                .map(|t| KimiTool {
                    tool_type: "function".to_string(),
                    function: KimiFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        let request = KimiRequest {
            model: self.model.clone(),
            messages: kimi_messages,
            tools: kimi_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;

        log::debug!("[Kimi] Request body: {}", body);

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
        log::info!("[Kimi] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[Kimi] API error {}: {}", status, error_text);

            return Err(AgentError::AiError(format!(
                "Kimi API error {}: {}",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        log::debug!("[Kimi] Response: {}", &response_text[..response_text.len().min(500)]);

        let kimi_response: KimiResponse = serde_json::from_str(&response_text).map_err(|e| {
            AgentError::SerializationError(format!("解析响应失败: {} | 响应: {}", e, response_text))
        })?;

        let choice = kimi_response
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

        log::info!("[Kimi] 收到响应, {} 个工具调用", tool_calls.len());

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
