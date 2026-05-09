//! 工作流执行历史管理
//!
//! 提供执行历史的记录、查询、回滚和清理功能

use super::AsyncWorkflowExecutor;
use super::*;

impl AsyncWorkflowExecutor {
    /// 回滚到指定的历史状态
    pub async fn rollback_to_iteration(
        &self,
        execution_id: &str,
        target_iteration: u32,
    ) -> Result<(), String> {
        let histories = self.ralph_loop_histories.read().await;
        let history = histories.get(execution_id)
            .ok_or_else(|| format!("No history found for execution: {}", execution_id))?;

        let target_history = history.iterations.iter()
            .find(|iter| iter.iteration == target_iteration)
            .ok_or_else(|| format!("Iteration {} not found in history", target_iteration))?;

        // 这里可以实现具体的回滚逻辑，比如恢复文件状态、数据库状态等
        // 目前只是记录回滚操作
        println!("🔄 [ROLLBACK] Rolling back execution {} to iteration {}", execution_id, target_iteration);

        // 发送回滚事件
        let _ = self.event_sender.send(ExecutionEvent::Completed {
            execution_id: execution_id.to_string(),
            result: serde_json::json!({
                "rollback_to_iteration": target_iteration,
                "rollback_time": chrono::Utc::now().timestamp(),
                "original_result": target_history.result
            }),
        });

        Ok(())
    }

    /// 获取执行历史
    pub async fn get_execution_history(&self, execution_id: &str) -> Result<Option<RalphLoopExecutionHistory>, String> {
        let histories = self.ralph_loop_histories.read().await;
        Ok(histories.get(execution_id).cloned())
    }

    /// 清理旧的历史记录
    pub async fn cleanup_old_histories(&self, max_age_days: u32) {
        let cutoff_time = chrono::Utc::now().timestamp() - (max_age_days as i64 * 24 * 60 * 60);
        let mut histories = self.ralph_loop_histories.write().await;

        histories.retain(|_, history| {
            history.completed_at.unwrap_or(history.started_at) > cutoff_time
        });

        println!("🧹 [HISTORY] Cleaned up old execution histories, {} remaining", histories.len());
    }

    /// 获取所有执行历史
    pub async fn get_all_execution_histories(&self) -> Vec<RalphLoopExecutionHistory> {
        let histories = self.ralph_loop_histories.read().await;
        histories.values().cloned().collect()
    }

    /// 获取执行历史的统计信息
    pub async fn get_history_statistics(&self) -> serde_json::Value {
        let histories = self.ralph_loop_histories.read().await;

        let total_executions = histories.len();
        let completed_executions = histories.values()
            .filter(|h| h.final_status.as_deref() == Some("completed"))
            .count();
        let failed_executions = histories.values()
            .filter(|h| h.final_status.as_deref() == Some("failed"))
            .count();

        let total_iterations: u32 = histories.values()
            .map(|h| h.total_iterations)
            .sum();

        let avg_iterations = if total_executions > 0 {
            total_iterations as f64 / total_executions as f64
        } else {
            0.0
        };

        let total_cost: f64 = histories.values()
            .map(|h| h.total_cost)
            .sum();

        serde_json::json!({
            "total_executions": total_executions,
            "completed_executions": completed_executions,
            "failed_executions": failed_executions,
            "success_rate": if total_executions > 0 {
                completed_executions as f64 / total_executions as f64
            } else {
                0.0
            },
            "total_iterations": total_iterations,
            "average_iterations_per_execution": avg_iterations,
            "total_cost": total_cost
        })
    }

    /// 导出执行历史
    pub async fn export_execution_history(&self, execution_id: &str) -> Result<String, String> {
        let history = self.get_execution_history(execution_id).await?
            .ok_or_else(|| format!("Execution history not found for: {}", execution_id))?;

        serde_json::to_string_pretty(&history)
            .map_err(|e| format!("Failed to serialize history: {}", e))
    }

    /// 导入执行历史
    pub async fn import_execution_history(&self, data: &str) -> Result<(), String> {
        let history: RalphLoopExecutionHistory = serde_json::from_str(data)
            .map_err(|e| format!("Failed to deserialize history: {}", e))?;

        let mut histories = self.ralph_loop_histories.write().await;
        histories.insert(history.execution_id.clone(), history);

        Ok(())
    }
}