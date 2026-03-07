//! Telegram Bot 适配器
//!
//! 处理 Telegram Bot 消息的接收和发送

use crate::bot_gateway::config::TelegramConfig;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Telegram Bot 适配器
pub struct TelegramAdapter {
    config: TelegramConfig,
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub update_id: u64,
    pub message: Option<TelegramChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChatMessage {
    pub message_id: u64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub text: Option<String>,
    pub date: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub id: u64,
    pub is_bot: bool,
    pub first_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramResponse {
    pub ok: bool,
    pub result: Option<serde_json::Value>,
    pub description: Option<String>,
}

impl TelegramAdapter {
    pub fn new(config: TelegramConfig) -> Self {
        let base_url = format!("https://api.telegram.org/bot{}", config.bot_token);
        Self {
            config,
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// 获取更新
    pub async fn get_updates(
        &self,
        offset: Option<u64>,
        timeout: Option<u64>,
    ) -> Result<Vec<TelegramMessage>, String> {
        let mut url = format!("{}/getUpdates", self.base_url);
        let mut params = Vec::new();

        if let Some(offset) = offset {
            params.push(format!("offset={}", offset));
        }
        if let Some(timeout) = timeout {
            params.push(format!("timeout={}", timeout));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("请求失败：{}", e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败：{}", e))?;

        if let Some(result_array) = result.get("result").and_then(|v| v.as_array()) {
            let messages: Vec<TelegramMessage> = result_array
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            Ok(messages)
        } else {
            Ok(vec![])
        }
    }

    /// 发送消息
    pub async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
        reply_to_message_id: Option<u64>,
    ) -> Result<TelegramResponse, String> {
        let url = format!("{}/sendMessage", self.base_url);

        let mut params = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
        });

        if let Some(reply_id) = reply_to_message_id {
            params["reply_to_message_id"] = serde_json::json!(reply_id);
        }

        let response = self
            .client
            .post(&url)
            .json(&params)
            .send()
            .await
            .map_err(|e| format!("发送消息失败：{}", e))?;

        let result: TelegramResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败：{}", e))?;

        Ok(result)
    }

    /// 检查用户是否被允许
    pub fn is_user_allowed(&self, user_id: u64) -> bool {
        if self.config.allowed_user_ids.is_empty() {
            return true; // 空列表表示允许所有用户
        }
        self.config
            .allowed_user_ids
            .iter()
            .any(|id| id.parse::<u64>().ok() == Some(user_id))
    }

    /// 检查聊天是否被允许
    pub fn is_chat_allowed(&self, chat_id: i64) -> bool {
        if self.config.allowed_chat_ids.is_empty() {
            return true;
        }
        self.config
            .allowed_chat_ids
            .iter()
            .any(|id| id.parse::<i64>().ok() == Some(chat_id))
    }

    /// 解析命令
    pub fn parse_command(&self, text: &str) -> Option<(String, Vec<String>)> {
        let text = text.trim();
        if !text.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command = parts[0].trim_start_matches('/').to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        Some((command, args))
    }

    /// 获取 Bot 信息
    pub async fn get_me(&self) -> Result<TelegramUser, String> {
        let url = format!("{}/getMe", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("请求失败：{}", e))?;

        let result: TelegramResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败：{}", e))?;

        if result.ok {
            let user: TelegramUser = serde_json::from_value(
                result.result.unwrap_or_default()
            ).map_err(|e| format!("解析 Bot 信息失败：{}", e))?;
            Ok(user)
        } else {
            Err(result.description.unwrap_or_else(|| "未知错误".to_string()))
        }
    }
}

/// 消息处理结果
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    pub user_id: u64,
    pub chat_id: i64,
    pub message_id: u64,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub text: String,
    pub is_private: bool,
}

impl TelegramAdapter {
    /// 处理消息
    pub fn process_message(&self, msg: &TelegramChatMessage) -> Option<ProcessedMessage> {
        let text = msg.text.as_deref().unwrap_or("").to_string();
        let (command, args) = self.parse_command(&text).unzip();

        Some(ProcessedMessage {
            user_id: msg.from.as_ref()?.id,
            chat_id: msg.chat.id,
            message_id: msg.message_id,
            command,
            args: args.unwrap_or_default(),
            text,
            is_private: msg.chat.chat_type == "private",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let config = TelegramConfig::default();
        let adapter = TelegramAdapter::new(config);

        let (cmd, args) = adapter.parse_command("/help arg1 arg2").unwrap();
        assert_eq!(cmd, "help");
        assert_eq!(args, vec!["arg1", "arg2"]);

        let (cmd, args) = adapter.parse_command("/spec create product").unwrap();
        assert_eq!(cmd, "spec");
        assert_eq!(args, vec!["create", "product"]);
    }

    #[test]
    fn test_parse_command_no_args() {
        let config = TelegramConfig::default();
        let adapter = TelegramAdapter::new(config);

        let (cmd, args) = adapter.parse_command("/status").unwrap();
        assert_eq!(cmd, "status");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_command_invalid() {
        let config = TelegramConfig::default();
        let adapter = TelegramAdapter::new(config);

        assert!(adapter.parse_command("not a command").is_none());
        assert!(adapter.parse_command("").is_none());
    }
}
