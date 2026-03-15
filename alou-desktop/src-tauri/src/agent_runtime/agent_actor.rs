//! Agent Actor - Agent Actor（顺序处理）

use crate::agent_runtime::agent_registry::{AgentInfo, AgentConfig};
use crate::agent_runtime::message_bus::GroupMessage;
use tokio::sync::mpsc;

/// Agent Actor 消息
pub enum ActorMessage {
    /// 处理群聊消息
    HandleMessage {
        message: GroupMessage,
        history: Vec<GroupMessage>,
    },
    /// 停止 actor
    Stop,
}

/// Agent Actor 句柄（用于发送消息到 actor）
pub struct AgentHandle {
    tx: mpsc::Sender<ActorMessage>,
}

impl AgentHandle {
    pub fn new(tx: mpsc::Sender<ActorMessage>) -> Self {
        Self { tx }
    }
    
    pub async fn send_message(&self, message: GroupMessage, history: Vec<GroupMessage>) -> Result<(), String> {
        self.tx.send(ActorMessage::HandleMessage { message, history }).await
            .map_err(|e| format!("发送消息失败：{}", e))
    }
    
    pub async fn stop(&self) -> Result<(), String> {
        self.tx.send(ActorMessage::Stop).await
            .map_err(|e| format!("发送停止消息失败：{}", e))
    }
}

/// Agent Actor
pub struct AgentActor {
    pub agent: AgentInfo,
    pub config: AgentConfig,
    pub inbox: mpsc::Receiver<ActorMessage>,
}

impl AgentActor {
    pub fn new(agent: AgentInfo, config: AgentConfig, inbox: mpsc::Receiver<ActorMessage>) -> Self {
        Self { agent, config, inbox }
    }
    
    /// 运行 actor（顺序处理所有消息）
    pub async fn run(mut self) -> Result<(), String> {
        log::info!("Agent Actor 启动：{}", self.agent.name);
        
        while let Some(msg) = self.inbox.recv().await {
            match msg {
                ActorMessage::HandleMessage { message, history } => {
                    if let Err(e) = self.handle_message(message, history).await {
                        log::error!("Agent 处理消息失败：{}", e);
                    }
                }
                ActorMessage::Stop => {
                    log::info!("Agent Actor 停止：{}", self.agent.name);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// 处理单条消息
    async fn handle_message(&self, message: GroupMessage, history: Vec<GroupMessage>) -> Result<(), String> {
        if self.config.mention_only && !self.is_mentioned(&message) {
            return Ok(());
        }
        
        let limited_history = history
            .into_iter()
            .rev()
            .take(self.config.max_context_messages)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        
        log::debug!(
            "Agent {} 处理消息：{} (history: {})",
            self.agent.name,
            message.content.chars().take(50).collect::<String>(),
            limited_history.len()
        );
        
        Ok(())
    }
    
    /// 检查是否被提及
    fn is_mentioned(&self, message: &GroupMessage) -> bool {
        message.content.contains(&format!("@{}", self.agent.name))
            || message.content.contains(&format!("@{}", self.agent.display_name))
    }
}

impl Clone for AgentActor {
    fn clone(&self) -> Self {
        Self {
            agent: self.agent.clone(),
            config: self.config.clone(),
            inbox: mpsc::channel(100).1,
        }
    }
}
