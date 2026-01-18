//! AITaskDO 工作流执行模块
//!
//! 负责工作流的异步分步执行、AI 决策和状态管理

use crate::compatibility::models::CompatibleResponse;
use crate::durable_objects::{
    ai_task_state::{get_state_key, get_workflow_key, get_workflow_results_key, get_tool_result_key, get_result_key},
};
use crate::mcp::tools::workflow::{Workflow, WorkflowStep, StepStatus};
use crate::agent::ai_client::{AiClient, AiMessage};
use crate::durable_objects::ai_task_ai::DefaultAiCaller;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use worker::{console_log, console_error};

/// AI 决策结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDecision {
    /// 操作类型：continue/retry/skip/terminate
    pub action: String,
    /// 决策原因
    pub reason: String,
    /// 下一步骤ID（仅当 action 为 continue 时有效）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step_id: Option<String>,
}

/// 工作流执行器 trait
pub trait WorkflowExecutor {
    /// 检查是否为工作流任务
    async fn is_workflow_task(&self) -> bool;

    /// 执行工作流任务
    async fn execute_workflow_task(&self) -> worker::Result<()>;

    /// 让 AI 判断工作流的下一步操作
    async fn continue_workflow_with_ai(&self) -> worker::Result<()>;
}

/// 工作流存储 trait
pub trait WorkflowStorage {
    /// 加载工作流
    async fn load_workflow(&self) -> worker::Result<Option<Workflow>>;

    /// 保存工作流
    async fn save_workflow(&self, workflow: &Workflow) -> worker::Result<()>;

    /// 加载工作流结果
    async fn load_workflow_results(&self) -> worker::Result<Option<HashMap<String, Value>>>;

    /// 保存工作流结果
    async fn save_workflow_results(&self, results: &HashMap<String, Value>) -> worker::Result<()>;
}

/// 工作流 AI 决策器
pub struct WorkflowAIDecider<'a> {
    pub ai_caller: &'a DefaultAiCaller,
    pub state: &'a worker::State,
    pub task_name: String,
}

impl<'a> WorkflowAIDecider<'a> {
    pub fn new(ai_caller: &'a DefaultAiCaller, state: &'a worker::State, task_name: String) -> Self {
        Self {
            ai_caller,
            state,
            task_name,
        }
    }

    /// 构建 AI 提示词
    pub fn build_ai_prompt(&self, workflow: &Workflow, tool_result: &Value) -> String {
        let workflow_summary = self.build_workflow_summary(workflow);

        format!(
            "工作流当前状态：\n{}\n\n上一步工具执行结果：\n{}\n\n请判断下一步应该做什么。回答格式：JSON，包含 action（continue/retry/skip/terminate）、reason（原因）和 next_step_id（如果action是continue）。",
            workflow_summary,
            serde_json::to_string_pretty(tool_result).unwrap_or_else(|_| "无法解析".to_string())
        )
    }

    /// 构建工作流状态摘要
    fn build_workflow_summary(&self, workflow: &Workflow) -> String {
        let mut summary = format!("工作流：{}\n状态：{:?}\n步骤：\n", workflow.name, workflow.status);

        for step in &workflow.steps {
            summary.push_str(&format!(
                "  - {} ({}): {:?}\n",
                step.id, step.name, step.status
            ));
            if let Some(result) = &step.result {
                summary.push_str(&format!("    结果: {}\n", serde_json::to_string(result).unwrap_or_default()));
            }
            if let Some(error) = &step.error {
                summary.push_str(&format!("    错误: {}\n", error));
            }
        }

        summary
    }

    /// 解析 AI 决策
    pub fn parse_ai_decision(&self, content: &str) -> worker::Result<WorkflowDecision> {
        console_log!("[WORKFLOW-AI] Parsing AI decision from: {}", content);

        // 尝试提取 JSON
        let json_str = if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        match serde_json::from_str::<WorkflowDecision>(json_str) {
            Ok(decision) => Ok(decision),
            Err(e) => {
                console_error!("[WORKFLOW-AI] Failed to parse decision: {}", e);
                // 返回默认决策
                Ok(WorkflowDecision {
                    action: "continue".to_string(),
                    reason: "AI 解析失败，默认继续".to_string(),
                    next_step_id: None,
                })
            }
        }
    }

