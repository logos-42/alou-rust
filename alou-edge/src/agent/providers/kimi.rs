use crate::agent::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall};
use crate::utils::error::{AloudError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::{console_log, Fetch, Headers, Method, RequestInit};

const KIMI_API_URL: &str = "https://api.moonshot.cn/v1/chat/completions";

pub struct KimiProvider {
    api_key: String,
    model: String,
}

impl KimiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
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

#[async_trait(?Send)]
impl AiProvider for KimiProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        console_log!("Kimi: Sending request to {}", KIMI_API_URL);

        let kimi_messages: Vec<KimiMessage> = messages
            .into_iter()
            .map(|m| KimiMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let kimi_tools = tools.map(|tools| {
            tools
                .into_iter()
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
            .map_err(|e| AloudError::AgentError(format!("Serialize error: {}", e)))?;

        let headers = Headers::new();
        headers
            .set("Content-Type", "application/json")
            .map_err(|e| AloudError::AgentError(e.to_string()))?;
        headers
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .map_err(|e| AloudError::AgentError(e.to_string()))?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(body.into()));

        let mut response = Fetch::Request(
            worker::Request::new_with_init(KIMI_API_URL, &init)
                .map_err(|e| AloudError::AgentError(e.to_string()))?,
        )
        .send()
        .await
        .map_err(|e| AloudError::AgentError(e.to_string()))?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AloudError::AgentError(format!(
                "Kimi API error {}: {}",
                response.status_code(),
                error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(e.to_string()))?;

        let kimi_response: KimiResponse = serde_json::from_str(&response_text)
            .map_err(|e| AloudError::AgentError(format!("Parse error: {}", e)))?;

        let choice = kimi_response
            .choices
            .first()
            .ok_or_else(|| AloudError::AgentError("No choices in response".to_string()))?;

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

        console_log!("Kimi: Response received, {} tool calls", tool_calls.len());

        Ok(AiResponse {
            content,
            tool_calls,
            finish_reason: choice.finish_reason.clone(),
        })
    }
}

trait StatusCodeExt {
    fn is_success(&self) -> bool;
}

impl StatusCodeExt for u16 {
    fn is_success(&self) -> bool {
        (200..300).contains(self)
    }
}

