//! 智能体协作工具 - IPFS PubSub 版本
//!
//! 使用 IPFS PubSub 实现去中心化的智能体群聊系统
//! 特点：
//! - 无需中央服务器，完全去中心化
//! - 消息通过 IPFS 网络广播
//! - 支持端到端加密（可选）
//! - 消息持久化到 IPFS（可选）

use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

/// IPFS API 客户端
pub struct IpfsClient {
    api_url: String,
    client: reqwest::Client,
}

impl IpfsClient {
    #[allow(dead_code)]
    pub fn new(api_url: String) -> Self {
        let client = reqwest::Client::builder()
            .http1_only()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .no_proxy()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self { api_url, client }
    }

    /// 发布消息到 PubSub 主题
    pub async fn pubsub_publish(&self, topic: &str, message: &str) -> Result<(), String> {
        let endpoint = format!("{}/api/v0/pubsub/pub", self.normalize_url());
        let encoded_message = urlencoding::encode(message);
        
        let response = self.client
            .post(&endpoint)
            .query(&[("arg", topic), ("arg", &encoded_message)])
            .send()
            .await
            .map_err(|e| format!("PubSub publish failed: {}", e))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("PubSub publish failed: {}", response.status()))
        }
    }

    /// 获取主题的订阅者
    pub async fn pubsub_peers(&self, topic: &str) -> Result<Vec<String>, String> {
        let endpoint = format!("{}/api/v0/pubsub/peers", self.normalize_url());
        
        let response = self.client
            .post(&endpoint)
            .query(&[("arg", topic)])
            .send()
            .await
            .map_err(|e| format!("PubSub peers failed: {}", e))?;
        
        if response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            #[derive(Deserialize)]
            struct PeersResponse {
                #[serde(rename = "Strings")]
                strings: Option<Vec<String>>,
            }
            let parsed: PeersResponse = serde_json::from_str(&text)
                .unwrap_or(PeersResponse { strings: None });
            Ok(parsed.strings.unwrap_or_default())
        } else {
            Ok(vec![])
        }
    }

    fn normalize_url(&self) -> String {
        self.api_url.trim_end_matches('/').to_string()
    }
}

/// 智能体协作工具
pub struct AgentCollaborationTool {
    metadata: ToolMetadata,
    /// IPFS 客户端
    ipfs_client: Arc<IpfsClient>,
    /// 本地会话缓存（用于快速查询）
    sessions: Arc<RwLock<HashMap<String, CollaborationSession>>>,
    /// 消息接收通道（预留用于未来实现实时订阅）
    #[allow(dead_code)]
    message_receiver: Arc<RwLock<HashMap<String, tokio::sync::mpsc::UnboundedReceiver<PubSubChatMessage>>>>,
}

/// 群聊会话结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// 会话ID（也是 PubSub 主题名称）
    pub id: String,
    /// 会话名称
    pub name: String,
    /// 会话描述
    pub description: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 创建者智能体ID
    pub creator: String,
    /// 参与者列表
    pub participants: HashMap<String, ParticipantInfo>,
    /// 创建时间
    pub created_at: i64,
    /// 最后活动时间
    pub last_activity: i64,
    /// 会话状态
    pub status: SessionStatus,
    /// 是否启用消息持久化
    pub persist_messages: bool,
    /// 消息历史 CID（如果启用了持久化）
    pub message_log_cid: Option<String>,
}

/// 参与者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    /// 智能体ID
    pub agent_id: String,
    /// 加入时间
    pub joined_at: i64,
    /// 最后活跃时间
    pub last_seen: i64,
    /// 角色
    pub role: ParticipantRole,
    /// IPFS 节点 ID（用于直接通信）
    pub ipfs_peer_id: Option<String>,
}

/// 参与者角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParticipantRole {
    Creator,
    Admin,
    Member,
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Inactive,
    Archived,
}

