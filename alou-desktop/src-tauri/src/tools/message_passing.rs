use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolCategory};
use crate::tools::ExecutionContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum MessageOperation {
    /// 发送消息到指定主题
    #[serde(rename = "send_message")]
    SendMessage {
        /// 主题名称
        topic: String,
        /// 消息内容
        content: String,
        /// 消息类型 (可选，默认为 "text")
        message_type: Option<String>,
        /// 消息标签 (可选)
        tags: Option<Vec<String>>,
    },
    
    /// 订阅主题并接收消息
    #[serde(rename = "subscribe_topic")]
    SubscribeTopic {
        /// 主题名称
        topic: String,
        /// 是否仅获取最新消息 (可选，默认为 false)
        latest_only: Option<bool>,
        /// 消息数量限制 (可选，默认为 10)
        limit: Option<usize>,
    },
    
    /// 获取主题列表
    #[serde(rename = "list_topics")]
    ListTopics {},
    
    /// 获取特定主题的消息历史
    #[serde(rename = "get_message_history")]
    GetMessageHistory {
        /// 主题名称
        topic: String,
        /// 消息数量限制 (可选，默认为 10)
        limit: Option<usize>,
    },
    
    /// 创建新主题
    #[serde(rename = "create_topic")]
    CreateTopic {
        /// 主题名称
        topic: String,
        /// 主题描述 (可选)
        description: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct MessagePassingTool {
    id: String,
    name: String,
    description: String,
    category: ToolCategory,
}

impl MessagePassingTool {
    pub fn new() -> Self {
        Self {
            id: "message_passing".to_string(),
            name: "Message Passing Tool".to_string(),
            description: "Tool for sending and receiving messages via Iroh and PubSub networks".to_string(),
            category: ToolCategory::Communication,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct MessagePassingResult {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
    pub output: Option<String>,
}

// 模拟消息传递客户端
pub struct MessageClient {
    topics: HashMap<String, TopicData>,
    node_id: String,
}

#[derive(Clone, Debug)]
struct TopicData {
    name: String,
    description: Option<String>,
    messages: Vec<Message>,
    created_at: i64,
}

#[derive(Clone, Debug, Serialize)]
struct Message {
    id: String,
    content: String,
    message_type: String,
    sender: String,
    timestamp: i64,
    tags: Vec<String>,
}

impl MessageClient {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            node_id: format!("node_{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
        }
    }

    pub async fn send_message(
        &mut self,
        topic: &str,
        content: String,
        message_type: Option<String>,
        tags: Option<Vec<String>>
    ) -> Result<String, String> {
        // 确保主题存在
        if !self.topics.contains_key(topic) {
            self.create_topic(topic, None).await?;
        }

        let message = Message {
            id: format!("msg_{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            content,
            message_type: message_type.unwrap_or_else(|| "text".to_string()),
            sender: self.node_id.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            tags: tags.unwrap_or_default(),
        };

        if let Some(topic_data) = self.topics.get_mut(topic) {
            topic_data.messages.push(message.clone());
        }

        Ok(message.id)
    }

    pub async fn subscribe_topic(
        &self,
        topic: &str,
        latest_only: Option<bool>,
        limit: Option<usize>
    ) -> Result<Vec<Message>, String> {
        if !self.topics.contains_key(topic) {
            return Err(format!("Topic '{}' not found", topic));
        }

        let topic_data = self.topics.get(topic).unwrap();
        let mut messages = topic_data.messages.clone();

        // 根据参数过滤消息
        if latest_only.unwrap_or(false) {
            if let Some(last_msg) = messages.last() {
                messages = vec![last_msg.clone()];
            } else {
                messages.clear();
            }
        } else {
            let limit = limit.unwrap_or(10);
            if messages.len() > limit {
                messages = messages[messages.len() - limit..].to_vec();
            }
        }

        Ok(messages)
    }

    pub async fn list_topics(&self) -> Vec<TopicInfo> {
        self.topics
            .values()
            .map(|topic| TopicInfo {
                name: topic.name.clone(),
                description: topic.description.clone(),
                message_count: topic.messages.len(),
                created_at: topic.created_at,
            })
            .collect()
    }

    pub async fn get_message_history(
        &self,
        topic: &str,
        limit: Option<usize>
    ) -> Result<Vec<Message>, String> {
        if !self.topics.contains_key(topic) {
            return Err(format!("Topic '{}' not found", topic));
        }

        let topic_data = self.topics.get(topic).unwrap();
        let mut messages = topic_data.messages.clone();
        
        let limit = limit.unwrap_or(10);
        if messages.len() > limit {
            messages = messages[messages.len() - limit..].to_vec();
        }

        Ok(messages)
    }

    pub async fn create_topic(&mut self, name: &str, description: Option<String>) -> Result<(), String> {
        if self.topics.contains_key(name) {
            return Err(format!("Topic '{}' already exists", name));
        }

        let topic_data = TopicData {
            name: name.to_string(),
            description,
            messages: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
        };

        self.topics.insert(name.to_string(), topic_data);
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
struct TopicInfo {
    name: String,
    description: Option<String>,
    message_count: usize,
    created_at: i64,
}

impl MessagePassingTool {
    async fn execute_impl(&self, args: Value) -> Result<MessagePassingResult, Box<dyn std::error::Error>> {
        let operation: MessageOperation = serde_json::from_value(args)?;
        let mut client = MessageClient::new();

        match operation {
            MessageOperation::SendMessage { topic, content, message_type, tags } => {
                match client.send_message(&topic, content, message_type, tags).await {
                    Ok(message_id) => {
                        Ok(MessagePassingResult {
                            success: true,
                            message: format!("Successfully sent message to topic '{}'", topic),
                            data: Some(serde_json::json!({
                                "topic": topic,
                                "message_id": message_id,
                                "sender": client.node_id
                            })),
                            output: Some(format!("Message sent to topic '{}', message ID: {}", topic, message_id)),
                        })
                    }
                    Err(e) => {
                        Ok(MessagePassingResult {
                            success: false,
                            message: format!("Failed to send message: {}", e),
                            data: None,
                            output: Some(format!("Error sending message: {}", e)),
                        })
                    }
                }
            }

            MessageOperation::SubscribeTopic { topic, latest_only, limit } => {
                match client.subscribe_topic(&topic, latest_only, limit).await {
                    Ok(messages) => {
                        let messages_json: Vec<Value> = messages
                            .into_iter()
                            .map(|msg| {
                                serde_json::json!({
                                    "id": msg.id,
                                    "content": msg.content,
                                    "type": msg.message_type,
                                    "sender": msg.sender,
                                    "timestamp": msg.timestamp,
                                    "tags": msg.tags
                                })
                            })
                            .collect();

                        Ok(MessagePassingResult {
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
                        Ok(MessagePassingResult {
                            success: false,
                            message: format!("Failed to subscribe to topic: {}", e),
                            data: None,
                            output: Some(format!("Error subscribing to topic: {}", e)),
                        })
                    }
                }
            }

            MessageOperation::ListTopics {} => {
                let topics = client.list_topics().await;
                let topics_json: Vec<Value> = topics
                    .into_iter()
                    .map(|topic| {
                        serde_json::json!({
                            "name": topic.name,
                            "description": topic.description,
                            "message_count": topic.message_count,
                            "created_at": topic.created_at
                        })
                    })
                    .collect();

                Ok(MessagePassingResult {
                    success: true,
                    message: "Successfully retrieved topic list".to_string(),
                    data: Some(serde_json::json!({
                        "topics": topics_json,
                        "count": topics_json.len()
                    })),
                    output: Some(format!("Found {} topics", topics_json.len())),
                })
            }

            MessageOperation::GetMessageHistory { topic, limit } => {
                match client.get_message_history(&topic, limit).await {
                    Ok(messages) => {
                        let messages_json: Vec<Value> = messages
                            .into_iter()
                            .map(|msg| {
                                serde_json::json!({
                                    "id": msg.id,
                                    "content": msg.content,
                                    "type": msg.message_type,
                                    "sender": msg.sender,
                                    "timestamp": msg.timestamp,
                                    "tags": msg.tags
                                })
                            })
                            .collect();

                        Ok(MessagePassingResult {
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
                        Ok(MessagePassingResult {
                            success: false,
                            message: format!("Failed to get message history: {}", e),
                            data: None,
                            output: Some(format!("Error getting message history: {}", e)),
                        })
                    }
                }
            }

            MessageOperation::CreateTopic { topic, description } => {
                match client.create_topic(&topic, description.clone()).await {
                    Ok(_) => {
                        Ok(MessagePassingResult {
                            success: true,
                            message: format!("Successfully created topic: {}", topic),
                            data: Some(serde_json::json!({
                                "topic": topic,
                                "description": description,
                                "node_id": client.node_id
                            })),
                            output: Some(format!("Created new topic: '{}'", topic)),
                        })
                    }
                    Err(e) => {
                        Ok(MessagePassingResult {
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
impl ToolExecutor for MessagePassingTool {
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
            dependencies: vec!["iroh".to_string(), "ipfs".to_string()],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec!["network".to_string(), "read".to_string(), "write".to_string()],
            tags: vec!["messaging".to_string(), "communication".to_string()],
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
                    output: Some(format!("Error executing message passing tool: {}", e)),
                    warnings: vec![],
                    context: None,
                })
            }
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        match serde_json::from_value::<MessageOperation>(args.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToolError::InvalidArguments(e.to_string())),
        }
    }

    fn help(&self) -> String {
        "Message Passing Tool for sending/receiving messages via Iroh and PubSub networks. Actions: send_message, subscribe_topic, list_topics, get_message_history, create_topic".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_send_and_receive_message() {
        let tool = MessagePassingTool::new();
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
            "action": "create_topic",
            "topic": "test_topic",
            "description": "Test topic for messaging"
        });
        let create_result = tool.execute(create_args, &context).await.unwrap();
        assert!(create_result.success);

        // 发送消息
        let send_args = json!({
            "action": "send_message",
            "topic": "test_topic",
            "content": "Hello, world!",
            "message_type": "text"
        });
        let send_result = tool.execute(send_args, &context).await.unwrap();
        assert!(send_result.success);
        assert!(send_result.output.as_ref().unwrap().contains("Message sent"));

        // 订阅并接收消息
        let subscribe_args = json!({
            "action": "subscribe_topic",
            "topic": "test_topic"
        });
        let subscribe_result = tool.execute(subscribe_args, &context).await.unwrap();
        assert!(subscribe_result.success);
        assert_eq!(subscribe_result.data["messages"].as_array().unwrap().len(), 1);
        assert_eq!(subscribe_result.data["messages"][0]["content"], "Hello, world!");
    }

    #[tokio::test]
    async fn test_list_topics() {
        let tool = MessagePassingTool::new();
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
            "action": "create_topic",
            "topic": "topic_1",
            "description": "First test topic"
        });
        let create_args2 = json!({
            "action": "create_topic",
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
        
        let topics = list_result.data["topics"].as_array().unwrap();
        assert!(topics.len() >= 2);
    }
}