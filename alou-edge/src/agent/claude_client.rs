use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::{console_log, console_error};
use crate::utils::error::{AloudError, Result};

// Claude API Configuration
const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_API_VERSION: &str = "2023-06-01";
#[allow(dead_code)]
const CLAUDE_MODEL: &str = "claude-3-5-sonnet-20241022";
const MAX_RETRIES: u32 = 3;

trait StatusCodeExt {
    fn is_success(&self) -> bool;
}

impl StatusCodeExt for u16 {
    fn is_success(&self) -> bool {
        (200..300).contains(self)
    }
}

/// Claude message format (Messages API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

impl ClaudeMessage {
    /// Create a simple text message
    pub fn text(role: &str, text: String) -> Self {
        Self {
            role: role.to_string(),
            content: vec![ContentBlock::Text { text }],
        }
    }
    
    /// Create a message with tool use
    #[allow(dead_code)]
    pub fn with_tool_use(role: &str, id: String, name: String, input: Value) -> Self {
        Self {
            role: role.to_string(),
            content: vec![ContentBlock::ToolUse { id, name, input }],
        }
    }
    
    /// Create a message with tool result
    pub fn with_tool_result(role: &str, tool_use_id: String, content: String) -> Self {
        Self {
            role: role.to_string(),
            content: vec![ContentBlock::ToolResult { tool_use_id, content }],
        }
    }
}

/// Content block in Claude message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

/// Tool definition in Claude format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Tool use request from Claude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: Value,
}

/// Claude API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub content: String,
    pub tool_calls: Vec<ToolUse>,
    pub stop_reason: String,
}

/// Claude API request (Messages API format)
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeRequestMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ClaudeTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct ClaudeRequestMessage {
    role: String,
    content: Value,
}

/// Claude API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Some fields are for future use
struct ClaudeApiResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ResponseContent>,
    model: String,
    stop_reason: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Claude API client (using official Anthropic API)
pub struct ClaudeClient {
    api_key: String,
    model: String,
}

impl ClaudeClient {
    /// Create a new Claude client with API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: CLAUDE_MODEL.to_string(),
        }
    }
    
    /// Create a new Claude client with custom model
    #[allow(dead_code)]
    pub fn with_model(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
    
    /// Send a message and get a response
    pub async fn send_message(
        &self,
        messages: Vec<ClaudeMessage>,
        tools: Option<Vec<ClaudeTool>>,
    ) -> Result<ClaudeResponse> {
        let mut retries = 0;
        
        loop {
            match self.send_message_internal(messages.clone(), tools.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        return Err(e);
                    }
                    
                    console_log!("Retry {}/{} after error: {}", retries, MAX_RETRIES, e);
                    // Simple delay (in WASM, we'd use a proper async sleep)
                    // For now, just retry immediately
                    continue;
                }
            }
        }
    }
    
    async fn send_message_internal(
        &self,
        messages: Vec<ClaudeMessage>,
        tools: Option<Vec<ClaudeTool>>,
    ) -> Result<ClaudeResponse> {
        // Convert messages to Claude API format
        let api_messages: Vec<ClaudeRequestMessage> = messages
            .into_iter()
            .map(|m| {
                // Convert content blocks to JSON
                let content = if m.content.len() == 1 {
                    // Single content block - can be simplified to string for text
                    match &m.content[0] {
                        ContentBlock::Text { text } => json!(text),
                        _ => json!(m.content),
                    }
                } else {
                    json!(m.content)
                };
                
                ClaudeRequestMessage {
                    role: m.role,
                    content,
                }
            })
            .collect();
        
        let request = ClaudeRequest {
            model: self.model.clone(),
            messages: api_messages,
            tools,
            system: None,
            max_tokens: 4096,
            temperature: Some(0.7),
        };
        
        // Make HTTP request using fetch API (worker-compatible)
        let response = self.make_request(&request).await?;
        
        // Parse response
        self.parse_response(response)
    }
    
    async fn make_request(&self, request: &ClaudeRequest) -> Result<ClaudeApiResponse> {
        use worker::{Fetch, Headers, Method, RequestInit};
        
        let body = serde_json::to_string(request)
            .map_err(|e| AloudError::ClaudeApiError(format!("Failed to serialize request: {}", e)))?;
        
        console_log!("Claude API Request to: {}", CLAUDE_API_URL);
        
        // Create headers following Claude API requirements
        let headers = {
            let h = Headers::new();
            h.set("Content-Type", "application/json")
                .map_err(|e| AloudError::ClaudeApiError(format!("Failed to set header: {}", e)))?;
            h.set("x-api-key", &self.api_key)
                .map_err(|e| AloudError::ClaudeApiError(format!("Failed to set API key: {}", e)))?;
            h.set("anthropic-version", CLAUDE_API_VERSION)
                .map_err(|e| AloudError::ClaudeApiError(format!("Failed to set version: {}", e)))?;
            h
        };
        
        // Create request init
        let init = {
            let mut i = RequestInit::new();
            i.with_method(Method::Post)
                .with_headers(headers)
                .with_body(Some(body.into()));
            i
        };
        
        // Make fetch request
        let mut response = Fetch::Request(
            worker::Request::new_with_init(CLAUDE_API_URL, &init)
                .map_err(|e| AloudError::ClaudeApiError(format!("Failed to create request: {}", e)))?
        )
        .send()
        .await
        .map_err(|e| AloudError::ClaudeApiError(format!("Request failed: {}", e)))?;
        
        // Check status
        if !response.status_code().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            console_error!("Claude API Error ({}): {}", response.status_code(), error_text);
            return Err(AloudError::ClaudeApiError(format!(
                "API returned error {}: {}",
                response.status_code(),
                error_text
            )));
        }
        
        // Parse response
        let response_text = response
            .text()
            .await
            .map_err(|e| AloudError::ClaudeApiError(format!("Failed to read response: {}", e)))?;
        
        console_log!("Claude API Response received ({} bytes)", response_text.len());
        
        serde_json::from_str(&response_text)
            .map_err(|e| AloudError::ClaudeApiError(format!("Failed to parse response: {} - Response: {}", e, response_text)))
    }
    
    fn parse_response(&self, response: ClaudeApiResponse) -> Result<ClaudeResponse> {
        let mut text_content = String::new();
        let mut tool_calls = Vec::new();
        
        // Extract text and tool uses from content blocks
        for content in response.content {
            match content {
                ResponseContent::Text { text } => {
                    if !text_content.is_empty() {
                        text_content.push('\n');
                    }
                    text_content.push_str(&text);
                }
                ResponseContent::ToolUse { id, name, input } => {
                    tool_calls.push(ToolUse { id, name, input });
                }
            }
        }
        
        Ok(ClaudeResponse {
            content: text_content,
            tool_calls,
            stop_reason: response.stop_reason.unwrap_or_else(|| "end_turn".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_claude_message_creation() {
        let msg = ClaudeMessage::text("user", "Hello".to_string());
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content.len(), 1);
    }
    
    #[test]
    fn test_tool_definition() {
        let tool = ClaudeTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param": { "type": "string" }
                }
            }),
        };
        
        assert_eq!(tool.name, "test_tool");
    }
}
