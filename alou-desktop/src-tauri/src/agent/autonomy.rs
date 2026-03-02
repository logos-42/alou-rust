//! Agent Autonomy Framework
//!
//! Enables agents to make independent decisions and take initiative
//! Following "The Mythical Man-Month" principles for autonomous execution

use super::ai_client::{AiClient, AiMessage};
use super::task::{TaskManager, TaskStatus};
use super::executor::RalphLoopExecutor;
use super::swarm::{SwarmCoordinator, TaskPlan};
use crate::bridges::ToolBridge;
use crate::tools::{ToolRegistry, ToolMetadata, ToolExecutor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;

/// Agent Autonomy Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyConfig {
    /// Enable auto-discovery of skills
    pub auto_discover_skills: bool,
    /// Enable auto-selection of tools
    pub auto_select_tools: bool,
    /// Enable auto-decomposition of complex tasks
    pub auto_decompose_tasks: bool,
    /// Enable auto-coordination with other agents
    pub auto_coordinate_swarm: bool,
    /// Maximum iterations without user input
    pub max_autonomous_iterations: u32,
    /// Operations requiring user confirmation
    pub requires_confirmation_for: Vec<ConfirmationRule>,
    /// Autonomy level (0.0 = fully manual, 1.0 = fully autonomous)
    pub autonomy_level: f32,
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self {
            auto_discover_skills: true,
            auto_select_tools: true,
            auto_decompose_tasks: false, // Conservative default
            auto_coordinate_swarm: false,
            max_autonomous_iterations: 5,
            requires_confirmation_for: vec![
                ConfirmationRule::FinancialTransaction(AmountThreshold::Any),
            ],
            autonomy_level: 0.5, // Balanced autonomy
        }
    }
}

impl AutonomyConfig {
    /// Fully autonomous mode (use with caution)
    pub fn fully_autonomous() -> Self {
        Self {
            auto_discover_skills: true,
            auto_select_tools: true,
            auto_decompose_tasks: true,
            auto_coordinate_swarm: true,
            max_autonomous_iterations: 20,
            requires_confirmation_for: vec![],
            autonomy_level: 1.0,
        }
    }

    /// Conservative mode (require confirmation for most operations)
    pub fn conservative() -> Self {
        Self {
            auto_discover_skills: true,
            auto_select_tools: false,
            auto_decompose_tasks: false,
            auto_coordinate_swarm: false,
            max_autonomous_iterations: 1,
            requires_confirmation_for: vec![
                ConfirmationRule::FileWrite(PathPattern::Any),
                ConfirmationRule::BashCommand(CommandPattern::Any),
                ConfirmationRule::NetworkRequest(UrlPattern::Any),
                ConfirmationRule::FinancialTransaction(AmountThreshold::Any),
            ],
            autonomy_level: 0.2,
        }
    }

    /// Check if an operation requires confirmation
    pub fn requires_confirmation(&self, operation: &OperationType) -> bool {
        for rule in &self.requires_confirmation_for {
            if rule.matches(operation) {
                return true;
            }
        }
        false
    }
}

/// Confirmation Rules for sensitive operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationRule {
    /// File write operations
    FileWrite(PathPattern),
    /// Bash command execution
    BashCommand(CommandPattern),
    /// Network requests
    NetworkRequest(UrlPattern),
    /// Financial transactions
    FinancialTransaction(AmountThreshold),
    /// Custom pattern
    Custom(String),
}

impl ConfirmationRule {
    pub fn matches(&self, operation: &OperationType) -> bool {
        match (self, operation) {
            (ConfirmationRule::FileWrite(pattern), OperationType::FileWrite(path)) => {
                pattern.matches(path)
            }
            (ConfirmationRule::BashCommand(pattern), OperationType::BashCommand(cmd)) => {
                pattern.matches(cmd)
            }
            (ConfirmationRule::NetworkRequest(pattern), OperationType::NetworkRequest(url)) => {
                pattern.matches(url)
            }
            (ConfirmationRule::FinancialTransaction(threshold), OperationType::FinancialTransaction(amount)) => {
                threshold.exceeds(*amount)
            }
            _ => false,
        }
    }
}

/// Path pattern for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathPattern {
    /// Any path
    Any,
    /// Specific path
    Exact(String),
    /// Glob pattern
    Glob(String),
    /// Protected directories
    Protected,
}

