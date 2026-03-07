//! QQ Bot 适配器 (OneBot 协议)
//!
//! 处理 QQ Bot 消息的接收和发送，支持 OneBot 协议

use crate::bot_gateway::config::QQConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// QQ Bot 适配器
pub struct QQAdapter {
    config: QQConfig,
    ws_url: String,
    access_token: Option<String>,
}

/// OneBot 消息事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBotMessageEvent {
    pub post_type: String,
    pub message_type: String,
    pub sub_type: Option<String>,
    pub message_id: i64,
    pub user_id: i64,
    pub group_id: Option<i64>,
    pub message: Vec<OneBotMessageSegment>,
    pub raw_message: String,
    pub time: i64,
}

/// OneBot 消息段
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OneBotMessageSegment {
    #[serde(rename = "text")]
    Text { data: TextData },
    #[serde(rename = "at")]
    At { data: AtData },
    #[serde(rename = "reply")]
    Reply { data: ReplyData },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextData {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtData {
    pub qq: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyData {
    pub id: String,
}

/// OneBot 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBotResponse {
    pub status: String,
    pub retcode: i32,
    pub data: Option<serde_json::Value>,
    pub message: Option<String>,
}

/// OneBot 动作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBotAction {
    pub action: String,
    pub params: serde_json::Value,
    pub echo: Option<String>,
}

impl QQAdapter {
    pub fn new(config: QQConfig) -> Self {
        Self {
            ws_url: config.ws_url.clone(),
            access_token: config.access_token.clone(),
            config,
        }
    }

    /// 发送私聊消息
    pub async fn send_private_message(
        &self,
        user_id: i64,
        message: &str,
    ) -> Result<(), String> {
        self.call_action("send_msg", serde_json::json!({
            "user_id": user_id,
            "message": message,
            "message_type": "private",
        }))
        .await?;
        Ok(())
    }

    /// 发送群消息
    pub async fn send_group_message(
        &self,
        group_id: i64,
        message: &str,
    ) -> Result<(), String> {
        self.call_action("send_msg", serde_json::json!({
            "group_id": group_id,
            "message": message,
            "message_type": "group",
        }))
        .await?;
        Ok(())
    }

    /// 调用 OneBot 动作
    async fn call_action(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> Result<OneBotResponse, String> {
        // 这里简化实现，实际应该通过 WebSocket 连接
        // 使用 HTTP API 调用
        let url = format!("{}/{}", self.ws_url.replace("ws://", "http://").replace("wss://", "https://"), action);

        let mut request = reqwest::Client::new()
            .post(&url)
            .header("Content-Type", "application/json");

        if let Some(token) = &self.access_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let action_request = OneBotAction {
            action: action.to_string(),
            params,
            echo: None,
        };

        let response = request
            .json(&action_request)
            .send()
            .await
            .map_err(|e| format!("调用动作失败：{}", e))?;

        let result: OneBotResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败：{}", e))?;

        if result.retcode != 0 {
            return Err(format!("OneBot 返回错误：{}", result.message.unwrap_or_default()));
        }

        Ok(result)
    }

    /// 检查用户是否被允许
    pub fn is_user_allowed(&self, user_id: i64) -> bool {
        if self.config.allowed_user_ids.is_empty() {
            return true;
        }
        self.config
            .allowed_user_ids
            .iter()
            .any(|id| id.parse::<i64>().ok() == Some(user_id))
    }

    /// 检查群组是否被允许
    pub fn is_group_allowed(&self, group_id: i64) -> bool {
        if self.config.allowed_group_ids.is_empty() {
            return true;
        }
        self.config
            .allowed_group_ids
            .iter()
            .any(|id| id.parse::<i64>().ok() == Some(group_id))
    }

    /// 解析命令
    pub fn parse_command(&self, content: &str) -> Option<(String, Vec<String>)> {
        let content = content.trim();
        
        // 支持多种命令前缀
        let prefixes = ['/', '!', '.', '#'];
        if !prefixes.iter().any(|p| content.starts_with(*p)) {
            return None;
        }

        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command = parts[0]
            .trim_start_matches(|c| prefixes.contains(&c))
            .to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        Some((command, args))
    }

    /// 提取消息文本
    pub fn extract_text(&self, message: &[OneBotMessageSegment]) -> String {
        message
            .iter()
            .filter_map(|seg| {
                if let OneBotMessageSegment::Text { data } = seg {
                    Some(data.text.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// 消息处理结果
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    pub user_id: i64,
    pub group_id: Option<i64>,
    pub message_id: i64,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub content: String,
    pub is_private: bool,
}

impl QQAdapter {
    /// 处理 QQ 消息
    pub fn process_message(&self, event: &OneBotMessageEvent) -> Option<ProcessedMessage> {
        // 验证用户权限
        if !self.is_user_allowed(event.user_id) {
            return None;
        }

        // 验证群组权限（如果是群消息）
        if let Some(group_id) = event.group_id {
            if !self.is_group_allowed(group_id) {
                return None;
            }
        }

        // 提取消息文本
        let content = self.extract_text(&event.message);

        let (command, args) = self.parse_command(&content).unzip();

        Some(ProcessedMessage {
            user_id: event.user_id,
            group_id: event.group_id,
            message_id: event.message_id,
            command,
            args: args.unwrap_or_default(),
            content,
            is_private: event.group_id.is_none(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let config = QQConfig::default();
        let adapter = QQAdapter::new(config);

        let (cmd, args) = adapter.parse_command("!help arg1 arg2").unwrap();
        assert_eq!(cmd, "help");
        assert_eq!(args, vec!["arg1", "arg2"]);

        let (cmd, args) = adapter.parse_command("/spec create product").unwrap();
        assert_eq!(cmd, "spec");
        assert_eq!(args, vec!["create", "product"]);

        let (cmd, args) = adapter.parse_command(".status").unwrap();
        assert_eq!(cmd, "status");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_command_invalid() {
        let config = QQConfig::default();
        let adapter = QQAdapter::new(config);

        assert!(adapter.parse_command("not a command").is_none());
        assert!(adapter.parse_command("").is_none());
    }

    #[test]
    fn test_extract_text() {
        let config = QQConfig::default();
        let adapter = QQAdapter::new(config);

        let message = vec![
            OneBotMessageSegment::Text { data: TextData { text: "Hello ".to_string() } },
            OneBotMessageSegment::At { data: AtData { qq: "123".to_string() } },
            OneBotMessageSegment::Text { data: TextData { text: " world".to_string() } },
        ];

        let text = adapter.extract_text(&message);
        assert_eq!(text, "Hello  world");
    }
}
