//! AI决策模块
//!
//! 实现AI自动决策下一步行动的逻辑

use super::super::AsyncWorkflowExecutor;
use super::super::*;

impl AsyncWorkflowExecutor {
    /// AI自动决策下一步行动（集成上下文和学习进度）
    pub async fn ai_decide_next_action_with_context(
        &self,
        execution_id: &str,
        iteration: u32,
        previous_result: &serde_json::Value,
        api_key: &str,
    ) -> Result<String, String> {
        println!("🤖 [AI-DECISION-CONTEXT] AI analyzing with context for iteration {}", iteration);

        // 获取历史上下文
        let context_summary = self.get_execution_context_summary(execution_id).await;

        // 获取学习进度
        let learning_progress = self.get_learning_progress_summary(execution_id).await;

        // 构建增强的AI决策提示（控制token）
        let decision_prompt = format!(
            r#"
你是智能工作流协调器。基于以下信息决策下一步：

迭代: {}
历史摘要: {}
学习进度: {}
当前结果: {}

决策选项（简洁回复）：
1. COMPLETED - 任务完成
2. RETRY:<原因> - 重试
3. RESEARCH:<查询> - 调研
4. ADJUST:<策略> - 调整
5. CONTINUE - 继续

考虑历史和学习，选择最优行动。
"#,
            iteration,
            context_summary,
            learning_progress,
            self.simplify_result_for_ai(previous_result)
        );

        // 调用AI进行决策（控制token）
        match self.call_ai_for_decision(&decision_prompt, api_key).await {
            Ok(decision) => {
                println!("🧠 [AI-DECISION-CONTEXT] AI decided: {}", decision);
                Ok(decision)
            }
            Err(e) => {
                println!("❌ [AI-DECISION-CONTEXT] AI decision failed: {}", e);
                // 默认继续执行
                Ok("CONTINUE".to_string())
            }
        }
    }

    /// 调用AI进行决策
    pub async fn call_ai_for_decision(&self, prompt: &str, api_key: &str) -> Result<String, String> {
        // 使用bridge_manager调用AI工具
        let request = crate::bridges::ToolCallRequest {
            session_id: "ralph_loop_decision".to_string(),
            user_id: None,
            tool_id: "claude".to_string(),
            args: serde_json::json!({
                "prompt": prompt,
                "max_tokens": 100,
                "temperature": 0.3
            }),
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            environment: std::env::vars().collect(),
            timeout_seconds: Some(30),
            permissions: vec!["read".to_string()],
        };

        match self.bridge_manager.tool_bridge().handle_request(request).await {
            Ok(response) => {
                if response.success {
                    if let Some(result) = response.result {
                        Ok(result.data.as_str().unwrap_or("").trim().to_string())
                    } else {
                        Err("AI返回空结果".to_string())
                    }
                } else {
                    Err(response.error.unwrap_or_else(|| "AI调用失败".to_string()))
                }
            }
            Err(e) => Err(format!("AI调用错误: {}", e))
        }
    }

    /// 获取执行上下文摘要
    async fn get_execution_context_summary(&self, execution_id: &str) -> String {
        let histories = self.ralph_loop_histories.read().await;
        if let Some(history) = histories.get(execution_id) {
            let total_iterations = history.total_iterations;
            let success_rate = if total_iterations > 0 {
                history.iterations.iter()
                    .filter(|iter| iter.result.is_some())
                    .count() as f64 / total_iterations as f64
            } else {
                0.0
            };

            format!("迭代:{}, 成功率:{:.1}%", total_iterations, success_rate * 100.0)
        } else {
            "首次执行".to_string()
        }
    }

    /// 简化结果用于AI分析（控制token）
    fn simplify_result_for_ai(&self, result: &serde_json::Value) -> String {
        if let Some(steps) = result.get("steps").and_then(|s| s.as_array()) {
            let completed = steps.iter()
                .filter(|step| step.get("status").and_then(|s| s.as_str()) == Some("completed"))
                .count();

            let total = steps.len();

            // 检查是否有错误
            let has_errors = steps.iter()
                .any(|step| step.get("error").is_some());

            // 检查是否有调研结果
            let has_research = result.get("research_results").is_some();

            format!(
                "步骤:{}/{}, 错误:{}, 调研:{}",
                completed,
                total,
                if has_errors { "有" } else { "无" },
                if has_research { "有" } else { "无" }
            )
        } else {
            "结果已处理".to_string()
        }
    }

    /// AI错误分析
    pub async fn ai_analyze_error(&self, error: &str, iteration: u32) -> Result<String, String> {
        println!("🤖 [AI-ANALYSIS] AI analyzing error at iteration {}: {}", iteration, error);

        let analysis_prompt = format!(
            r#"
你是一个错误分析专家。请分析以下工作流执行错误并提供修复建议：

迭代次数: {}
错误信息: {}

请分析错误的严重程度并提供建议：
1. 如果是临时性错误（网络超时、资源暂不可用等），返回 "RETRYABLE: <描述>"
2. 如果是配置错误或参数问题，返回 "CONFIG_ERROR: <修复建议>"
3. 如果是致命错误（权限、依赖缺失等），返回 "CRITICAL: <描述>"
4. 如果是逻辑错误，返回 "LOGIC_ERROR: <修复建议>"
5. 如果是未知错误，返回 "UNKNOWN: <分析结果>"

请只返回上述格式之一，简洁明了。
"#,
            iteration, error
        );

        // 使用bridge_manager调用AI工具
        let request = crate::bridges::ToolCallRequest {
            session_id: "ralph_loop_error_analysis".to_string(),
            user_id: None,
            tool_id: "claude".to_string(),
            args: serde_json::json!({
                "prompt": analysis_prompt,
                "max_tokens": 150,
                "temperature": 0.2
            }),
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            environment: std::env::vars().collect(),
            timeout_seconds: Some(20),
            permissions: vec!["read".to_string()],
        };

        match self.bridge_manager.tool_bridge().handle_request(request).await {
            Ok(response) => {
                if response.success {
                    if let Some(result) = response.result {
                        Ok(result.data.as_str().unwrap_or("").trim().to_string())
                    } else {
                        Err("AI返回空结果".to_string())
                    }
                } else {
                    Err(response.error.unwrap_or_else(|| "AI分析失败".to_string()))
                }
            }
            Err(e) => Err(format!("AI分析错误: {}", e))
        }
    }
}
