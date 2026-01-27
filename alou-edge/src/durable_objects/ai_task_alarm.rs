//! AITaskDO Alarm 处理模块
//!
//! 负责处理 Alarm 逻辑、状态转换和工作流调度

use crate::compatibility::models::ToolCall;
use crate::durable_objects::ai_task_state::{TaskState, get_state_key, get_pending_tool_calls_key, get_tool_result_key};
use crate::compatibility::models::TaskStatus;
use crate::durable_objects::ai_task_persistence::TaskPersistence;
use crate::durable_objects::ai_task_execution::TaskExecutionContext;
use crate::durable_objects::ai_task_workflow::{WorkflowDecisionHandler, WorkflowDecision};
use crate::mcp::tools::workflow::Workflow;
use worker::{Result, console_log, console_error};

/// Alarm 处理器
pub struct AlarmHandler<'a> {
    pub ctx: TaskExecutionContext<'a>,
    pub state: &'a worker::State,
}

impl<'a> AlarmHandler<'a> {
    pub fn new(ctx: TaskExecutionContext<'a>, state: &'a worker::State) -> Self {
        Self { ctx, state }
    }

    /// 处理 Alarm 逻辑
    pub async fn handle_alarm_logic(
        &self,
        execute_workflow_task: impl Fn() -> Result<()>,
        execute_regular_task: impl Fn() -> Result<()>,
        continue_with_results: impl Fn() -> Result<()>,
    ) -> Result<()> {
        console_log!("[ALARM-LOGIC] Handling alarm for task: {}", self.ctx.task_name);
        console_log!("[ALARM-LOGIC] Current timestamp: {}", self.ctx.get_current_timestamp_millis());

        // 加载状态
        let state = self.load_state().await?;
        console_log!("[STATE-TRANSITION] Task {}: status={}, progress={}, step='{}', error={:?}",
                      self.ctx.task_name, state.status, state.progress, state.current_step, state.error);

        match state.status {
            TaskStatus::Processing => {
                self.handle_processing_state(state, execute_workflow_task, continue_with_results).await?;
            }
            TaskStatus::Queued => {
                self.handle_queued_state(state, execute_workflow_task, execute_regular_task).await?;
            }
            TaskStatus::Completed => {
                console_log!("[ALARM-LOGIC] Task already completed, no action needed");
            }
            TaskStatus::Failed => {
                console_log!("[ALARM-LOGIC] Task already failed, no action needed");
            }
            TaskStatus::Running => {
                console_log!("[ALARM-LOGIC] Task is running, checking if needs attention");
            }
        }

        Ok(())
    }

    /// 处理 Processing 状态
    async fn handle_processing_state(&self, state: TaskState, execute_workflow_task: impl Fn() -> Result<()>, continue_with_results: impl Fn() -> Result<()>) -> Result<()> {
        console_log!("[ALARM-LOGIC] Task is processing, checking for tool results");

        // 检查是否为工作流任务
        let is_workflow = TaskPersistence::is_workflow_task(self.ctx.storage, self.ctx.task_name).await?;

        // 检查是否有待处理的工具调用
        let storage = self.ctx.storage;
        let pending_key = get_pending_tool_calls_key(self.ctx.task_name);

        // 检查是否有待处理的工具调用（非空数组）
        let has_pending = match storage.get::<Vec<ToolCall>>(&pending_key).await {
            Ok(tool_calls) if !tool_calls.is_empty() => {
                console_log!("[ALARM-LOGIC] Found {} pending tool calls", tool_calls.len());
                true
            }
            _ => {
                console_log!("[ALARM-LOGIC] No pending tool calls");
                false
            }
        };

        if is_workflow {
            self.handle_workflow_processing(state, has_pending, execute_workflow_task).await?;
        } else {
            self.handle_regular_processing(state, has_pending, continue_with_results).await?;
        }

        Ok(())
    }

