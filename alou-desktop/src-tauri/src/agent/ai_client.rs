//! AI 客户端
//!
//! 提供统一的 AI 接口，支持多种 Provider（DeepSeek/OpenAI/Claude/Kimi）

use super::providers;
use super::error::{AgentError, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// AI Provider trait
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// 发送消息
    async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse>;

    /// 发送消息（流式响应）
    async fn send_message_stream(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<BoxStream<'static, std::result::Result<AiStreamEvent, AgentError>>>;
}

/// 统一的消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<AiToolCall>>,
}

impl AiMessage {
    /// 创建普通文本消息
    pub fn text(role: &str, content: String) -> Self {
        Self {
            role: role.to_string(),
            content,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    /// 创建工具结果消息
    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content,
            tool_call_id: Some(tool_call_id),
            tool_calls: None,
        }
    }

    /// 创建包含工具调用的助手消息
    pub fn assistant_with_tools(content: String, tool_calls: Vec<AiToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        }
    }
}

/// 统一的工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// AI 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub tool_calls: Vec<AiToolCall>,
    pub finish_reason: String,
    pub usage: Option<TokenUsage>,
}

/// Token 使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 流式事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiStreamEvent {
    /// 内容增量
    Content { delta: String },

    /// 工具调用开始
    ToolCallStart { id: String, name: String },

    /// 工具调用增量
    ToolCallDelta { id: String, arguments: String },

    /// 工具调用结束
    ToolCallEnd { id: String },

    /// 完成
    Done,

    /// 错误
    Error { message: String },
}

/// AI 客户端
pub struct AiClient {
    provider: Box<dyn AiProvider>,
    pub config: super::config::UserApiConfig,
}

impl AiClient {
    /// 创建 AI 客户端
    pub fn new(config: &super::config::UserApiConfig) -> Result<Self> {
        let provider: Box<dyn AiProvider> = match config.provider.to_lowercase().as_str() {
            "deepseek" => Box::new(providers::DeepSeekProvider::new(
                config.api_key.clone(),
                config.model.clone().unwrap_or_else(|| "deepseek-chat".to_string()),
                config.base_url.clone(),
            )),
            "openai" | "opencode" => Box::new(providers::OpenAiProvider::new(
                config.api_key.clone(),
                config.model.clone().unwrap_or_else(|| "gpt-4".to_string()),
                config.base_url.clone(),
            )),
            "claude" => Box::new(providers::ClaudeProvider::new(
                config.api_key.clone(),
                config.model.clone().unwrap_or_else(|| "claude-3-opus-20240229".to_string()),
                config.base_url.clone(),
            )),
            "kimi" => Box::new(providers::KimiProvider::new(
                config.api_key.clone(),
                config.model.clone().unwrap_or_else(|| "kimi-k2-turbo-preview".to_string()),
                config.base_url.clone(),
            )),
            _ => {
                return Err(AgentError::InvalidInput(format!(
                    "未知的 provider: {}",
                    config.provider
                )))
            }
        };

        Ok(Self {
            provider,
            config: config.clone(),
        })
    }

    /// 发送消息
    pub async fn send_message(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<AiResponse> {
        self.provider.send_message(messages, tools).await
    }

    /// 发送消息（流式）
    pub async fn send_message_stream(
        &self,
        messages: Vec<AiMessage>,
        tools: Option<Vec<AiTool>>,
    ) -> Result<BoxStream<'static, std::result::Result<AiStreamEvent, AgentError>>> {
        self.provider.send_message_stream(messages, tools).await
    }
}