    /// 调用 AI 获取决策
    pub async fn get_ai_decision(
        &self,
        ai_client: AiClient,
        workflow: &Workflow,
        tool_result: &Value,
    ) -> worker::Result<WorkflowDecision> {
        let prompt = self.build_ai_prompt(workflow, tool_result);

        console_log!("[WORKFLOW-AI] Sending prompt to AI");

        let ai_messages = vec![
            AiMessage {
                role: "system".to_string(),
                content: "你是一个工作流执行助手，负责根据工具执行结果决定工作流的下一步操作。".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            AiMessage {
                role: "user".to_string(),
                content: prompt,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        match self.ai_caller.call_ai_with_timeout(ai_client, ai_messages, vec![], &self.task_name).await {
            Ok(ai_response) => {
                let decision = self.parse_ai_decision(&ai_response.content)?;
                console_log!("[WORKFLOW-AI] AI decision: {:?}", decision);
                Ok(decision)
            }
            Err(e) => {
                console_error!("[WORKFLOW-AI] AI call failed: {}", e);
                Err(e)
            }
        }
    }
}

/// 工作流执行器实现
pub struct WorkflowStepExecutor<'a> {
    pub state: &'a worker::State,
    pub task_name: String,
}

impl<'a> WorkflowStepExecutor<'a> {
    pub fn new(state: &'a worker::State, task_name: String) -> Self {
        Self { state, task_name }
    }

    /// 查找下一个可执行的步骤
    pub fn find_next_executable_step(&self, workflow: &Workflow) -> worker::Result<Option<WorkflowStep>> {
        for step in &workflow.steps {
            if step.status == StepStatus::Pending {
                // 检查依赖是否都已完成
                let dependencies_met = step.depends_on.iter().all(|dep_id| {
                    workflow.steps.iter().any(|s| s.id == *dep_id && s.status == StepStatus::Completed)
                });

                if dependencies_met {
                    return Ok(Some(step.clone()));
                }
            }
        }
        Ok(None)
    }

    /// 执行单个工作流步骤
    pub async fn execute_workflow_step(&self, step: &WorkflowStep) -> worker::Result<Value> {
        console_log!("[WORKFLOW-STEP] Executing step {}: tool={}", step.id, step.tool);

        // 将工具调用保存为待处理状态，等待外部工具执行器通过 /tool-result 端点返回结果
        let tool_call = crate::compatibility::models::ToolCall {
            tool: step.tool.clone(),
            arguments: step.args.clone(),
            id: None, // 工作流步骤没有预定义的 tool_call_id
        };

        // 保存待处理的工具调用
        let storage = self.state.storage();
        let pending_key = crate::durable_objects::ai_task_state::get_pending_tool_calls_key(&self.task_name);
        storage.put(&pending_key, &[tool_call]).await?;

        // 返回待执行状态，等待工具结果
        console_log!("[WORKFLOW-STEP] Step {} pending tool execution", step.id);

        Ok(serde_json::json!({
            "step_id": step.id,
            "tool": step.tool,
            "status": "pending",
            "message": "Waiting for tool execution"
        }))
    }

    /// 计算工作流进度
    pub fn calculate_workflow_progress(&self, workflow: &Workflow) -> f32 {
        if workflow.steps.is_empty() {
            return 0.0;
        }

        let completed = workflow.steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .count();

        (completed as f32) / (workflow.steps.len() as f32)
    }

    /// 检查工作流是否完成
    pub fn is_workflow_completed(&self, workflow: &Workflow) -> bool {
        workflow
            .steps
            .iter()
            .all(|s| s.status == StepStatus::Completed || s.status == StepStatus::Failed)
    }
}

/// 工作流决策处理器
pub struct WorkflowDecisionHandler<'a> {
    pub state: &'a worker::State,
    pub task_name: String,
}

impl<'a> WorkflowDecisionHandler<'a> {
    pub fn new(state: &'a worker::State, task_name: String) -> Self {
        Self { state, task_name }
    }

