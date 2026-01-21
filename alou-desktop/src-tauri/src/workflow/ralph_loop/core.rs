//! Ralph Loop 核心执行逻辑
//!
//! 实现 Ralph Loop 的主要循环执行逻辑

use super::super::AsyncWorkflowExecutor;
use super::super::*;
use crate::workflow::{Workflow, WorkflowStep};
use crate::bridges::BridgeManager;
use tokio::time::{sleep, Duration};

use super::ai_decision::*;
use super::research::*;
use super::learning::*;
use super::retry_strategy::*;

impl AsyncWorkflowExecutor {
    /// 使用Ralph Loop执行工作流
    pub async fn execute_workflow_with_ralph_loop(
        &self,
        execution_id: String,
        workflow: Workflow,
        api_key: String,
        agent_info: Option<serde_json::Value>,
        ralph_config: RalphLoopConfig,
    ) -> Result<(), String> {
        println!("🚀 [RALPH-LOOP] Starting Ralph Loop execution for workflow: {}", workflow.id);

        let start_time = chrono::Utc::now().timestamp_millis() as u64;
        let mut iteration = 0;
        let mut total_cost = 0.0;

        // 初始化执行历史（如果启用）
        if ralph_config.enable_history {
            let history = RalphLoopExecutionHistory {
                execution_id: execution_id.clone(),
                workflow_id: workflow.id.clone(),
                total_iterations: 0,
                iterations: Vec::new(),
                started_at: chrono::Utc::now().timestamp(),
                completed_at: None,
                final_status: None,
                total_cost: 0.0,
                total_execution_time_ms: None,
            };

            let mut histories = self.ralph_loop_histories.write().await;
            histories.insert(execution_id.clone(), history);
        }

        // Ralph Loop主循环
        loop {
            iteration += 1;
            println!("🔄 [RALPH-LOOP] Starting iteration {} for execution {}", iteration, execution_id);

            // 检查迭代次数限制
            if iteration > ralph_config.max_iterations {
                println!("⚠️ [RALPH-LOOP] Max iterations ({}) reached, terminating", ralph_config.max_iterations);
                self.complete_execution(&execution_id, ExecutionStatus::Failed).await;
                return Err(format!("Ralph Loop: Max iterations ({}) exceeded", ralph_config.max_iterations));
            }

            // 检查总执行时间限制
            if let Some(max_time) = ralph_config.max_total_time_ms {
                let elapsed = chrono::Utc::now().timestamp_millis() as u64 - start_time;
                if elapsed > max_time {
                    println!("⚠️ [RALPH-LOOP] Max total time ({})ms exceeded, terminating", max_time);
                    self.complete_execution(&execution_id, ExecutionStatus::Failed).await;
                    return Err(format!("Ralph Loop: Max total time ({})ms exceeded", max_time));
                }
            }

            // 检查成本限制
            if let Some(max_cost) = ralph_config.max_cost {
                if total_cost > max_cost {
                    println!("⚠️ [RALPH-LOOP] Max cost ({}) exceeded, terminating", max_cost);
                    self.complete_execution(&execution_id, ExecutionStatus::Failed).await;
                    return Err(format!("Ralph Loop: Max cost ({}) exceeded", max_cost));
                }
            }

            // 执行一次工作流
            let iteration_start = chrono::Utc::now().timestamp_millis();
            let result = self.execute_workflow_single_iteration(
                &execution_id,
                &workflow,
                &api_key,
                &agent_info,
                iteration,
            ).await;

            let iteration_time = chrono::Utc::now().timestamp_millis() - iteration_start;
            println!("⏱️ [RALPH-LOOP] Iteration {} completed in {}ms", iteration, iteration_time);

            // 记录迭代历史
            if ralph_config.enable_history {
                let iteration_history = RalphLoopIterationHistory {
                    iteration,
                    started_at: (iteration_start / 1000) as i64,
                    completed_at: Some(chrono::Utc::now().timestamp()),
                    result: result.as_ref().ok().cloned(),
                    error: result.as_ref().err().map(|e| e.to_string()),
                    cost: 0.01, // 模拟成本
                    execution_time_ms: Some(iteration_time as u64),
                    retry_count: 0, // 这里可以根据智能重试策略计算
                };

                let mut histories = self.ralph_loop_histories.write().await;
                if let Some(history) = histories.get_mut(&execution_id) {
                    history.iterations.push(iteration_history);
                    history.total_iterations = iteration;
                    history.total_cost = total_cost + 0.01;
                }
            }

            // 处理执行结果
            match self.handle_iteration_result(
                &execution_id,
                &workflow,
                iteration,
                result,
                &api_key,
                &agent_info,
                &ralph_config,
                start_time,
                total_cost,
            ).await {
                LoopResult::Continue => {
                    // 继续下一次迭代
                }
                LoopResult::Completed => {
                    // 任务完成
                    return Ok(());
                }
                LoopResult::Failed(error) => {
                    // 执行失败
                    return Err(error);
                }
            }

            // 迭代间延迟
            if iteration < ralph_config.max_iterations {
                println!("⏳ [RALPH-LOOP] Waiting {}ms before next iteration", ralph_config.iteration_delay_ms);
                sleep(Duration::from_millis(ralph_config.iteration_delay_ms)).await;
            }
        }
    }