impl PathPattern {
    pub fn matches(&self, path: &str) -> bool {
        match self {
            PathPattern::Any => true,
            PathPattern::Exact(p) => path == *p,
            PathPattern::Glob(pattern) => {
                // Simple glob matching (supports * and **)
                path.contains(pattern.trim_matches('*'))
            }
            PathPattern::Protected => {
                let protected_dirs = ["/etc", "/usr", "/bin", "/sbin", "/root"];
                protected_dirs.iter().any(|d| path.starts_with(d))
            }
        }
    }
}

/// Command pattern for bash operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandPattern {
    Any,
    Prefix(String),
    Contains(String),
    Exact(String),
}

impl CommandPattern {
    pub fn matches(&self, command: &str) -> bool {
        match self {
            CommandPattern::Any => true,
            CommandPattern::Prefix(prefix) => command.starts_with(prefix),
            CommandPattern::Contains(pattern) => command.contains(pattern),
            CommandPattern::Exact(cmd) => command == *cmd,
        }
    }
}

/// URL pattern for network requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrlPattern {
    Any,
    Domain(String),
    Pattern(String),
}

impl UrlPattern {
    pub fn matches(&self, url: &str) -> bool {
        match self {
            UrlPattern::Any => true,
            UrlPattern::Domain(domain) => url.contains(domain),
            UrlPattern::Pattern(pattern) => url.contains(pattern),
        }
    }
}

/// Amount threshold for financial transactions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AmountThreshold {
    Any,
    GreaterThan(f64),
}

impl AmountThreshold {
    pub fn exceeds(&self, amount: f64) -> bool {
        match self {
            AmountThreshold::Any => true,
            AmountThreshold::GreaterThan(threshold) => amount > *threshold,
        }
    }
}

/// Operation types that may require confirmation
#[derive(Debug, Clone)]
pub enum OperationType {
    FileWrite(String),
    BashCommand(String),
    NetworkRequest(String),
    FinancialTransaction(f64),
}

