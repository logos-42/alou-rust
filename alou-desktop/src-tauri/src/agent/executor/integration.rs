//! 整合层模块
//!
//! 负责：
//! - 整合行动结果到对话历史
//! - 更新任务状态
//! - 管理循环终止条件
//! - 发送事件到前端

use std::sync::Arc;

use serde_json::Value;
use tauri::Emitter;

use crate::agent::ai_client::AiMessage;
use crate::agent::executor::types::{EnvironmentState, Thought, Action, ActionResult, ExecutionResult};
use crate::agent::executor::core::ExecutorCore;
use crate::agent::task::TaskStatus;

/// 整合层
pub struct IntegrationLayer;

impl IntegrationLayer {
    /// 创建整合层实例
    pub fn new() -> Self {
        Self
    }

    /// 整合结果
    pub async fn integrate(
        &self,
        core: &ExecutorCore,
        task_id: &str,
        thought: &Thought,
        results: &[ActionResult],
    ) -> Result<ExecutionResult, crate::agent::executor::types::ExecutorError> {
        let task_manager = core.task_manager();

        // 1. 处理特殊情况
        for result in results {
            match result.tool_name.as_str() {
                "complete" => {
                    if let Some(ref output) = result.output {
                        let content = match output {
                            Value::String(s) => s.clone(),
                            _ => output.to_string(),
                        };

                        // 更新任务为完成状态
                        let _ = task_manager.update_task(task_id, |task| {
                            task.status = TaskStatus::Completed;
                            task.final_response = Some(content.clone());
                        }).await;

                        // 发送完成事件
                        self.emit_event(core, task_id, "completed", Some(&serde_json::json!({
                            "task_id": task_id,
                            "result": content,
                        })));

                        return Ok(ExecutionResult::Completed(content));
                    }
                }
                "ask_user" => {
                    if let Some(ref output) = result.output {
                        let question = match output {
                            Value::String(s) => s.clone(),
                            _ => output.to_string(),
                        };
                        // 需要用户输入 - 添加到消息历史
                        let _ = task_manager.update_task(task_id, |task| {
                            task.messages.push(AiMessage {
                                role: "assistant".to_string(),
                                content: question,
                                tool_call_id: None,
                                tool_calls: None,
                            });
                        }).await;
                    }
                    return Ok(ExecutionResult::NeedsMoreIterations);
                }
                _ => {}
            }
        }

        // 2. 发送工具执行事件
        for result in results {
            if result.tool_name == "continue" || result.tool_name == "complete" {
                continue;
            }

            self.emit_event(core, task_id, "tool_done", Some(&serde_json::json!({
                "task_id": task_id,
                "tool_name": result.tool_name,
                "success": result.success,
                "error": result.error,
            })));
        }

        // 3. 构建结果摘要
        let summary = self.build_results_summary(thought, results);

        // 4. 添加到对话历史
        let _ = task_manager.update_task(task_id, |task| {
            task.messages.push(AiMessage {
                role: "assistant".to_string(),
                content: summary.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
        }).await;

        // 5. 更新迭代计数
        let _ = task_manager.update_task(task_id, |task| {
            task.metadata.iteration_count += 1;
        }).await;

        // 6. 检查是否需要终止
        let should_terminate = self.check_termination_conditions(core, task_id, thought).await?;

        if should_terminate {
            let final_answer = self.extract_final_answer(thought, results);

            // 更新任务为完成状态
            let _ = task_manager.update_task(task_id, |task| {
                task.status = TaskStatus::Completed;
                task.final_response = Some(final_answer.clone());
            }).await;

            // 发送完成事件
            self.emit_event(core, task_id, "completed", Some(&serde_json::json!({
                "task_id": task_id,
                "result": final_answer,
            })));

            return Ok(ExecutionResult::Completed(final_answer));
        }

        Ok(ExecutionResult::NeedsMoreIterations)
    }

    /// 构建结果摘要
    fn build_results_summary(&self, thought: &Thought, results: &[ActionResult]) -> String {
        let mut summary = format!("**分析**: {}\n\n", thought.analysis);

        if let Some(ref reflection) = thought.reflection {
            summary.push_str(&format!(
                "**反思**:\n- 置信度：{:.0}%\n- 进度：{:.0}%\n- 建议：{}\n\n",
                reflection.confidence * 100.0,
                reflection.task_progress.overall_progress * 100.0,
                reflection.suggested_next_step
            ));
        }

        summary.push_str("**执行结果**:\n");
        for result in results {
            if result.tool_name == "continue" || result.tool_name == "complete" {
                continue;
            }

            summary.push_str(&format!(
                "- {}: {}\n",
                result.tool_name,
                if result.success { "✓ 成功" } else { "✗ 失败" }
            ));

            if let Some(ref error) = result.error {
                summary.push_str(&format!("  - 错误：{}\n", error));
            }
        }

        summary
    }

    /// 提取最终答案
    fn extract_final_answer(&self, thought: &Thought, results: &[ActionResult]) -> String {
        // 优先使用 Complete 动作的内容
        if let Action::Complete(content) = &thought.action {
            return content.clone();
        }

        // 否则从结果中提取
        for result in results {
            if result.tool_name == "complete" {
                if let Some(ref output) = result.output {
                    return match output {
                        Value::String(s) => s.clone(),
                        _ => output.to_string(),
                    };
                }
            }
        }

        // 默认返回分析内容
        thought.analysis.clone()
    }

    /// 检查终止条件
    async fn check_termination_conditions(
        &self,
        core: &ExecutorCore,
        task_id: &str,
        thought: &Thought,
    ) -> Result<bool, crate::agent::executor::types::ExecutorError> {
        // 1. 检查是否显式完成
        if matches!(thought.action, Action::Complete(_)) {
            return Ok(true);
        }

        // 2. 检查最大迭代次数
        let task_manager = core.task_manager();
        let task = task_manager
            .get_task(task_id)
            .await
            .ok_or_else(|| crate::agent::executor::types::ExecutorError::TaskNotFound(
                task_id.to_string()
            ))?;

        if task.metadata.iteration_count >= crate::agent::executor::core::RalphLoopExecutor::MAX_ITERATIONS {
            log::warn!("[IntegrationLayer] 达到最大迭代次数，强制终止");
            return Ok(true);
        }

        // 3. 检查反思层建议
        if let Some(ref reflection) = thought.reflection {
            // 如果任务已完成（进度 100%）
            if reflection.task_progress.overall_progress >= 1.0 {
                return Ok(true);
            }

            // 如果置信度高且没有待处理步骤
            if reflection.confidence > 0.9 && reflection.task_progress.pending_steps.is_empty() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 处理错误
    pub async fn handle_error(
        &self,
        core: &ExecutorCore,
        task_id: &str,
        error: &crate::agent::executor::types::ExecutorError,
    ) -> Result<(), crate::agent::executor::types::ExecutorError> {
        let task_manager = core.task_manager();

        let error_message = format!("执行错误：{}", error);

        // 添加系统错误消息
        let _ = task_manager.update_task(task_id, |task| {
            task.messages.push(AiMessage {
                role: "system".to_string(),
                content: error_message.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
            task.status = TaskStatus::Failed;
            task.error = Some(error.to_string());
        }).await;

        // 发送失败事件
        self.emit_event(core, task_id, "failed", Some(&serde_json::json!({
            "task_id": task_id,
            "error": error_message,
        })));

        Ok(())
    }

    /// 发送事件到前端
    fn emit_event(&self, core: &ExecutorCore, task_id: &str, event_type: &str, payload: Option<&Value>) {
        if let Some(app_handle) = core.app_handle() {
            let event_payload = payload.cloned().unwrap_or_else(|| serde_json::json!({
                "task_id": task_id,
            }));

            if let Err(e) = app_handle.emit("agent:progress", &event_payload) {
                log::warn!("[IntegrationLayer] 发送 agent:progress 事件失败：{}", e);
            }
        }
    }
}

impl Default for IntegrationLayer {
    fn default() -> Self {
        Self::new()
    }
}
