use crate::storage::kv::KvStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::*;

use super::{json_response, json_response_with_status, ErrorResponse};

const PUBSUB_MESSAGE_TTL: u64 = 3600; // 1 hour
const MAX_MESSAGES_PER_TOPIC: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub content: String,
    pub topic: String,
    pub timestamp: i64,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Deserialize)]
pub struct PublishRequest {
    pub topic: String,
    pub message: PubSubMessage,
}

#[derive(Serialize)]
pub struct PublishResponse {
    pub success: bool,
    pub message_id: String,
}

#[derive(Serialize)]
pub struct MessagesResponse {
    pub topic: String,
    pub messages: Vec<PubSubMessage>,
    pub count: usize,
}

/// PubSub 管理器
pub struct PubSubManager {
    kv: KvStore,
}

impl PubSubManager {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }

    /// 发布消息到主题
    pub async fn publish(&self, topic: &str, message: PubSubMessage) -> Result<String> {
        let key = Self::topic_key(topic);
        
        // 获取现有消息
        let mut messages: Vec<PubSubMessage> = self.kv
            .get(&key)
            .await
            .map_err(|e| worker::Error::RustError(e.to_string()))?
            .unwrap_or_default();
        
        // 添加新消息
        let message_id = message.id.clone();
        messages.push(message);
        
        // 限制消息数量
        let len = messages.len();
        let messages: Vec<PubSubMessage> = if len > MAX_MESSAGES_PER_TOPIC {
            messages.into_iter()
                .skip(len - MAX_MESSAGES_PER_TOPIC)
                .collect()
        } else {
            messages
        };
        
        // 保存
        self.kv
            .put(&key, &messages, Some(PUBSUB_MESSAGE_TTL))
            .await
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
        
        Ok(message_id)
    }

    /// 获取主题消息（从指定时间戳之后）
    pub async fn get_messages(&self, topic: &str, since: Option<i64>) -> Result<Vec<PubSubMessage>> {
        let key = Self::topic_key(topic);
        
        let messages: Vec<PubSubMessage> = self.kv
            .get(&key)
            .await
            .map_err(|e| worker::Error::RustError(e.to_string()))?
            .unwrap_or_default();
        
        // 过滤时间戳
        let filtered = if let Some(since_ts) = since {
            messages.into_iter()
                .filter(|m| m.timestamp > since_ts)
                .collect()
        } else {
            messages
        };
        
        Ok(filtered)
    }

    fn topic_key(topic: &str) -> String {
        format!("pubsub:{}", topic)
    }
}

/// 处理发布消息请求
pub async fn handle_publish(
    pubsub: &PubSubManager,
    req: &mut Request,
) -> Result<Response> {
    let body: PublishRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    if body.topic.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "topic is required".to_string(),
        };
        return json_response_with_status(&error_response, 400);
    }

    match pubsub.publish(&body.topic, body.message).await {
        Ok(message_id) => {
            let response = PublishResponse {
                success: true,
                message_id,
            };
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to publish message: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

/// 处理获取消息请求
pub async fn handle_get_messages(
    pubsub: &PubSubManager,
    req: &Request,
) -> Result<Response> {
    let url = req.url()?;
    let mut topic: Option<String> = None;
    let mut since: Option<i64> = None;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "topic" => topic = Some(value.into_owned()),
            "since" => {
                if let Ok(parsed) = value.parse::<i64>() {
                    since = Some(parsed);
                }
            }
            _ => {}
        }
    }

    let topic = match topic {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            let error_response = ErrorResponse {
                error: "Missing topic query parameter".to_string(),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    match pubsub.get_messages(&topic, since).await {
        Ok(messages) => {
            let response = MessagesResponse {
                topic,
                count: messages.len(),
                messages,
            };
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get messages: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}
