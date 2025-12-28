use async_trait::async_trait;
use serde_json::{json, Value};

use crate::agent::context::AgentContext;
use crate::agent::spec::{
    TaskSpec, StepSpec, StepType, Precondition, ExpectedOutcome,
    ValidationRule, ExecutionPlan, SpecMetadata, RetryConfig,
};
use crate::agent::spec_validator::{SpecValidator, ValidationResult};
use crate::mcp::registry::McpTool;
use crate::storage::kv::KvStore;
use crate::utils::error::{AloudError, Result};
use worker::console_log;

const SPEC_TTL_SECONDS: u64 = 24 * 60 * 60; // 24 hours
const TRACE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60; // 7 days

/// Execution trace for tracking spec execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionTrace {
    pub trace_id: String,
    pub task_id: String,
    pub session_id: String,
    pub status: ExecutionStatus,
    pub started_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    pub steps: Vec<ExecutionStepRecord>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionStepRecord {
    pub step_id: String,
    pub started_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    pub status: StepStatus,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub attempt_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl ExecutionTrace {
    pub fn new(task_id: String, session_id: String) -> Self {
        let now = crate::utils::time::now_timestamp();
        Self {
            trace_id: format!("trace_{}_{}", task_id, now),
            task_id,
            session_id,
            status: ExecutionStatus::Pending,
            started_at: now,
            completed_at: None,
            steps: Vec::new(),
            summary: None,
        }
    }
    
    pub fn start_step(&mut self, step_id: String) {
        self.steps.push(ExecutionStepRecord {
            step_id,
            started_at: crate::utils::time::now_timestamp(),
            completed_at: None,
            status: StepStatus::Running,
            result: None,
            error: None,
            attempt_count: 0,
        });
    }
    
    pub fn complete_step(&mut self, step_id: String, result: Value) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.step_id == step_id) {
            step.status = StepStatus::Completed;
            step.result = Some(result);
            step.completed_at = Some(crate::utils::time::now_timestamp());
        }
    }
    
    pub fn fail_step(&mut self, step_id: String, error: String) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.step_id == step_id) {
            step.status = StepStatus::Failed;
            step.error = Some(error);
            step.completed_at = Some(crate::utils::time::now_timestamp());
        }
    }
    
    pub fn skip_step(&mut self, step_id: String, reason: String) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.step_id == step_id) {
            step.status = StepStatus::Skipped;
            step.error = Some(reason);
            step.completed_at = Some(crate::utils::time::now_timestamp());
        }
    }
}

/// Enhanced Spec Tool for managing task specifications
pub struct SpecEnhancedTool {
    kv: KvStore,
}