    /// 处理迭代结果
    async fn handle_iteration_result(
        &self,
        execution_id: &str,
        workflow: &Workflow,
        iteration: u32,
        result: Result<serde_json::Value, String>,
        api_key: &str,
        agent_info: &Option<serde_json::Value>,
        ralph_config: &RalphLoopConfig,
        start_time: u64,
        total_cost: f64,
    ) -> LoopResult {
        match result {
            Ok(iteration_result) => {
                // 追踪AI学习进度
                self.track_ai_learning_progress(
                    execution_id,
                    iteration,
                    "workflow_execution",
                    &iteration_result
                ).await;

                // AI自动决策下一步行动（集成历史和学习进度）
                let ai_decision = match self.ai_decide_next_action_with_context(
                    execution_id,
                    iteration,
                    &iteration_result,
                    api_key,
                ).await {
                    Ok(decision) => decision,
                    Err(e) => {
                        eprintln!("❌ [AI-DECISION] Failed to get AI decision: {}", e);
                        "CONTINUE".to_string() // 默认继续
                    }
                };

                // 根据AI决策执行相应行动
                match self.execute_ai_decision(
                    execution_id,
                    workflow,
                    iteration,
                    &iteration_result,
                    &ai_decision,
                    api_key,
                    agent_info,
                    ralph_config,
                    start_time,
                    total_cost,
                ).await {
                    DecisionResult::Continue => LoopResult::Continue,
                    DecisionResult::Completed => LoopResult::Completed,
                    DecisionResult::Retry => LoopResult::Continue,
                }
            }
            Err(e) => {
                eprintln!("❌ [RALPH-LOOP] Iteration {} failed: {}", iteration, e);

                // 应用智能重试策略
                if ralph_config.smart_retry.enabled {
                    if self.should_retry_with_smart_strategy(execution_id, ralph_config, iteration, &e).await {
                        println!("🔄 [SMART-RETRY] Retrying iteration {} with adjusted strategy", iteration);
                        return LoopResult::Continue;
                    }
                }

                // 更新历史状态为失败
                if ralph_config.enable_history {
                    let mut histories = self.ralph_loop_histories.write().await;
                    if let Some(history) = histories.get_mut(execution_id) {
                        history.completed_at = Some(chrono::Utc::now().timestamp());
                        history.final_status = Some("failed".to_string());
                        history.total_execution_time_ms = Some((chrono::Utc::now().timestamp_millis() as u64 - start_time) as u64);
                    }
                }

                self.complete_execution(execution_id, ExecutionStatus::Failed).await;
                LoopResult::Failed(format!("Ralph Loop iteration {} failed: {}", iteration, e))
            }
        }
    }

