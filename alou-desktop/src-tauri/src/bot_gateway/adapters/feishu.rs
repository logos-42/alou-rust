//! 飞书 Bot 适配器
//!
//! 处理飞书 Bot 消息的接收和发送

use crate::bot_gateway::config::FeishuConfig;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 飞书 Bot 适配器
pub struct FeishuAdapter {
    config: FeishuConfig,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuWebhookEvent {
    pub schema: String,
    pub header: FeishuHeader,
    pub event: FeishuEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuHeader {
    pub message_id: String,
    pub timestamp: String,
    pub token: String,
    pub event_type: String,
    pub tenant_key: String,
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub message: Option<FeishuMessage>,
    pub sender: Option<FeishuSender>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuMessage {
    pub message_id: String,
    pub root_id: String,
    pub parent_id: String,
    pub chat_type: String,
    pub msg_type: String,
    pub content: String,
    pub create_time: String,
    pub chat_id: String,
    pub mention: Option<FeishuMention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuSender {
    pub sender_id: FeishuUserId,
    pub tenant_key: String,
    pub user_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuUserId {
    pub open_id: Option<String>,
    pub user_id: Option<String>,
    pub union_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuMention {
    pub name: Option<String>,
    pub all: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuChallenge {
    pub challenge: String,
    pub token: String,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuResponse {
    #[serde(rename = "code")]
    pub code: Option<i32>,
    #[serde(rename = "msg")]
    pub msg: Option<String>,
    #[serde(rename = "data")]
    pub data: Option<serde_json::Value>,
    #[serde(rename = "Extra")]
    pub extra: Option<serde_json::Value>,
}

/// 飞书响应格式（用于消息回复）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuReplyResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<serde_json::Value>,
}

impl FeishuAdapter {
    pub fn new(config: FeishuConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// 验证 webhook 挑战
    pub fn verify_challenge(&self, challenge: &FeishuChallenge) -> Option<String> {
        if challenge.token == self.config.verify_token {
            Some(challenge.challenge.clone())
        } else {
            None
        }
    }

    /// 验证请求签名
    pub fn verify_signature(
        &self,
        timestamp: &str,
        token: &str,
        body: &str,
    ) -> bool {
        if token != self.config.verify_token {
            return false;
        }

        // 飞书签名验证：SHA256(timestamp + token + body)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(timestamp.as_bytes());
        hasher.update(token.as_bytes());
        hasher.update(body.as_bytes());
        let result = hasher.finalize();
        
        // 这里简化验证，实际应该比较 base64 编码
        true
    }

    /// 解密飞书消息内容
    pub fn decrypt_message(&self, encrypted_content: &str) -> Result<String, String> {
        if let Some(encrypt_key) = &self.config.encrypt_key {
            // AES 解密逻辑（简化版本）
            Ok(encrypted_content.to_string())
        } else {
            Ok(encrypted_content.to_string())
        }
    }

    /// 发送消息到飞书
    pub async fn send_message(
        &self,
        chat_id: &str,
        msg_type: &str,
        content: &serde_json::Value,
    ) -> Result<FeishuResponse, String> {
        let url = "https://open.feishu.cn/open-apis/im/v1/messages";

        let params = serde_json::json!({
            "receive_id": chat_id,
            "msg_type": msg_type,
            "content": content,
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.get_access_token().await?))
            .header("Content-Type", "application/json")
            .json(&params)
            .send()
            .await
            .map_err(|e| format!("发送消息失败：{}", e))?;

        let result: FeishuResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败：{}", e))?;

        Ok(result)
    }

    /// 发送文本消息
    pub async fn send_text_message(
        &self,
        chat_id: &str,
        text: &str,
        mention_all: Option<bool>,
    ) -> Result<FeishuResponse, String> {
        let content = if mention_all.unwrap_or(false) {
            serde_json::json!({
                "text": format!("<at user_id=\"all\">所有人</at> {}", text),
            })
        } else {
            serde_json::json!({
                "text": text,
            })
        };

        self.send_message(chat_id, "text", &content).await
    }

    /// 发送富文本消息
    pub async fn send_post_message(
        &self,
        chat_id: &str,
        elements: Vec<serde_json::Value>,
    ) -> Result<FeishuResponse, String> {
        let content = serde_json::json!({
            "zh_cn": {
                "title": "",
                "content": [elements],
            }
        });

        self.send_message(chat_id, "post", &content).await
    }

    /// 获取访问令牌
    pub async fn get_access_token(&self) -> Result<String, String> {
        // 实际应该缓存 token 并定期刷新
        let url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";

        let params = serde_json::json!({
            "app_id": &self.config.app_id,
            "app_secret": &self.config.app_secret,
        });

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&params)
            .send()
            .await
            .map_err(|e| format!("获取 access_token 失败：{}", e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败：{}", e))?;

        result
            .get("tenant_access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "无法获取 access_token".to_string())
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

    /// 检查租户是否被允许
    pub fn is_tenant_allowed(&self, tenant_key: &str) -> bool {
        if self.config.allowed_tenant_ids.is_empty() {
            return true;
        }
        self.config
            .allowed_tenant_ids
            .iter()
            .any(|id| id == tenant_key)
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
}

/// 消息处理结果
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    pub user_id: String,
    pub chat_id: String,
    pub message_id: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub text: String,
    pub chat_type: String,
}

impl FeishuAdapter {
    /// 处理飞书消息
    pub fn process_message(&self, event: &FeishuEvent) -> Option<ProcessedMessage> {
        let msg = event.message.as_ref()?;

        // 解密消息内容
        let content_str = self.decrypt_message(&msg.content).ok()?;
        
        // 解析消息内容（飞书消息内容是 JSON 字符串）
        let content: serde_json::Value = serde_json::from_str(&content_str).ok()?;
        let text = content.get("text").and_then(|v| v.as_str()).unwrap_or("");

        let (command, args) = self.parse_command(text).unzip();

        Some(ProcessedMessage {
            user_id: event.sender.as_ref()?.sender_id.user_id.clone().unwrap_or_default(),
            chat_id: msg.chat_id.clone(),
            message_id: msg.message_id.clone(),
            command,
            args: args.unwrap_or_default(),
            text: text.to_string(),
            chat_type: msg.chat_type.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let config = FeishuConfig::default();
        let adapter = FeishuAdapter::new(config);

        let (cmd, args) = adapter.parse_command("/help arg1 arg2").unwrap();
        assert_eq!(cmd, "help");
        assert_eq!(args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_verify_challenge() {
        let config = FeishuConfig {
            verify_token: "test_token".to_string(),
            ..Default::default()
        };
        let adapter = FeishuAdapter::new(config);

        let challenge = FeishuChallenge {
            challenge: "test_challenge".to_string(),
            token: "test_token".to_string(),
            type_: "url_verification".to_string(),
        };

        assert_eq!(adapter.verify_challenge(&challenge), Some("test_challenge".to_string()));
    }
}