/// Agent Autonomy Manager
pub struct AgentAutonomy {
    config: AutonomyConfig,
    task_manager: Arc<TaskManager>,
    tool_bridge: Arc<ToolBridge>,
    tool_registry: Arc<ToolRegistry>,
    ai_client: Arc<AiClient>,
    swarm_coordinator: Option<Arc<SwarmCoordinator>>,
    discovered_skills: Arc<Mutex<Vec<String>>>,
    learned_patterns: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl AgentAutonomy {
    /// Create new autonomy manager
    pub fn new(
        task_manager: Arc<TaskManager>,
        tool_bridge: Arc<ToolBridge>,
        tool_registry: Arc<ToolRegistry>,
        ai_client: Arc<AiClient>,
    ) -> Self {
        Self {
            config: AutonomyConfig::default(),
            task_manager,
            tool_bridge,
            tool_registry,
            ai_client,
            swarm_coordinator: None,
            discovered_skills: Arc::new(Mutex::new(Vec::new())),
            learned_patterns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set autonomy configuration
    pub fn configure(&mut self, config: AutonomyConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AutonomyConfig {
        &self.config
    }

    /// Set swarm coordinator for multi-agent coordination
    pub fn with_swarm_coordinator(mut self, coordinator: Arc<SwarmCoordinator>) -> Self {
        self.swarm_coordinator = Some(coordinator);
        self
    }

    /// Autonomous task execution with decision making
    pub async fn execute_autonomously(
        &self,
        task_id: &str,
    ) -> Result<AutonomousExecutionResult, AutonomyError> {
        log::info!("[Autonomy] Starting autonomous execution for task: {}", task_id);

        // Check autonomy level
        if self.config.autonomy_level < 0.1 {
            return Err(AutonomyError::InsufficientAutonomy);
        }

        // Get task
        let task = self.task_manager
            .get_task(task_id)
            .await
            .ok_or_else(|| AutonomyError::TaskNotFound(task_id.to_string()))?;

        // Analyze task complexity
        let complexity = self.analyze_task_complexity(&task).await?;

        // Decide execution strategy
        let strategy = self.decide_execution_strategy(complexity).await?;

        match strategy {
            ExecutionStrategy::Direct => {
                // Execute directly with Ralph Loop
                self.execute_direct(task_id).await
            }
            ExecutionStrategy::Decompose => {
                // Decompose and execute in parallel
                if let Some(coordinator) = &self.swarm_coordinator {
                    self.execute_decomposed(task_id, coordinator.clone()).await
                } else {
                    // Fallback to direct execution
                    self.execute_direct(task_id).await
                }
            }
            ExecutionStrategy::Collaborate => {
                // Request collaboration from other agents
                self.execute_collaborative(task_id).await
            }
        }
    }

    /// Analyze task complexity
    async fn analyze_task_complexity(
        &self,
        task: &super::task::Task,
    ) -> Result<TaskComplexity, AutonomyError> {
        let message_length = task.messages.iter().map(|m| m.content.len()).sum::<usize>();
        let has_tools = !task.pending_tools.is_empty();

        // Simple heuristic-based complexity analysis
        let complexity = if message_length < 100 && !has_tools {
            TaskComplexity::Low
        } else if message_length < 500 {
            TaskComplexity::Medium
        } else {
            TaskComplexity::High
        };

        Ok(complexity)
    }

    /// Decide execution strategy based on complexity
    async fn decide_execution_strategy(
        &self,
        complexity: TaskComplexity,
    ) -> Result<ExecutionStrategy, AutonomyError> {
        if !self.config.auto_decompose_tasks {
            return Ok(ExecutionStrategy::Direct);
        }

        match complexity {
            TaskComplexity::Low => Ok(ExecutionStrategy::Direct),
            TaskComplexity::Medium => {
                if self.config.auto_coordinate_swarm && self.swarm_coordinator.is_some() {
                    Ok(ExecutionStrategy::Decompose)
                } else {
                    Ok(ExecutionStrategy::Direct)
                }
            }
            TaskComplexity::High => {
                if self.swarm_coordinator.is_some() {
                    Ok(ExecutionStrategy::Collaborate)
                } else {
                    Ok(ExecutionStrategy::Direct)
                }
            }
        }
    }

    /// Execute task directly
    async fn execute_direct(&self, task_id: &str) -> Result<AutonomousExecutionResult, AutonomyError> {
        let executor = RalphLoopExecutor::new(
            self.ai_client.clone(),
            self.task_manager.clone(),
            self.tool_bridge.clone(),
            self.tool_registry.clone(),
        );

        match executor.execute(task_id).await {
            Ok(result) => Ok(AutonomousExecutionResult {
                success: true,
                result: result.result,
                strategy: ExecutionStrategy::Direct,
                iterations: result.iteration_count,
                confirmation_requests: vec![],
            }),
            Err(e) => Err(AutonomyError::ExecutionFailed(e.to_string())),
        }
    }

    /// Execute decomposed task
    async fn execute_decomposed(
        &self,
        task_id: &str,
        coordinator: Arc<SwarmCoordinator>,
    ) -> Result<AutonomousExecutionResult, AutonomyError> {
        // Get available agents
        let agents = coordinator.get_agents().await;
        let agent_ids: Vec<String> = agents.iter().map(|a| a.id.clone()).collect();

        // Decompose task
        let plan = coordinator
            .decompose_task(task_id, agent_ids)
            .await
            .map_err(|e| AutonomyError::DecompositionFailed(e.to_string()))?;

        // Execute swarm
        let result = coordinator
            .execute_swarm(plan)
            .await
            .map_err(|e| AutonomyError::ExecutionFailed(e.to_string()))?;

        Ok(AutonomousExecutionResult {
            success: result.success,
            result: result.result,
            strategy: ExecutionStrategy::Decompose,
            iterations: 0,
            confirmation_requests: vec![],
        })
    }

    /// Execute collaborative task
    async fn execute_collaborative(&self, task_id: &str) -> Result<AutonomousExecutionResult, AutonomyError> {
        // For now, fallback to direct execution
        // Future: Implement true multi-agent collaboration
        self.execute_direct(task_id).await
    }

    /// Auto-discover available skills
    pub async fn discover_skills(&self) -> Result<Vec<String>, AutonomyError> {
        if !self.config.auto_discover_skills {
            return Ok(vec![]);
        }

        // Use the agent_skills tool to discover skills
        let skills_tool: crate::tools::agent_skills::AgentSkillsTool = crate::tools::agent_skills::AgentSkillsTool::new()
            .map_err(|e| AutonomyError::SkillDiscoveryFailed(e.to_string()))?;

        let args = serde_json::json!({
            "action": "discover"
        });

        let context = crate::tools::ExecutionContext {
            session_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            working_directory: std::env::current_dir().ok().and_then(|p| p.to_str().map(String::from)),
            environment: std::env::vars().collect(),
            timeout_seconds: Some(30),
            permissions: vec!["read".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        match skills_tool.execute(args, &context).await {
            Ok(result) => {
                // Parse skill names from result
                let skills: Vec<String> = serde_json::from_value(result.data).unwrap_or_default();

                let mut discovered = self.discovered_skills.lock().await;
                *discovered = skills.clone();

                Ok(skills)
            }
            Err(e) => Err(AutonomyError::SkillDiscoveryFailed(e.to_string())),
        }
    }

    /// Auto-select appropriate tools for a task
    pub async fn select_tools(&self, task_description: &str) -> Result<Vec<ToolMetadata>, AutonomyError> {
        if !self.config.auto_select_tools {
            return Ok(vec![]);
        }

        // Get all available tools
        let all_tools = self.tool_registry.list_all().await;

        // Use AI to select appropriate tools
        let prompt = format!(
            "Given the task: \"{}\"\n\n\
             Select the most appropriate tools from this list: {:?}\n\n\
             Return only the tool IDs as a JSON array.",
            task_description,
            all_tools.iter().map(|t| &t.id).collect::<Vec<_>>()
        );

        let messages = vec![AiMessage {
            role: "user".to_string(),
            content: prompt,
            tool_call_id: None,
            tool_calls: None,
        }];

        let response = self.ai_client
            .send_message(messages, None)
            .await
            .map_err(|e| AutonomyError::ToolSelectionFailed(e.to_string()))?;

        // Parse selected tool IDs
        let selected_ids: Vec<String> = serde_json::from_str(&response.content)
            .unwrap_or_default();

        // Filter tools
        let selected_tools: Vec<ToolMetadata> = all_tools
            .into_iter()
            .filter(|t| selected_ids.contains(&t.id))
            .collect();

        Ok(selected_tools)
    }

    /// Learn from execution patterns
    pub async fn learn_from_execution(
        &self,
        task_type: &str,
        successful_tools: Vec<String>,
    ) {
        let successful_tools_clone = successful_tools.clone();
        let mut patterns = self.learned_patterns.lock().await;
        patterns.insert(task_type.to_string(), successful_tools);
        log::info!("[Autonomy] Learned pattern for {}: {:?}", task_type, successful_tools_clone);
    }

    /// Get learned patterns for a task type
    pub async fn get_learned_pattern(&self, task_type: &str) -> Option<Vec<String>> {
        let patterns = self.learned_patterns.lock().await;
        patterns.get(task_type).cloned()
    }
}

/// Task complexity levels
#[derive(Debug, Clone, Copy)]
pub enum TaskComplexity {
    Low,
    Medium,
    High,
}

/// Execution strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Direct,
    Decompose,
    Collaborate,
}

/// Autonomous execution result
#[derive(Debug, Clone)]
pub struct AutonomousExecutionResult {
    pub success: bool,
    pub result: String,
    pub strategy: ExecutionStrategy,
    pub iterations: u32,
    pub confirmation_requests: Vec<ConfirmationRequest>,
}

/// Confirmation request for user approval
#[derive(Debug, Clone)]
pub struct ConfirmationRequest {
    pub operation: OperationType,
    pub reason: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Autonomy errors
#[derive(Debug)]
pub enum AutonomyError {
    TaskNotFound(String),
    InsufficientAutonomy,
    DecompositionFailed(String),
    ExecutionFailed(String),
    SkillDiscoveryFailed(String),
    ToolSelectionFailed(String),
    ConfirmationRequired(ConfirmationRequest),
}

impl std::fmt::Display for AutonomyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutonomyError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            AutonomyError::InsufficientAutonomy => write!(f, "Insufficient autonomy level"),
            AutonomyError::DecompositionFailed(msg) => write!(f, "Decomposition failed: {}", msg),
            AutonomyError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            AutonomyError::SkillDiscoveryFailed(msg) => write!(f, "Skill discovery failed: {}", msg),
            AutonomyError::ToolSelectionFailed(msg) => write!(f, "Tool selection failed: {}", msg),
            AutonomyError::ConfirmationRequired(req) => {
                write!(f, "Confirmation required: {} - {}", req.reason, match req.risk_level {
                    RiskLevel::Low => "Low Risk",
                    RiskLevel::Medium => "Medium Risk",
                    RiskLevel::High => "High Risk",
                    RiskLevel::Critical => "Critical Risk",
                })
            }
        }
    }
}

impl std::error::Error for AutonomyError {}
