//! Group Chat Manager - 群聊管理器
//!
//! 管理所有 Session 的群聊订阅和消息广播

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// 群聊消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub group_id: String,
    pub from_agent: Option<String>,
    pub from_session: Option<String>,
    pub content: String,
    pub timestamp: i64,
}

impl GroupMessage {
    pub fn new(group_id: String, from_session: Option<String>, content: String) -> Self {
        Self {
            group_id,
            from_agent: None,
            from_session,
            content,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn from_agent(group_id: String, from_agent: String, content: String) -> Self {
        Self {
            group_id,
            from_agent: Some(from_agent),
            from_session: None,
            content,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// 群聊订阅信息
#[derive(Debug, Clone)]
pub struct GroupSubscription {
    pub session_id: String,
    pub subscribed_at: i64,
    pub auto_reply: bool,
}

/// 群聊房间
pub struct GroupRoom {
    pub group_id: String,
    pub members: Vec<GroupSubscription>,
    pub message_history: Vec<GroupMessage>,
    pub max_history: usize,
}

impl GroupRoom {
    pub fn new(group_id: String, max_history: usize) -> Self {
        Self {
            group_id,
            members: Vec::new(),
            message_history: Vec::new(),
            max_history,
        }
    }

    pub fn add_member(&mut self, subscription: GroupSubscription) {
        // 检查是否已存在
        if !self.members.iter().any(|m| m.session_id == subscription.session_id) {
            self.members.push(subscription);
        }
    }

    pub fn remove_member(&mut self, session_id: &str) {
        self.members.retain(|m| m.session_id != session_id);
    }

    pub fn add_message(&mut self, message: GroupMessage) {
        self.message_history.push(message);
        // 限制历史记录大小
        if self.message_history.len() > self.max_history {
            self.message_history.remove(0);
        }
    }

    pub fn get_history(&self, limit: usize) -> Vec<GroupMessage> {
        self.message_history
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

/// 群聊管理器
pub struct GroupChatManager {
    /// 群聊房间映射：group_id -> GroupRoom
    rooms: DashMap<String, GroupRoom>,
    /// Session 订阅映射：session_id -> [group_id]
    session_subscriptions: DashMap<String, Vec<String>>,
    /// 配置
    config: GroupChatConfig,
}

/// 群聊配置
#[derive(Debug, Clone)]
pub struct GroupChatConfig {
    /// 最大历史记录数量
    pub max_history_size: usize,
    /// 最大群成员数量
    pub max_members_per_group: usize,
}

impl Default for GroupChatConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100,
            max_members_per_group: 50,
        }
    }
}

impl GroupChatManager {
    /// 创建新的群聊管理器
    pub fn new() -> Self {
        Self::with_config(GroupChatConfig::default())
    }

    /// 创建带配置的群聊管理器
    pub fn with_config(config: GroupChatConfig) -> Self {
        Self {
            rooms: DashMap::new(),
            session_subscriptions: DashMap::new(),
            config,
        }
    }

    /// 订阅群聊
    pub async fn subscribe(&self, group_id: String, session_id: String, auto_reply: bool) {
        log::info!("Session {} 订阅群聊 {}", session_id, group_id);

        // 更新 session 订阅列表
        self.session_subscriptions
            .entry(session_id.clone())
            .or_insert_with(Vec::new)
            .push(group_id.clone());

        // 创建或获取群聊房间
        let mut room = self.rooms
            .entry(group_id.clone())
            .or_insert_with(|| GroupRoom::new(group_id, self.config.max_history_size));

        // 添加成员
        room.add_member(GroupSubscription {
            session_id,
            subscribed_at: chrono::Utc::now().timestamp(),
            auto_reply,
        });

        log::info!("群聊 {} 当前成员数：{}", group_id, room.members.len());
    }

    /// 取消订阅群聊
    pub async fn unsubscribe(&self, group_id: String, session_id: String) {
        log::info!("Session {} 取消订阅群聊 {}", session_id, group_id);

        // 更新 session 订阅列表
        if let Some(mut groups) = self.session_subscriptions.get_mut(&session_id) {
            groups.retain(|g| g != &group_id);
        }

        // 从群聊房间移除成员
        if let Some(mut room) = self.rooms.get_mut(&group_id) {
            room.remove_member(&session_id);
            log::info!("群聊 {} 剩余成员数：{}", group_id, room.members.len());

            // 如果房间为空，可以选择删除
            if room.members.is_empty() {
                log::info!("群聊 {} 已无成员，清理房间", group_id);
            }
        }
    }

    /// 广播消息到群聊
    pub async fn broadcast(&self, message: GroupMessage) {
        let group_id = message.group_id.clone();
        log::info!("广播消息到群聊 {}: {}", group_id, message.content.chars().take(50).collect::<String>());

        // 添加到房间历史
        if let Some(mut room) = self.rooms.get_mut(&group_id) {
            room.add_message(message.clone());
        }

        // 注意：实际的消息推送需要通过 SessionRouter 发送到各个 Session
        // 这里只负责存储和路由信息
    }

    /// 获取群聊历史
    pub async fn get_history(&self, group_id: &str, limit: usize) -> Vec<GroupMessage> {
        if let Some(room) = self.rooms.get(group_id) {
            room.get_history(limit)
        } else {
            Vec::new()
        }
    }

    /// 获取 Session 订阅的所有群聊
    pub async fn get_subscriptions(&self, session_id: &str) -> Vec<String> {
        self.session_subscriptions
            .get(session_id)
            .map(|groups| groups.clone())
            .unwrap_or_default()
    }

    /// 获取群聊成员列表
    pub async fn get_members(&self, group_id: &str) -> Vec<String> {
        if let Some(room) = self.rooms.get(group_id) {
            room.members.iter().map(|m| m.session_id.clone()).collect()
        } else {
            Vec::new()
        }
    }

    /// 获取群聊信息
    pub async fn get_group_info(&self, group_id: &str) -> Option<GroupInfo> {
        self.rooms.get(group_id).map(|room| {
            GroupInfo {
                group_id: room.group_id.clone(),
                member_count: room.members.len(),
                message_count: room.message_history.len(),
            }
        })
    }

    /// 列出所有活跃群聊
    pub async fn list_groups(&self) -> Vec<GroupInfo> {
        self.rooms
            .iter()
            .map(|entry| {
                GroupInfo {
                    group_id: entry.group_id.clone(),
                    member_count: entry.members.len(),
                    message_count: entry.message_history.len(),
                }
            })
            .collect()
    }

    /// 清理空房间
    pub async fn cleanup_empty_rooms(&self) -> usize {
        let to_remove: Vec<String> = self.rooms
            .iter()
            .filter(|entry| entry.members.is_empty())
            .map(|entry| entry.key().clone())
            .collect();

        let count = to_remove.len();
        for group_id in to_remove {
            self.rooms.remove(&group_id);
        }

        if count > 0 {
            log::info!("清理了 {} 个空群聊房间", count);
        }

        count
    }
}

impl Default for GroupChatManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 群聊信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub member_count: usize,
    pub message_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let manager = GroupChatManager::new();

        // 订阅
        manager.subscribe("group1".to_string(), "session1".to_string(), true).await;
        manager.subscribe("group1".to_string(), "session2".to_string(), false).await;

        // 验证订阅
        let subs = manager.get_subscriptions("session1").await;
        assert_eq!(subs, vec!["group1"]);

        let members = manager.get_members("group1").await;
        assert_eq!(members.len(), 2);

        // 取消订阅
        manager.unsubscribe("group1".to_string(), "session1".to_string()).await;

        // 验证取消订阅
        let subs = manager.get_subscriptions("session1").await;
        assert!(subs.is_empty());

        let members = manager.get_members("group1").await;
        assert_eq!(members.len(), 1);
    }

    #[tokio::test]
    async fn test_broadcast_history() {
        let manager = GroupChatManager::new();

        // 订阅
        manager.subscribe("group1".to_string(), "session1".to_string(), true).await;

        // 广播消息
        let msg = GroupMessage::new(
            "group1".to_string(),
            Some("session1".to_string()),
            "Hello, group!".to_string(),
        );
        manager.broadcast(msg).await;

        // 获取历史
        let history = manager.get_history("group1", 10).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "Hello, group!");
    }
}
