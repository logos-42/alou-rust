//! Session Actor
//!
//! 每个 Session 对应一个独立的 Actor（Tokio Task），处理该 Session 的所有消息。
//! 
//! 并发模型：
//! - 消息接收是串行的（通过 channel）
//! - 任务执行是并行的（每个任务启动独立的 tokio::spawn）
//! - 通过 TaskManager 的事件订阅跟踪任务进度

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::runtime::session::SessionRuntime;
use crate::runtime::message::SessionMessage;
use crate::runtime::handle::ActorHandle;
use crate::bridges::BridgeManager;
// Agent 模块
use crate::agent::executor::RalphLoopExecutor;
use crate::agent::task::TaskManager;
use crate::agent::ai_client::AiClient;
use crate::tools::ToolRegistry;

/// Session Actor
///
/// 每个 Session 对应一个独立的 Actor，负责处理该 Session 的所有消息。
/// 消息接收是串行的，但任务执行是并行的（每个任务启动独立 task）。
pub struct SessionActor {
    session_id: String,
    runtime: SessionRuntime,
    message_rx: mpsc::Receiver<SessionMessage>,
    bridge_manager: Arc<BridgeManager>,
    // Agent 执行器
    executor: Arc<RalphLoopExecutor>,
    task_manager: Arc<TaskManager>,
    // AppHandle 用于发送事件到前端
    app_handle: Option<tauri::AppHandle>,
}

impl SessionActor {
    /// 创建并启动 Session Actor
    pub fn spawn(
        session_id: String,
        bridge_manager: Arc<BridgeManager>,
        ai_client: Arc<AiClient>,
        tool_registry: Arc<ToolRegistry>,
    ) -> ActorHandle {
        let (tx, rx) = mpsc::channel(100);

        // 创建任务管理器和执行器
        let task_manager = Arc::new(TaskManager::new());
        let executor = Arc::new(RalphLoopExecutor::new(
            ai_client,
            task_manager.clone(),
            bridge_manager.tool_bridge(),
            tool_registry,
        ));

        let actor = Self {
            session_id: session_id.clone(),
            runtime: SessionRuntime::new(session_id.clone()),
            message_rx: rx,
            bridge_manager,
            executor,
            task_manager,
            app_handle: None, // 初始为 None，可以通过消息设置
        };

        let handle = tokio::spawn(actor.run());

        ActorHandle::new(session_id, tx, handle)
    }

    /// 设置 AppHandle（用于发送事件到前端）
    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    /// Actor 主循环
    pub async fn run(mut self) {
        let session_id = self.session_id.clone();
        let executor = self.executor.clone();
        let task_manager = self.task_manager.clone();
        let app_handle = self.app_handle.clone();

        // 发送 SessionCreated 消息
        self.runtime = Self::handle_message(&session_id, SessionMessage::SessionCreated, self.runtime, executor.clone(), task_manager.clone(), app_handle.clone()).await;

        while let Some(msg) = self.message_rx.recv().await {
            self.runtime = Self::handle_message(&session_id, msg, self.runtime, executor.clone(), task_manager.clone(), app_handle.clone()).await;
        }

        // 通道关闭，发送 SessionDestroy 消息
        Self::handle_message(&session_id, SessionMessage::SessionDestroy, self.runtime, executor, task_manager, app_handle).await;

        log::info!("[SessionActor] Session {} actor shutdown", session_id);
    }

    /// 处理消息
    async fn handle_message(
        session_id: &str,
        msg: SessionMessage,
        runtime: SessionRuntime,
        executor: Arc<RalphLoopExecutor>,
        task_manager: Arc<TaskManager>,
        app_handle: Option<tauri::AppHandle>,
    ) -> SessionRuntime {
        match msg {
            SessionMessage::UserMessage { content, metadata } => {
                log::info!(
                    "[SessionActor:{}] received user message: {}",
                    session_id,
                    content.chars().take(50).collect::<String>()
                );

                // 🔥 异步并发执行：启动独立 task，不阻塞消息队列
                // 注意：这里不执行任务，只记录状态，实际执行由命令完成
                let session_id = session_id.to_string();
                let task_manager = task_manager.clone();
                let app_handle = app_handle.clone();
                let content_clone = content.clone();
                let metadata_clone = metadata.clone();

                tokio::spawn(async move {
                    // 创建任务（仅用于状态跟踪）
                    let task_id = task_manager.create_task(session_id.clone(), content_clone.clone()).await;
                    log::info!("[SessionActor:{}] Task created for tracking: {}", session_id, task_id);

                    // 如果提供了 AppHandle，发送任务创建事件到前端
                    if let Some(app) = app_handle {
                        let _ = app.emit("session:task_created", serde_json::json!({
                            "task_id": task_id,
                            "session_id": session_id,
                            "content": content_clone.chars().take(200).to_string(),
                            "stream": metadata_clone.stream.unwrap_or(false),
                            "timestamp": metadata_clone.timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp()),
                        }));
                    }
                });

                // 立即返回 runtime，不阻塞
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

            SessionMessage::SystemEvent { event_type, data } => {
                log::info!(
                    "[SessionActor:{}] system event: {:?}",
                    session_id,
                    event_type
                );
                log::debug!("[SessionActor:{}] system data: {:?}", session_id, data);
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
