use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::{console_log, Fetch, Headers, Method, RequestInit};
use crate::agent::ai_client::{AiProvider, AiMessage, AiTool, AiResponse, AiToolCall};
use crate::utils::error::{AloudError, Result};

const DEEPSEEK_API_URL: &str = "https://api.deepseek.com/v1/chat/completions";

pub struct DeepSeekProvider {
    api_key: String,
    model: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
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

#[async_trait(?Send)]
impl AiProvider for DeepSeekProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        console_log!("DeepSeek: Sending request to {}", DEEPSEEK_API_URL);
        console_log!("DeepSeek: Model: {}, Messages: {}", self.model, messages.len());
        
        let deepseek_messages: Vec<DeepSeekMessage> = messages
            .into_iter()
            .map(|m| {
                let tool_calls_in_msg = m.tool_calls.map(|tcs| {
                    tcs.into_iter()
                        .map(|tc| DeepSeekToolCallInMessage {
                            id: tc.id,
                            call_type: "function".to_string(),
                            function: DeepSeekFunctionCallInMessage {
                                name: tc.name,
                                arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                            },
                        })
                        .collect()
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
            tools
                .into_iter()
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
            .map_err(|e| AloudError::AgentError(format!("Serialize error: {}", e)))?;
        
        let headers = {
            let h = Headers::new();
            h.set("Content-Type", "application/json")
                .map_err(|e| AloudError::AgentError(e.to_string()))?;
            h.set("Authorization", &format!("Bearer {}", self.api_key))
                .map_err(|e| AloudError::AgentError(e.to_string()))?;
            h
        };
        
        let init = {
            let mut i = RequestInit::new();
            i.with_method(Method::Post)
                .with_headers(headers)
                .with_body(Some(body.into()));
            i
        };
        
        let mut response = Fetch::Request(
            worker::Request::new_with_init(DEEPSEEK_API_URL, &init)
                .map_err(|e| AloudError::AgentError(e.to_string()))?
        )
        .send()
        .await
        .map_err(|e| AloudError::AgentError(e.to_string()))?;
        
        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AloudError::AgentError(format!(
                "DeepSeek API error {}: {}",
                response.status_code(),
                error_text
            )));
        }
        
        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(e.to_string()))?;
        
        let preview = response_text.chars().take(500).collect::<String>();
        console_log!("DeepSeek: Raw response: {}", preview);
        
        let deepseek_response: DeepSeekResponse = serde_json::from_str(&response_text)
            .map_err(|e| AloudError::AgentError(format!("Parse error: {} | Response: {}", e, response_text)))?;
        
        console_log!("DeepSeek: Parsed response, {} choices", deepseek_response.choices.len());
        
        let choice = deepseek_response
            .choices
            .first()
            .ok_or_else(|| AloudError::AgentError("No choices in response".to_string()))?;
        
        console_log!("DeepSeek: Choice finish_reason: {}", choice.finish_reason);
        console_log!("DeepSeek: Message content is_some: {}", choice.message.content.is_some());
        
        let content = choice.message.content.clone().unwrap_or_default();
        let content_preview = content.chars().take(100).collect::<String>();
        console_log!("DeepSeek: Content length: {}, content: '{}'", content.len(), content_preview);
        
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
        
        console_log!("DeepSeek: Response received, {} tool calls", tool_calls.len());
        
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