    /// 执行AI决策
    async fn execute_ai_decision(
        &self,
        execution_id: &str,
        workflow: &Workflow,
        iteration: u32,
        iteration_result: &serde_json::Value,
        ai_decision: &str,
        api_key: &str,
        agent_info: &Option<serde_json::Value>,
        ralph_config: &RalphLoopConfig,
        start_time: u64,
        total_cost: f64,
    ) -> DecisionResult {
        match ai_decision {
            "COMPLETED" => {
                println!("✅ [AI-DECISION] AI determined task is completed");
                
                // 记录完成状态
                self.track_ai_learning_progress(
                    execution_id,
                    iteration,
                    "task_completed",
                    iteration_result
                ).await;
                
                // 更新历史状态
                if ralph_config.enable_history {
                    let mut histories = self.ralph_loop_histories.write().await;
                    if let Some(history) = histories.get_mut(execution_id) {
                        history.completed_at = Some(chrono::Utc::now().timestamp());
                        history.final_status = Some("completed".to_string());
                        history.total_execution_time_ms = Some((chrono::Utc::now().timestamp_millis() as u64 - start_time) as u64);
                    }
                }

                self.complete_execution(execution_id, ExecutionStatus::Completed).await;

                // 发送Ralph Loop完成事件
                let _ = self.event_sender.send(ExecutionEvent::Completed {
                    execution_id: execution_id.to_string(),
                    result: serde_json::json!({
                        "ralph_loop_iterations": iteration,
                        "total_time_ms": chrono::Utc::now().timestamp_millis() as u64 - start_time,
                        "total_cost": total_cost,
                        "final_result": iteration_result,
                        "ai_decision": ai_decision,
                        "learning_progress": "completed"
                    }),
                });

                DecisionResult::Completed
            }
            decision if decision.starts_with("RETRY:") => {
                let error_desc = decision.strip_prefix("RETRY:").unwrap_or("未知错误");
                println!("🔄 [AI-DECISION] AI decided to retry: {}", error_desc);
                
                // 记录重试决策
                self.track_ai_learning_progress(
                    execution_id,
                    iteration,
                    &format!("retry:{}", error_desc),
                    iteration_result
                ).await;
                
                DecisionResult::Retry
            }
            decision if decision.starts_with("RESEARCH:") => {
                let research_query = decision.strip_prefix("RESEARCH:").unwrap_or("未知查询");
                println!("📚 [AI-DECISION] AI decided to research: {}", research_query);
                
                // 记录调研决策
                self.track_ai_learning_progress(
                    execution_id,
                    iteration,
                    &format!("research:{}", research_query),
                    iteration_result
                ).await;
                
                // 执行文档调研
                match self.research_documentation_with_ai(research_query, api_key).await {
                    Ok(research_results) => {
                        println!("📊 [RESEARCH] Research completed, integrating results into loop");
                        
                        // 记录调研完成
                        self.track_ai_learning_progress(
                            execution_id,
                            iteration,
                            "research_completed",
                            &serde_json::json!({
                                "research_results": research_results,
                                "research_summary": self.summarize_research_results(&research_results).await
                            })
                        ).await;
                        
                        // 记录调研结果到历史
                        self.record_research_to_history(execution_id, &research_results).await;
                        
                        DecisionResult::Continue
                    }
                    Err(e) => {
                        println!("❌ [RESEARCH] Research failed: {}", e);
                        
                        // 记录调研失败
                        self.track_ai_learning_progress(
                            execution_id,
                            iteration,
                            &format!("research_failed:{}", e),
                            iteration_result
                        ).await;
                        
                        DecisionResult::Continue
                    }
                }
            }
            decision if decision.starts_with("ADJUST:") => {
                let adjustment = decision.strip_prefix("ADJUST:").unwrap_or("未知调整");
                println!("🔧 [AI-DECISION] AI decided to adjust strategy: {}", adjustment);
                
                // 记录策略调整
                self.track_ai_learning_progress(
                    execution_id,
                    iteration,
                    &format!("adjust:{}", adjustment),
                    iteration_result
                ).await;
                
                DecisionResult::Continue
            }
            "CONTINUE" | _ => {
                println!("🔄 [AI-DECISION] AI decided to continue execution");
                
                // 记录继续执行
                self.track_ai_learning_progress(
                    execution_id,
                    iteration,
                    "continue_execution",
                    iteration_result
                ).await;
                
                DecisionResult::Continue
            }
        }
    }

    /// 执行单次Ralph Loop迭代
    async fn execute_workflow_single_iteration(
        &self,
        execution_id: &str,
        workflow: &Workflow,
        api_key: &str,
        agent_info: &Option<serde_json::Value>,
        iteration: u32,
    ) -> Result<serde_json::Value, String> {
        println!("🔄 [RALPH-ITERATION] Executing iteration {} for workflow {}", iteration, workflow.id);

        // 重置执行状态为运行中
        self.update_execution_status(execution_id, ExecutionStatus::Running).await;

        // 执行工作流步骤（简化版本，实际可以调用现有的execute_workflow_async）
        let total_steps = workflow.steps.len();
        let mut completed_steps = 0;
        let mut iteration_results = serde_json::json!({
            "iteration": iteration,
            "steps": []
        });

        for step in &workflow.steps {
            // 执行步骤
            let step_result = self.execute_step_logic(step, execution_id, api_key, agent_info).await?;

            // 记录步骤结果
            if let Some(steps_array) = iteration_results.get_mut("steps").and_then(|s| s.as_array_mut()) {
                steps_array.push(serde_json::json!({
                    "step_id": step.id,
                    "status": "completed",
                    "result": step_result
                }));
            }

            completed_steps += 1;
            let progress = completed_steps as f32 / total_steps as f32;

            // 发送进度更新
            let _ = self.event_sender.send(ExecutionEvent::ProgressUpdated {
                execution_id: execution_id.to_string(),
                progress,
                current_step: Some(format!("Iteration {}: {}", iteration, step.name)),
            });
        }

        // 返回迭代结果
        Ok(iteration_results)
    }