    /// 处理 "continue" 决策
    pub async fn handle_continue_decision(
        &self,
        tool_result: Value,
        workflow: &Workflow,
        _next_step_id: Option<String>,
    ) -> worker::Result<()> {
        console_log!("[WORKFLOW-AI] Continuing workflow");

        // 1. 找到并更新最近执行的步骤
        let mut updated_workflow = workflow.clone();
        for step in &mut updated_workflow.steps {
            if step.status == StepStatus::Pending {
                step.status = StepStatus::Completed;
                step.result = Some(tool_result.clone());
                console_log!("[WORKFLOW-AI] Marked step {} as completed", step.id);
                break;
            }
        }

        self.save_workflow(&updated_workflow).await?;

        // 2. 清除工具结果
        let result_key = get_tool_result_key(&self.task_name);
        let _ = self.state.storage().delete(&result_key).await;

        // 3. 继续执行工作流步骤
        self.execute_workflow_steps(&updated_workflow).await?;

        Ok(())
    }

    /// 处理 "retry" 决策
    pub async fn handle_retry_decision(
        &self,
        _tool_result: Value,
        workflow: &Workflow,
    ) -> worker::Result<()> {
        console_log!("[WORKFLOW-AI] Retrying current step");

        let state_key = get_state_key(&self.task_name);
        let mut state_data = self.state.storage().get::<Value>(&state_key).await?;

        let mut current_step = state_data
            .get_mut("current_step")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        current_step = "重试当前步骤（根据 AI 判断）".to_string();
        state_data["current_step"] = serde_json::Value::String(current_step);

        self.state.storage().put(&state_key, &state_data).await?;

        // 清除工具结果，准备重新执行
        let result_key = get_tool_result_key(&self.task_name);
        let _ = self.state.storage().delete(&result_key).await;

        // 重新执行工作流步骤（会重新触发工具调用）
        self.execute_workflow_steps(workflow).await?;

        Ok(())
    }

    /// 处理 "skip" 决策
    pub async fn handle_skip_decision(
        &self,
        tool_result: Value,
        workflow: &Workflow,
        reason: String,
    ) -> worker::Result<()> {
        console_log!("[WORKFLOW-AI] Skipping steps: {}", reason);

        // 标记当前步骤为跳过
        let mut updated_workflow = workflow.clone();
        for step in &mut updated_workflow.steps {
            if step.status == StepStatus::Pending {
                step.status = StepStatus::Skipped;
                step.result = Some(serde_json::json!({
                    "skipped": true,
                    "reason": reason
                }));
                console_log!("[WORKFLOW-AI] Marked step {} as skipped", step.id);
                break;
            }
        }

        self.save_workflow(&updated_workflow).await?;

        // 清除工具结果
        let result_key = get_tool_result_key(&self.task_name);
        let _ = self.state.storage().delete(&result_key).await;

        // 继续执行工作流步骤
        self.execute_workflow_steps(&updated_workflow).await?;

        Ok(())
    }

    /// 处理 "terminate" 决策
    pub async fn handle_terminate_decision(&self, reason: String) -> worker::Result<()> {
        console_log!("[WORKFLOW-AI] Terminating workflow: {}", reason);

        let state_key = get_state_key(&self.task_name);
        let mut state_data = self.state.storage().get::<Value>(&state_key).await?;

        state_data["status"] = serde_json::Value::String("completed".to_string());
        state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap());
        state_data["current_step"] = serde_json::Value::String(format!("工作流终止：{}", reason));

        self.state.storage().put(&state_key, &state_data).await?;

        // 保存结果
        let result_key = get_result_key(&self.task_name);
        let result = CompatibleResponse::success_response(
            format!("工作流已终止：{}", reason),
            None,
        );
        self.state.storage().put(&result_key, &result).await?;

        Ok(())
    }

