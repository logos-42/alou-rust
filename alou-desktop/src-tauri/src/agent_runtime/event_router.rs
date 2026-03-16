//! Event Router - 事件路由器
//! 
//! 将 MessageBus 的消息路由到不同的处理队列

use crate::agent_runtime::message_bus::{Event, MessageBus};
use tokio::sync::mpsc;

/// 事件路由器
pub struct EventRouter {
    message_bus: MessageBus,
    agent_tx: mpsc::Sender<Event>,
    group_tx: mpsc::Sender<Event>,
    task_tx: mpsc::Sender<Event>,
}

impl EventRouter {
    pub async fn new(message_bus: MessageBus) -> Self {
        let (agent_tx, mut agent_rx) = mpsc::channel(1000);
        let (group_tx, mut group_rx) = mpsc::channel(1000);
        let (task_tx, mut task_rx) = mpsc::channel(1000);
        
        // 启动路由循环
        let mut rx = message_bus.subscribe().await;
        
        let agent_tx_clone = agent_tx.clone();
        let group_tx_clone = group_tx.clone();
        let task_tx_clone = task_tx.clone();
        
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await.ok() {
                match &event {
                    Event::AgentMessage(_) => {
                        let _ = agent_tx_clone.send(event).await;
                    }
                    Event::GroupMessage(_) => {
                        let _ = group_tx_clone.send(event).await;
                    }
                    Event::TaskMessage(_) => {
                        let _ = task_tx_clone.send(event).await;
                    }
                    Event::System(_) => {
                        // 系统事件广播到所有队列
                        let _ = agent_tx_clone.send(event.clone()).await;
                        let _ = group_tx_clone.send(event.clone()).await;
                        let _ = task_tx_clone.send(event.clone()).await;
                    }
                }
            }
        });
        
        Self {
            message_bus,
            agent_tx,
            group_tx,
            task_tx,
        }
    }
    
    pub fn get_message_bus(&self) -> &MessageBus {
        &self.message_bus
    }
    
    pub fn subscribe_agent(&self) -> mpsc::Receiver<Event> {
        self.agent_tx.subscribe()
    }
    
    pub fn subscribe_group(&self) -> mpsc::Receiver<Event> {
        self.group_tx.subscribe()
    }
    
    pub fn subscribe_task(&self) -> mpsc::Receiver<Event> {
        self.task_tx.subscribe()
    }
    
    pub async fn publish(&self, event: Event) {
        self.message_bus.publish(event).await;
    }
}
