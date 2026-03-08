//! Discord Bot 适配器
//!
//! 处理 Discord Bot 消息的接收和发送

use crate::bot_gateway::config::DiscordConfig;
use serde::{Deserialize, Serialize};

/// Discord Bot 适配器
pub struct DiscordAdapter {
    config: DiscordConfig,
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMessage {
    pub id: String,
    pub channel_id: String,
    pub author: DiscordUser,
    pub content: String,
    pub timestamp: String,
    pub guild_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub is_bot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordWebhookPayload {
    pub content: String,
    pub embeds: Vec<DiscordEmbed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordEmbed {
    pub title: String,
    pub description: String,
    pub color: i32,
}

impl DiscordAdapter {
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            base_url: "https://discord.com/api/v10".to_string(),
        }
    }

    /// 发送消息到 Discord 频道
    pub async fn send_message(
        &self,
        channel_id: &str,
        content: &str,
    ) -> Result<(), String> {
        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);

        let payload = serde_json::json!({
            "content": content,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.config.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("发送消息失败：{}", e))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Discord API 错误：{}", error));
        }

        Ok(())
    }

    /// 发送嵌入消息
    pub async fn send_embed(
        &self,
        channel_id: &str,
        title: &str,
        description: &str,
        color: i32,
    ) -> Result<(), String> {
        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);

        let payload = serde_json::json!({
            "embeds": [{
                "title": title,
                "description": description,
                "color": color,
            }]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.config.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("发送嵌入消息失败：{}", e))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Discord API 错误：{}", error));
        }

        Ok(())
    }

    /// 检查用户是否被允许
    pub fn is_user_allowed(&self, user_id: &str) -> bool {
        if self.config.allowed_user_ids.is_empty() {
            return true;
        }
        self.config
            .allowed_user_ids
            .iter()
            .any(|id| id == user_id)
    }

    /// 检查服务器是否被允许
    pub fn is_guild_allowed(&self, guild_id: &str) -> bool {
        if self.config.allowed_guild_ids.is_empty() {
            return true;
        }
        self.config
            .allowed_guild_ids
            .iter()
            .any(|id| id == guild_id)
    }

    /// 检查频道是否被允许
    pub fn is_channel_allowed(&self, channel_id: &str) -> bool {
        if self.config.allowed_channel_ids.is_empty() {
            return true;
        }
        self.config
            .allowed_channel_ids
            .iter()
            .any(|id| id == channel_id)
    }

    /// 解析命令
    pub fn parse_command(&self, content: &str) -> Option<(String, Vec<String>)> {
        let content = content.trim();
        if !content.starts_with('!') && !content.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command = parts[0]
            .trim_start_matches('!')
            .trim_start_matches('/')
            .to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        Some((command, args))
    }

    /// 获取 Bot 信息
    pub async fn get_me(&self) -> Result<DiscordUser, String> {
        let url = format!("{}/users/@me", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.config.bot_token))
            .send()
            .await
            .map_err(|e| format!("请求失败：{}", e))?;

        let user: DiscordUser = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败：{}", e))?;

        Ok(user)
    }
}

/// 消息处理结果
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    pub user_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub content: String,
    pub guild_id: Option<String>,
}

impl DiscordAdapter {
    /// 处理 Discord 消息
    pub fn process_message(&self, msg: &DiscordMessage) -> Option<ProcessedMessage> {
        // 忽略 Bot 自己的消息
        if msg.author.is_bot {
            return None;
        }

        // 验证用户权限
        if !self.is_user_allowed(&msg.author.id) {
            return None;
        }

        // 验证服务器权限
        if let Some(guild_id) = &msg.guild_id {
            if !self.is_guild_allowed(guild_id) {
                return None;
            }
        }

        // 验证频道权限
        if !self.is_channel_allowed(&msg.channel_id) {
            return None;
        }

        let (command, args) = self.parse_command(&msg.content).unzip();

        Some(ProcessedMessage {
            user_id: msg.author.id.clone(),
            channel_id: msg.channel_id.clone(),
            message_id: msg.id.clone(),
            command,
            args: args.unwrap_or_default(),
            content: msg.content.clone(),
            guild_id: msg.guild_id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let config = DiscordConfig::default();
        let adapter = DiscordAdapter::new(config);

        let (cmd, args) = adapter.parse_command("!help arg1 arg2").unwrap();
        assert_eq!(cmd, "help");
        assert_eq!(args, vec!["arg1", "arg2"]);

        let (cmd, args) = adapter.parse_command("/spec create product").unwrap();
        assert_eq!(cmd, "spec");
        assert_eq!(args, vec!["create", "product"]);
    }

    #[test]
    fn test_parse_command_invalid() {
        let config = DiscordConfig::default();
        let adapter = DiscordAdapter::new(config);

        assert!(adapter.parse_command("not a command").is_none());
        assert!(adapter.parse_command("").is_none());
    }
}
