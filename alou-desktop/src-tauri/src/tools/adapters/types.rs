//! 群聊适配器类型定义
//!
//! 定义群聊相关的核心数据类型，供所有适配器实现共享

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 群聊模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum GroupChatMode {
    /// 内存模式 - 本地离线群聊
    Memory,
    /// DIAP PubSub 模式
    Pubsub,
    /// Iroh P2P 模式
    Iroh,
}

impl Default for GroupChatMode {
    fn default() -> Self {
        GroupChatMode::Memory
    }
}

impl std::fmt::Display for GroupChatMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupChatMode::Memory => write!(f, "Memory"),
            GroupChatMode::Pubsub => write!(f, "Pubsub"),
            GroupChatMode::Iroh => write!(f, "Iroh"),
        }
    }
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Chat,
    System,
    Join,
    Leave,
    #[serde(rename = "agent_request")]
    AgentRequest,
    #[serde(rename = "agent_response")]
    AgentResponse,
    #[serde(rename = "task_assign")]
    TaskAssign,
    #[serde(rename = "task_progress")]
    TaskProgress,
    #[serde(rename = "task_complete")]
    TaskComplete,
    Error,
    Welcome,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Chat
    }
}

impl From<&str> for MessageType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "system" => MessageType::System,
            "join" => MessageType::Join,
            "leave" => MessageType::Leave,
            "agent_request" => MessageType::AgentRequest,
            "agent_response" => MessageType::AgentResponse,
            "task_assign" => MessageType::TaskAssign,
            "task_progress" => MessageType::TaskProgress,
            "task_complete" => MessageType::TaskComplete,
            "error" => MessageType::Error,
            "welcome" => MessageType::Welcome,
            _ => MessageType::Chat,
        }
    }
}

impl From<Option<String>> for MessageType {
    fn from(opt: Option<String>) -> Self {
        match opt.as_deref() {
            Some("text") | None => MessageType::Chat,
            Some("markdown") => MessageType::Chat,
            Some("code") => MessageType::Chat,
            Some("file") => MessageType::Chat,
            Some("image") => MessageType::Chat,
            Some("system") => MessageType::System,
            Some("join") => MessageType::Join,
            Some("leave") => MessageType::Leave,
            Some("agent_request") => MessageType::AgentRequest,
            Some("agent_response") => MessageType::AgentResponse,
            Some(_custom) => MessageType::Chat,
        }
    }
}

/// 智能体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

impl AgentInfo {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            did: None,
            avatar: None,
            mode: Some("agent".to_string()),
        }
    }

    pub fn with_did(mut self, did: impl Into<String>) -> Self {
        self.did = Some(did.into());
        self
    }

    pub fn with_avatar(mut self, avatar: impl Into<String>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }

    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }
}

