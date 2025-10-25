use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::{console_log, Fetch, Headers, Method, RequestInit};
use crate::agent::ai_client::{AiProvider, AiMessage, AiTool, AiResponse, AiToolCall};
use crate::utils::error::{AloudError, Result};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct ClaudeProvider {
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
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
    content: String,
}

#[derive(Serialize)]
struct ClaudeTool {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    stop_reason: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClaudeContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

#[async_trait(?Send)]
impl AiProvider for ClaudeProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        console_log!("Claude: Sending request to {}", CLAUDE_API_URL);
        
        let claude_messages: Vec<ClaudeMessage> = messages
            .into_iter()
            .map(|m| ClaudeMessage {
                role: m.role,
                content: m.content,
            })
            .collect();
        
        let claude_tools = tools.map(|tools| {
            tools
                .into_iter()
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
            .map_err(|e| AloudError::AgentError(format!("Serialize error: {}", e)))?;
        
        let headers = {
            let h = Headers::new();
            h.set("Content-Type", "application/json")
                .map_err(|e| AloudError::AgentError(e.to_string()))?;
            h.set("x-api-key", &self.api_key)
                .map_err(|e| AloudError::AgentError(e.to_string()))?;
            h.set("anthropic-version", "2023-06-01")
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
            worker::Request::new_with_init(CLAUDE_API_URL, &init)
                .map_err(|e| AloudError::AgentError(e.to_string()))?
        )
        .send()
        .await
        .map_err(|e| AloudError::AgentError(e.to_string()))?;
        
        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AloudError::AgentError(format!(
                "Claude API error {}: {}",
                response.status_code(),
                error_text
            )));
        }
        
        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::AgentError(e.to_string()))?;
        
        let claude_response: ClaudeResponse = serde_json::from_str(&response_text)
            .map_err(|e| AloudError::AgentError(format!("Parse error: {}", e)))?;
        
        let mut content = String::new();
        let mut tool_calls = Vec::new();
        
        for item in claude_response.content {
            match item {
                ClaudeContent::Text { text } => {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&text);
                }
                ClaudeContent::ToolUse { id, name, input } => {
                    tool_calls.push(AiToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
            }
        }
        
        console_log!("Claude: Response received, {} tool calls", tool_calls.len());
        
        Ok(AiResponse {
            content,
            tool_calls,
            finish_reason: claude_response.stop_reason,
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