    /// 执行工作流步骤（异步分步执行）
    pub async fn execute_workflow_steps(&self, workflow: &Workflow) -> worker::Result<()> {
        console_log!("[WORKFLOW] Executing {} steps", workflow.steps.len());

        // Ralph Loop: 检查是否需要先进行探索阶段
        let needs_exploration = self.needs_exploration_phase(workflow).await?;
        if needs_exploration {
            console_log!("[RALPH-LOOP] 🚀 Starting exploration phase for complex workflow");
            self.execute_exploration_phase(workflow).await?;
        }

        // 检查是否有工具结果需要处理
        let tool_result_key = get_tool_result_key(&self.task_name);
        let mut updated_workflow = workflow.clone();
        if let Ok(tool_result) = self.state.storage().get::<Value>(&tool_result_key).await {
            console_log!("[WORKFLOW] Found tool result, updating step status");

            // 查找最近执行的步骤并更新状态
            let mut updated_workflow = workflow.clone();
            let mut found_pending = false;

            for step in &mut updated_workflow.steps {
                if step.status == StepStatus::Pending {
                    // 检查依赖是否都已完成
                    let dependencies_met = step.depends_on.iter().all(|dep_id| {
                        workflow.steps.iter()
                            .any(|s| s.id == *dep_id && s.status == StepStatus::Completed)
                    });

                    if dependencies_met {
                        // 更新步骤状态为完成
                        step.status = StepStatus::Completed;
                        step.result = Some(tool_result.clone());
                        found_pending = true;

                        console_log!("[WORKFLOW] Marked step {} as completed", step.id);

                        // 保存步骤结果
                        let mut results: HashMap<String, Value> = HashMap::new();
                        if let Ok(Some(existing_results)) = self.load_workflow_results().await {
                            results = existing_results;
                        }
                        results.insert(step.id.clone(), tool_result.clone());
                        self.save_workflow_results(&results).await?;

                        // 清除工具结果，准备下一步
                        self.state.storage().delete(&tool_result_key).await?;

                        break;
                    }
                }
            }

            if found_pending {
                self.save_workflow(&updated_workflow).await?;
            }
        }

        // 查找下一个可执行的步骤
        let executor = WorkflowStepExecutor::new(self.state, self.task_name.clone());
        let next_step = executor.find_next_executable_step(workflow)?;

        if let Some(step) = next_step {
            console_log!("[WORKFLOW] Executing step: {}", step.name);

            // 更新状态
            let state_key = get_state_key(&self.task_name);
            let mut state_data = self.state.storage().get::<Value>(&state_key).await?;
            state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(executor.calculate_workflow_progress(workflow) as f64).unwrap());
            state_data["current_step"] = serde_json::Value::String(format!("执行步骤: {}", step.name));
            self.state.storage().put(&state_key, &state_data).await?;

            // 执行步骤
            let step_result = executor.execute_workflow_step(&step).await?;

            // 保存步骤结果
            let mut results: HashMap<String, Value> = HashMap::new();
            if let Ok(Some(existing_results)) = self.load_workflow_results().await {
                results = existing_results;
            }
            results.insert(step.id.clone(), step_result);
            self.save_workflow_results(&results).await?;

            // 更新步骤状态
            let mut final_workflow = updated_workflow.clone();
            for s in &mut final_workflow.steps {
                if s.id == step.id {
                    s.status = if results.get(&step.id).is_some() {
                        StepStatus::Completed
                    } else {
                        StepStatus::Failed
                    };
                    s.result = results.get(&step.id).cloned();
                    break;
                }
            }
            self.save_workflow(&final_workflow).await?;

            // 检查是否所有步骤都完成
            if executor.is_workflow_completed(&final_workflow) {
                console_log!("[WORKFLOW] Workflow completed");

                // 保存最终结果
                let final_result = CompatibleResponse::success_response(
                    format!("工作流完成，共执行 {} 个步骤", workflow.steps.len()),
                    None,
                );

                let result_key = get_result_key(&self.task_name);
                self.state.storage().put(&result_key, &final_result).await?;

                // 更新任务状态
                let state_key = get_state_key(&self.task_name);
                let mut state_data = self.state.storage().get::<Value>(&state_key).await?;
                state_data["status"] = serde_json::Value::String("completed".to_string());
                state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap());
                state_data["current_step"] = serde_json::Value::String("工作流完成".to_string());
                self.state.storage().put(&state_key, &state_data).await?;
            } else {
                // 继续执行下一步（设置延迟alarm）
                let now_ms = self.get_current_timestamp_millis();
                self.state.storage().set_alarm((now_ms + 1000) as i64).await?;
                console_log!("[WORKFLOW] Scheduled next step execution");
            }
        } else {
            console_log!("[WORKFLOW] No executable steps found");

            // 检查工作流状态
            if executor.is_workflow_completed(&updated_workflow) {
                let state_key = get_state_key(&self.task_name);
                let mut state_data = self.state.storage().get::<Value>(&state_key).await?;
                state_data["status"] = serde_json::Value::String("completed".to_string());
                state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap());
                state_data["current_step"] = serde_json::Value::String("工作流完成".to_string());
                self.state.storage().put(&state_key, &state_data).await?;
            } else {
                // 工作流卡住了
                let state_key = get_state_key(&self.task_name);
                let mut state_data = self.state.storage().get::<Value>(&state_key).await?;
                state_data["status"] = serde_json::Value::String("failed".to_string());
                state_data["error"] = serde_json::Value::String("工作流执行失败：无法继续执行".to_string());
                state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap());
                self.state.storage().put(&state_key, &state_data).await?;
            }
        }

        Ok(())
    }

    /// 获取当前时间戳（毫秒）
    fn get_current_timestamp_millis(&self) -> u64 {
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

    // 存储辅助方法
    async fn save_workflow(&self, workflow: &Workflow) -> worker::Result<()> {
        let key = get_workflow_key(&self.task_name);
        self.state.storage().put(&key, workflow).await?;
        Ok(())
    }

    async fn save_workflow_results(&self, results: &HashMap<String, Value>) -> worker::Result<()> {
        let key = get_workflow_results_key(&self.task_name);
        self.state.storage().put(&key, results).await?;
        Ok(())
    }

    async fn load_workflow_results(&self) -> worker::Result<Option<HashMap<String, Value>>> {
        let key = get_workflow_results_key(&self.task_name);
        Ok(self.state.storage().get::<HashMap<String, Value>>(&key).await.ok())
    }

    /// 检查是否需要探索阶段（Ralph Loop）
    async fn needs_exploration_phase(&self, workflow: &Workflow) -> worker::Result<bool> {
        // 检查是否已经进行过探索
        let exploration_key = format!("{}_exploration_done", &self.task_name);
        let exploration_done = match self.state.storage().get::<serde_json::Value>(&exploration_key).await {
            Ok(value) => value.as_bool().unwrap_or(false),
            _ => false,
        };

        if exploration_done {
            console_log!("[RALPH-LOOP] Exploration already completed, proceeding to execution");
            return Ok(false);
        }

        // 检查工作流复杂度 - 如果步骤数多于3个或包含复杂工具，则需要探索
        let is_complex = workflow.steps.len() > 3 ||
            workflow.steps.iter().any(|step| {
                // 检查是否使用了复杂工具
                matches!(step.tool.as_str(), "web_search" | "web_fetch" | "plan" | "subagents")
            });

        if is_complex {
            console_log!("[RALPH-LOOP] Complex workflow detected ({} steps), exploration needed", workflow.steps.len());
            Ok(true)
        } else {
            console_log!("[RALPH-LOOP] Simple workflow ({} steps), skipping exploration", workflow.steps.len());
            // 标记探索已完成，即使跳过了
            self.state.storage().put(&exploration_key, &true).await?;
            Ok(false)
        }
    }

    /// 执行探索阶段（Ralph Loop）
    async fn execute_exploration_phase(&self, workflow: &Workflow) -> worker::Result<()> {
        console_log!("[RALPH-LOOP] 🔍 Executing exploration phase");

        // 创建一个特殊的探索步骤
        let exploration_step = WorkflowStep {
            id: "exploration_phase".to_string(),
            name: "环境探索阶段".to_string(),
            tool: "bash".to_string(),
            args: serde_json::json!({
                "command": "pwd && ls -la && echo '=== Environment Info ===' && which node && which npm && which git",
                "working_directory": "."
            }),
            depends_on: vec![],
            status: StepStatus::Pending,
            result: None,
            error: None,
        };

        // 执行探索步骤
        let executor = WorkflowStepExecutor::new(self.state, self.task_name.clone());
        let exploration_result = executor.execute_workflow_step(&exploration_step).await?;

        console_log!("[RALPH-LOOP] ✅ Exploration phase completed, result: {:?}", exploration_result);

        // 标记探索已完成
        let exploration_key = format!("{}_exploration_done", &self.task_name);
        self.state.storage().put(&exploration_key, &true).await?;

        // 现在进入正常的执行阶段 - 直接执行步骤逻辑，不递归调用
        console_log!("[RALPH-LOOP] 🎯 Transitioning to execution phase");

        // 查找下一个可执行的步骤
        let executor = WorkflowStepExecutor::new(self.state, self.task_name.clone());
        let next_step = executor.find_next_executable_step(workflow)?;

        if let Some(step) = next_step {
            console_log!("[WORKFLOW] Executing step: {}", step.name);

            // 更新状态
            let state_key = get_state_key(&self.task_name);
            let mut state_data = self.state.storage().get::<Value>(&state_key).await?;
            state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(executor.calculate_workflow_progress(workflow) as f64).unwrap());
            state_data["current_step"] = serde_json::Value::String(format!("执行步骤: {}", step.name));
            self.state.storage().put(&state_key, &state_data).await?;

            // 执行步骤
            let step_result = executor.execute_workflow_step(&step).await?;

            // 保存步骤结果
            let mut results: HashMap<String, Value> = HashMap::new();
            if let Ok(Some(existing_results)) = self.load_workflow_results().await {
                results = existing_results;
            }
            results.insert(step.id.clone(), step_result);
            self.save_workflow_results(&results).await?;

            // 更新步骤状态
            let mut final_workflow = workflow.clone();
            for s in &mut final_workflow.steps {
                if s.id == step.id {
                    s.status = if results.get(&step.id).is_some() {
                        StepStatus::Completed
                    } else {
                        StepStatus::Failed
                    };
                    s.result = results.get(&step.id).cloned();
                    break;
                }
            }
            self.save_workflow(&final_workflow).await?;

            // 检查是否所有步骤都完成
            if executor.is_workflow_completed(&final_workflow) {
                console_log!("[WORKFLOW] Workflow completed");

                // 保存最终结果
                let final_result = CompatibleResponse::success_response(
                    format!("工作流完成，共执行 {} 个步骤", workflow.steps.len()),
                    None,
                );

                let result_key = get_result_key(&self.task_name);
                self.state.storage().put(&result_key, &final_result).await?;

                // 更新任务状态
                let state_key = get_state_key(&self.task_name);
                let mut state_data = self.state.storage().get::<Value>(&state_key).await?;
                state_data["status"] = serde_json::Value::String("completed".to_string());
                state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap());
                state_data["current_step"] = serde_json::Value::String("工作流完成".to_string());
                self.state.storage().put(&state_key, &state_data).await?;
            } else {
                // 继续执行下一步（设置延迟alarm）
                let now_ms = self.get_current_timestamp_millis();
                self.state.storage().set_alarm((now_ms + 1000) as i64).await?;
                console_log!("[WORKFLOW] Scheduled next step execution");
            }
        } else {
            console_log!("[WORKFLOW] No executable steps found");

            // 检查工作流状态
            if executor.is_workflow_completed(workflow) {
                let state_key = get_state_key(&self.task_name);
                let mut state_data = self.state.storage().get::<Value>(&state_key).await?;
                state_data["status"] = serde_json::Value::String("completed".to_string());
                state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap());
                state_data["current_step"] = serde_json::Value::String("工作流完成".to_string());
                self.state.storage().put(&state_key, &state_data).await?;
            } else {
                // 工作流卡住了
                let state_key = get_state_key(&self.task_name);
                let mut state_data = self.state.storage().get::<Value>(&state_key).await?;
                state_data["status"] = serde_json::Value::String("failed".to_string());
                state_data["error"] = serde_json::Value::String("工作流执行失败：无法继续执行".to_string());
                state_data["progress"] = serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap());
                self.state.storage().put(&state_key, &state_data).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ai_decision() {
        let json_content = r#"{"action":"continue","reason":"All steps completed","next_step_id":"step3"}"#;
        // 需要在实际上下文中测试
    }
}