/// 统一群聊结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGroup {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub mode: GroupChatMode,
    pub members: Vec<AgentInfo>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl UnifiedGroup {
    pub fn new(name: impl Into<String>, mode: GroupChatMode) -> Self {
        let prefix = match mode {
            GroupChatMode::Memory => "memory_group",
            GroupChatMode::Pubsub => "diap_group",
            GroupChatMode::Iroh => "iroh_group",
        };
        Self {
            id: format!(
                "{}_{}_{}",
                prefix,
                Utc::now().timestamp_millis(),
                &Uuid::new_v4().to_string()[..8]
            ),
            name: name.into(),
            description: None,
            mode,
            members: Vec::new(),
            created_at: Utc::now().timestamp_millis(),
            created_by: None,
            topic: None,
            ticket: None,
            metadata: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    pub fn with_ticket(mut self, ticket: impl Into<String>) -> Self {
        self.ticket = Some(ticket.into());
        self
    }

    pub fn with_created_by(mut self, created_by: impl Into<String>) -> Self {
        self.created_by = Some(created_by.into());
        self
    }

    pub fn add_member(&mut self, member: AgentInfo) {
        if !self.members.iter().any(|m| m.id == member.id) {
            self.members.push(member);
        }
    }

    pub fn remove_member(&mut self, member_id: &str) {
        self.members.retain(|m| m.id != member_id);
    }

    pub fn has_member(&self, member_id: &str) -> bool {
        self.members.iter().any(|m| m.id == member_id)
    }
}

/// 统一消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub id: String,
    pub group_id: String,
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub sender: AgentInfo,
    pub content: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered: Option<bool>,
}

impl UnifiedMessage {
    pub fn new(group_id: impl Into<String>, sender: AgentInfo, content: impl Into<String>) -> Self {
        Self {
            id: format!(
                "msg_{}_{}",
                Utc::now().timestamp_millis(),
                &Uuid::new_v4().to_string()[..8]
            ),
            group_id: group_id.into(),
            message_type: MessageType::Chat,
            sender,
            content: content.into(),
            timestamp: Utc::now().timestamp_millis(),
            metadata: None,
            delivered: Some(true),
        }
    }

    pub fn with_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_delivered(mut self, delivered: bool) -> Self {
        self.delivered = Some(delivered);
        self
    }
}

/// 群聊配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<AgentInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<GroupChatMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

impl GroupChatConfig {
    pub fn new(name: impl Into<String>, mode: GroupChatMode) -> Self {
        Self {
            name: name.into(),
            description: None,
            members: None,
            mode: Some(mode),
            topic: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_members(mut self, members: Vec<AgentInfo>) -> Self {
        self.members = Some(members);
        self
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }
}

/// 群聊适配器错误类型
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum GroupChatError {
    #[error("群聊不存在：{group_id}")]
    GroupNotFound { group_id: String },

    #[error("消息不存在：{message_id}")]
    MessageNotFound { message_id: String },

    #[error("连接失败：{message}")]
    ConnectionFailed { message: String },

    #[error("认证失败：{message}")]
    AuthenticationFailed { message: String },

    #[error("权限拒绝：{message}")]
    PermissionDenied { message: String },

    #[error("无效配置：{message}")]
    InvalidConfiguration { message: String },

    #[error("后端不可用：{backend}")]
    BackendNotAvailable { backend: String },

    #[error("操作超时：{message}")]
    Timeout { message: String },

    #[error("序列化错误：{message}")]
    SerializationError { message: String },

    #[error("内部错误：{message}")]
    InternalError { message: String },

    #[error("已是群成员：{group_id}")]
    AlreadyMember { group_id: String },

    #[error("不是群成员：{group_id}")]
    NotMember { group_id: String },

    #[error("群聊已满：{group_id}")]
    GroupFull { group_id: String },

    #[error("无效消息格式：{message}")]
    InvalidMessageFormat { message: String },

    #[error("未知错误：{message}")]
    Unknown { message: String },
}

impl GroupChatError {
    pub fn group_not_found(group_id: impl Into<String>) -> Self {
        GroupChatError::GroupNotFound { group_id: group_id.into() }
    }

    pub fn connection_failed(message: impl Into<String>) -> Self {
        GroupChatError::ConnectionFailed { message: message.into() }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        GroupChatError::InternalError { message: message.into() }
    }

    pub fn unknown(message: impl Into<String>) -> Self {
        GroupChatError::Unknown { message: message.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_chat_mode_display() {
        assert_eq!(GroupChatMode::Memory.to_string(), "Memory");
        assert_eq!(GroupChatMode::Pubsub.to_string(), "Pubsub");
        assert_eq!(GroupChatMode::Iroh.to_string(), "Iroh");
    }

    #[test]
    fn test_unified_group_creation() {
        let group = UnifiedGroup::new("Test Group", GroupChatMode::Memory);
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.mode, GroupChatMode::Memory);
        assert!(group.is_active());
    }

    #[test]
    fn test_unified_group_members() {
        let mut group = UnifiedGroup::new("Test Group", GroupChatMode::Memory);
        let member = AgentInfo::new("user1", "User 1");
        group.add_member(member.clone());
        assert!(group.has_member("user1"));
        assert!(!group.has_member("user2"));
        group.remove_member("user1");
        assert!(!group.has_member("user1"));
    }

    #[test]
    fn test_unified_message_creation() {
        let sender = AgentInfo::new("user1", "User 1");
        let message = UnifiedMessage::new("group1", sender, "Hello");
        assert_eq!(message.group_id, "group1");
        assert_eq!(message.content, "Hello");
        assert_eq!(message.message_type, MessageType::Chat);
    }

    #[test]
    fn test_message_type_from_str() {
        assert_eq!(MessageType::from("system"), MessageType::System);
        assert_eq!(MessageType::from("join"), MessageType::Join);
        assert_eq!(MessageType::from("leave"), MessageType::Leave);
        assert_eq!(MessageType::from("unknown"), MessageType::Chat);
    }
}

impl UnifiedGroup {
    /// Check if the group is active (has at least one member)
    pub fn is_active(&self) -> bool {
        !self.members.is_empty()
    }
}
