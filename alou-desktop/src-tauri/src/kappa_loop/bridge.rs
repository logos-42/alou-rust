//! Alou AiClient → Hyperagent LLMClient 桥接
//!
//! 将 Alou 的 `AiClient`/`UserApiConfig` 适配为 Hyperagent 的 `LLMClient` trait，
//! 使 Hyperagent 的 AutoResearch 引擎可以复用 Alou 已配置的 API Key 和 Provider。

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use hyperagent::llm::{LLMClient, LLMConfig, LLMProvider, LLMResponse, TokenUsage};

/// Alou → Hyperagent LLM 桥接
///
/// 通过 Alou 的 `AiClient` 发送消息，适配为 Hyperagent 的 `LLMClient` trait。
/// 使用 `Arc<Mutex<AiClient>>` 包装以满足 Clone 要求（AiClient 本身不可 Clone）。
#[derive(Clone)]
pub struct AlouLlmBridge {
    client: Arc<tokio::sync::Mutex<crate::agent::ai_client::AiClient>>,
    provider_name: String,
    model_name: String,
}

impl AlouLlmBridge {
    /// 从 Alou 的 UserApiConfig 创建桥接
    pub fn new(config: &crate::agent::config::UserApiConfig) -> Result<Self> {
        let client = crate::agent::ai_client::AiClient::new(config)?;
        let model_name = config.model.clone().unwrap_or_else(|| "unknown".to_string());
        let provider_name = config.provider.clone();

        Ok(Self {
            client: Arc::new(tokio::sync::Mutex::new(client)),
            provider_name,
            model_name,
        })
    }

    /// 转换为 Hyperagent 的 LLMConfig（用于兼容性）
    pub fn to_llm_config(config: &crate::agent::config::UserApiConfig) -> LLMConfig {
        let provider = match config.provider.to_lowercase().as_str() {
            "openai" | "opencode" => LLMProvider::OpenAI,
            "glm" | "zhipuai" | "智谱" => LLMProvider::GLM,
            "minimax" | "minimax-text" => LLMProvider::MiniMax,
            "qwen" => LLMProvider::Qwen,
            _ => LLMProvider::OpenAI, // 默认走 OpenAI 兼容
        };

        LLMConfig {
            provider,
            model: config.model.clone().unwrap_or_else(|| "gpt-4o".to_string()),
            api_key: config.api_key.clone(),
            base_url: config.base_url.clone(),
            max_concurrent: 4,
            temperature: Some(0.7),
            max_tokens: Some(4000),
        }
    }
}

#[async_trait]
impl LLMClient for AlouLlmBridge {
    async fn complete(&self, prompt: &str) -> Result<LLMResponse> {
        let messages = vec![
            crate::agent::ai_client::AiMessage::text("user", prompt.to_string()),
        ];

        let client = self.client.lock().await;
        let response = client.send_message(messages, None).await
            .map_err(|e| anyhow::anyhow!("Alou LLM Bridge error: {}", e))?;

        Ok(LLMResponse {
            content: response.content,
            model: self.model_name.clone(),
            provider: self.provider_name.clone(),
            usage: response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens as i32,
                completion_tokens: u.completion_tokens as i32,
                total_tokens: u.total_tokens as i32,
            }),
        })
    }

    async fn complete_with_system(&self, system_prompt: &str, user_prompt: &str) -> Result<LLMResponse> {
        let messages = vec![
            crate::agent::ai_client::AiMessage::text("system", system_prompt.to_string()),
            crate::agent::ai_client::AiMessage::text("user", user_prompt.to_string()),
        ];

        let client = self.client.lock().await;
        let response = client.send_message(messages, None).await
            .map_err(|e| anyhow::anyhow!("Alou LLM Bridge error: {}", e))?;

        Ok(LLMResponse {
            content: response.content,
            model: self.model_name.clone(),
            provider: self.provider_name.clone(),
            usage: response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens as i32,
                completion_tokens: u.completion_tokens as i32,
                total_tokens: u.total_tokens as i32,
            }),
        })
    }

    async fn complete_with_messages(
        &self,
        messages: Vec<hyperagent::llm::Message>,
    ) -> Result<LLMResponse> {
        let ai_messages: Vec<crate::agent::ai_client::AiMessage> = messages
            .into_iter()
            .map(|m| {
                let role = match m.role {
                    hyperagent::llm::MessageRole::System => "system",
                    hyperagent::llm::MessageRole::User => "user",
                    hyperagent::llm::MessageRole::Assistant => "assistant",
                };
                crate::agent::ai_client::AiMessage::text(role, m.content)
            })
            .collect();

        let client = self.client.lock().await;
        let response = client.send_message(ai_messages, None).await
            .map_err(|e| anyhow::anyhow!("Alou LLM Bridge error: {}", e))?;

        Ok(LLMResponse {
            content: response.content,
            model: self.model_name.clone(),
            provider: self.provider_name.clone(),
            usage: response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens as i32,
                completion_tokens: u.completion_tokens as i32,
                total_tokens: u.total_tokens as i32,
            }),
        })
    }
}
