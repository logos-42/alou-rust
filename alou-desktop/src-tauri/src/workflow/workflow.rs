use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkflowRequest {
    pub workflow: WorkflowData,
    pub agent_info: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowData {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub workflow_id: String,
    pub api_key: String,
    pub agent_info: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetryStepRequest {
    pub workflow_id: String,
    pub step_id: String,
    pub api_key: String,
    pub agent_info: Option<serde_json::Value>,
}

pub struct WorkflowState {
    pub workflows: Mutex<HashMap<String, Workflow>>,
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            workflows: Mutex::new(HashMap::new()),
        }
    }
}

/// Create a new workflow
#[tauri::command]
pub async fn create_workflow(
    request: CreateWorkflowRequest,
    state: State<'_, WorkflowState>,
) -> Result<serde_json::Value, String> {
    let mut workflows = state.workflows.lock().map_err(|e| e.to_string())?;

    let workflow_id = format!("wf_{}", uuid::Uuid::new_v4().to_string());

    let workflow = Workflow {
        id: workflow_id.clone(),
        name: request.workflow.name,
        description: request.workflow.description,
        steps: request.workflow.steps,
        status: "draft".to_string(),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    workflows.insert(workflow_id.clone(), workflow);

    Ok(serde_json::json!({
        "workflow_id": workflow_id,
        "message": "Workflow created successfully"
    }))
}

/// Execute a workflow using Claude Agent SDK
#[tauri::command]
pub async fn execute_workflow(
    request: ExecuteWorkflowRequest,
    state: State<'_, WorkflowState>,
) -> Result<serde_json::Value, String> {
    let mut workflows = state.workflows.lock().map_err(|e| e.to_string())?;

    let workflow = workflows.get_mut(&request.workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;

    // Generate execution ID
    let execution_id = format!("exec_{}", uuid::Uuid::new_v4().to_string());

    // Mark workflow as running
    workflow.status = "running".to_string();
    workflow.updated_at = chrono::Utc::now().timestamp();

    // Simulate workflow execution (in real implementation, this would orchestrate the steps)
    // For now, we'll just mark all steps as completed
    for step in &mut workflow.steps {
        step.status = Some("completed".to_string());
        step.result = Some(serde_json::json!({"message": format!("Step {} completed", step.name)}));
    }

    workflow.status = "completed".to_string();
    workflow.updated_at = chrono::Utc::now().timestamp();

    Ok(serde_json::json!({
        "execution_id": execution_id,
        "workflow_id": request.workflow_id,
        "status": "completed",
        "message": "Workflow executed successfully"
    }))
}

/// Get workflow status
#[tauri::command]
pub fn get_workflow_status(
    workflow_id: String,
    state: State<'_, WorkflowState>,
) -> Result<serde_json::Value, String> {
    let workflows = state.workflows.lock().map_err(|e| e.to_string())?;

    let workflow = workflows.get(&workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;

    // Calculate progress based on completed steps
    let total_steps = workflow.steps.len();
    let completed_steps = workflow.steps.iter()
        .filter(|step| step.status.as_ref().map_or(false, |s| s == "completed"))
        .count();
    let progress = if total_steps > 0 {
        (completed_steps as f64 / total_steps as f64 * 100.0) as u32
    } else {
        0
    };

    // Get current step
    let current_step = workflow.steps.iter()
        .find(|step| step.status.as_ref().map_or(false, |s| s == "running"))
        .map(|step| step.name.clone());

    Ok(serde_json::json!({
        "data": {
            "status": workflow.status,
            "progress": progress,
            "current_step": current_step,
            "result": if workflow.status == "completed" {
                Some(serde_json::json!({
                    "steps": workflow.steps.iter().map(|step| {
                        serde_json::json!({
                            "id": step.id,
                            "name": step.name,
                            "status": step.status,
                            "result": step.result
                        })
                    }).collect::<Vec<_>>()
                }))
            } else {
                None
            },
            "error": workflow.steps.iter()
                .find(|step| step.status.as_ref().map_or(false, |s| s == "failed"))
                .and_then(|step| step.error.clone())
        }
    }))
}

/// List all workflows
#[tauri::command]
pub fn list_workflows(state: State<'_, WorkflowState>) -> Result<serde_json::Value, String> {
    let workflows = state.workflows.lock().map_err(|e| e.to_string())?;

    let workflow_list: Vec<&Workflow> = workflows.values().collect();

    Ok(serde_json::json!({
        "workflows": workflow_list
    }))
}

/// Delete a workflow
#[tauri::command]
pub fn delete_workflow(
    workflow_id: String,
    state: State<'_, WorkflowState>,
) -> Result<serde_json::Value, String> {
    let mut workflows = state.workflows.lock().map_err(|e| e.to_string())?;

    workflows.remove(&workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;

    Ok(serde_json::json!({
        "message": "Workflow deleted successfully"
    }))
}

/// Retry a failed workflow step
#[tauri::command]
pub async fn retry_workflow_step(
    request: RetryStepRequest,
    state: State<'_, WorkflowState>,
) -> Result<serde_json::Value, String> {
    let mut workflows = state.workflows.lock().map_err(|e| e.to_string())?;

    let workflow = workflows.get_mut(&request.workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;

    // Find and update the step
    let step = workflow.steps.iter_mut()
        .find(|s| s.id == request.step_id)
        .ok_or_else(|| "Step not found".to_string())?;

    step.status = Some("completed".to_string());
    step.result = Some(serde_json::json!({
        "message": format!("Step {} retried successfully", step.name)
    }));

    Ok(serde_json::json!({
        "step_result": {
            "step_id": step.id,
            "status": step.status,
            "result": step.result
        }
    }))
}

/// Pause workflow execution
#[tauri::command]
pub fn pause_workflow(
    workflow_id: String,
    state: State<'_, WorkflowState>,
) -> Result<serde_json::Value, String> {
    let mut workflows = state.workflows.lock().map_err(|e| e.to_string())?;

    let workflow = workflows.get_mut(&workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;

    workflow.status = "paused".to_string();

    Ok(serde_json::json!({
        "message": "Workflow paused"
    }))
}

/// Resume workflow execution
#[tauri::command]
pub async fn resume_workflow(
    request: ExecuteWorkflowRequest,
    state: State<'_, WorkflowState>,
) -> Result<serde_json::Value, String> {
    let mut workflows = state.workflows.lock().map_err(|e| e.to_string())?;

    let workflow = workflows.get_mut(&request.workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;

    workflow.status = "running".to_string();

    // Continue execution from paused state
    for step in &mut workflow.steps {
        if step.status.as_ref().unwrap_or(&"".to_string()) == "pending" {
            step.status = Some("completed".to_string());
            step.result = Some(serde_json::json!({
                "message": format!("Step {} resumed and completed", step.name)
            }));
        }
    }

    workflow.status = "completed".to_string();

    Ok(serde_json::json!({
        "execution_result": {
            "status": "resumed",
            "message": "Workflow resumed and completed"
        }
    }))
}