/// PubSub 聊天消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubChatMessage {
    /// 消息ID
    pub id: String,
    /// 会话ID（主题）
    pub session_id: String,
    /// 发送者智能体ID
    pub sender: String,
    /// 消息内容
    pub content: String,
    /// 时间戳
    pub timestamp: i64,
    /// 消息类型
    pub message_type: MessageType,
    /// 回复的消息ID
    pub reply_to: Option<String>,
    /// 发送者的 IPFS 节点 ID
    pub sender_peer_id: Option<String>,
    /// 消息签名（用于验证）
    pub signature: Option<String>,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Text,
    Command,
    System,
    Response,
    File,
}

/// 系统消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum SystemEvent {
    AgentJoined { agent_id: String, inviter: Option<String> },
    AgentLeft { agent_id: String },
    SessionCreated { creator: String },
    SessionArchived { archiver: String },
    RoleChanged { agent_id: String, new_role: ParticipantRole, changed_by: String },
}

impl AgentCollaborationTool {
    /// 创建新的智能体协作工具
    pub fn new(ipfs_api_url: String) -> Self {
        let ipfs_client = Arc::new(IpfsClient::new(ipfs_api_url));
        
        Self {
            metadata: ToolMetadata {
                id: "agent_collaboration".to_string(),
                name: "Agent Collaboration Tool".to_string(),
                description: "基于 IPFS PubSub 的去中心化智能体群聊".to_string(),
                category: ToolCategory::Communication,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "2.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                dependencies: vec!["ipfs".to_string()],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "network".to_string()],
            },
            ipfs_client,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            message_receiver: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 生成 PubSub 主题名称
    pub fn generate_topic(&self, session_id: &str) -> String {
        format!("alou-chat-{}", session_id)
    }

    /// 创建新的协作会话
    pub async fn create_session(
        &self,
        name: &str,
        creator: &str,
        description: Option<String>,
        tags: Vec<String>,
        persist_messages: bool,
    ) -> Result<String, ToolError> {
        let session_id = format!("session_{}", uuid::Uuid::new_v4());
        let topic = self.generate_topic(&session_id);
        let now = Utc::now().timestamp();
        
        // 创建参与者信息
        let mut participants = HashMap::new();
        participants.insert(creator.to_string(), ParticipantInfo {
            agent_id: creator.to_string(),
            joined_at: now,
            last_seen: now,
            role: ParticipantRole::Creator,
            ipfs_peer_id: None,
        });
        
        let session = CollaborationSession {
            id: session_id.clone(),
            name: name.to_string(),
            description,
            tags,
            creator: creator.to_string(),
            participants,
            created_at: now,
            last_activity: now,
            status: SessionStatus::Active,
            persist_messages,
            message_log_cid: None,
        };

        // 保存到本地缓存
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }
        
        // 发布系统消息到 PubSub
        let system_event = SystemEvent::SessionCreated {
            creator: creator.to_string(),
        };
        self.publish_system_event(&topic, &system_event).await?;
        
        println!("[AgentCollaboration] Created session '{}' on topic '{}'", session_id, topic);
        
        Ok(session_id)
    }

    /// 邀请智能体加入会话
    pub async fn invite_agent(
        &self,
        session_id: &str,
        agent_id: &str,
        inviter_id: &str,
    ) -> Result<bool, ToolError> {
        let topic = self.generate_topic(session_id);
        
        // 更新本地会话
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                // 检查邀请者权限
                if let Some(inviter) = session.participants.get(inviter_id) {
                    if inviter.role != ParticipantRole::Creator && inviter.role != ParticipantRole::Admin {
                        return Err(ToolError::PermissionDenied("Only creator or admin can invite".to_string()));
                    }
                } else {
                    return Err(ToolError::PermissionDenied("Inviter is not a participant".to_string()));
                }
                
                if session.participants.contains_key(agent_id) {
                    return Ok(false); // 已在会话中
                }
                
                let now = Utc::now().timestamp();
                session.participants.insert(agent_id.to_string(), ParticipantInfo {
                    agent_id: agent_id.to_string(),
                    joined_at: now,
                    last_seen: now,
                    role: ParticipantRole::Member,
                    ipfs_peer_id: None,
                });
                session.last_activity = now;
            } else {
                return Err(ToolError::InvalidArguments(format!("Session '{}' not found", session_id)));
            }
        }
        
