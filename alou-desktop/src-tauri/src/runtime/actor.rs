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
use tauri::Emitter;
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

    /// Actor 主循环（带自主心跳）
    pub async fn run(mut self) {
        let session_id = self.session_id.clone();
        let executor = self.executor.clone();
        let task_manager = self.task_manager.clone();
        let app_handle = self.app_handle.clone();

        // 发送 SessionCreated 消息
        self.runtime = Self::handle_message(&session_id, SessionMessage::SessionCreated, self.runtime, executor.clone(), task_manager.clone(), app_handle.clone()).await;

        // 🔥 启动自主心跳循环（每 5 秒发送 AgentTick）
        let tick_session_id = session_id.clone();
        let tick_executor = executor.clone();
        let tick_task_manager = task_manager.clone();
        let tick_app_handle = app_handle.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                // 发送 AgentTick 消息（让 Agent 自主思考）
                let _ = Self::handle_message(
                    &tick_session_id,
                    SessionMessage::AgentTick { timestamp: chrono::Utc::now().timestamp() },
                    SessionRuntime::new(tick_session_id.clone()),
                    tick_executor.clone(),
                    tick_task_manager.clone(),
                    tick_app_handle.clone(),
                ).await;
            }
        });

        // 主消息循环
        while let Some(msg) = self.message_rx.recv().await {
            // 如果是 AgentTick，跳过（心跳循环已处理）
            if matches!(msg, SessionMessage::AgentTick { .. }) {
                continue;
            }
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
                let session_id = session_id.to_string();
                let task_manager = task_manager.clone();
                let executor = executor.clone();
                let app_handle = app_handle.clone();
                let content_clone = content.clone();
                let metadata_clone = metadata.clone();

                tokio::spawn(async move {
                    // 创建任务（用于状态跟踪）
                    let task_id = task_manager.create_task(session_id.clone(), content_clone.clone()).await;
                    log::info!("[SessionActor:{}] Task created for tracking: {}", session_id, task_id);

                    // 如果提供了 AppHandle，发送任务创建事件到前端
                    if let Some(app) = &app_handle {
                        let _ = app.emit("session:task_created", serde_json::json!({
                            "task_id": task_id,
                            "session_id": session_id,
                            "content": content_clone.chars().take(200).to_string(),
                            "stream": metadata_clone.stream.unwrap_or(false),
                            "timestamp": metadata_clone.timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp()),
                        }));
                    }

                    // 🔥 关键优化：立即启动 executor.execute，而不是等待 AgentTick
                    // 这样 Agent 可以立即响应用户消息，而不是被动等待心跳
                    log::info!("[SessionActor:{}] Starting task execution: {}", session_id, task_id);
                    if let Err(e) = executor.execute(&task_id).await {
                        log::error!("[SessionActor:{}] Task {} execution failed: {}", session_id, task_id, e);
                    } else {
                        log::info!("[SessionActor:{}] Task {} completed successfully", session_id, task_id);
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

            SessionMessage::AgentTick { timestamp } => {
                // 🔥 自主心跳：让 Agent 定期自主思考
                log::info!("[SessionActor:{}] AgentTick received at {}", session_id, timestamp);

                // 检查是否有待处理的任务或需要自主执行的操作
                let pending_tasks = task_manager.get_pending_tasks().await;
                if !pending_tasks.is_empty() {
                    log::info!(
                        "[SessionActor:{}] Found {} pending tasks, continuing execution",
                        session_id,
                        pending_tasks.len()
                    );

                    // 继续执行待处理任务
                    for task in pending_tasks {
                        let task_id = task.task_id.clone();
                        let exec = executor.clone();
                        tokio::spawn(async move {
                            log::info!("[SessionActor:{}] Resuming task: {}", session_id, task_id);
                            if let Err(e) = exec.execute(&task_id).await {
                                log::error!("[SessionActor:{}] Task {} execution failed: {}", session_id, task_id, e);
                            }
                        });
                    }
                } else {
                    // 没有待处理任务，Agent 可以自主决定做什么
                    log::debug!("[SessionActor:{}] No pending tasks, agent is idle", session_id);

                    // 这里可以添加更多自主逻辑，例如：
                    // 1. 检查长期记忆是否需要更新
                    // 2. 检查是否有定时任务需要执行
                    // 3. 主动学习或总结之前的交互
                }
            }
        }

        runtime
    }
}
