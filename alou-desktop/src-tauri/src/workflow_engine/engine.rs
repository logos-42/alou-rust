//! Workflow Engine - 执行引擎

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::workflow_engine::types::{Workflow, WorkflowExecution, ExecutionStatus, StepResult};

/// Workflow Engine
///
/// 独立的确定性状态机，管理所有工作流执行
pub struct WorkflowEngine {
    /// 工作流定义存储
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    
    /// 活跃的执行实例
    executions: Arc<RwLock<HashMap<String, WorkflowExecution>>>,
    
    /// 执行历史
    history: Arc<RwLock<Vec<WorkflowExecution>>>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 注册工作流定义
    pub async fn register_workflow(&self, workflow: Workflow) {
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow);
    }
    
    /// 启动工作流执行
    pub async fn start_execution(&self, workflow_id: &str) -> Result<String, String> {
        // 获取工作流定义
        let workflows = self.workflows.read().await;
        let workflow = workflows
            .get(workflow_id)
            .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?
            .clone();
        drop(workflows);
        
        // 创建执行实例
        let execution_id = format!("exec_{}", uuid::Uuid::new_v4());
        let mut execution = WorkflowExecution::new(execution_id.clone(), workflow_id.to_string());
        execution.status = ExecutionStatus::Running;
        
        // 存储执行实例
        let mut executions = self.executions.write().await;
        executions.insert(execution_id.clone(), execution);
        drop(executions);
        
        // 启动执行任务
        let engine = Self {
            workflows: self.workflows.clone(),
            executions: self.executions.clone(),
            history: self.history.clone(),
        };
        
        tokio::spawn(async move {
            engine.execute_workflow(execution_id.clone(), workflow).await;
        });
        
        Ok(execution_id)
    }
    
    /// 执行工作流（内部方法）
    async fn execute_workflow(&self, execution_id: String, workflow: Workflow) {
        log::info!("[WorkflowEngine] Starting execution: {}", execution_id);
        
        // 按顺序执行步骤（简化版本，实际应该支持 DAG 并行）
        for step in &workflow.steps {
            // 更新当前步骤
            {
                let mut executions = self.executions.write().await;
                if let Some(execution) = executions.get_mut(&execution_id) {
                    execution.current_step = Some(step.id.clone());
                    execution.progress = (workflow.steps.iter()
                        .position(|s| s.id == step.id)
                        .unwrap_or(0) as f32) / workflow.steps.len() as f32 * 100.0;
                }
            }
            
            // 执行步骤（简化：实际应该调用工具）
            let step_result = StepResult {
                step_id: step.id.clone(),
                status: ExecutionStatus::Completed,
                result: Some(serde_json::json!({"message": format!("Step {} completed", step.name)})),
                error: None,
                started_at: chrono::Utc::now().timestamp(),
                completed_at: Some(chrono::Utc::now().timestamp()),
            };
            
            // 存储步骤结果
            {
                let mut executions = self.executions.write().await;
                if let Some(execution) = executions.get_mut(&execution_id) {
                    execution.step_results.insert(step.id.clone(), step_result);
                }
            }
        }
        
        // 标记执行完成
        {
            let mut executions = self.executions.write().await;
            if let Some(execution) = executions.remove(&execution_id) {
                let mut completed = execution.clone();
                completed.status = ExecutionStatus::Completed;
                completed.completed_at = Some(chrono::Utc::now().timestamp());
                completed.progress = 100.0;
                
                let mut history = self.history.write().await;
                history.push(completed);
            }
        }
        
        log::info!("[WorkflowEngine] Execution completed: {}", execution_id);
    }
    
    /// 获取执行状态
    pub async fn get_execution_status(&self, execution_id: &str) -> Option<WorkflowExecution> {
        let executions = self.executions.read().await;
        executions.get(execution_id).cloned()
    }
    
    /// 暂停执行
    pub async fn pause_execution(&self, execution_id: &str) -> Result<(), String> {
        let mut executions = self.executions.write().await;
        let execution = executions
            .get_mut(execution_id)
            .ok_or_else(|| "Execution not found".to_string())?;
        
        execution.status = ExecutionStatus::Paused;
        Ok(())
    }
    
    /// 恢复执行
    pub async fn resume_execution(&self, execution_id: &str) -> Result<(), String> {
        let mut executions = self.executions.write().await;
        let execution = executions
            .get_mut(execution_id)
            .ok_or_else(|| "Execution not found".to_string())?;
        
        execution.status = ExecutionStatus::Running;
        Ok(())
    }
    
    /// 取消执行
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), String> {
        let mut executions = self.executions.write().await;
        let execution = executions
            .get_mut(execution_id)
            .ok_or_else(|| "Execution not found".to_string())?;
        
        execution.status = ExecutionStatus::Cancelled;
        Ok(())
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}