    /// 检查完成条件
    async fn check_completion_condition(
        &self,
        iteration_result: &serde_json::Value,
        ralph_config: &RalphLoopConfig,
    ) -> bool {
        if let Some(checker) = &ralph_config.completion_checker {
            // 简单的字符串匹配检查
            if let Some(result_str) = iteration_result.to_string().to_lowercase().find(&checker.to_lowercase()) {
                println!("🎯 [COMPLETION] Found completion signal '{}' in result", checker);
                return true;
            }

            // 检查JSON路径（简化实现）
            if checker.starts_with("$.") {
                // 这里可以实现JSONPath逻辑
                // 暂时返回false
                println!("🔍 [COMPLETION] JSONPath checking not implemented yet: {}", checker);
                return false;
            }

            // 检查文件编辑操作的完成条件
            if checker.starts_with("file:") {
                return self.check_file_completion_condition(checker, iteration_result).await;
            }

            println!("❌ [COMPLETION] Completion condition '{}' not met", checker);
            false
        } else {
            // 如果没有设置完成检查器，默认检查是否所有步骤都成功
            println!("⚠️ [COMPLETION] No completion checker configured, checking step results");
            if let Some(steps) = iteration_result.get("steps").and_then(|s| s.as_array()) {
                steps.iter().all(|step| {
                    step.get("status").and_then(|s| s.as_str()) == Some("completed")
                })
            } else {
                false
            }
        }
    }

    /// 检查文件编辑操作的完成条件
    async fn check_file_completion_condition(
        &self,
        checker: &str,
        iteration_result: &serde_json::Value,
    ) -> bool {
        // 解析文件检查条件，例如: "file:deleted:/path/to/file.txt"
        // 或 "file:lines_deleted:/path/to/file.txt:5"
        // 或 "file:block_deleted:/path/to/file.txt:some_text"

        let parts: Vec<&str> = checker.split(':').collect();
        if parts.len() < 3 {
            println!("❌ [FILE-COMPLETION] Invalid file completion checker format: {}", checker);
            return false;
        }

        let operation = parts[1];
        let file_path = parts[2];

        match operation {
            "deleted" => {
                // 检查文件是否已被删除
                if !std::path::Path::new(file_path).exists() {
                    println!("✅ [FILE-COMPLETION] File '{}' has been deleted", file_path);
                    return true;
                }
                println!("❌ [FILE-COMPLETION] File '{}' still exists", file_path);
                false
            }
            "lines_deleted" => {
                if parts.len() >= 4 {
                    if let Ok(expected_lines) = parts[3].parse::<usize>() {
                        // 检查步骤结果中是否包含预期的行删除数量
                        if let Some(steps) = iteration_result.get("steps").and_then(|s| s.as_array()) {
                            for step in steps {
                                if let Some(result) = step.get("result") {
                                    if let Some(lines_deleted) = result.get("lines_deleted") {
                                        if let Some(deleted_count) = lines_deleted.as_u64() {
                                            if deleted_count >= expected_lines as u64 {
                                                println!("✅ [FILE-COMPLETION] Expected {} lines deleted, got {}", expected_lines, deleted_count);
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        println!("❌ [FILE-COMPLETION] Expected {} lines to be deleted from '{}'", expected_lines, file_path);
                        false
                    } else {
                        println!("❌ [FILE-COMPLETION] Invalid lines count in checker: {}", checker);
                        false
                    }
                } else {
                    println!("❌ [FILE-COMPLETION] Missing lines count in checker: {}", checker);
                    false
                }
            }
            "block_deleted" => {
                if parts.len() >= 4 {
                    let block_text = parts[3];
                    // 检查步骤结果中是否包含块删除操作
                    if let Some(steps) = iteration_result.get("steps").and_then(|s| s.as_array()) {
                        for step in steps {
                            if let Some(result) = step.get("result") {
                                if let Some(blocks_deleted) = result.get("blocks_deleted") {
                                    if let Some(deleted_count) = blocks_deleted.as_u64() {
                                        if deleted_count > 0 {
                                            println!("✅ [FILE-COMPLETION] Block '{}' deleted {} times", block_text, deleted_count);
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    println!("❌ [FILE-COMPLETION] Block '{}' not found/deleted in '{}'", block_text, file_path);
                    false
                } else {
                    println!("❌ [FILE-COMPLETION] Missing block text in checker: {}", checker);
                    false
                }
            }
            _ => {
                println!("❌ [FILE-COMPLETION] Unknown file operation: {}", operation);
                false
            }
        }
    }
}

/// 循环结果枚举
enum LoopResult {
    Continue,
    Completed,
    Failed(String),
}

/// 决策结果枚举
enum DecisionResult {
    Continue,
    Completed,
    Retry,
}
