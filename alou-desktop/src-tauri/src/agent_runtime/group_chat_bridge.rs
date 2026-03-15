//! GroupChat Bridge - 群聊桥接器
//!
//! 桥接外部群聊（PubSub/Iroh）到内部 MessageBus

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::agent_runtime::message_bus::{MessageBus, Event, GroupMessage};
use crate::agent_runtime::agent_registry::GroupChatMode;

/// 群聊订阅句柄
pub struct GroupSubscription {
    pub group_id: String,
    pub mode: GroupChatMode,
    pub unsubscribe: Option<Box<dyn FnOnce() + Send + Sync>>,
}

/// 群聊桥接器
pub struct GroupChatBridge {
    message_bus: MessageBus,
    subscriptions: RwLock<Vec<GroupSubscription>>,
}

impl GroupChatBridge {
    pub fn new(message_bus: MessageBus) -> Self {
        Self {
            message_bus,
            subscriptions: RwLock::new(Vec::new()),
        }
    }

    /// 订阅群聊并桥接到 MessageBus
    pub async fn subscribe(
        &self,
        group_id: String,
        mode: GroupChatMode,
    ) -> Result<(), String> {
        log::info!("订阅群聊：{} (mode: {:?})", group_id, mode);

        // 根据模式订阅不同的群聊
        match mode {
            GroupChatMode::PubSub => {
                self.subscribe_pubsub(&group_id).await?;
            }
            GroupChatMode::Iroh => {
                self.subscribe_iroh(&group_id).await?;
            }
            GroupChatMode::Memory => {
                self.subscribe_memory(&group_id).await?;
            }
        }

        Ok(())
    }

    /// 订阅 PubSub 群聊
    async fn subscribe_pubsub(&self, group_id: &str) -> Result<(), String> {
        // TODO: 使用现有的 pubsub_tool 订阅
        // 这里需要调用实际的 PubSub 订阅逻辑
        log::info!("PubSub 订阅：{}", group_id);
        
        // 模拟订阅成功
        let subscription = GroupSubscription {
            group_id: group_id.to_string(),
            mode: GroupChatMode::PubSub,
            unsubscribe: None,
        };
        
        self.subscriptions.write().await.push(subscription);
        Ok(())
    }

    /// 订阅 Iroh 群聊
    async fn subscribe_iroh(&self, group_id: &str) -> Result<(), String> {
        // TODO: 使用现有的 iroh_tool 订阅
        log::info!("Iroh 订阅：{}", group_id);
        
        let subscription = GroupSubscription {
            group_id: group_id.to_string(),
            mode: GroupChatMode::Iroh,
            unsubscribe: None,
        };
        
        self.subscriptions.write().await.push(subscription);
        Ok(())
    }

    /// 订阅 Memory 群聊
    async fn subscribe_memory(&self, group_id: &str) -> Result<(), String> {
        log::info!("Memory 订阅：{}", group_id);
        
        let subscription = GroupSubscription {
            group_id: group_id.to_string(),
            mode: GroupChatMode::Memory,
            unsubscribe: None,
        };
        
        self.subscriptions.write().await.push(subscription);
        Ok(())
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, group_id: &str) -> Result<(), String> {
        let mut subscriptions = self.subscriptions.write().await;
        
        if let Some(pos) = subscriptions.iter().position(|s| s.group_id == group_id) {
            let subscription = subscriptions.remove(pos);
            log::info!("取消订阅群聊：{}", group_id);
            
            if let Some(unsubscribe) = subscription.unsubscribe {
                unsubscribe();
            }
        }
        
        Ok(())
    }

    /// 发布消息到群聊
    pub async fn publish(
        &self,
        group_id: &str,
        message: GroupMessage,
    ) -> Result<(), String> {
        log::info!("发布消息到群聊：{} - {}", group_id, message.content.chars().take(50).collect::<String>());

        // 发布到内部 MessageBus
        self.message_bus.publish(Event::GroupMessage(message.clone())).await;

        // 根据模式发布到外部群聊
        let subscriptions = self.subscriptions.read().await;
        let subscription = subscriptions.iter().find(|s| s.group_id == group_id);
        
        if let Some(sub) = subscription {
            match sub.mode {
                GroupChatMode::PubSub => {
                    self.publish_pubsub(group_id, &message).await?;
                }
                GroupChatMode::Iroh => {
                    self.publish_iroh(group_id, &message).await?;
                }
                GroupChatMode::Memory => {
                    self.publish_memory(group_id, &message).await?;
                }
            }
        }

        Ok(())
    }

    /// 发布到 PubSub
    async fn publish_pubsub(&self, group_id: &str, message: &GroupMessage) -> Result<(), String> {
        // TODO: 使用现有的 pubsub_tool 发布
        log::info!("PubSub 发布：{} - {}", group_id, message.content);
        Ok(())
    }

    /// 发布到 Iroh
    async fn publish_iroh(&self, group_id: &str, message: &GroupMessage) -> Result<(), String> {
        // TODO: 使用现有的 iroh_tool 发布
        log::info!("Iroh 发布：{} - {}", group_id, message.content);
        Ok(())
    }

    /// 发布到 Memory
    async fn publish_memory(&self, group_id: &str, message: &GroupMessage) -> Result<(), String> {
        // Memory 模式只需要发布到 MessageBus
        log::info!("Memory 发布：{} - {}", group_id, message.content);
        Ok(())
    }

    /// 获取所有订阅的群聊
    pub async fn list_subscriptions(&self) -> Vec<GroupSubscriptionInfo> {
        self.subscriptions.read().await.iter().map(|s| {
            GroupSubscriptionInfo {
                group_id: s.group_id.clone(),
                mode: format!("{:?}", s.mode),
            }
        }).collect()
    }
}

/// 群聊订阅信息
#[derive(Debug, Clone, Serialize)]
pub struct GroupSubscriptionInfo {
    pub group_id: String,
    pub mode: String,
}

impl Clone for GroupChatBridge {
    fn clone(&self) -> Self {
        Self {
            message_bus: self.message_bus.clone(),
            subscriptions: self.subscriptions.clone(),
        }
    }
}
