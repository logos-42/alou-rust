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
