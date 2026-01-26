use crate::agent::ai_client::{AiMessage, AiProvider, AiResponse, AiTool, AiToolCall};
use crate::utils::error::{AloudError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::{console_log, Fetch, Headers, Method, RequestInit};

const GLM_API_URL: &str = "https://open.bigmodel.cn/api/paas/v4/chat/completions";

pub struct GlmProvider {
    api_key: String,
    model: String,
}

impl GlmProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[derive(Serialize)]
struct GlmRequest {
    model: String,
    messages: Vec<GlmMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GlmTool>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct GlmMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<GlmToolCallInMessage>>,
}

#[derive(Serialize)]
struct GlmToolCallInMessage {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: GlmFunctionCallInMessage,
}

#[derive(Serialize)]
struct GlmFunctionCallInMessage {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct GlmTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: GlmFunction,
}

#[derive(Serialize)]
struct GlmFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct GlmResponse {
    choices: Vec<GlmChoice>,
}

#[derive(Deserialize)]
struct GlmChoice {
    message: GlmResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct GlmResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<GlmToolCall>,
}

#[derive(Deserialize)]
struct GlmToolCall {
    id: String,
    function: GlmFunctionCall,
}

#[derive(Deserialize)]
struct GlmFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait(?Send)]
impl AiProvider for GlmProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        console_log!("GLM: Sending request to {}", GLM_API_URL);
        console_log!(
            "GLM: Model: {}, Messages: {}",
            self.model,
            messages.len()
        );

        let glm_messages: Vec<GlmMessage> = messages
            .into_iter()
            .map(|m| {
                let tool_calls_in_msg = m.tool_calls.map(|tcs| {
                    tcs.into_iter()
                        .map(|tc| GlmToolCallInMessage {
                            id: tc.id,
                            call_type: "function".to_string(),
                            function: GlmFunctionCallInMessage {
                                name: tc.name,
                                arguments: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                            },
                        })
                        .collect()
                });

                GlmMessage {
                    role: m.role,
                    content: m.content,
                    tool_call_id: m.tool_call_id,
                    tool_calls: tool_calls_in_msg,
                }
            })
            .collect();

        let glm_tools = tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| GlmTool {
                    tool_type: "function".to_string(),
                    function: GlmFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.parameters,
                    },
                })
                .collect()
        });

        let request = GlmRequest {
            model: self.model.clone(),
            messages: glm_messages,
            tools: glm_tools,
            temperature: 0.7,
            max_tokens: 4096,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| AloudError::AgentError(format!("Serialize error: {}", e)))?;

        let request_preview = body.chars().take(200).collect::<String>();
        console_log!("GLM: Request preview: {}", request_preview);

        if self.api_key.is_empty() {
            return Err(AloudError::AgentError(
                "GLM API key is empty. Please configure AI_API_KEY secret.".to_string()
            ));
        }
        
        let api_key_preview = if self.api_key.len() > 8 {
            format!("{}...", &self.api_key[..8])
        } else {
            "***".to_string()
        };
        console_log!("GLM: Using API key: {}", api_key_preview);
        
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
            worker::Request::new_with_init(GLM_API_URL, &init)
                .map_err(|e| AloudError::AgentError(e.to_string()))?,
        )
        .send()
        .await
        .map_err(|e| AloudError::AgentError(e.to_string()))?;

        let status_code = response.status_code();
        console_log!("GLM: Response status: {}", status_code);
        
        if !status_code.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            console_log!("GLM API error {}: {}", status_code, error_text);
            
            return Err(AloudError::AgentError(format!(
                "GLM API error {}: {}",
                status_code, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(e.to_string()))?;

        let preview = response_text.chars().take(500).collect::<String>();
        console_log!("GLM: Raw response: {}", preview);

        let glm_response: GlmResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                AloudError::AgentError(format!("Parse error: {} | Response: {}", e, response_text))
            })?;

        console_log!(
            "GLM: Parsed response, {} choices",
            glm_response.choices.len()
        );

        let choice = glm_response
            .choices
            .first()
            .ok_or_else(|| AloudError::AgentError("No choices in response".to_string()))?;

        console_log!("GLM: Choice finish_reason: {}", choice.finish_reason);
        console_log!(
            "GLM: Message content is_some: {}",
            choice.message.content.is_some()
        );

        let content = choice.message.content.clone().unwrap_or_default();
        let content_preview = content.chars().take(100).collect::<String>();
        console_log!(
            "GLM: Content length: {}, content: '{}'",
            content.len(),
            content_preview
        );

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

        console_log!(
            "GLM: Response received, {} tool calls",
            tool_calls.len()
        );

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