        // 发布系统事件到 PubSub
        let system_event = SystemEvent::AgentJoined {
            agent_id: agent_id.to_string(),
            inviter: Some(inviter_id.to_string()),
        };
        self.publish_system_event(&topic, &system_event).await?;
        
        Ok(true)
    }

    /// 智能体离开会话
    pub async fn leave_session(&self, session_id: &str, agent_id: &str) -> Result<bool, ToolError> {
        let topic = self.generate_topic(session_id);
        
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                if session.participants.remove(agent_id).is_none() {
                    return Ok(false);
                }
                
                // 如果创建者离开，转移权限
                if session.creator == agent_id && !session.participants.is_empty() {
                    if let Some((new_creator_id, _)) = session.participants.iter().next() {
                        let new_creator_id = new_creator_id.clone();
                        if let Some(participant) = session.participants.get_mut(&new_creator_id) {
                            participant.role = ParticipantRole::Creator;
                        }
                        session.creator = new_creator_id;
                    }
                }
                
                // 如果没有参与者，归档会话
                if session.participants.is_empty() {
                    session.status = SessionStatus::Archived;
                }
                
                session.last_activity = Utc::now().timestamp();
            } else {
                return Err(ToolError::InvalidArguments(format!("Session '{}' not found", session_id)));
            }
        }
        
        // 发布系统事件
        let system_event = SystemEvent::AgentLeft {
            agent_id: agent_id.to_string(),
        };
        self.publish_system_event(&topic, &system_event).await?;
        
        Ok(true)
    }

    /// 发送消息
    pub async fn send_message(
        &self,
        session_id: &str,
        sender: &str,
        content: &str,
        message_type: MessageType,
        reply_to: Option<String>,
    ) -> Result<String, ToolError> {
        // 检查会话和权限
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                if session.status != SessionStatus::Active {
                    return Err(ToolError::ExecutionFailed("Session is not active".to_string()));
                }
                if !session.participants.contains_key(sender) {
                    return Err(ToolError::PermissionDenied("Sender is not a participant".to_string()));
                }
            } else {
                return Err(ToolError::InvalidArguments(format!("Session '{}' not found", session_id)));
            }
        }
        
        let topic = self.generate_topic(session_id);
        let message_id = format!("msg_{}", uuid::Uuid::new_v4());
        
        let message = PubSubChatMessage {
            id: message_id.clone(),
            session_id: session_id.to_string(),
            sender: sender.to_string(),
            content: content.to_string(),
            timestamp: Utc::now().timestamp(),
            message_type,
            reply_to,
            sender_peer_id: None,
            signature: None,
        };
        
        // 序列化并发布到 PubSub
        let message_json = serde_json::to_string(&message)
            .map_err(|e| ToolError::InternalError(format!("Failed to serialize message: {}", e)))?;
        
        self.ipfs_client.pubsub_publish(&topic, &message_json).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to publish message: {}", e)))?;
        
        // 更新最后活动时间
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.last_activity = Utc::now().timestamp();
            }
        }
        
        Ok(message_id)
    }

    /// 发布系统事件
    pub async fn publish_system_event(&self, topic: &str, event: &SystemEvent) -> Result<(), ToolError> {
        let event_json = serde_json::to_string(event)
            .map_err(|e| ToolError::InternalError(format!("Failed to serialize event: {}", e)))?;
        
        let message = PubSubChatMessage {
            id: format!("sys_{}", uuid::Uuid::new_v4()),
            session_id: topic.replace("alou-chat-", ""),
            sender: "system".to_string(),
            content: event_json,
            timestamp: Utc::now().timestamp(),
            message_type: MessageType::System,
            reply_to: None,
            sender_peer_id: None,
            signature: None,
        };
        
        let message_json = serde_json::to_string(&message)
            .map_err(|e| ToolError::InternalError(format!("Failed to serialize message: {}", e)))?;
        
        self.ipfs_client.pubsub_publish(topic, &message_json).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to publish system event: {}", e)))?;
        
        Ok(())
    }

    /// 获取会话信息
    pub async fn get_session(&self, session_id: &str) -> Result<Option<CollaborationSession>, ToolError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    /// 列出所有活跃会话
    pub async fn list_sessions(&self) -> Result<Vec<CollaborationSession>, ToolError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values()
            .filter(|s| s.status == SessionStatus::Active)
            .cloned()
            .collect())
    }

    /// 获取会话的在线订阅者
    pub async fn get_online_peers(&self, session_id: &str) -> Result<Vec<String>, ToolError> {
        let topic = self.generate_topic(session_id);
        self.ipfs_client.pubsub_peers(&topic).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get peers: {}", e)))
    }

    /// 归档会话
    pub async fn archive_session(&self, session_id: &str, requester: &str) -> Result<bool, ToolError> {
        let topic = self.generate_topic(session_id);
        
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                // 检查权限
                if let Some(participant) = session.participants.get(requester) {
                    if participant.role != ParticipantRole::Creator && participant.role != ParticipantRole::Admin {
                        return Err(ToolError::PermissionDenied("Only creator or admin can archive".to_string()));
                    }
                } else {
                    return Err(ToolError::PermissionDenied("Requester is not a participant".to_string()));
                }
                
                session.status = SessionStatus::Archived;
            } else {
                return Ok(false);
            }
        }
        
        // 发布系统事件
        let system_event = SystemEvent::SessionArchived {
            archiver: requester.to_string(),
        };
        self.publish_system_event(&topic, &system_event).await?;
        
        Ok(true)
    }
}

