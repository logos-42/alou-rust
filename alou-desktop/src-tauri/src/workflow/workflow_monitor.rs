//! 工作流监控和日志记录
//!
//! 提供工作流执行的监控、日志记录和性能分析功能

use super::{ExecutionStatus, WorkflowExecution, StepResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 执行日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogEntry {
    pub timestamp: i64,
    pub level: LogLevel,
    pub execution_id: String,
    pub workflow_id: String,
    pub step_id: Option<String>,
    pub message: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub total_steps_executed: u64,
    pub average_steps_per_execution: f64,
    pub retry_rate: f64,
    pub error_rate: f64,
}

/// 工作流监控器
pub struct WorkflowMonitor {
    /// 执行日志
    logs: Arc<RwLock<Vec<ExecutionLogEntry>>>,
    /// 性能指标
    metrics: Arc<RwLock<PerformanceMetrics>>,
    /// 最大日志条目数
    max_logs: usize,
}

impl WorkflowMonitor {
    /// 创建新的监控器
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                average_execution_time_ms: 0.0,
                total_steps_executed: 0,
                average_steps_per_execution: 0.0,
                retry_rate: 0.0,
                error_rate: 0.0,
            })),
            max_logs,
        }
    }

    /// 记录执行开始
    pub async fn log_execution_started(&self, execution: &WorkflowExecution) {
        self.log(LogLevel::Info, &execution.execution_id, &execution.workflow_id, None, "Execution started", HashMap::new()).await;

        // 更新指标
        let mut metrics = self.metrics.write().await;
        metrics.total_executions += 1;
    }

    /// 记录步骤开始
    pub async fn log_step_started(&self, execution_id: &str, workflow_id: &str, step_id: &str) {
        self.log(LogLevel::Debug, execution_id, workflow_id, Some(step_id.to_string()), "Step started", HashMap::new()).await;
    }

    /// 记录步骤完成
    pub async fn log_step_completed(&self, execution_id: &str, workflow_id: &str, step_result: &StepResult) {
        let mut metadata = HashMap::new();
        metadata.insert("retry_count".to_string(), serde_json::json!(step_result.retry_count));
        metadata.insert("execution_time_ms".to_string(), serde_json::json!(step_result.execution_time_ms));

        self.log(LogLevel::Info, execution_id, workflow_id, Some(step_result.step_id.clone()), "Step completed", metadata).await;

        // 更新指标
        let mut metrics = self.metrics.write().await;
        metrics.total_steps_executed += 1;
        if step_result.retry_count > 0 {
            // 这里可以计算重试率，但需要更复杂的逻辑
        }
    }

    /// 记录步骤失败
    pub async fn log_step_failed(&self, execution_id: &str, workflow_id: &str, step_result: &StepResult) {
        let mut metadata = HashMap::new();
        metadata.insert("retry_count".to_string(), serde_json::json!(step_result.retry_count));
        metadata.insert("error".to_string(), serde_json::json!(step_result.error));

        self.log(LogLevel::Warn, execution_id, workflow_id, Some(step_result.step_id.clone()), "Step failed", metadata).await;
    }

    /// 记录执行完成
    pub async fn log_execution_completed(&self, execution: &WorkflowExecution) {
        let mut metadata = HashMap::new();
        metadata.insert("status".to_string(), serde_json::json!(execution.status));
        metadata.insert("total_execution_time_ms".to_string(), serde_json::json!(execution.total_execution_time_ms));
        metadata.insert("steps_count".to_string(), serde_json::json!(execution.step_results.len()));

        let level = match execution.status {
            ExecutionStatus::Completed => LogLevel::Info,
            ExecutionStatus::Failed => LogLevel::Error,
            _ => LogLevel::Warn,
        };

        let message = match execution.status {
            ExecutionStatus::Completed => "Execution completed successfully",
            ExecutionStatus::Failed => "Execution failed",
            ExecutionStatus::Cancelled => "Execution cancelled",
            _ => "Execution finished",
        };

        self.log(level, &execution.execution_id, &execution.workflow_id, None, message, metadata).await;

        // 更新指标
        let mut metrics = self.metrics.write().await;
        match execution.status {
            ExecutionStatus::Completed => {
                metrics.successful_executions += 1;
            }
            ExecutionStatus::Failed => {
                metrics.failed_executions += 1;
            }
            _ => {}
        }

        // 更新平均执行时间
        if let Some(total_time) = execution.total_execution_time_ms {
            let total_time_f64 = total_time as f64;
            let count = metrics.successful_executions + metrics.failed_executions;
            if count > 0 {
                metrics.average_execution_time_ms = (metrics.average_execution_time_ms * (count - 1) as f64 + total_time_f64) / count as f64;
            }
        }

        // 更新平均步骤数
        let steps_count = execution.step_results.len() as f64;
        let count = metrics.successful_executions + metrics.failed_executions;
        if count > 0 {
            metrics.average_steps_per_execution = (metrics.average_steps_per_execution * (count - 1) as f64 + steps_count) / count as f64;
        }

        // 计算错误率
        if metrics.total_executions > 0 {
            metrics.error_rate = metrics.failed_executions as f64 / metrics.total_executions as f64;
        }
    }

    /// 记录进度更新
    pub async fn log_progress_update(&self, execution_id: &str, workflow_id: &str, progress: f32, current_step: Option<&str>) {
        let mut metadata = HashMap::new();
        metadata.insert("progress".to_string(), serde_json::json!(progress));
        if let Some(step) = current_step {
            metadata.insert("current_step".to_string(), serde_json::json!(step));
        }

        self.log(LogLevel::Debug, execution_id, workflow_id, None, "Progress updated", metadata).await;
    }

    /// 获取日志条目
    pub async fn get_logs(&self, execution_id: Option<&str>, limit: Option<usize>) -> Vec<ExecutionLogEntry> {
        let logs = self.logs.read().await;
        let mut filtered_logs: Vec<ExecutionLogEntry> = if let Some(exec_id) = execution_id {
            logs.iter().filter(|log| log.execution_id == exec_id).cloned().collect()
        } else {
            logs.clone()
        };

        // 按时间戳排序（最新的在前）
        filtered_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 应用限制
        if let Some(limit) = limit {
            filtered_logs.truncate(limit);
        }

        filtered_logs
    }

    /// 获取性能指标
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// 清理旧日志
    pub async fn cleanup_old_logs(&self) {
        let mut logs = self.logs.write().await;
        if logs.len() > self.max_logs {
            // 保留最新的日志
            logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            logs.truncate(self.max_logs);
        }
    }

    /// 内部日志记录方法
    async fn log(
        &self,
        level: LogLevel,
        execution_id: &str,
        workflow_id: &str,
        step_id: Option<String>,
        message: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) {
        let entry = ExecutionLogEntry {
            timestamp: chrono::Utc::now().timestamp(),
            level,
            execution_id: execution_id.to_string(),
            workflow_id: workflow_id.to_string(),
            step_id,
            message: message.to_string(),
            metadata,
        };

        let mut logs = self.logs.write().await;
        logs.push(entry);

        // 清理旧日志
        self.cleanup_old_logs().await;
    }
}

impl Default for WorkflowMonitor {
    fn default() -> Self {
        Self::new(10000) // 默认保留10000条日志
    }
}