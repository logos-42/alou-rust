//! Agent Swarm Coordination
//!
//! Implements multi-agent task decomposition and parallel execution
//! Following "The Mythical Man-Month" principles for autonomous agent coordination

use super::task::{Task, TaskManager, TaskStatus, TaskMetadata, TaskEvent, TaskFinalResult};
use super::ai_client::{AiClient, AiMessage};
use super::executor::RalphLoopExecutor;
use crate::bridges::ToolBridge;
use crate::tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

/// Agent Status in the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Busy(String), // Current task ID
    Offline,
    Error(String),
}

/// Agent Role in swarm coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentRole {
    Coordinator,  // Leads task decomposition
    Worker,       // Executes subtasks
    Specialist(String), // Specialized role (e.g., "coder", "reviewer")
}

/// Agent Instance in the swarm
#[derive(Debug, Clone)]
pub struct AgentInstance {
    pub id: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub skills: Vec<String>,
    pub current_task: Option<String>,
    pub capabilities: Vec<String>,
}

impl AgentInstance {
    pub fn new(id: String, role: AgentRole) -> Self {
        Self {
            id,
            role,
            status: AgentStatus::Idle,
            skills: Vec::new(),
            current_task: None,
            capabilities: Vec::new(),
        }
    }

    pub fn with_skills(mut self, skills: Vec<String>) -> Self {
        self.skills = skills;
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }
}

