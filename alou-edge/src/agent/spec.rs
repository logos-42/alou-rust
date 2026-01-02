use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Task specification for planned execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskSpec {
    pub task_id: String,
    pub title: String,
    pub description: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    
    pub preconditions: Vec<Precondition>,
    pub steps: Vec<StepSpec>,
    pub expected_outcome: ExpectedOutcome,
    pub validation_rules: Vec<ValidationRule>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SpecMetadata>,
    
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

impl TaskSpec {
    #[allow(dead_code)]
    pub fn new(task_id: String, title: String, description: String) -> Self {
        let now = crate::utils::time::now_timestamp();
        Self {
            task_id,
            title,
            description,
            category: None,
            preconditions: Vec::new(),
            steps: Vec::new(),
            expected_outcome: ExpectedOutcome {
                success_criteria: None,
                expected_value: None,
                acceptance_criteria: Vec::new(),
            },
            validation_rules: Vec::new(),
            metadata: None,
            created_at: now,
            updated_at: None,
        }
    }
}

/// Precondition for task execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Precondition {
    pub condition_id: String,
    pub description: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_tool: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_args: Option<Value>,
    
    #[serde(default)]
    pub required: bool,
}

/// Task execution step
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepSpec {
    pub step_id: String,
    pub step_type: StepType,
    pub description: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
    
    pub dependencies: Vec<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_result: Option<Value>,
    
    #[serde(default)]
    pub required: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_config: Option<RetryConfig>,
}

/// Step execution type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    ToolCall,
    Condition,
    Loop,
    Parallel,
    ManualAction,
}

impl Default for StepType {
    fn default() -> Self {
        StepType::ToolCall
    }
}

/// Retry configuration for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_delay_ms: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_strategy: Option<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: Some(1000),
            backoff_strategy: Some("exponential".to_string()),
        }
    }
}

/// Expected outcome of task
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpectedOutcome {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_criteria: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_value: Option<Value>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<String>,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_id: String,
    pub rule_type: ValidationRuleType,
    pub description: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_tool: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_args: Option<Value>,
    
    #[serde(default)]
    pub severity: ValidationSeverity,
}

/// Validation rule type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRuleType {
    OutputFormat,
    OutputRange,
    OutputPattern,
    CustomCheck,
}

impl Default for ValidationRuleType {
    fn default() -> Self {
        ValidationRuleType::OutputFormat
    }
}

/// Validation severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for ValidationSeverity {
    fn default() -> Self {
        ValidationSeverity::Info
    }
}

/// Spec metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Execution plan from spec
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct ExecutionPlan {
    pub task_id: String,
    pub phases: Vec<ExecutionPhase>,
    pub total_steps: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration_ms: Option<u64>,
    pub created_at: i64,
}

impl ExecutionPlan {
    #[allow(dead_code)]
    pub fn new(task_id: String, phases: Vec<ExecutionPhase>, total_steps: usize) -> Self {
        Self {
            task_id,
            phases,
            total_steps,
            estimated_duration_ms: None,
            created_at: crate::utils::time::now_timestamp(),
        }
    }
}

/// Execution phase (group of steps)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct ExecutionPhase {
    pub phase_id: String,
    pub name: String,
    pub step_ids: Vec<String>,
    #[serde(default)]
    pub can_execute_in_parallel: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ValidationResult {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_error(&mut self, error: String) -> &mut Self {
        self.errors.push(error);
        self.is_valid = false;
        self
    }

    pub fn with_warning(&mut self, warning: String) -> &mut Self {
        self.warnings.push(warning);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_spec_creation() {
        let spec = TaskSpec::new(
            "task_001".to_string(),
            "Test Task".to_string(),
            "Test description".to_string(),
        );
        
        assert_eq!(spec.task_id, "task_001");
        assert_eq!(spec.title, "Test Task");
        assert_eq!(spec.description, "Test description");
        assert!(spec.steps.is_empty());
        assert!(spec.created_at > 0);
    }
    
    #[test]
    fn test_step_spec() {
        let step = StepSpec {
            step_id: "step_001".to_string(),
            step_type: StepType::ToolCall,
            description: "Execute tool".to_string(),
            tool: Some("query".to_string()),
            args: Some(serde_json::json!({"test": true})),
            dependencies: Vec::new(),
            expected_result: None,
            required: true,
            timeout_ms: None,
            retry_config: None,
        };
        
        assert_eq!(step.step_type, StepType::ToolCall);
        assert_eq!(step.tool, Some("query".to_string()));
        assert!(step.dependencies.is_empty());
    }
    
    #[test]
    fn test_validation_result() {
        let result = ValidationResult::new()
            .with_error("Test error".to_string())
            .with_warning("Test warning".to_string());
        
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1);
    }
}