impl SpecEnhancedTool {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }
    
    fn spec_key(&self, spec_id: &str) -> String {
        format!("spec:{}", spec_id)
    }
    
    fn trace_key(&self, trace_id: &str) -> String {
        format!("trace:{}", trace_id)
    }
    
    async fn create_spec_internal(
        &self,
        spec_id: String,
        spec: TaskSpec,
        _context: &AgentContext,
    ) -> Result<Value> {
        let key = self.spec_key(&spec_id);
        self.kv
            .put(&key, &spec, Some(SPEC_TTL_SECONDS))
            .await?;
        
        console_log!("Created spec: {}", spec_id);
        
        Ok(json!({
            "spec_id": spec_id,
            "status": "created",
            "message": "任务规范已创建",
            "spec": spec
        }))
    }
    
    async fn validate_spec_internal(
        &self,
        spec_id: &str,
        _context: &AgentContext,
    ) -> Result<Value> {
        let key = self.spec_key(spec_id);
        let spec = self.kv.get::<TaskSpec>(&key).await?
            .ok_or_else(|| AloudError::InvalidInput(format!("Spec not found: {}", spec_id)))?;
        
        let validator = SpecValidator;
        let result = validator.validate(&spec);
        
        Ok(json!({
            "spec_id": spec_id,
            "is_valid": result.is_valid,
            "errors": result.errors,
            "warnings": result.warnings
        }))
    }
    
    async fn generate_plan_internal(
        &self,
        spec_id: &str,
        _context: &AgentContext,
    ) -> Result<Value> {
        let key = self.spec_key(spec_id);
        let spec = self.kv.get::<TaskSpec>(&key).await?
            .ok_or_else(|| AloudError::InvalidInput(format!("Spec not found: {}", spec_id)))?;
        
        let validator = SpecValidator;
        let plan = validator.generate_execution_plan(&spec);
        
        console_log!("Generated execution plan for spec {}", spec_id);
        
        Ok(json!({
            "spec_id": spec_id,
            "plan": plan
        }))
    }
    
    async fn start_execution_internal(
        &self,
        spec_id: &str,
        session_id: String,
        _context: &AgentContext,
    ) -> Result<Value> {
        let key = self.spec_key(spec_id);
        let spec = self.kv.get::<TaskSpec>(&key).await?
            .ok_or_else(|| AloudError::InvalidInput(format!("Spec not found: {}", spec_id)))?;
        
        let trace = ExecutionTrace::new(spec_id.clone(), session_id);
        
        let trace_key = self.trace_key(&trace.trace_id);
        self.kv
            .put(&trace_key, &trace, Some(TRACE_TTL_SECONDS))
            .await?;
        
        console_log!("Started execution of spec {}", spec_id);
        
        Ok(json!({
            "spec_id": spec_id,
            "trace_id": trace.trace_id,
            "status": "started",
            "message": "开始执行任务规范"
        }))
    }
    
    async fn get_status_internal(
        &self,
        trace_id: &str,
        _context: &AgentContext,
    ) -> Result<Value> {
        let key = self.trace_key(trace_id);
        let trace = self.kv.get::<ExecutionTrace>(&key).await?
            .ok_or_else(|| AloudError::InvalidInput(format!("Trace not found: {}", trace_id)))?;
        
        let completed_steps = trace.steps.iter()
            .filter(|s| matches!(s.status, StepStatus::Completed))
            .count();
        let failed_steps = trace.steps.iter()
            .filter(|s| matches!(s.status, StepStatus::Failed))
            .count();
        
        Ok(json!({
            "trace_id": trace.trace_id,
            "task_id": trace.task_id,
            "status": format!("{:?}", trace.status),
            "started_at": trace.started_at,
            "completed_at": trace.completed_at,
            "total_steps": trace.steps.len(),
            "completed_steps": completed_steps,
            "failed_steps": failed_steps,
            "steps": trace.steps
        }))
    }
    
    async fn generate_report_internal(
        &self,
        trace_id: &str,
        _context: &AgentContext,
    ) -> Result<Value> {
        let key = self.trace_key(trace_id);
        let trace = self.kv.get::<ExecutionTrace>(&key).await?
            .ok_or_else(|| AloudError::InvalidInput(format!("Trace not found: {}", trace_id)))?;
        
        let completed_steps = trace.steps.iter()
            .filter(|s| matches!(s.status, StepStatus::Completed))
            .count();
        let failed_steps = trace.steps.iter()
            .filter(|s| matches!(s.status, StepStatus::Failed))
            .count();
        
        let success_rate = if trace.steps.is_empty() {
            0.0
        } else {
            (completed_steps as f64 / trace.steps.len() as f64) * 100.0
        };
        
        let message = if matches!(trace.status, ExecutionStatus::Completed) {
            "任务执行成功完成".to_string()
        } else if matches!(trace.status, ExecutionStatus::Failed) {
            "任务执行失败".to_string()
        } else {
            "任务执行中".to_string()
        };
        
        Ok(json!({
            "trace_id": trace.trace_id,
            "task_id": trace.task_id,
            "status": format!("{:?}", trace.status),
            "message": message,
            "success_rate": success_rate,
            "completed_steps": completed_steps,
            "failed_steps": failed_steps,
            "total_steps": trace.steps.len(),
            "started_at": trace.started_at,
            "completed_at": trace.completed_at,
            "duration_ms": trace.completed_at.map(|c| c - trace.started_at)
        }))
    }
}