#[async_trait]
impl ToolExecutor for AgentCollaborationTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "create_session" => {
                let name = args.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'name' field".to_string()))?
                    .to_string();

                let creator = args.get("creator")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'creator' field".to_string()))?
                    .to_string();

                let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                
                let tags: Vec<String> = args.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                
                let persist_messages = args.get("persist_messages").and_then(|v| v.as_bool()).unwrap_or(false);

                let session_id = self.create_session(&name, &creator, description, tags, persist_messages).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "session_id": session_id,
                        "name": name,
                        "creator": creator,
                        "topic": self.generate_topic(&session_id)
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Created collaboration session: {} (topic: {})", session_id, self.generate_topic(&session_id))),
                    warnings: vec![],
                    context: None,
                })
            }

            "invite_agent" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'session_id' field".to_string()))?
                    .to_string();

                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let inviter_id = args.get("inviter_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'inviter_id' field".to_string()))?
                    .to_string();

                let invited = self.invite_agent(&session_id, &agent_id, &inviter_id).await?;

                Ok(ToolResult {
                    success: invited,
                    data: serde_json::json!({
                        "session_id": session_id,
                        "agent_id": agent_id,
                        "invited": invited
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(if invited {
                        format!("Invited agent '{}' to session '{}'", agent_id, session_id)
                    } else {
                        format!("Agent '{}' already in session '{}'", agent_id, session_id)
                    }),
                    warnings: vec![],
                    context: None,
                })
            }

            "leave_session" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'session_id' field".to_string()))?
                    .to_string();

                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let left = self.leave_session(&session_id, &agent_id).await?;

                Ok(ToolResult {
                    success: left,
                    data: serde_json::json!({
                        "session_id": session_id,
                        "agent_id": agent_id,
                        "left": left
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(if left {
                        format!("Agent '{}' left session '{}'", agent_id, session_id)
                    } else {
                        format!("Agent '{}' was not in session '{}'", agent_id, session_id)
                    }),
                    warnings: vec![],
                    context: None,
                })
            }

            "send_message" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'session_id' field".to_string()))?
                    .to_string();

                let sender = args.get("sender")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'sender' field".to_string()))?
                    .to_string();

                let content = args.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'content' field".to_string()))?
                    .to_string();

                let message_type_str = args.get("message_type").and_then(|v| v.as_str()).unwrap_or("text");
                let message_type = match message_type_str {
                    "text" => MessageType::Text,
                    "command" => MessageType::Command,
                    "system" => MessageType::System,
                    "response" => MessageType::Response,
                    "file" => MessageType::File,
                    _ => MessageType::Text,
                };

                let reply_to = args.get("reply_to").and_then(|v| v.as_str()).map(|s| s.to_string());

                let message_id = self.send_message(&session_id, &sender, &content, message_type, reply_to).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "message_id": message_id,
                        "session_id": session_id,
                        "sender": sender
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Sent message to session '{}'", session_id)),
                    warnings: vec![],
                    context: None,
                })
            }

            "get_session" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'session_id' field".to_string()))?
                    .to_string();

                let session = self.get_session(&session_id).await?;

                Ok(ToolResult {
                    success: session.is_some(),
                    data: serde_json::json!({
                        "session": session,
                        "found": session.is_some()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: session.as_ref().map(|s| format!("Found session: {}", s.name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "list_sessions" => {
                let sessions = self.list_sessions().await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "sessions": sessions,
                        "count": sessions.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} active sessions", sessions.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "get_online_peers" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'session_id' field".to_string()))?
                    .to_string();

                let peers = self.get_online_peers(&session_id).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "session_id": session_id,
                        "peers": peers,
                        "count": peers.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} online peers in session '{}'", peers.len(), session_id)),
                    warnings: vec![],
                    context: None,
                })
            }

            "archive_session" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'session_id' field".to_string()))?
                    .to_string();

                let requester = args.get("requester")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'requester' field".to_string()))?
                    .to_string();

                let archived = self.archive_session(&session_id, &requester).await?;

                Ok(ToolResult {
                    success: archived,
                    data: serde_json::json!({
                        "session_id": session_id,
                        "archived": archived
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(if archived {
                        format!("Archived session '{}'", session_id)
                    } else {
                        format!("Failed to archive session '{}'", session_id)
                    }),
                    warnings: vec![],
                    context: None,
                })
            }

            _ => Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }
        if args.get("action").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: action".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        r#"Agent Collaboration Tool (IPFS PubSub)

