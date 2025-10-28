use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::agent::context::AgentContext;
use crate::mcp::registry::McpTool;
use crate::utils::error::{AloudError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step identifier
    pub id: String,
    
    /// Step name/description
    pub name: String,
    
    /// Tool to execute in this step
    pub tool: String,
    
    /// Arguments for the tool
    pub args: Value,
    
    /// Dependencies: IDs of steps that must complete before this step
    pub depends_on: Vec<String>,
    
    /// Step status
    pub status: StepStatus,
    
    /// Step result (if completed)
    pub result: Option<Value>,
    
    /// Error message (if failed)
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow identifier
    pub id: String,
    
    /// Workflow name
    pub name: String,
    
    /// Description
    pub description: String,
    
    /// Steps in the workflow
    pub steps: Vec<WorkflowStep>,
    
    /// Overall status
    pub status: WorkflowStatus,
    
    /// Created timestamp
    pub created_at: i64,
    
    /// Updated timestamp
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    Draft,
    Running,
    Completed,
    Failed,
}

/// Workflow Management Tool
pub struct WorkflowTool {
    /// In-memory workflow storage (in production, this would be in KV)
    workflows: HashMap<String, Workflow>,
}

impl WorkflowTool {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }
    
    fn create_workflow(&mut self, name: String, description: String, steps: Vec<WorkflowStep>) -> Result<String> {
        let id = format!("wf_{}", uuid::Uuid::new_v4().to_string());
        
        let workflow = Workflow {
            id: id.clone(),
            name,
            description,
            steps,
            status: WorkflowStatus::Draft,
            created_at: crate::utils::time::now_timestamp(),
            updated_at: crate::utils::time::now_timestamp(),
        };
        
        self.workflows.insert(id.clone(), workflow);
        Ok(id)
    }
    
    fn get_workflow(&self, id: &str) -> Result<&Workflow> {
        self.workflows
            .get(id)
            .ok_or_else(|| AloudError::InvalidInput(format!("Workflow not found: {}", id)))
    }
    
    fn get_workflow_mut(&mut self, id: &str) -> Result<&mut Workflow> {
        self.workflows
            .get_mut(id)
            .ok_or_else(|| AloudError::InvalidInput(format!("Workflow not found: {}", id)))
    }
    
    fn list_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }
    
    fn delete_workflow(&mut self, id: &str) -> Result<()> {
        self.workflows.remove(id);
        Ok(())
    }
}

#[async_trait(?Send)]
impl McpTool for WorkflowTool {
    fn name(&self) -> &str {
        "workflow"
    }
    
    fn description(&self) -> &str {
        "Manage workflow execution for complex multi-step tasks. Supports creating, executing, and managing workflows with dependencies between steps."
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "execute", "list", "get", "status", "delete"],
                    "description": "Action to perform on workflows"
                },
                "workflow_id": {
                    "type": "string",
                    "description": "Workflow identifier"
                },
                "name": {
                    "type": "string",
                    "description": "Workflow name (for create action)"
                },
                "description": {
                    "type": "string",
                    "description": "Workflow description (for create action)"
                },
                "steps": {
                    "type": "array",
                    "description": "List of workflow steps (for create action)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "name": {"type": "string"},
                            "tool": {"type": "string"},
                            "args": {"type": "object"},
                            "depends_on": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        },
                        "required": ["id", "name", "tool", "args"]
                    }
                }
            },
            "required": ["action"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing 'action' field".to_string()))?;
        
        // We need mutable access, so we'll create a new instance
        // In production, this would use KV storage for persistence
        let mut workflow_tool = WorkflowTool::new();
        
        match action {
            "create" => self.handle_create(&mut workflow_tool, args, context).await,
            "execute" => self.handle_execute(&mut workflow_tool, args, context).await,
            "list" => self.handle_list(&mut workflow_tool, args, context).await,
            "get" => self.handle_get(&mut workflow_tool, args, context).await,
            "status" => self.handle_status(&mut workflow_tool, args, context).await,
            "delete" => self.handle_delete(&mut workflow_tool, args, context).await,
            _ => Err(AloudError::InvalidInput(format!("Unknown action: {}", action))),
        }
    }
}

