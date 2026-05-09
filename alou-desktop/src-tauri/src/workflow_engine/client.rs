//! Workflow Client - Session 侧的轻量级客户端

use std::sync::Arc;
use crate::workflow_engine::engine::WorkflowEngine;

/// Workflow Client Handle
///
/// Session 侧的轻量级客户端，用于调用 WorkflowEngine
pub struct WorkflowClientHandle {
    engine: Arc<WorkflowEngine>,
}

impl WorkflowClientHandle {
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }
    
    /// 启动工作流
    pub async fn start_workflow(&self, workflow_id: &str) -> Result<String, String> {
        self.engine.start_execution(workflow_id).await
    }
    
    /// 获取执行状态
    pub async fn get_status(&self, execution_id: &str) -> Option<crate::workflow_engine::types::WorkflowExecution> {
        self.engine.get_execution_status(execution_id).await
    }
    
    /// 暂停执行
    pub async fn pause(&self, execution_id: &str) -> Result<(), String> {
        self.engine.pause_execution(execution_id).await
    }
    
    /// 恢复执行
    pub async fn resume(&self, execution_id: &str) -> Result<(), String> {
        self.engine.resume_execution(execution_id).await
    }
    
    /// 取消执行
    pub async fn cancel(&self, execution_id: &str) -> Result<(), String> {
        self.engine.cancel_execution(execution_id).await
    }
}

impl Clone for WorkflowClientHandle {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
        }
    }
}
