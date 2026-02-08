//! 流式响应支持
//!
//! 提供 Agent 执行的流式响应，支持打字机效果和实时进度更新

use super::task::{TaskEvent, TaskManager};
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

/// 流式执行器
pub struct StreamingExecutor {
    task_manager: Arc<TaskManager>,
}

impl StreamingExecutor {
    pub fn new(task_manager: Arc<TaskManager>) -> Self {
        Self { task_manager }
    }

    /// 执行并返回流
    pub async fn execute_stream(
        &self,
        task_id: String,
    ) -> impl Stream<Item = StreamEvent> + Send {
        let (tx, rx) = mpsc::channel(100);
        let manager = self.task_manager.clone();

        tokio::spawn(async move {
            // 订阅任务事件
            let mut event_rx = manager.subscribe_events();

            // 过滤并转发事件到流
            while let Ok(event) = event_rx.recv().await {
                if event.task_matches(&task_id) {
                    let stream_event = convert_to_stream_event(event);
                    if tx.send(stream_event).await.is_err() {
                        break;
                    }
                }
            }
        });

        // 将 mpsc 接收器转换为 Stream
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}

/// 流式事件
#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamEvent {
    pub r#type: String,
    pub content: Option<String>,
    pub name: Option<String>,
    pub success: Option<bool>,
    pub progress: Option<f32>,
    pub message: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
}

/// 扩展 TaskEvent 以支持匹配
trait TaskEventExt {
    fn task_matches(&self, task_id: &str) -> bool;
}

impl TaskEventExt for TaskEvent {
    fn task_matches(&self, task_id: &str) -> bool {
        match self {
            TaskEvent::TaskCreated { task_id: id } => id == task_id,
            TaskEvent::TaskStarted { task_id: id } => id == task_id,
            TaskEvent::AiResponse { task_id: id, .. } => id == task_id,
            TaskEvent::ToolCallsPending { task_id: id, .. } => id == task_id,
            TaskEvent::ToolExecuting { task_id: id, .. } => id == task_id,
            TaskEvent::ToolCompleted { task_id: id, .. } => id == task_id,
            TaskEvent::TaskProgress { task_id: id, .. } => id == task_id,
            TaskEvent::TaskCompleted { task_id: id, .. } => id == task_id,
            TaskEvent::TaskFailed { task_id: id, .. } => id == task_id,
        }
    }
}

/// 转换为流式事件
fn convert_to_stream_event(event: TaskEvent) -> StreamEvent {
    match event {
        TaskEvent::TaskCreated { task_id } => StreamEvent {
            r#type: "task_created".to_string(),
            content: None,
            name: None,
            success: None,
            progress: Some(0.0),
            message: Some("任务已创建".to_string()),
            result: None,
            error: None,
        },
        TaskEvent::TaskStarted { task_id } => StreamEvent {
            r#type: "task_started".to_string(),
            content: None,
            name: None,
            success: None,
            progress: Some(0.1),
            message: Some("任务开始执行".to_string()),
            result: None,
            error: None,
        },
        TaskEvent::AiResponse {
            task_id,
            content,
        } => StreamEvent {
            r#type: "ai_response".to_string(),
            content: Some(content),
            name: None,
            success: None,
            progress: None,
            message: None,
            result: None,
            error: None,
        },
        TaskEvent::ToolCallsPending { task_id, count } => StreamEvent {
            r#type: "tools_pending".to_string(),
            content: None,
            name: None,
            success: None,
            progress: Some(0.5),
            message: Some(format!("待执行 {} 个工具", count)),
            result: None,
            error: None,
        },
        TaskEvent::ToolExecuting { task_id, tool_name } => StreamEvent {
            r#type: "tool_executing".to_string(),
            content: None,
            name: Some(tool_name.clone()),
            success: None,
            progress: None,
            message: Some(format!("正在执行工具: {}", tool_name)),
            result: None,
            error: None,
        },
        TaskEvent::ToolCompleted {
            task_id,
            tool_name,
            result,
        } => StreamEvent {
            r#type: "tool_completed".to_string(),
            content: result.data_preview,
            name: Some(tool_name),
            success: Some(result.success),
            progress: None,
            message: Some(if result.success {
                "工具执行成功".to_string()
            } else {
                result.error.clone().unwrap_or_else(|| "工具执行失败".to_string())
            }),
            result: None,
            error: if result.success { None } else { result.error.clone() },
        },
        TaskEvent::TaskProgress {
            task_id,
            progress,
            message,
        } => StreamEvent {
            r#type: "progress".to_string(),
            content: None,
            name: None,
            success: None,
            progress: Some(progress),
            message: Some(message),
            result: None,
            error: None,
        },
        TaskEvent::TaskCompleted { task_id, result } => StreamEvent {
            r#type: "completed".to_string(),
            content: Some(result.clone()),
            name: None,
            success: Some(true),
            progress: Some(1.0),
            message: Some("任务完成".to_string()),
            result: Some(result),
            error: None,
        },
        TaskEvent::TaskFailed { task_id, error } => StreamEvent {
            r#type: "failed".to_string(),
            content: None,
            name: None,
            success: Some(false),
            progress: None,
            message: Some(error.clone()),
            result: None,
            error: Some(error),
        },
    }
}