    /// 处理工作流任务的 Processing 状态
    async fn handle_workflow_processing(&self, state: TaskState, has_pending: bool, execute_workflow_task: impl Fn() -> Result<()>) -> Result<()> {
        let storage = self.ctx.storage;

        if has_pending {
            console_log!("[ALARM-LOGIC] Workflow task waiting for tool execution");
            // 重新设置alarm继续等待
            let now_ms = self.ctx.get_current_timestamp_millis();
            storage.set_alarm((now_ms + 5000) as i64).await?;
            return Ok(());
        } else {
            console_log!("[ALARM-LOGIC] Workflow task checking for tool results");
            // 检查是否有工具结果
            let result_key = get_tool_result_key(self.ctx.task_name);
            if storage.get::<serde_json::Value>(&result_key).await.is_ok() {
                console_log!("[ALARM-LOGIC] Workflow tool results available, letting AI decide next step");
                // 让 AI 根据工具结果判断下一步
                if let Err(e) = self.continue_workflow_with_ai(execute_workflow_task).await {
                    console_error!("[ALARM-LOGIC] Workflow AI decision failed: {}", e);
                    let mut new_state = state;
                    new_state.status = TaskStatus::Failed;
                    new_state.error = Some(format!("工作流AI判断失败: {}", e));
                    new_state.progress = 1.0;
                    self.save_state(&new_state).await?;
                    return Err(e);
                }
            } else {
                console_log!("[ALARM-LOGIC] Workflow no tool results yet, waiting...");
                let now_ms = self.ctx.get_current_timestamp_millis();
                storage.set_alarm((now_ms + 5000) as i64).await?;
            }
        }
        Ok(())
    }

    /// 处理常规任务的 Processing 状态
    async fn handle_regular_processing(&self, state: TaskState, has_pending: bool, continue_with_results: impl Fn() -> Result<()>) -> Result<()> {
        let storage = self.ctx.storage;

        if has_pending {
            console_log!("[ALARM-LOGIC] Still waiting for tool results, scheduling next check");
            // 重新设置alarm
            let now_ms = self.ctx.get_current_timestamp_millis();
            storage.set_alarm((now_ms + 5000) as i64).await?;
            console_log!("[STATE-TRANSITION] Task {}: staying in Processing state, alarm set for next check", self.ctx.task_name);
        } else {
            console_log!("[ALARM-LOGIC] No pending tool calls, checking for tool results");
            // 检查是否有工具结果，如果有就继续执行
            let result_key = get_tool_result_key(self.ctx.task_name);
            if storage.get::<serde_json::Value>(&result_key).await.is_ok() {
                console_log!("[ALARM-LOGIC] Tool results available, continuing execution");
                console_log!("[STATE-TRANSITION] Task {}: Processing -> Continuing with tool results", self.ctx.task_name);
                // 继续执行任务
                continue_with_results()?;
            } else {
                console_log!("[ALARM-LOGIC] No tool results yet, waiting...");
                let now_ms = self.ctx.get_current_timestamp_millis();
                storage.set_alarm((now_ms + 5000) as i64).await?;
                console_log!("[STATE-TRANSITION] Task {}: staying in Processing state, waiting for tool results", self.ctx.task_name);
            }
        }
        Ok(())
    }

    /// 处理 Queued 状态
    async fn handle_queued_state(&self, state: TaskState, execute_workflow: impl Fn() -> Result<()>, execute_regular: impl Fn() -> Result<()>) -> Result<()> {
        console_log!("[ALARM-LOGIC] Task is queued, starting execution");
        console_log!("[ALARM-LOGIC] Checking if task {} is a workflow task", self.ctx.task_name);

        // 检查是否为工作流任务
        let is_workflow = TaskPersistence::is_workflow_task(self.ctx.storage, self.ctx.task_name).await?;
        console_log!("[ALARM-LOGIC] Task {} is_workflow: {}", self.ctx.task_name, is_workflow);

        if is_workflow {
            console_log!("[ALARM-LOGIC] Workflow task detected, starting workflow execution");
            console_log!("[STATE-TRANSITION] Task {}: Queued -> Starting workflow execution", self.ctx.task_name);

            // 更新状态为Processing
            let mut new_state = state;
            new_state.status = TaskStatus::Processing;
            new_state.progress = 0.1;
            new_state.current_step = "初始化工作流".to_string();
            self.save_state(&new_state).await?;

            // 执行工作流任务
            if let Err(e) = execute_workflow() {
                console_error!("[ALARM-LOGIC] Workflow execution failed: {}", e);
                let mut failed_state = new_state;
                failed_state.status = TaskStatus::Failed;
                failed_state.error = Some(format!("工作流执行失败: {}", e));
                failed_state.progress = 1.0;
                self.save_state(&failed_state).await?;
                return Err(e);
            }
        } else {
            console_log!("[ALARM-LOGIC] Regular AI task, starting AI execution");
            console_log!("[STATE-TRANSITION] Task {}: Queued -> Starting execution", self.ctx.task_name);
            
            // 更新状态为Running
            let mut new_state = state;
            new_state.status = TaskStatus::Running;
            new_state.progress = 0.1;
            new_state.current_step = "开始执行".to_string();
            self.save_state(&new_state).await?;
            
            // 执行常规任务
            if let Err(e) = execute_regular() {
                console_error!("[ALARM-LOGIC] Regular task execution failed: {}", e);
                let mut failed_state = new_state;
                failed_state.status = TaskStatus::Failed;
                failed_state.error = Some(format!("任务执行失败: {}", e));
                failed_state.progress = 1.0;
                self.save_state(&failed_state).await?;
                return Err(e);
            }
        }

        Ok(())
    }