Decentralized agent group chat using IPFS PubSub.

Actions:
  - create_session: Create a new collaboration session
    params: {
      "name": "Session Name",
      "creator": "Agent ID of creator",
      "description": "Session description (optional)",
      "tags": ["tag1", "tag2"],
      "persist_messages": false
    }

  - invite_agent: Invite an agent to join a session
    params: {
      "session_id": "Session ID",
      "agent_id": "Agent ID to invite",
      "inviter_id": "Agent ID who is inviting"
    }

  - leave_session: Leave a session
    params: {
      "session_id": "Session ID",
      "agent_id": "Agent ID leaving"
    }

  - send_message: Send a message to a session (broadcast via PubSub)
    params: {
      "session_id": "Session ID",
      "sender": "Agent ID of sender",
      "content": "Message content",
      "message_type": "text|command|system|response|file",
      "reply_to": "Message ID to reply to (optional)"
    }

  - get_session: Get session information
    params: {
      "session_id": "Session ID"
    }

  - list_sessions: List all active sessions

  - get_online_peers: Get online peers in a session
    params: {
      "session_id": "Session ID"
    }

  - archive_session: Archive a session
    params: {
      "session_id": "Session ID",
      "requester": "Agent ID requesting archive"
    }

Examples:

Create a collaboration session:
{
  "action": "create_session",
  "name": "Code Review Session",
  "creator": "code-reviewer-agent",
  "description": "Reviewing PR #123",
  "tags": ["code-review", "urgent"]
}

Invite an agent:
{
  "action": "invite_agent",
  "session_id": "session_xxx",
  "agent_id": "security-audit-agent",
  "inviter_id": "code-reviewer-agent"
}

Send a message (broadcast to all subscribers):
{
  "action": "send_message",
  "session_id": "session_xxx",
  "sender": "code-reviewer-agent",
  "content": "Let's review this pull request together.",
  "message_type": "text"
}

Check online peers:
{
  "action": "get_online_peers",
  "session_id": "session_xxx"
}"#.to_string()
    }
}