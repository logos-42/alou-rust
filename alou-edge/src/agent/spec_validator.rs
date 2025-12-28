use crate::agent::spec::{
    StepSpec, StepType, TaskSpec, Precondition, ValidationRule,
    ExecutionPlan, ExecutionPhase, ValidationResult,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Spec validator for validating task specifications
pub struct SpecValidator;

impl SpecValidator {
    /// Validate a complete task spec
    pub fn validate(&self, spec: &TaskSpec) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Validate basic fields
        if spec.task_id.is_empty() {
            result = result.with_error("task_id 不能为空".to_string());
        }
        
        if spec.title.trim().is_empty() {
            result = result.with_error("title 不能为空".to_string());
        }
        
        if spec.description.trim().is_empty() {
            result = result.with_error("description 不能为空".to_string());
        }
        
        // Validate steps
        self.validate_steps(&spec.steps, &mut result);
        
        // Validate dependencies
        self.validate_dependencies(&spec.steps, &mut result);
        
        // Validate preconditions
        self.validate_preconditions(&spec.preconditions, &mut result);
        
        // Validate expected outcome
        self.validate_expected_outcome(&spec.expected_outcome, &mut result);
        
        // Validate validation rules
        self.validate_validation_rules(&spec.validation_rules, &mut result);
        
        // Check for circular dependencies
        if self.has_circular_dependencies(&spec.steps) {
            result = result.with_error("检测到循环依赖".to_string());
        }
        
        // Warnings
        if spec.steps.is_empty() {
            result = result.with_warning("任务没有定义任何步骤".to_string());
        }
        
        if spec.steps.len() > 20 {
            result = result.with_warning("任务步骤过多，建议拆分为多个子任务".to_string());
        }
        
        result
    }
    
    /// Validate steps
    fn validate_steps(&self, steps: &[StepSpec], result: &mut ValidationResult) {
        let mut step_ids = HashSet::new();
        
        for (i, step) in steps.iter().enumerate() {
            // Validate step ID
            if step.step_id.is_empty() {
                result = result.with_error(format!("步骤 {} (索引 {}) 的 step_id 为空", i, i));
            }
            
            // Check for duplicate step IDs
            if step_ids.contains(&step.step_id) {
                result = result.with_error(format!("重复的 step_id: {}", step.step_id));
            } else {
                step_ids.insert(step.step_id.clone());
            }
            
            // Validate description
            if step.description.trim().is_empty() {
                result = result.with_warning(format!("步骤 {} 没有描述", step.step_id));
            }
            
            // Validate tool call steps have tool specified
            if step.step_type == StepType::ToolCall && step.tool.is_none() {
                result = result.with_error(format!(
                    "步骤 {} (ToolCall 类型) 必须指定 tool",
                    step.step_id
                ));
            }
            
            // Validate tool call steps have args if tool is specified
            if step.step_type == StepType::ToolCall {
                if step.tool.is_some() && step.args.is_none() {
                    result = result.with_warning(format!(
                        "步骤 {} 指定了 tool 但没有 args",
                        step.step_id
                    ));
                }
            }
            
            // Validate timeout is reasonable
            if let Some(timeout) = step.timeout_ms {
                if timeout > 300000 { // 5 minutes
                    result = result.with_warning(format!(
                        "步骤 {} 的超时时间过长 ({}ms > 300000ms)",
                        step.step_id, timeout
                    ));
                }
                if timeout == 0 {
                    result = result.with_error(format!(
                        "步骤 {} 的超时时间不能为 0",
                        step.step_id
                    ));
                }
            }
            
            // Validate retry config
            if let Some(ref retry_config) = step.retry_config {
                if retry_config.max_attempts == 0 {
                    result = result.with_error(format!(
                        "步骤 {} 的最大重试次数不能为 0",
                        step.step_id
                    ));
                }
                
                if retry_config.max_attempts > 10 {
                    result = result.with_warning(format!(
                        "步骤 {} 的最大重试次数过多 ({} > 10)",
                        step.step_id, retry_config.max_attempts
                    ));
                }
            }
        }
    }
    
    /// Validate step dependencies
    fn validate_dependencies(&self, steps: &[StepSpec], result: &mut ValidationResult) {
        let step_ids: HashSet<_> = steps.iter().map(|s| s.step_id.as_str()).collect();
        
        for step in steps {
            for dep_id in &step.dependencies {
                // Check if dependency exists
                if !step_ids.contains(dep_id.as_str()) {
                    result = result.with_error(format!(
                        "步骤 {} 依赖不存在的步骤 {}",
                        step.step_id, dep_id
                    ));
                }
                
                // Check if step depends on itself
                if step.step_id == *dep_id {
                    result = result.with_error(format!(
                        "步骤 {} 不能依赖自身",
                        step.step_id
                    ));
                }
            }
        }
    }
    
    /// Validate preconditions
    fn validate_preconditions(&self, preconditions: &[Precondition], result: &mut ValidationResult) {
        for (i, cond) in preconditions.iter().enumerate() {
            if cond.condition_id.is_empty() {
                result = result.with_error(format!("前置条件 {} (索引 {}) 的 ID 为空", i, i));
            }
            
            if cond.description.trim().is_empty() {
                result = result.with_warning(format!("前置条件 {} 没有描述", cond.condition_id));
            }
            
            // If check_tool is specified, it should not be empty
            if let Some(ref tool) = cond.check_tool {
                if tool.is_empty() {
                    result = result.with_error(format!(
                        "前置条件 {} 的 check_tool 为空字符串",
                        cond.condition_id
                    ));
                }
            }
        }
    }
    
    /// Validate expected outcome
    fn validate_expected_outcome(&self, outcome: &crate::agent::spec::ExpectedOutcome, result: &mut ValidationResult) {
        if let Some(ref criteria) = outcome.success_criteria {
            if criteria.trim().is_empty() {
                result = result.with_warning("success_criteria 为空字符串".to_string());
            }
        }
        
        if outcome.acceptance_criteria.is_empty() {
            result = result.with_warning("没有定义验收标准 (acceptance_criteria)".to_string());
        }
    }
    
    /// Validate validation rules
    fn validate_validation_rules(&self, rules: &[ValidationRule], result: &mut ValidationResult) {
        for (i, rule) in rules.iter().enumerate() {
            if rule.rule_id.is_empty() {
                result = result.with_error(format!("验证规则 {} (索引 {}) 的 ID 为空", i, i));
            }
            
            if rule.description.trim().is_empty() {
                result = result.with_warning(format!("验证规则 {} 没有描述", rule.rule_id));
            }
            
            // CustomCheck rules must have check_tool
            if rule.rule_type == crate::agent::spec::ValidationRuleType::CustomCheck 
                && rule.check_tool.is_none() {
                result = result.with_error(format!(
                    "验证规则 {} (CustomCheck 类型) 必须指定 check_tool",
                    rule.rule_id
                ));
            }
        }
    }
    
    /// Check for circular dependencies in steps
    fn has_circular_dependencies(&self, steps: &[StepSpec]) -> bool {
        if steps.is_empty() {
            return false;
        }
        
        // Build adjacency list
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        for step in steps {
            let deps: Vec<&str> = step.dependencies.iter().map(|s| s.as_str()).collect();
            graph.insert(step.step_id.as_str(), deps);
        }
        
        // Use DFS to detect cycles
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for step_id in graph.keys() {
            if self.has_cycle_dfs(*step_id, &graph, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        
        false
    }
    
    /// DFS helper for cycle detection
    fn has_cycle_dfs<'a>(
        &self,
        node: &'a str,
        graph: &HashMap<&'a str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);
        
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(*neighbor) {
                    if self.has_cycle_dfs(*neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(*neighbor) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(node);
        false
    }
    
    /// Generate execution plan from spec
    pub fn generate_execution_plan(&self, spec: &TaskSpec) -> ExecutionPlan {
        // Topological sort for execution order
        let execution_order = self.topological_sort(&spec.steps);
        
        // Group into phases based on dependencies
        let phases = self.group_into_phases(execution_order.clone(), &spec.steps);
        
        // Estimate duration
        let estimated_duration_ms = self.estimate_duration(&spec.steps);
        
        let total_steps = spec.steps.len();
        
        ExecutionPlan {
            task_id: spec.task_id.clone(),
            phases,
            total_steps,
            estimated_duration_ms: Some(estimated_duration_ms),
            created_at: crate::utils::time::now_timestamp(),
        }
    }
    
    /// Topological sort of steps
    fn topological_sort(&self, steps: &[StepSpec]) -> Vec<StepSpec> {
        if steps.is_empty() {
            return Vec::new();
        }
        
        // Build adjacency list and in-degree map
        let step_map: HashMap<&str, &StepSpec> = steps
            .iter()
            .map(|s| (s.step_id.as_str(), s))
            .collect();
        
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj_list: HashMap<&str, Vec<&str>> = HashMap::new();
        
        for step in steps {
            let step_id = step.step_id.as_str();
            in_degree.entry(step_id).or_insert(0);
            adj_list.entry(step_id).or_insert_with(Vec::new);
            
            for dep_id in &step.dependencies {
                adj_list.entry(dep_id.as_str())
                    .or_insert_with(Vec::new)
                    .push(step_id);
                *in_degree.entry(step_id).or_insert(0) += 1;
            }
        }
        
        // Kahn's algorithm for topological sort
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        
        let mut sorted_steps = Vec::new();
        
        while !queue.is_empty() {
            // Sort queue for deterministic output (by step_id)
            queue.sort();
            
            let current = queue.remove(0);
            
            if let Some(step) = step_map.get(current) {
                sorted_steps.push((*step).clone());
            }
            
            if let Some(neighbors) = adj_list.get(current) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(*neighbor) {
                        if *deg > 0 {
                            *deg -= 1;
                            if *deg == 0 {
                                queue.push(*neighbor);
                            }
                        }
                    }
                }
            }
        }
        
        sorted_steps
    }
    
    /// Group steps into phases
    fn group_into_phases(&self, sorted_steps: Vec<StepSpec>, all_steps: &[StepSpec]) -> Vec<ExecutionPhase> {
        let mut phases = Vec::new();
        
        if sorted_steps.is_empty() {
            return phases;
        }
        
        // Phase 1: Steps with no dependencies
        let phase_1: Vec<String> = sorted_steps
            .iter()
            .filter(|s| s.dependencies.is_empty())
            .map(|s| s.step_id.clone())
            .collect();
        
        if !phase_1.is_empty() {
            phases.push(ExecutionPhase {
                phase_id: "phase_1".to_string(),
                name: "初始化步骤".to_string(),
                step_ids: phase_1,
                can_execute_in_parallel: true,
                dependencies: Vec::new(),
            });
        }
        
        // Phase 2: Steps that depend on Phase 1
        let phase_1_ids: HashSet<_> = phases[0].step_ids.iter().collect();
        let phase_2: Vec<String> = sorted_steps
            .iter()
            .filter(|s| {
                s.dependencies.iter().any(|dep| phase_1_ids.contains(dep))
                    && !s.dependencies.iter().any(|dep| !phase_1_ids.contains(dep))
            })
            .map(|s| s.step_id.clone())
            .collect();
        
        if !phase_2.is_empty() {
            phases.push(ExecutionPhase {
                phase_id: "phase_2".to_string(),
                name: "依赖执行步骤".to_string(),
                step_ids: phase_2,
                can_execute_in_parallel: false,
                dependencies: vec!["phase_1".to_string()],
            });
        }
        
        // Phase 3: Remaining steps
        let phase_1_2_ids: HashSet<_> = phases
            .iter()
            .flat_map(|p| p.step_ids.iter())
            .collect();
        let phase_3: Vec<String> = sorted_steps
            .iter()
            .filter(|s| !phase_1_2_ids.contains(&s.step_id))
            .map(|s| s.step_id.clone())
            .collect();
        
        if !phase_3.is_empty() {
            phases.push(ExecutionPhase {
                phase_id: "phase_3".to_string(),
                name: "完成步骤".to_string(),
                step_ids: phase_3,
                can_execute_in_parallel: false,
                dependencies: vec!["phase_2".to_string()],
            });
        }
        
        phases
    }
    
    /// Estimate total execution duration
    fn estimate_duration(&self, steps: &[StepSpec]) -> u64 {
        let mut total_ms = 0u64;
        
        for step in steps {
            // Base duration by step type
            let base_duration = match step.step_type {
                StepType::ToolCall => 5000,   // 5 seconds
                StepType::Condition => 1000,  // 1 second
                StepType::Loop => 10000,     // 10 seconds
                StepType::Parallel => 5000,   // 5 seconds (average of parallel tasks)
                StepType::ManualAction => 60000, // 1 minute (user action)
            };
            
            // Add configured timeout
            let duration = step.timeout_ms.unwrap_or(base_duration);
            
            // Add retry overhead
            let retry_overhead = step.retry_config.as_ref()
                .map(|rc| rc.max_attempts * 1000)
                .unwrap_or(0);
            
            total_ms += duration + retry_overhead;
        }
        
        total_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::spec::{TaskSpec, StepSpec, StepType};
    use serde_json::json;
    
    #[test]
    fn test_validate_empty_spec() {
        let spec = TaskSpec::new(
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        
        let validator = SpecValidator;
        let result = validator.validate(&spec);
        
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("task_id")));
    }
    
    #[test]
    fn test_validate_valid_spec() {
        let spec = TaskSpec::new(
            "task_001".to_string(),
            "Test Task".to_string(),
            "Test Description".to_string(),
        );
        
        let result = SpecValidator::validate(&spec).validate(&spec);
        
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
    
    #[test]
    fn test_validate_circular_dependency() {
        let mut spec = TaskSpec::new(
            "task_001".to_string(),
            "Test Task".to_string(),
            "Test Description".to_string(),
        );
        
        spec.steps = vec![
            StepSpec {
                step_id: "step_1".to_string(),
                step_type: StepType::ToolCall,
                description: "Step 1".to_string(),
                tool: Some("tool_a".to_string()),
                args: None,
                dependencies: vec!["step_2".to_string()],
                expected_result: None,
                required: true,
                timeout_ms: None,
                retry_config: None,
            },
            StepSpec {
                step_id: "step_2".to_string(),
                step_type: StepType::ToolCall,
                description: "Step 2".to_string(),
                tool: Some("tool_b".to_string()),
                args: None,
                dependencies: vec!["step_1".to_string()],
                expected_result: None,
                required: true,
                timeout_ms: None,
                retry_config: None,
            },
        ];
        
        let result = SpecValidator::new().validate(&spec);
        
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("循环依赖")));
    }
    
    #[test]
    fn test_topological_sort() {
        let steps = vec![
            StepSpec {
                step_id: "step_1".to_string(),
                step_type: StepType::ToolCall,
                description: "Step 1".to_string(),
                tool: Some("tool".to_string()),
                args: None,
                dependencies: vec![],
                expected_result: None,
                required: true,
                timeout_ms: None,
                retry_config: None,
            },
            StepSpec {
                step_id: "step_2".to_string(),
                step_type: StepType::ToolCall,
                description: "Step 2".to_string(),
                tool: Some("tool".to_string()),
                args: None,
                dependencies: vec!["step_1".to_string()],
                expected_result: None,
                required: true,
                timeout_ms: None,
                retry_config: None,
            },
        ];
        
        let sorted = SpecValidator::new().topological_sort(&steps);
        
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].step_id, "step_1");
        assert_eq!(sorted[1].step_id, "step_2");
    }
    
    #[test]
    fn test_estimate_duration() {
        let steps = vec![
            StepSpec {
                step_id: "step_1".to_string(),
                step_type: StepType::ToolCall,
                description: "Step 1".to_string(),
                tool: Some("tool".to_string()),
                args: None,
                dependencies: vec![],
                expected_result: None,
                required: true,
                timeout_ms: None,
                retry_config: None,
            },
        ];
        
        let duration = SpecValidator::new().estimate_duration(&steps);
        
        // Base 5000ms + no retry overhead
        assert_eq!(duration, 5000);
    }
}
