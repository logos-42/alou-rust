//! Google Gemini Provider 实现
//! 
//! Google Gemini 是 Google 的最新多模态大模型
//! 官网：https://ai.google.dev/
//! 文档：https://ai.google.dev/docs

use super::super::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall, AiStreamEvent};
use super::super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct GeminiProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTool>>,
    generation_config: Option<GeminiConfig>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiTool {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Serialize)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Serialize)]
struct GeminiConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    finish_reason: String,
}

#[async_trait]
impl AiProvider for GeminiProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        log::info!("[Gemini] 发送请求到 {}", self.base_url);
        log::info!("[Gemini] Model: {}, Messages: {}", self.model, messages.len());

        let client = Client::new();

        // Gemini 使用不同的消息格式
        let contents: Vec<GeminiContent> = messages
            .into_iter()
            .map(|m| GeminiContent {
                role: if m.role == "assistant" { "model" } else { "user" }.to_string(),
                parts: vec![GeminiPart { text: m.content }],
            })
            .collect();

        let gemini_tools = tools.map(|tools| GeminiTool {
            function_declarations: tools
                .into_iter()
                .map(|t| GeminiFunctionDeclaration {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                })
                .collect(),
        });

        let request = GeminiRequest {
            contents,
            tools: gemini_tools,
            generation_config: Some(GeminiConfig {
                temperature: 0.7,
                max_output_tokens: 4096,
            }),
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AgentError::AgentError(format!("Serialize error: {}", e)))?;

        if self.api_key.is_empty() {
            return Err(AgentError::AgentError(
                "Gemini API key is empty. Please configure your API key.".to_string()
            ));
        }

        let url = format!("{}/{}:generateContent?key={}", self.base_url, self.model, self.api_key);

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AgentError::AgentError(format!("Request failed: {}", e)))?;

        let status = response.status();
        log::info!("[Gemini] 响应状态码：{}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("[Gemini] API 错误：{}", error_text);
            return Err(AgentError::AgentError(
                format!("Gemini API error ({}): {}", status, error_text)
            ));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::AgentError(format!("Read response failed: {}", e)))?;

        let gemini_response: GeminiResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::AgentError(format!("Parse error: {} | Response: {}", e, response_text)))?;

        let candidate = gemini_response
            .candidates
            .first()
            .ok_or_else(|| AgentError::AgentError("No candidates in response".to_string()))?;

        let content = candidate.content.parts.first()
            .map(|p| p.text.clone())
            .unwrap_or_default();

        // Gemini 工具调用解析待实现
        let tool_calls = vec![];

        log::info!("[Gemini] 响应收到，{} tool calls", tool_calls.len());

        Ok(AiResponse {
            content,
            tool_calls,
            finish_reason: candidate.finish_reason.clone(),
            usage: None,
        })
    }

    async fn send_message_stream(
        &self,
        _messages: Vec<AiMessage>,
        _tools: Option<Vec<AiTool>>,
    ) -> Result<BoxStream<'static, std::result::Result<AiStreamEvent, AgentError>>> {
        Err(AgentError::AgentError("Gemini streaming not yet implemented".to_string()))
    }
}
