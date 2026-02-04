use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolCategory};
use crate::tools::ExecutionContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum PubSubOperation {
    /// 发布消息到主题
    #[serde(rename = "publish")]
    Publish {
        /// 主题名称
        topic: String,
        /// 消息内容
        message: String,
        /// 消息类型 (可选)
        message_type: Option<String>,
        /// 消息标签 (可选)
        tags: Option<Vec<String>>,
    },
    
    /// 订阅主题
    #[serde(rename = "subscribe")]
    Subscribe {
        /// 主题名称
        topic: String,
        /// 消息数量限制 (可选，默认为 10)
        limit: Option<usize>,
    },
    
    /// 获取订阅者数量
    #[serde(rename = "subscriber_count")]
    SubscriberCount {
        /// 主题名称
        topic: String,
    },
    
    /// 获取所有主题
    #[serde(rename = "list_topics")]
    ListTopics {},
    
    /// 获取主题的历史消息
    #[serde(rename = "get_history")]
    GetHistory {
        /// 主题名称
        topic: String,
        /// 消息数量限制 (可选，默认为 10)
        limit: Option<usize>,
    },
    
    /// 创建持久化主题
    #[serde(rename = "create_persistent_topic")]
    CreatePersistentTopic {
        /// 主题名称
        topic: String,
        /// 主题描述 (可选)
        description: Option<String>,
        /// 是否持久化 (可选，默认为 true)
        persistent: Option<bool>,
    },
}

#[derive(Debug, Clone)]
pub struct PubSubTool {
    id: String,
    name: String,
    description: String,
    category: ToolCategory,
}

