//! 工作流事件桥接
//!
//! 将异步工作流执行器的事件与Tauri事件系统集成

use super::{AsyncWorkflowExecutor, ExecutionEvent};
use tauri::Emitter;

/// 工作流事件管理器
pub struct WorkflowEventManager {
    executor: AsyncWorkflowExecutor,
}

impl WorkflowEventManager {
    /// 创建新的事件管理器
    pub fn new(executor: AsyncWorkflowExecutor) -> Self {
        Self { executor }
    }

    /// 启动事件监听循环
    pub async fn start_event_loop(&self, app_handle: tauri::AppHandle) {
        loop {
            // 等待下一个事件
            if let Some(event) = self.executor.next_event().await {
                // 将事件发送到前端
                self.emit_event_to_frontend(&app_handle, event).await;
            }

            // 短暂延迟避免过度占用CPU
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// 将执行事件发送到前端
    async fn emit_event_to_frontend(&self, app_handle: &tauri::AppHandle, event: ExecutionEvent) {
        let event_name = "workflow-execution-event";
        let payload = serde_json::to_value(&event).unwrap_or(serde_json::Value::Null);

        // 发送事件到所有窗口
        let _ = app_handle.emit(event_name, payload);
    }

    /// 获取执行器引用
    pub fn executor(&self) -> &AsyncWorkflowExecutor {
        &self.executor
    }
}

/// 启动工作流事件监听器
pub fn start_workflow_event_listener(app_handle: tauri::AppHandle, executor: AsyncWorkflowExecutor) {
    let event_manager = WorkflowEventManager::new(executor);

    // 在Tauri的异步运行时中运行事件循环
    tauri::async_runtime::spawn(async move {
        event_manager.start_event_loop(app_handle).await;
    });
}