//! 工作流类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工作流定义（DAG）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub created_at: i64,
}

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub depends_on: Vec<String>,
}

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

/// 工作流执行实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub execution_id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub current_step: Option<String>,
    pub progress: f32,
    pub step_results: HashMap<String, StepResult>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error: Option<String>,
}

/// 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub status: ExecutionStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

impl WorkflowExecution {
    pub fn new(execution_id: String, workflow_id: String) -> Self {
        Self {
            execution_id,
            workflow_id,
            status: ExecutionStatus::Pending,
            current_step: None,
            progress: 0.0,
            step_results: HashMap::new(),
            started_at: chrono::Utc::now().timestamp(),
            completed_at: None,
            error: None,
        }
    }
}