impl PubSubTool {
    pub fn new() -> Self {
        Self {
            id: "pubsub".to_string(),
            name: "PubSub Tool".to_string(),
            description: "Tool for publish-subscribe messaging patterns".to_string(),
            category: ToolCategory::Communication,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct PubSubResult {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
    pub output: Option<String>,
}

// 模拟 PubSub 客户端
pub struct PubSubClient {
    topics: HashMap<String, TopicData>,
    message_history: HashMap<String, Vec<PubSubMessage>>,
    node_id: String,
}

#[derive(Clone, Debug)]
struct TopicData {
    name: String,
    description: Option<String>,
    subscribers: usize,
    persistent: bool,
    created_at: i64,
}

#[derive(Clone, Debug, Serialize)]
struct PubSubMessage {
    id: String,
    topic: String,
    message: String,
    message_type: String,
    sender: String,
    timestamp: i64,
    tags: Vec<String>,
}

impl PubSubClient {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            message_history: HashMap::new(),
            node_id: format!("pubsub_node_{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
        }
    }

    pub async fn publish(
        &mut self,
        topic: &str,
        message: String,
        message_type: Option<String>,
        tags: Option<Vec<String>>
    ) -> Result<String, String> {
        // 确保主题存在
        if !self.topics.contains_key(topic) {
            self.create_topic(topic, None, true).await?;
        }

        let pubsub_message = PubSubMessage {
            id: format!("msg_{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            topic: topic.to_string(),
            message,
            message_type: message_type.unwrap_or_else(|| "text".to_string()),
            sender: self.node_id.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            tags: tags.unwrap_or_default(),
        };

        // 添加到消息历史
        self.message_history.entry(topic.to_string()).or_insert_with(Vec::new).push(pubsub_message.clone());

        Ok(pubsub_message.id)
    }

    pub async fn subscribe(&self, topic: &str, limit: Option<usize>) -> Result<Vec<PubSubMessage>, String> {
        if !self.topics.contains_key(topic) {
            return Err(format!("Topic '{}' not found", topic));
        }

        let empty_vec = vec![];
        let messages = self.message_history.get(topic).unwrap_or(&empty_vec);
        let limit = limit.unwrap_or(10);

        let start_idx = if messages.len() > limit { messages.len() - limit } else { 0 };
        let result_messages = messages[start_idx..].to_vec();

        Ok(result_messages)
    }

    pub async fn subscriber_count(&self, topic: &str) -> Result<usize, String> {
        if let Some(topic_data) = self.topics.get(topic) {
            Ok(topic_data.subscribers)
        } else {
            Err(format!("Topic '{}' not found", topic))
        }
    }

    pub async fn list_topics(&self) -> Vec<TopicInfo> {
        self.topics
            .values()
            .map(|topic| TopicInfo {
                name: topic.name.clone(),
                description: topic.description.clone(),
                subscribers: topic.subscribers,
                persistent: topic.persistent,
                created_at: topic.created_at,
                message_count: self.message_history.get(&topic.name).map(|msgs| msgs.len()).unwrap_or(0),
            })
            .collect()
    }

    pub async fn get_history(&self, topic: &str, limit: Option<usize>) -> Result<Vec<PubSubMessage>, String> {
        if !self.topics.contains_key(topic) {
            return Err(format!("Topic '{}' not found", topic));
        }

        let empty_vec = vec![];
        let messages = self.message_history.get(topic).unwrap_or(&empty_vec);
        let limit = limit.unwrap_or(10);

        let start_idx = if messages.len() > limit { messages.len() - limit } else { 0 };
        let result_messages = messages[start_idx..].to_vec();

        Ok(result_messages)
    }

    pub async fn create_topic(&mut self, name: &str, description: Option<String>, persistent: bool) -> Result<(), String> {
        if self.topics.contains_key(name) {
            return Err(format!("Topic '{}' already exists", name));
        }

        let topic_data = TopicData {
            name: name.to_string(),
            description,
            subscribers: 0,
            persistent,
            created_at: chrono::Utc::now().timestamp(),
        };

        self.topics.insert(name.to_string(), topic_data);
        self.message_history.insert(name.to_string(), Vec::new());
        
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
struct TopicInfo {
    name: String,
    description: Option<String>,
    subscribers: usize,
    persistent: bool,
    created_at: i64,
    message_count: usize,
}

impl PubSubTool {
    async fn execute_impl(&self, args: Value) -> Result<PubSubResult, Box<dyn std::error::Error>> {
        let operation: PubSubOperation = serde_json::from_value(args)?;
        let mut client = PubSubClient::new();

        match operation {
            PubSubOperation::Publish { topic, message, message_type, tags } => {
                match client.publish(&topic, message, message_type, tags).await {
                    Ok(message_id) => {
                        Ok(PubSubResult {
                            success: true,
                            message: format!("Successfully published message to topic '{}'", topic),
                            data: Some(serde_json::json!({
                                "topic": topic,
                                "message_id": message_id,
                                "sender": client.node_id
                            })),
                            output: Some(format!("Published message to topic '{}', message ID: {}", topic, message_id)),
                        })
                    }
                    Err(e) => {
                        Ok(PubSubResult {
                            success: false,
                            message: format!("Failed to publish message: {}", e),
                            data: None,
                            output: Some(format!("Error publishing message: {}", e)),
                        })
                    }
                }
            }

            PubSubOperation::Subscribe { topic, limit } => {
                match client.subscribe(&topic, limit).await {
                    Ok(messages) => {
                        let messages_json: Vec<Value> = messages
                            .into_iter()
                            .map(|msg| {
                                serde_json::json!({
                                    "id": msg.id,
                                    "topic": msg.topic,
                                    "message": msg.message,
                                    "type": msg.message_type,
                                    "sender": msg.sender,
                                    "timestamp": msg.timestamp,
                                    "tags": msg.tags
                                })
                            })
                            .collect();

                        Ok(PubSubResult {
                            success: true,
                            message: format!("Successfully subscribed to topic '{}'", topic),
                            data: Some(serde_json::json!({
                                "topic": topic,
                                "messages": messages_json,
                                "count": messages_json.len()
                            })),
                            output: Some(format!("Received {} messages from topic '{}'", messages_json.len(), topic)),
                        })
                    }
                    Err(e) => {
                        Ok(PubSubResult {
                            success: false,
                            message: format!("Failed to subscribe to topic: {}", e),
                            data: None,
                            output: Some(format!("Error subscribing to topic: {}", e)),
                        })
                    }
                }
            }

            PubSubOperation::SubscriberCount { topic } => {
                match client.subscriber_count(&topic).await {
                    Ok(count) => {
                        Ok(PubSubResult {
                            success: true,
                            message: format!("Subscriber count for topic '{}': {}", topic, count),
                            data: Some(serde_json::json!({
                                "topic": topic,
                                "subscriber_count": count
                            })),
                            output: Some(format!("Topic '{}' has {} subscribers", topic, count)),
                        })
                    }
                    Err(e) => {
                        Ok(PubSubResult {
                            success: false,
                            message: format!("Failed to get subscriber count: {}", e),
                            data: None,
                            output: Some(format!("Error getting subscriber count: {}", e)),
                        })
                    }
                }
            }

            PubSubOperation::ListTopics {} => {
                let topics = client.list_topics().await;
                let topics_json: Vec<Value> = topics
                    .into_iter()
                    .map(|topic| {
                        serde_json::json!({
                            "name": topic.name,
                            "description": topic.description,
                            "subscribers": topic.subscribers,
                            "persistent": topic.persistent,
                            "created_at": topic.created_at,
                            "message_count": topic.message_count
                        })
                    })
                    .collect();

                Ok(PubSubResult {
                    success: true,
                    message: "Successfully retrieved topic list".to_string(),
                    data: Some(serde_json::json!({
                        "topics": topics_json,
                        "count": topics_json.len()
                    })),
                    output: Some(format!("Found {} topics", topics_json.len())),
                })
            }

            PubSubOperation::GetHistory { topic, limit } => {
                match client.get_history(&topic, limit).await {
                    Ok(messages) => {
                        let messages_json: Vec<Value> = messages
                            .into_iter()
                            .map(|msg| {
                                serde_json::json!({
                                    "id": msg.id,
                                    "topic": msg.topic,
                                    "message": msg.message,
                                    "type": msg.message_type,
                                    "sender": msg.sender,
                                    "timestamp": msg.timestamp,
                                    "tags": msg.tags
                                })
                            })
                            .collect();

                        Ok(PubSubResult {
                            success: true,
                            message: format!("Successfully retrieved message history for topic '{}'", topic),
                            data: Some(serde_json::json!({
                                "topic": topic,
                                "messages": messages_json,
                                "count": messages_json.len()
                            })),
                            output: Some(format!("Retrieved {} messages from topic '{}'", messages_json.len(), topic)),
                        })
                    }
                    Err(e) => {
                        Ok(PubSubResult {
                            success: false,
                            message: format!("Failed to get message history: {}", e),
                            data: None,
                            output: Some(format!("Error getting message history: {}", e)),
                        })
                    }
                }
            }

            PubSubOperation::CreatePersistentTopic { topic, description, persistent } => {
                let persistent = persistent.unwrap_or(true);
                match client.create_topic(&topic, description.clone(), persistent).await {
                    Ok(_) => {
                        Ok(PubSubResult {
                            success: true,
                            message: format!("Successfully created topic: {}", topic),
                            data: Some(serde_json::json!({
                                "topic": topic,
                                "description": description,
                                "persistent": persistent,
                                "node_id": client.node_id
                            })),
                            output: Some(format!("Created new topic: '{}', persistent: {}", topic, persistent)),
                        })
                    }
                    Err(e) => {
                        Ok(PubSubResult {
                            success: false,
                            message: format!("Failed to create topic: {}", e),
                            data: None,
                            output: Some(format!("Error creating topic: {}", e)),
                        })
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for PubSubTool {
    fn metadata(&self) -> &crate::tools::ToolMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<crate::tools::ToolMetadata> = OnceLock::new();
        
        METADATA.get_or_init(|| crate::tools::ToolMetadata {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category,
            priority: crate::tools::ToolPriority::High,
            status: crate::tools::ToolStatus::Available,
            version: "1.0.0".to_string(),
            author: "Alou Team".to_string(),
            created_at: 1700000000,
            updated_at: 1700000000,
            dependencies: vec!["pubsub".to_string()],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec!["network".to_string(), "read".to_string(), "write".to_string()],
        })
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        match self.execute_impl(args).await {
            Ok(result) => {
                Ok(ToolResult {
                    success: result.success,
                    data: result.data.unwrap_or(serde_json::Value::Null),
                    error: if result.success { None } else { Some(result.message) },
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    output: result.output,
                    warnings: vec![],
                    context: None,
                })
            }
            Err(e) => {
                Ok(ToolResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(e.to_string()),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    output: Some(format!("Error executing pubsub tool: {}", e)),
                    warnings: vec![],
                    context: None,
                })
            }
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        match serde_json::from_value::<PubSubOperation>(args.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToolError::InvalidArguments(e.to_string())),
        }
    }

    fn help(&self) -> String {
        "PubSub Tool for publish-subscribe messaging. Actions: publish, subscribe, subscriber_count, list_topics, get_history, create_persistent_topic".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let tool = PubSubTool::new();
        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["read".to_string(), "write".to_string(), "network".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 创建主题
        let create_args = json!({
            "action": "create_persistent_topic",
            "topic": "test_topic",
            "description": "Test topic for pubsub"
        });
        let create_result = tool.execute(create_args, &context).await.unwrap();
        assert!(create_result.success);

        // 发布消息
        let publish_args = json!({
            "action": "publish",
            "topic": "test_topic",
            "message": "Hello, pubsub world!",
            "message_type": "text"
        });
        let publish_result = tool.execute(publish_args, &context).await.unwrap();
        assert!(publish_result.success);
        assert!(publish_result.output.as_ref().unwrap().contains("Published message"));

        // 订阅消息
        let subscribe_args = json!({
            "action": "subscribe",
            "topic": "test_topic"
        });
        let subscribe_result = tool.execute(subscribe_args, &context).await.unwrap();
        assert!(subscribe_result.success);
        assert_eq!(subscribe_result.data.as_ref().unwrap()["messages"].as_array().unwrap().len(), 1);
        assert_eq!(subscribe_result.data.as_ref().unwrap()["messages"][0]["message"], "Hello, pubsub world!");
    }

    #[tokio::test]
    async fn test_list_topics() {
        let tool = PubSubTool::new();
        let context = ExecutionContext {
            session_id: "test_session".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["read".to_string(), "write".to_string(), "network".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 创建多个主题
        let create_args1 = json!({
            "action": "create_persistent_topic",
            "topic": "topic_1",
            "description": "First test topic"
        });
        let create_args2 = json!({
            "action": "create_persistent_topic",
            "topic": "topic_2",
            "description": "Second test topic"
        });
        
        tool.execute(create_args1, &context).await.unwrap();
        tool.execute(create_args2, &context).await.unwrap();

        // 列出主题
        let list_args = json!({
            "action": "list_topics"
        });
        let list_result = tool.execute(list_args, &context).await.unwrap();
        assert!(list_result.success);
        
        let topics = list_result.data.as_ref().unwrap()["topics"].as_array().unwrap();
        assert!(topics.len() >= 2);
    }
}