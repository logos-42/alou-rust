//! Message Bus - 不丢消息的事件总线

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "agent_message")]
    AgentMessage(AgentMessage),
    #[serde(rename = "group_message")]
    GroupMessage(GroupMessage),
    #[serde(rename = "task_message")]
    TaskMessage(TaskMessage),
    #[serde(rename = "system")]
    System(SystemEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub agent_id: String,
    pub group_id: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub id: String,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    pub id: String,
    pub task_id: String,
    pub agent_id: Option<String>,
    pub action: String,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_type: String,
    pub data: serde_json::Value,
}

type SubscriberTx = mpsc::Sender<Event>;

pub struct MessageBus {
    subscribers: Arc<RwLock<Vec<SubscriberTx>>>,
    capacity: usize,
}

impl MessageBus {
    pub fn new(capacity: usize) -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            capacity,
        }
    }
    
    pub fn with_default_capacity() -> Self {
        Self::new(1000)
    }
    
    pub async fn subscribe(&self) -> mpsc::Receiver<Event> {
        let (tx, rx) = mpsc::channel(self.capacity);
        self.subscribers.write().await.push(tx);
        rx
    }
    
    pub async fn publish(&self, event: Event) {
        let subscribers = self.subscribers.read().await;
        let mut failed_indices = Vec::new();
        
        for (idx, tx) in subscribers.iter().enumerate() {
            if tx.send(event.clone()).await.is_err() {
                failed_indices.push(idx);
            }
        }
        
        drop(subscribers);
        
        if !failed_indices.is_empty() {
            let mut subs = self.subscribers.write().await;
            for idx in failed_indices.into_iter().rev() {
                subs.remove(idx);
            }
        }
    }
    
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

impl Clone for MessageBus {
    fn clone(&self) -> Self {
        Self {
            subscribers: Arc::clone(&self.subscribers),
            capacity: self.capacity,
        }
    }
}
