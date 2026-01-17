//! AITaskDO - AI任务Durable Object
//!
//! 负责管理AI任务的执行状态、进度跟踪和结果存储
//!
//! 模块结构：
//! - ai_task.rs: Durable Object 核心实现
//! - ai_task_state.rs: 状态模型和存储键
//! - ai_task_handlers.rs: HTTP 路由处理器
//! - ai_task_persistence.rs: 持久化操作
//! - ai_task_ai.rs: AI 调用模块
//! - ai_task_execution.rs: 任务执行逻辑
//! - ai_task_workflow.rs: 工作流决策逻辑
//! - ai_task_alarm.rs: Alarm 处理逻辑

use crate::durable_objects::{
    ai_task_state::{TaskState, TaskCache},
    ai_task_execution::{TaskExecutorImpl, TaskExecutionContext},
    ai_task_workflow::WorkflowDecisionHandler,
    ai_task_persistence::TaskPersistence,
    ai_task_alarm::AlarmHandler,
    ai_task_handlers::AITaskHandlers,
};
use crate::mcp::tools::workflow::Workflow;
use std::cell::RefCell;

use worker::{
    durable_object, Env, Method, Request, Response, Result, State, console_error, console_log,
};

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

/// AI任务Durable Object
#[durable_object]
pub struct AITaskDO {
    state: State,
    env: Env,
    cache: RefCell<Option<TaskCache>>,
    is_executing: RefCell<bool>,
    debug_logs: RefCell<Vec<String>>,
    ai_caller: DefaultAiCaller,
}

impl DurableObject for AITaskDO {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
            cache: RefCell::new(None),
            is_executing: RefCell::new(false),
            debug_logs: RefCell::new(Vec::new()),
            ai_caller: DefaultAiCaller,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        console_log!("[AITaskDO] fetch called, path: {}", req.path());

        let path = req.path();
        let method = req.method();

        match (&method, path.as_str()) {
            (Method::Get, "/status") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_get_status(&self.task_name(), &storage).await
            }
            (Method::Post, "/init") => {
                let storage = self.state.storage();
                let request: CompatibleRequest = req.json().await?;
                AITaskHandlers::handle_init(&self.task_name(), &storage, request).await
            }
            (Method::Post, "/start") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_start(&self.task_name(), &storage, || self.get_current_timestamp_millis()).await
            }
            (Method::Post, "/init-and-start") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_init_and_start(&self.task_name(), &storage, req, || self.get_current_timestamp_millis()).await
            }
            (Method::Post, "/cancel") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_cancel(&self.task_name(), &storage, || self.get_current_timestamp_millis()).await
            }
            (Method::Post, "/tool-result") => {
                let storage = self.state.storage();
                AITaskHandlers::handle_tool_result(&self.task_name(), &storage, req, || self.get_current_timestamp_millis()).await
            }
            _ => {
                console_log!("[AITaskDO] Unknown route: {:?} {}", method, path);
                Response::error("Not found", 404)
            }
        }
    }

    async fn alarm(&self) -> Result<Response> {
        console_error!("!!! ALARM ACTIVE !!! Task: {}", self.task_name());
        console_log!("🚨 AITaskDO ALARM STARTED");
        console_log!("[DEBUG] Current DO ID: {}", self.state.id().to_string());

        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        // 使用 AlarmHandler 处理 alarm 逻辑
        let ctx = TaskExecutionContext::new(
            &self.state.storage(),
            &self.task_name(),
            &self.env,
        );
        let alarm_handler = AlarmHandler::new(ctx);

        match alarm_handler.handle_alarm_logic(
            || self.execute_workflow_task(),
            || self.execute_task(),
            || self.continue_with_tool_results(),
        ).await {
            Ok(_) => Response::ok("Alarm handled"),
            Err(e) => {
                console_error!("Alarm handling failed: {}", e);
                Response::error(format!("Alarm failed: {}", e), 500)
            }
        }
    }
}

impl AITaskDO {
    // 内部辅助方法
    pub fn task_name(&self) -> String {
        match self.state.id().name() {
            Some(name) => name.to_string(),
            None => self.state.id().to_string()
        }
    }

    pub fn get_current_timestamp_millis(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::Date;
            Date::now() as u64
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        }
    }

    /// 执行工作流任务
    async fn execute_workflow_task(&self) -> Result<()> {
        console_log!("[WORKFLOW] Starting workflow execution");

        // 尝试加载已保存的工作流
        let workflow: Workflow = match TaskPersistence::load_workflow(&self.state.storage(), &self.task_name()).await? {
            Some(wf) => {
                console_log!("[WORKFLOW] Loaded existing workflow with {} steps", wf.steps.len());
                wf
            }
            _ => {
                // 首次执行，从请求中加载工作流配置
                let request = match TaskPersistence::get_request(&self.state.storage(), &self.task_name()).await? {
                    Some(req) => req,
                    None => {
                        return Err(worker::Error::RustError("Request data not found".to_string()));
                    }
                };

                // 获取工作流配置
                let workflow_config = request.workflow_config.as_ref()
                    .ok_or_else(|| worker::Error::RustError("Workflow config not found".to_string()))?;

                // 解析工作流步骤
                let wf: Workflow = serde_json::from_value(workflow_config.clone())
                    .map_err(|e| worker::Error::RustError(format!("Failed to parse workflow: {}", e)))?;

                // 保存工作流
                TaskPersistence::save_workflow(&self.state.storage(), &self.task_name(), &wf).await?;

                wf
            }
        };

        // 使用决策处理器执行工作流步骤
        let decision_handler = WorkflowDecisionHandler::new(&self.state, self.task_name());
        decision_handler.execute_workflow_steps(&workflow).await?;

        Ok(())
    }

    /// 基于工具结果继续对话
    async fn continue_with_tool_results(&self) -> Result<()> {
        console_log!("[CONTINUE] Continuing conversation with tool results");

        // 创建执行上下文
        let ctx = TaskExecutionContext::new(&self.state.storage(), &self.task_name(), &self.env);
        let executor = TaskExecutorImpl::new(ctx);

        executor.continue_with_tool_results().await
    }

    /// 继续执行任务（在收到工具结果后）- 保留用于兼容性
    async fn continue_execution(&self) -> Result<()> {
        console_log!("[CONTINUE] Legacy continue_execution called, redirecting to continue_with_tool_results");
        self.continue_with_tool_results().await
    }

    /// 执行任务
    async fn execute_task(&self) -> Result<()> {
        // 创建执行上下文
        let ctx = TaskExecutionContext::new(&self.state.storage(), &self.task_name(), &self.env);
        let executor = TaskExecutorImpl::new(ctx);

        executor.execute_full_task().await
    }
}

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_creation() {
        let state = TaskState::new_initial();
        assert_eq!(state.status, TaskStatus::Queued);
        assert_eq!(state.progress, 0.0);
    }

    #[test]
    fn test_task_state_failed() {
        let state = TaskState::new_failed("test error".to_string());
        assert_eq!(state.status, TaskStatus::Failed);
        assert_eq!(state.progress, 1.0);
        assert_eq!(state.error, Some("test error".to_string()));
    }
}