    /// 让 AI 判断工作流的下一步操作
    async fn continue_workflow_with_ai(&self, execute_workflow_task: impl Fn() -> Result<()>) -> Result<()> {
        console_log!("[WORKFLOW-AI] Letting AI decide next step");

        // 1. 获取工具结果
        let result_key = get_tool_result_key(self.ctx.task_name);
        let tool_result = match self.ctx.storage.get::<serde_json::Value>(&result_key).await {
            Ok(res) => res,
            Err(_) => {
                console_log!("[WORKFLOW-AI] No tool result found");
                return Ok(());
            }
        };

        // 2. 获取工作流
        let workflow = match TaskPersistence::load_workflow(self.ctx.storage, self.ctx.task_name).await? {
            Some(wf) => wf,
            None => return Err(worker::Error::RustError("Workflow not found".to_string())),
        };

        // 3. 获取原始请求（用于获取 AI 配置）
        let request = match TaskPersistence::get_request(self.ctx.storage, self.ctx.task_name).await? {
            Some(req) => req,
            None => return Err(worker::Error::RustError("Request not found".to_string())),
        };

        // 4. 初始化 AI 客户端
        let ai_api_key = match self.ctx.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.ctx.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    return Err(worker::Error::RustError("AI key not configured".to_string()));
                }
            }
        };

        let ai_client = crate::agent::ai_client::AiClient::new(
            request.provider.as_deref().unwrap_or("deepseek"), 
            ai_api_key, 
            Some(request.model.clone())
        )?;

        // 5. 使用工作流 AI 决策器获取决策
        use crate::durable_objects::ai_task_workflow::{WorkflowAIDecider, WorkflowDecisionHandler};
        let ai_caller = crate::durable_objects::ai_task_ai::DefaultAiCaller;
        let ai_decider = WorkflowAIDecider::new(&ai_caller, self.state, self.ctx.task_name.to_string());
        let decision = ai_decider.get_ai_decision(ai_client, &workflow, &tool_result).await;

        let decision = match decision {
            Ok(dec) => dec,
            Err(e) => {
                console_error!("[WORKFLOW-AI] AI call failed: {}, continuing anyway", e);
                // AI 调用失败，默认继续
                WorkflowDecision {
                    action: "continue".to_string(),
                    reason: "AI 调用失败，默认继续".to_string(),
                    next_step_id: None,
                }
            }
        };

        console_log!("[WORKFLOW-AI] AI decision: {:?}", decision);

        // 6. 使用决策处理器执行 AI 的决策
        let decision_handler = WorkflowDecisionHandler::new(self.state, self.ctx.task_name.to_string());
        match decision.action.as_str() {
            "continue" => {
                decision_handler.handle_continue_decision(tool_result, &workflow, decision.next_step_id).await?;
            }
            "retry" => {
                decision_handler.handle_retry_decision(tool_result, &workflow).await?;
            }
            "skip" => {
                decision_handler.handle_skip_decision(tool_result, &workflow, decision.reason).await?;
            }
            "terminate" => {
                decision_handler.handle_terminate_decision(decision.reason).await?;
            }
            _ => {
                console_error!("[WORKFLOW-AI] Unknown action: {}", decision.action);
                decision_handler.handle_continue_decision(tool_result, &workflow, None).await?;
            }
        }

        Ok(())
    }

    /// 加载状态
    async fn load_state(&self) -> Result<TaskState> {
        TaskPersistence::load_state(self.ctx.storage, self.ctx.task_name).await
    }

    /// 保存状态
    async fn save_state(&self, state: &TaskState) -> Result<()> {
        let storage = self.ctx.storage;
        let task_name = self.ctx.task_name;

        let state_data = serde_json::json!({
            "status": state.status.to_string(),
            "progress": state.progress,
            "current_step": state.current_step,
            "created_at": state.created_at,
            "updated_at": state.updated_at,
            "error": state.error,
        });

        let state_key = get_state_key(task_name);
        console_log!("[DEBUG-SAVE] Saving state for task: {}, key: {}, data: {:?}", task_name, state_key, state_data);
        storage.put(&state_key, state_data).await?;
        console_log!("[DEBUG-SAVE] State saved successfully for task: {}", task_name);
        Ok(())
    }
}