/// Task Decomposition for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub task_id: String,
    pub subtasks: Vec<Subtask>,
    pub dependencies: TaskDependencyGraph,
    pub assigned_agents: HashMap<String, String>, // subtask_id -> agent_id
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: String,
    pub parent_task_id: String,
    pub description: String,
    pub assigned_agent: Option<String>,
    pub dependencies: Vec<String>, // Other subtask IDs this depends on
    pub estimated_duration_secs: u64,
    pub priority: Priority,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskDependencyGraph {
    pub edges: HashMap<String, Vec<String>>, // task_id -> dependent task IDs
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Swarm Coordinator - manages multi-agent task execution
pub struct SwarmCoordinator {
    agents: Arc<Mutex<HashMap<String, AgentInstance>>>,
    task_manager: Arc<TaskManager>,
    tool_bridge: Arc<ToolBridge>,
    tool_registry: Arc<ToolRegistry>,
    ai_client: Arc<AiClient>,
    event_sender: broadcast::Sender<SwarmEvent>,
}

/// Swarm Events for frontend notification
#[derive(Debug, Clone, Serialize)]
pub enum SwarmEvent {
    SwarmInitialized { swarm_id: String },
    TaskDecomposed { task_id: String, subtasks: Vec<String> },
    SubtaskAssigned { subtask_id: String, agent_id: String },
    SubtaskStarted { subtask_id: String, agent_id: String },
    SubtaskCompleted { subtask_id: String, result: String },
    SubtaskFailed { subtask_id: String, error: String },
    SwarmCompleted { task_id: String, result: TaskFinalResult },
    SwarmFailed { task_id: String, error: String },
    AgentStatusChanged { agent_id: String, status: AgentStatus },
}

impl SwarmCoordinator {
    pub fn new(
        task_manager: Arc<TaskManager>,
        tool_bridge: Arc<ToolBridge>,
        tool_registry: Arc<ToolRegistry>,
        ai_client: Arc<AiClient>,
    ) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            task_manager,
            tool_bridge,
            tool_registry,
            ai_client,
            event_sender: tx,
        }
    }

    /// Register an agent in the swarm
    pub async fn register_agent(&self, agent: AgentInstance) {
        let mut agents = self.agents.lock().await;
        log::info!("[Swarm] Registering agent: {}", agent.id);
        agents.insert(agent.id.clone(), agent);
        
        let _ = self.event_sender.send(SwarmEvent::AgentStatusChanged {
            agent_id: agents.keys().next().unwrap().clone(),
            status: AgentStatus::Idle,
        });
    }

    /// Unregister an agent from the swarm
    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut agents = self.agents.lock().await;
        agents.remove(agent_id);
        log::info!("[Swarm] Unregistered agent: {}", agent_id);
    }

    /// Get all registered agents
    pub async fn get_agents(&self) -> Vec<AgentInstance> {
        let agents = self.agents.lock().await;
        agents.values().cloned().collect()
    }

    /// Decompose a task into parallel subtasks using AI
    pub async fn decompose_task(
        &self,
        task_id: &str,
        available_agents: Vec<String>,
    ) -> Result<TaskPlan, SwarmError> {
        log::info!("[Swarm] Decomposing task {} for {} agents", task_id, available_agents.len());

        // Get the original task
        let task = self.task_manager
            .get_task(task_id)
            .await
            .ok_or_else(|| SwarmError::TaskNotFound(task_id.to_string()))?;

        // Use AI to decompose the task
        let decomposition_prompt = format!(
            "Decompose the following task into independent subtasks that can be executed in parallel by multiple agents.\n\n\
             Task: {}\n\n\
             Available agents: {:?}\n\n\
             For each subtask, specify:\n\
             1. A unique ID\n\
             2. A clear description\n\
             3. Dependencies on other subtasks (if any)\n\
             4. Which agent role would be best suited\n\n\
             Return the decomposition as a JSON array of subtasks.",
            task.messages.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n"),
            available_agents
        );

        let messages = vec![AiMessage {
            role: "user".to_string(),
            content: decomposition_prompt,
            tool_call_id: None,
            tool_calls: None,
        }];

        // Send to AI for decomposition
        let response = self.ai_client
            .send_message(messages, None)
            .await
            .map_err(|e| SwarmError::DecompositionFailed(e.to_string()))?;

        // Parse the AI response into a TaskPlan
        let subtasks = self.parse_decomposition(&response.content, task_id, &available_agents)?;
        
        let plan = TaskPlan {
            task_id: task_id.to_string(),
            subtasks,
            dependencies: TaskDependencyGraph::default(),
            assigned_agents: HashMap::new(),
            created_at: chrono::Utc::now().timestamp(),
        };

        // Emit event
        let subtask_ids: Vec<String> = plan.subtasks.iter().map(|s| s.id.clone()).collect();
        let _ = self.event_sender.send(SwarmEvent::TaskDecomposed {
            task_id: task_id.to_string(),
            subtasks: subtask_ids,
        });

        Ok(plan)
    }

    /// Parse AI decomposition response into subtasks
    fn parse_decomposition(
        &self,
        content: &str,
        parent_task_id: &str,
        available_agents: &[String],
    ) -> Result<Vec<Subtask>, SwarmError> {
        // Try to parse as JSON array
        if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(array) = json_array.as_array() {
                let mut subtasks = Vec::new();
                for (i, item) in array.iter().enumerate() {
                    let subtask = Subtask {
                        id: format!("subtask_{}_{}", parent_task_id, i),
                        parent_task_id: parent_task_id.to_string(),
                        description: item.get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown task")
                            .to_string(),
                        assigned_agent: item.get("agent")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| available_agents.first().cloned()),
                        dependencies: item.get("dependencies")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect())
                            .unwrap_or_default(),
                        estimated_duration_secs: item.get("estimated_duration")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(60),
                        priority: Priority::Medium,
                        status: TaskStatus::Pending,
                    };
                    subtasks.push(subtask);
                }
                return Ok(subtasks);
            }
        }

        // Fallback: create a single subtask
        Ok(vec![Subtask {
            id: format!("subtask_{}_0", parent_task_id),
            parent_task_id: parent_task_id.to_string(),
            description: content.to_string(),
            assigned_agent: available_agents.first().cloned(),
            dependencies: Vec::new(),
            estimated_duration_secs: 300,
            priority: Priority::Medium,
            status: TaskStatus::Pending,
        }])
    }

    /// Execute a task plan with swarm coordination
    pub async fn execute_swarm(&self, mut plan: TaskPlan) -> Result<TaskFinalResult, SwarmError> {
        log::info!("[Swarm] Executing swarm with {} subtasks", plan.subtasks.len());

        // Assign agents to subtasks if not already assigned
        self.assign_agents(&mut plan).await?;

        // Execute subtasks respecting dependencies
        let mut completed_subtasks: HashMap<String, String> = HashMap::new();
        let mut failed_subtasks: HashMap<String, String> = HashMap::new();

        // Get executable subtasks (no dependencies or dependencies met)
        let mut pending: Vec<&Subtask> = plan.subtasks.iter().collect();
        
        while !pending.is_empty() {
            // Find subtasks that can be executed now
            let mut executable = Vec::new();
            let mut still_pending = Vec::new();

            for subtask in pending {
                let deps_met = subtask.dependencies.iter().all(|dep| {
                    completed_subtasks.contains_key(dep)
                });

                if deps_met {
                    executable.push(subtask);
                } else {
                    still_pending.push(subtask);
                }
            }

            if executable.is_empty() {
                // No progress possible - check for failures
                if !still_pending.is_empty() {
                    return Err(SwarmError::Deadlock(
                        still_pending.iter().map(|s| s.id.clone()).collect()
                    ));
                }
                break;
            }

            // Execute subtasks in parallel
            let futures: Vec<_> = executable.iter().map(|subtask| {
                self.execute_subtask(subtask, &plan.assigned_agents)
            }).collect();

            let results = futures::future::join_all(futures).await;

            // Process results
            for (subtask, result) in executable.iter().zip(results) {
                match result {
                    Ok(output) => {
                        log::info!("[Swarm] Subtask {} completed", subtask.id);
                        let output_clone = output.clone();
                        completed_subtasks.insert(subtask.id.clone(), output_clone);

                        let _ = self.event_sender.send(SwarmEvent::SubtaskCompleted {
                            subtask_id: subtask.id.clone(),
                            result: output,
                        });
                    }
                    Err(e) => {
                        log::error!("[Swarm] Subtask {} failed: {}", subtask.id, e);
                        failed_subtasks.insert(subtask.id.clone(), e.to_string());

                        let _ = self.event_sender.send(SwarmEvent::SubtaskFailed {
                            subtask_id: subtask.id.clone(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            pending = still_pending;
        }

        // Aggregate results
        if failed_subtasks.is_empty() {
            let aggregated_result = self.aggregate_results(&completed_subtasks);
            let aggregated_result_clone = aggregated_result.clone();

            let _ = self.event_sender.send(SwarmEvent::SwarmCompleted {
                task_id: plan.task_id.clone(),
                result: TaskFinalResult {
                    task_id: plan.task_id.clone(),
                    success: true,
                    result: aggregated_result,
                    error: None,
                    iteration_count: 0,
                },
            });

            Ok(TaskFinalResult {
                task_id: plan.task_id.clone(),
                success: true,
                result: aggregated_result_clone,
                error: None,
                iteration_count: 0,
            })
        } else {
            let error_msg = format!(
                "{} subtasks failed: {:?}",
                failed_subtasks.len(),
                failed_subtasks.keys().collect::<Vec<_>>()
            );

            let _ = self.event_sender.send(SwarmEvent::SwarmFailed {
                task_id: plan.task_id.clone(),
                error: error_msg.clone(),
            });

            Err(SwarmError::SubtaskFailures(failed_subtasks))
        }
    }

    /// Assign agents to subtasks
    async fn assign_agents(&self, plan: &mut TaskPlan) -> Result<(), SwarmError> {
        let agents = self.get_agents().await;
        let idle_agents: Vec<_> = agents.iter()
            .filter(|a| matches!(a.status, AgentStatus::Idle))
            .collect();

        for subtask in &mut plan.subtasks {
            if subtask.assigned_agent.is_none() {
                // Find a suitable agent
                if let Some(agent) = idle_agents.first() {
                    subtask.assigned_agent = Some(agent.id.clone());
                    plan.assigned_agents.insert(subtask.id.clone(), agent.id.clone());
                    
                    let _ = self.event_sender.send(SwarmEvent::SubtaskAssigned {
                        subtask_id: subtask.id.clone(),
                        agent_id: agent.id.clone(),
                    });
                } else {
                    return Err(SwarmError::NoAvailableAgents);
                }
            }
        }

        Ok(())
    }

    /// Execute a single subtask
    async fn execute_subtask(
        &self,
        subtask: &Subtask,
        assigned_agents: &HashMap<String, String>,
    ) -> Result<String, SwarmError> {
        let agent_id = assigned_agents.get(&subtask.id)
            .or(subtask.assigned_agent.as_ref())
            .ok_or_else(|| SwarmError::NoAgentAssigned(subtask.id.clone()))?;

        log::info!("[Swarm] Executing subtask {} with agent {}", subtask.id, agent_id);

        let _ = self.event_sender.send(SwarmEvent::SubtaskStarted {
            subtask_id: subtask.id.clone(),
            agent_id: agent_id.clone(),
        });

        // Create a task for this subtask
        let subtask_id = self.task_manager
            .create_task(agent_id.clone(), subtask.description.clone())
            .await;

        // Execute using Ralph Loop
        let executor = RalphLoopExecutor::new(
            self.ai_client.clone(),
            self.task_manager.clone(),
            self.tool_bridge.clone(),
            self.tool_registry.clone(),
        );

        match executor.execute(&subtask_id).await {
            Ok(result) => Ok(result.result),
            Err(e) => Err(SwarmError::ExecutionFailed(e.to_string())),
        }
    }

    /// Aggregate results from completed subtasks
    fn aggregate_results(&self, results: &HashMap<String, String>) -> String {
        let mut aggregated = String::from("## Swarm Execution Results\n\n");
        
        for (subtask_id, result) in results {
            aggregated.push_str(&format!("### {}\n{}\n\n", subtask_id, result));
        }

        aggregated
    }

    /// Subscribe to swarm events
    pub fn subscribe_events(&self) -> broadcast::Receiver<SwarmEvent> {
        self.event_sender.subscribe()
    }
}

/// Swarm execution errors
#[derive(Debug)]
pub enum SwarmError {
    TaskNotFound(String),
    DecompositionFailed(String),
    NoAvailableAgents,
    NoAgentAssigned(String),
    ExecutionFailed(String),
    SubtaskFailures(HashMap<String, String>),
    Deadlock(Vec<String>),
}

impl std::fmt::Display for SwarmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwarmError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            SwarmError::DecompositionFailed(msg) => write!(f, "Decomposition failed: {}", msg),
            SwarmError::NoAvailableAgents => write!(f, "No available agents in swarm"),
            SwarmError::NoAgentAssigned(id) => write!(f, "No agent assigned to subtask: {}", id),
            SwarmError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            SwarmError::SubtaskFailures(failures) => {
                write!(f, "Subtask failures: {:?}", failures.keys().collect::<Vec<_>>())
            }
            SwarmError::Deadlock(tasks) => {
                write!(f, "Deadlock detected in subtasks: {:?}", tasks)
            }
        }
    }
}

impl std::error::Error for SwarmError {}