impl WorkflowTool {
    async fn handle_create(
        &self,
        workflow_tool: &mut WorkflowTool,
        args: Value,
        context: &AgentContext,
    ) -> Result<Value> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing 'name' field".to_string()))?
            .to_string();
        
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let steps = args
            .get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AloudError::InvalidInput("Missing 'steps' field".to_string()))?;
        
        let workflow_steps: Vec<WorkflowStep> = steps
            .iter()
            .map(|step| {
                WorkflowStep {
                    id: step["id"].as_str().unwrap().to_string(),
                    name: step["name"].as_str().unwrap().to_string(),
                    tool: step["tool"].as_str().unwrap().to_string(),
                    args: step["args"].clone(),
                    depends_on: step["depends_on"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_else(|| vec![]),
                    status: StepStatus::Pending,
                    result: None,
                    error: None,
                }
            })
            .collect();
        
        let workflow_id = workflow_tool.create_workflow(name, description, workflow_steps)?;
        
        Ok(json!({
            "workflow_id": workflow_id,
            "message": "Workflow created successfully",
            "session_id": context.session_id
        }))
    }
    
    async fn handle_execute(
        &self,
        workflow_tool: &mut WorkflowTool,
        args: Value,
        _context: &AgentContext,
    ) -> Result<Value> {
        let workflow_id = args
            .get("workflow_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing 'workflow_id' field".to_string()))?
            .to_string();
        
        // Get workflow and execute it
        let workflow = workflow_tool.get_workflow(&workflow_id)?;
        
        // Simple execution: mark all steps as completed
        // In production, this would actually execute the tools in the right order
        Ok(json!({
            "workflow_id": workflow_id,
            "status": "completed",
            "message": "Workflow execution completed (simulation mode)",
            "steps_completed": workflow.steps.len()
        }))
    }
    
    async fn handle_list(
        &self,
        workflow_tool: &mut WorkflowTool,
        _args: Value,
        context: &AgentContext,
    ) -> Result<Value> {
        let workflows = workflow_tool.list_workflows();
        
        let workflow_list: Vec<Value> = workflows
            .iter()
            .map(|wf| {
                json!({
                    "id": wf.id,
                    "name": wf.name,
                    "description": wf.description,
                    "status": format!("{:?}", wf.status),
                    "step_count": wf.steps.len(),
                    "created_at": wf.created_at
                })
            })
            .collect();
        
        Ok(json!({
            "workflows": workflow_list,
            "count": workflow_list.len(),
            "session_id": context.session_id
        }))
    }
    
    async fn handle_get(
        &self,
        workflow_tool: &mut WorkflowTool,
        args: Value,
        context: &AgentContext,
    ) -> Result<Value> {
        let workflow_id = args
            .get("workflow_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing 'workflow_id' field".to_string()))?
            .to_string();
        
        let workflow = workflow_tool.get_workflow(&workflow_id)?;
        
        Ok(json!({
            "workflow": json!({
                "id": workflow.id,
                "name": workflow.name,
                "description": workflow.description,
                "steps": workflow.steps.iter().map(|step| {
                    json!({
                        "id": step.id,
                        "name": step.name,
                        "tool": step.tool,
                        "status": format!("{:?}", step.status),
                        "depends_on": step.depends_on
                    })
                }).collect::<Vec<_>>(),
                "status": format!("{:?}", workflow.status),
                "created_at": workflow.created_at
            }),
            "session_id": context.session_id
        }))
    }
    
    async fn handle_status(
        &self,
        workflow_tool: &mut WorkflowTool,
        args: Value,
        context: &AgentContext,
    ) -> Result<Value> {
        let workflow_id = args
            .get("workflow_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing 'workflow_id' field".to_string()))?
            .to_string();
        
        let workflow = workflow_tool.get_workflow(&workflow_id)?;
        
        let status_summary: Vec<Value> = workflow
            .steps
            .iter()
            .map(|step| {
                json!({
                    "id": step.id,
                    "name": step.name,
                    "status": format!("{:?}", step.status)
                })
            })
            .collect();
        
        Ok(json!({
            "workflow_id": workflow_id,
            "overall_status": format!("{:?}", workflow.status),
            "steps": status_summary,
            "session_id": context.session_id
        }))
    }
    
    async fn handle_delete(
        &self,
        workflow_tool: &mut WorkflowTool,
        args: Value,
        context: &AgentContext,
    ) -> Result<Value> {
        let workflow_id = args
            .get("workflow_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AloudError::InvalidInput("Missing 'workflow_id' field".to_string()))?
            .to_string();
        
        workflow_tool.delete_workflow(&workflow_id)?;
        
        Ok(json!({
            "workflow_id": workflow_id,
            "message": "Workflow deleted successfully",
            "session_id": context.session_id
        }))
    }
}

impl Default for WorkflowTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::context::AgentContext;
    
    #[tokio::test]
    async fn test_workflow_tool_create() {
        let tool = WorkflowTool::new();
        let context = AgentContext::new("test_session".to_string());
        
        let args = json!({
            "action": "create",
            "name": "Test Workflow",
            "description": "A test workflow",
            "steps": [
                {
                    "id": "step1",
                    "name": "Step 1",
                    "tool": "echo",
                    "args": {"message": "hello"},
                    "depends_on": []
                }
            ]
        });
        
        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_workflow_tool_list() {
        let tool = WorkflowTool::new();
        let context = AgentContext::new("test_session".to_string());
        
        let args = json!({
            "action": "list"
        });
        
        let result = tool.execute(args, &context).await;
        assert!(result.is_ok());
    }
}

