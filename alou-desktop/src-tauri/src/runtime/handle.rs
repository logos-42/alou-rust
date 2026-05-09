//! Actor 句柄

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use crate::runtime::message::SessionMessage;

/// Actor 句柄
/// 
/// 用于向 SessionActor 发送消息
/// 
/// 注意：不实现 Clone，因为 JoinHandle 不能 Clone
/// 使用 Arc<ActorHandle> 来共享引用
pub struct ActorHandle {
    pub session_id: String,
    pub tx: mpsc::Sender<SessionMessage>,
    /// JoinHandle 用于跟踪 actor 状态，但不用于 Clone
    handle: Option<JoinHandle<()>>,
}

impl ActorHandle {
    pub fn new(session_id: String, tx: mpsc::Sender<SessionMessage>, handle: JoinHandle<()>) -> Self {
        Self {
            session_id,
            tx,
            handle: Some(handle),
        }
    }

    /// 发送消息到 actor（异步）
    pub async fn send(&self, msg: SessionMessage) -> Result<(), mpsc::error::SendError<SessionMessage>> {
        self.tx.send(msg).await
    }

    /// 发送消息到 actor（同步版本）
    pub fn send_blocking(&self, msg: SessionMessage) -> Result<(), mpsc::error::SendError<SessionMessage>> {
        self.tx.blocking_send(msg)
    }

    /// 检查 actor 是否存活
    pub fn is_alive(&self) -> bool {
        if let Some(handle) = &self.handle {
            !handle.is_finished()
        } else {
            false
        }
    }
}

// 为 Arc<ActorHandle> 实现 Clone
impl Clone for ActorHandle {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            tx: self.tx.clone(),
            handle: None,  // Clone 不包含 handle，避免 ownership 问题
        }
    }
}

use std::sync::Arc;

/// ArcActorHandle - 用于共享的 ActorHandle
pub type ArcActorHandle = Arc<ActorHandle>;

impl ActorHandle {
    /// 创建 Arc 包装的 ActorHandle
    pub fn into_arc(self) -> ArcActorHandle {
        Arc::new(self)
    }
}
