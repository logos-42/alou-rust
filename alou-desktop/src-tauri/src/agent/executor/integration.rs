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

        // 4. 🔥 分析工具执行结果，判断是否需要调整策略
        let tool_analysis = self.analyze_tool_results(results);
        
        // 5. 添加到对话历史
        let _ = task_manager.update_task(task_id, |task| {
            task.messages.push(AiMessage {
                role: "assistant".to_string(),
                content: summary.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
        }).await;

        // 6. 更新迭代计数
        let _ = task_manager.update_task(task_id, |task| {
            task.metadata.iteration_count += 1;
        }).await;
        
        // 🔥 如果工具分析发现问题，提前终止或继续
        if let Some(analysis_msg) = tool_analysis {
            log::info!("[Integration] 工具分析结果：{}", analysis_msg);
            // 添加到任务消息，让 AI 在下一轮能看到
            let _ = task_manager.update_task(task_id, |task| {
                task.messages.push(AiMessage {
                    role: "system".to_string(),
                    content: analysis_msg,
                    tool_call_id: None,
                    tool_calls: None,
                });
            }).await;
        }

        // 7. 检查是否需要终止
        let should_terminate = self.check_termination_conditions(core, task_id, thought, results).await?;

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

            // 🔥 添加工具输出结果，让 agent 可以看到实际返回内容
            if let Some(ref output) = result.output {
                let output_str = match output {
                    Value::String(s) => s.clone(),
                    _ => output.to_string(),
                };
                // 只显示前 500 个字符，避免过长
                let truncated = if output_str.len() > 500 {
                    format!("{}...", &output_str[..500])
                } else {
                    output_str
                };
                summary.push_str(&format!("  - 输出：{}\n", truncated));
            }

            if let Some(ref error) = result.error {
                summary.push_str(&format!("  - 错误：{}\n", error));
            }
        }

        summary
    }

    /// 分析工具执行结果
    fn analyze_tool_results(&self, results: &[ActionResult]) -> Option<String> {
        let mut failed_tools = Vec::new();
        let mut repeated_tools = std::collections::HashMap::new();
        
        for result in results {
            log::info!("[Integration] 工具结果：tool={}, success={}, error={:?}", 
                result.tool_name, result.success, result.error);
            
            // 记录失败的工具
            if !result.success {
                failed_tools.push((result.tool_name.clone(), result.error.clone()));
            }
            
            // 统计工具调用次数
            *repeated_tools.entry(result.tool_name.clone()).or_insert(0) += 1;
        }
        
        log::info!("[Integration] 失败工具数：{}, 重复工具数：{}", failed_tools.len(), repeated_tools.len());
        
        // 🔥 如果有工具失败，给 AI 提示
        if !failed_tools.is_empty() {
            let mut msg = String::from("⚠️ **工具执行失败提醒**\n\n");
            for (tool, error) in &failed_tools {
                msg.push_str(&format!(
                    "- **{}** 失败：{}\n",
                    tool,
                    error.as_deref().unwrap_or("未知错误")
                ));
            }
            msg.push_str("\n**建议**：\n");
            msg.push_str("1. 分析失败原因，检查参数是否正确\n");
            msg.push_str("2. 如果工具不可用，尝试其他替代工具\n");
            msg.push_str("3. 同一工具失败 2 次后，请切换到其他方法\n");
            msg.push_str("4. 如果所有方法都失败，向用户报告问题\n");
            log::info!("[Integration] 发送失败提醒：{}", msg);
            return Some(msg);
        }
        
        // 🔥 如果同一工具被调用超过 2 次，提醒 AI
        for (tool, count) in &repeated_tools {
            if *count > 2 && tool != "continue" {
                let msg = format!(
                    "⚠️ **工具重复调用提醒**\n\n\
                     工具 **{}** 已被调用 {} 次。\n\n\
                     **建议**：\n\
                     1. 如果工具持续失败，请尝试其他方法\n\
                     2. 考虑使用替代工具（如用 filesystem 代替 bash）\n\
                     3. 如果任务已完成，请直接返回结果",
                    tool, count
                );
                log::info!("[Integration] 发送重复调用提醒：{}", msg);
                return Some(msg);
            }
        }
        
        log::info!("[Integration] 无需提醒");
        None
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
        results: &[ActionResult],
    ) -> Result<bool, crate::agent::executor::types::ExecutorError> {
        // 1. 检查是否显式完成
        if matches!(thought.action, Action::Complete(_)) {
            return Ok(true);
        }
        
        // 🔥 2. 检查是否有连续失败的工具
        let consecutive_failures = results.iter().filter(|r| !r.success).count();
        if consecutive_failures >= 3 {
            log::warn!("[IntegrationLayer] 连续 {} 个工具失败，建议终止", consecutive_failures);
            // 不强制终止，但会让 AI 知道问题
        }

        // 3. 检查最大迭代次数
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

        // 4. 检查反思层建议
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
