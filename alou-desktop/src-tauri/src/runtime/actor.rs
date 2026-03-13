//! Session Actor
//! 
//! 每个 Session 对应一个独立的 Actor（Tokio Task），处理该 Session 的所有消息。

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::runtime::session::SessionRuntime;
use crate::runtime::message::SessionMessage;
use crate::runtime::handle::ActorHandle;
use crate::bridges::BridgeManager;

/// Session Actor
/// 
/// 每个 Session 对应一个独立的 Actor，负责处理该 Session 的所有消息。
/// 消息按顺序处理，避免并发问题。
pub struct SessionActor {
    session_id: String,
    runtime: SessionRuntime,
    message_rx: mpsc::Receiver<SessionMessage>,
    bridge_manager: Arc<BridgeManager>,
}

impl SessionActor {
    /// 创建并启动 Session Actor
    pub fn spawn(
        session_id: String,
        bridge_manager: Arc<BridgeManager>,
    ) -> ActorHandle {
        let (tx, rx) = mpsc::channel(100);

        let actor = Self {
            session_id: session_id.clone(),
            runtime: SessionRuntime::new(session_id.clone()),
            message_rx: rx,
            bridge_manager,
        };

        let handle = tokio::spawn(actor.run());

        ActorHandle::new(session_id, tx, handle)
    }

    /// Actor 主循环
    pub async fn run(mut self) {
        let session_id = self.session_id.clone();

        // 发送 SessionCreated 消息
        self.runtime = Self::handle_message(&session_id, SessionMessage::SessionCreated, self.runtime).await;

        while let Some(msg) = self.message_rx.recv().await {
            self.runtime = Self::handle_message(&session_id, msg, self.runtime).await;
        }

        // 通道关闭，发送 SessionDestroy 消息
        Self::handle_message(&session_id, SessionMessage::SessionDestroy, self.runtime).await;

        log::info!("[SessionActor] Session {} actor shutdown", session_id);
    }

    /// 处理消息
    async fn handle_message(
        session_id: &str,
        msg: SessionMessage,
        runtime: SessionRuntime,
    ) -> SessionRuntime {
        match msg {
            SessionMessage::UserMessage { content, .. } => {
                log::info!(
                    "[SessionActor:{}] received user message: {}",
                    session_id,
                    content.chars().take(50).collect::<String>()
                );
                // 记录到 agent 状态
                let mut runtime = runtime;
                runtime.agent_state.add_message("user", content);
                return runtime;
            }

            SessionMessage::ToolResult { tool_call_id, result } => {
                log::info!(
                    "[SessionActor:{}] received tool result: {} = {:?}",
                    session_id,
                    tool_call_id,
                    result.success
                );
            }

            SessionMessage::WorkflowEvent { workflow_id, event_type, data } => {
                log::info!(
                    "[SessionActor:{}] workflow event: {} - {}",
                    session_id,
                    workflow_id,
                    event_type
                );
                log::debug!("[SessionActor:{}] workflow data: {:?}", session_id, data);
            }

            SessionMessage::SystemEvent { event_type, .. } => {
                log::info!(
                    "[SessionActor:{}] system event: {:?}",
                    session_id,
                    event_type
                );
            }

            SessionMessage::GroupChatMessage { group_id, from_agent, content } => {
                log::info!(
                    "[SessionActor:{}] group chat message from {:?}: {}",
                    session_id,
                    from_agent,
                    content
                );
            }

            SessionMessage::GroupChatEvent { group_id, event_type } => {
                log::info!(
                    "[SessionActor:{}] group chat event: {} - {}",
                    session_id,
                    group_id,
                    event_type
                );
            }

            SessionMessage::Command { command } => {
                log::info!(
                    "[SessionActor:{}] received command: {:?}",
                    session_id,
                    command
                );
            }

            SessionMessage::SessionCreated => {
                log::info!("[SessionActor:{}] session created", session_id);
            }

            SessionMessage::SessionDestroy => {
                log::info!("[SessionActor:{}] session destroy", session_id);
            }

            SessionMessage::Ping => {
                log::debug!("[SessionActor:{}] ping", session_id);
            }

            SessionMessage::Pong => {
                log::debug!("[SessionActor:{}] pong", session_id);
            }

            // 新增消息类型（简化处理）
            SessionMessage::ToolCall { tool_call_id, tool_name, arguments } => {
                log::info!(
                    "[SessionActor:{}] tool call: {} - {} {:?}",
                    session_id,
                    tool_call_id,
                    tool_name,
                    arguments
                );
            }

            SessionMessage::WorkflowStep { workflow_id, step_id, action } => {
                log::info!(
                    "[SessionActor:{}] workflow step: {} - {} {:?}",
                    session_id,
                    workflow_id,
                    step_id,
                    action
                );
            }

            SessionMessage::WorkflowExecute { workflow_id, input } => {
                log::info!(
                    "[SessionActor:{}] workflow execute: {} {:?}",
                    session_id,
                    workflow_id,
                    input
                );
            }
        }

        runtime
    }
}
