use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::{console_log, Fetch, Headers, Method, RequestInit};
use crate::agent::ai_client::{AiProvider, AiMessage, AiTool, AiResponse, AiToolCall};
use crate::utils::error::{AloudError, Result};

const QWEN_API_URL: &str = "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation";

pub struct QwenProvider {
    api_key: String,
    model: String,
}

impl QwenProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[derive(Serialize)]
struct QwenRequest {
    model: String,
    input: QwenInput,
    parameters: QwenParameters,
}

#[derive(Serialize)]
struct QwenInput {
    messages: Vec<QwenMessage>,
}

#[derive(Serialize)]
struct QwenParameters {
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<QwenTool>>,
}

#[derive(Serialize)]
struct QwenMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct QwenTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: QwenFunction,
}

#[derive(Serialize)]
struct QwenFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Deserialize)]
struct QwenResponse {
    output: QwenOutput,
}

#[derive(Deserialize)]
struct QwenOutput {
    choices: Vec<QwenChoice>,
}

#[derive(Deserialize)]
struct QwenChoice {
    message: QwenResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct QwenResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<QwenToolCall>,
}

#[derive(Deserialize)]
struct QwenToolCall {
    id: String,
    function: QwenFunctionCall,
}

#[derive(Deserialize)]
struct QwenFunctionCall {
    name: String,
    arguments: String,
}

#[async_trait(?Send)]
impl AiProvider for QwenProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        console_log!("Qwen: Sending request to {}", QWEN_API_URL);
        
        let qwen_messages: Vec<QwenMessage> = messages
            .into_iter()
            .map(|m| QwenMessage {
                role: m.role,
                content: m.content,
            })
            .collect();
        
        let qwen_tools = tools.map(|tools| {
            tools
                .iter()
                .map(|t| QwenTool {
                    tool_type: "function".to_string(),
                    function: QwenFunction {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.parameters.clone(),
                    },
                })
                .collect()
        });
        
        let request = QwenRequest {
            model: self.model.clone(),
            input: QwenInput {
                messages: qwen_messages,
            },
            parameters: QwenParameters {
                temperature: 0.7,
                max_tokens: 4096,
                tools: qwen_tools,
            },
        };
        
        let body = serde_json::to_string(&request)
            .map_err(|e| AloudError::AgentError(format!("Serialize error: {}", e)))?;
        
        let headers = Headers::new();
        headers.set("Content-Type", "application/json")
            .map_err(|e| AloudError::AgentError(e.to_string()))?;
        headers.set("Authorization", &format!("Bearer {}", self.api_key))
            .map_err(|e| AloudError::AgentError(e.to_string()))?;
        
        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(body.into()));
        
        let mut response = Fetch::Request(
            worker::Request::new_with_init(QWEN_API_URL, &init)
                .map_err(|e| AloudError::AgentError(e.to_string()))?
        )
        .send()
        .await
        .map_err(|e| AloudError::AgentError(e.to_string()))?;
        
        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AloudError::AgentError(format!(
                "Qwen API error {}: {}",
                response.status_code(),
                error_text
            )));
        }
        
        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(e.to_string()))?;
        
        let qwen_response: QwenResponse = serde_json::from_str(&response_text)
            .map_err(|e| AloudError::AgentError(format!("Parse error: {}", e)))?;
        
        let choice = qwen_response
            .output
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
        
        console_log!("Qwen: Response received, {} tool calls", tool_calls.len());
        
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
