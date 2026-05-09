//! MiniMax 文本 Provider 实现
//! 
//! MiniMax 是中国领先的 AI 大模型公司
//! 官网：https://platform.minimaxi.com/
//! 文档：https://platform.minimaxi.com/document

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://api.minimaxi.com/v1/text/chatcompletion_v2";

pub struct MiniMaxTextProvider {
    api_key: String,
    model: String,
    base_url: String,
    group_id: String,
}

impl MiniMaxTextProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>, group_id: String) -> Self {
        // 修复：处理空字符串的情况，避免 "relative URL without a base" 错误
        let final_base_url = base_url
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_API_URL.to_string());

        Self {
            api_key,
            model,
            base_url: final_base_url,
            group_id,
        }
    }
}

#[derive(Serialize)]
struct MiniMaxRequest {
    model: String,
    messages: Vec<MiniMaxMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<MiniMaxTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct MiniMaxMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<MiniMaxToolCallInMessage>>,
}

#[derive(Serialize)]
struct MiniMaxToolCallInMessage {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: MiniMaxFunctionCallInMessage,
}

#[derive(Serialize)]
struct MiniMaxFunctionCallInMessage {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct MiniMaxTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: MiniMaxFunction,
}

#[derive(Serialize)]
struct MiniMaxFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct MiniMaxResponse {
    choices: Vec<MiniMaxChoice>,
}

#[derive(Deserialize)]
struct MiniMaxChoice {
    message: MiniMaxResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct MiniMaxResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<MiniMaxToolCall>,
}

#[derive(Deserialize)]
struct MiniMaxToolCall {
    id: String,
    function: MiniMaxFunctionCall,
}

#[derive(Deserialize)]
struct MiniMaxFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait]
impl AiProvider for MiniMaxTextProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[MiniMax Text] 发送请求到 {}", self.base_url);
        log::info!("[MiniMax Text] Model: {}, Messages: {}", self.model, messages.len());

        let client = Client::new();

        let minimax_messages: Vec<MiniMaxMessage> = messages
            .into_iter()
            .map(|m| {
                let tool_calls_in_msg = m.tool_calls.and_then(|tcs| {
                    if tcs.is_empty() {
                        None
                    } else {
                        Some(tcs.into_iter()
                            .map(|tc| MiniMaxToolCallInMessage {
                                id: tc.id,
                                call_type: "function".to_string(),
                                function: MiniMaxFunctionCallInMessage {
                                    name: tc.name,
                                    arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                                },
                            })
                            .collect())
                    }
                });

                MiniMaxMessage {
                    role: m.role,
                    content: m.content,
                    tool_call_id: m.tool_call_id,
                    tool_calls: tool_calls_in_msg,
                }
            })
            .collect();

        let minimax_tools = tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| MiniMaxTool {
                    tool_type: "function".to_string(),
                    function: MiniMaxFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        let request = MiniMaxRequest {
            model: self.model.clone(),
            messages: minimax_messages,
            tools: minimax_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::AgentError(format!("Serialize error: {}", e)))?;

        if self.api_key.is_empty() {
            return Err(AgentError::AgentError(
                "MiniMax API key is empty. Please configure your API key.".to_string()
            ));
        }

        let url = format!("{}?GroupId={}", self.base_url, self.group_id);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AgentError::AgentError(format!("Request failed: {}", e)))?;

        let status = response.status();
        log::info!("[MiniMax Text] 响应状态码：{}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[MiniMax Text] API 错误：{}", error_text);
            return Err(AgentError::AgentError(
                format!("MiniMax API error ({}): {}", status, error_text)
            ));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::AgentError(format!("Read response failed: {}", e)))?;

        let minimax_response: MiniMaxResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::AgentError(format!("Parse error: {} | Response: {}", e, response_text)))?;

        let choice = minimax_response
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

        log::info!("[MiniMax Text] 响应收到，{} tool calls", tool_calls.len());

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
        Err(AgentError::AgentError("MiniMax streaming not yet implemented".to_string()))
    }
}
