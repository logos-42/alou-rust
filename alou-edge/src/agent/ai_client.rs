use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::utils::error::Result;
use super::providers;

/// AI Provider trait for different model providers
#[async_trait(?Send)]
pub trait AiProvider {
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse>;
}

/// Unified message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
    /// Tool call ID (for tool result messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Tool calls (for assistant messages with tool calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<AiToolCall>>,
}

impl AiMessage {
    /// Create a simple text message
    pub fn text(role: &str, content: String) -> Self {
        Self {
            role: role.to_string(),
            content,
            tool_call_id: None,
            tool_calls: None,
        }
    }
    
    /// Create a tool result message
    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content,
            tool_call_id: Some(tool_call_id),
            tool_calls: None,
        }
    }
    
    /// Create an assistant message with tool calls
    pub fn assistant_with_tools(content: String, tool_calls: Vec<AiToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        }
    }
}

/// Unified tool format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Unified response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub tool_calls: Vec<AiToolCall>,
    pub finish_reason: String,
}

/// Tool call from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// AI Client that supports multiple providers
pub struct AiClient {
    provider: Box<dyn AiProvider>,
}

impl AiClient {
    /// Create client with specified provider
    pub fn new(provider_type: &str, api_key: String, model: Option<String>) -> Result<Self> {
        let provider: Box<dyn AiProvider> = match provider_type.to_lowercase().as_str() {
            "deepseek" => Box::new(providers::DeepSeekProvider::new(
                api_key,
                model.unwrap_or_else(|| "deepseek-chat".to_string()),
            )),
            "qwen" => Box::new(providers::QwenProvider::new(
                api_key,
                model.unwrap_or_else(|| "qwen-max".to_string()),
            )),
            "openai" => Box::new(providers::OpenAiProvider::new(
                api_key,
                model.unwrap_or_else(|| "gpt-4".to_string()),
            )),
            "claude" => Box::new(providers::ClaudeProvider::new(
                api_key,
                model.unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
            )),
            _ => return Err(crate::utils::error::AloudError::InvalidInput(
                format!("Unknown provider: {}", provider_type)
            )),
        };
        
        Ok(Self { provider })
    }
    
    /// Send message to AI
    pub async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        self.provider.send_message(messages, tools).await
    }
}