#[async_trait(?Send)]
impl McpTool for SpecEnhancedTool {
    fn name(&self) -> &str {
        "spec_enhanced"
    }
    
    fn description(&self) -> &str {
        "增强的任务规范工具。支持：
1. create - 创建任务规范
2. validate - 验证规范完整性
3. plan - 生成执行计划
4. start - 开始执行
5. status - 获取执行状态
6. report - 生成执行报告
7. delete - 删除规范"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "validate", "plan", "start", "status", "report", "delete"],
                    "description": "要执行的操作"
                },
                "spec_id": {
                    "type": "string",
                    "description": "规范 ID (用于 validate, plan, start, status, report, delete)"
                },
                "spec": {
                    "type": "object",
                    "description": "任务规范内容 (用于 create)"
                },
                "session_id": {
                    "type": "string",
                    "description": "会话 ID (用于 start)"
                }
            },
            "required": ["action"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing action".to_string()))?;
        
        match action {
            "create" => {
                let spec_json = args
                    .get("spec")
                    .ok_or_else(|| AloudError::InvalidInput("Missing spec for create".to_string()))?;
                
                let spec: TaskSpec = serde_json::from_value(spec_json.clone())
                    .map_err(|e| AloudError::InvalidInput(format!("Invalid spec JSON: {}", e)))?;
                
                let spec_id = spec.task_id.clone();
                self.create_spec_internal(spec_id, spec, context).await
            },
            "validate" => {
                let spec_id = args
                    .get("spec_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing spec_id for validate".to_string()))?;
                
                self.validate_spec_internal(spec_id.to_string(), context).await
            },
            "plan" => {
                let spec_id = args
                    .get("spec_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing spec_id for plan".to_string()))?;
                
                self.generate_plan_internal(spec_id.to_string(), context).await
            },
            "start" => {
                let spec_id = args
                    .get("spec_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing spec_id for start".to_string()))?;
                
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing session_id for start".to_string()))?;
                
                self.start_execution_internal(spec_id.to_string(), session_id.to_string(), context).await
            },
            "status" => {
                let trace_id = args
                    .get("trace_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing trace_id for status".to_string()))?;
                
                self.get_status_internal(trace_id.to_string(), context).await
            },
            "report" => {
                let trace_id = args
                    .get("trace_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing trace_id for report".to_string()))?;
                
                self.generate_report_internal(trace_id.to_string(), context).await
            },
            "delete" => {
                let spec_id = args
                    .get("spec_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AloudError::InvalidInput("Missing spec_id for delete".to_string()))?;
                
                let key = self.spec_key(spec_id);
                self.kv.delete(&key).await?;
                
                Ok(json!({
                    "spec_id": spec_id,
                    "status": "deleted",
                    "message": "任务规范已删除"
                }))
            },
            _ => Err(AloudError::InvalidInput(format!("Unknown action: {}", action)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_execution_trace_creation() {
        let trace = ExecutionTrace::new("task_001".to_string(), "session_123".to_string());
        
        assert_eq!(trace.task_id, "task_001");
        assert_eq!(trace.session_id, "session_123");
        assert!(matches!(trace.status, ExecutionStatus::Pending));
        assert!(trace.started_at > 0);
    }
    
    #[test]
    fn test_start_step() {
        let mut trace = ExecutionTrace::new("task_001".to_string(), "session_123".to_string());
        trace.start_step("step_1".to_string());
        
        assert_eq!(trace.steps.len(), 1);
        assert_eq!(trace.steps[0].step_id, "step_1");
        assert!(matches!(trace.steps[0].status, StepStatus::Running));
    }
    
    #[test]
    fn test_complete_step() {
        let mut trace = ExecutionTrace::new("task_001".to_string(), "session_123".to_string());
        trace.start_step("step_1".to_string());
        trace.complete_step("step_1".to_string(), json!({"result": "success"}));
        
        assert!(matches!(trace.steps[0].status, StepStatus::Completed));
        assert!(trace.steps[0].completed_at.is_some());
        assert!(trace.steps[0].result.is_some());
    }
}
