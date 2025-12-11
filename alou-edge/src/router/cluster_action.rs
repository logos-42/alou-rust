use crate::agent::agent_coordinator::AgentCoordinator;
use crate::agent::cluster_action::{
    ClusterAction, ClusterActionManager, ClusterActionStatus, TaskAnalysis,
};
use crate::agent::cluster_executor::ClusterExecutor;
use crate::agent::session::SessionManager;
use crate::router::pubsub::PubSubManager;
use crate::utils::error::{AloudError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::*;

use super::{json_response, json_response_with_status, ErrorResponse};

/// 创建集群行动请求
#[derive(Deserialize)]
pub struct CreateClusterActionRequest {
    pub description: String,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// 执行集群行动请求
#[derive(Deserialize)]
pub struct ExecuteClusterActionRequest {
    pub action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
}

/// 分析任务请求
#[derive(Deserialize)]
pub struct AnalyzeTaskRequest {
    pub task_description: String,
    pub available_agents: Vec<String>,
}

/// 集群行动响应
#[derive(Serialize)]
pub struct ClusterActionResponse {
    pub success: bool,
    pub action: Option<ClusterAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 状态响应
#[derive(Serialize)]
pub struct StatusResponse {
    pub action_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 结果响应
#[derive(Serialize)]
pub struct ResultsResponse {
    pub action_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 分析响应
#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub analysis: TaskAnalysis,
}

/// 处理创建集群行动请求
pub async fn handle_create_cluster_action(
    manager: &ClusterActionManager,
    req: &mut Request,
) -> Result<Response> {
    let body: CreateClusterActionRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    match manager
        .create_cluster_action(body.description, body.created_by)
        .await
    {
        Ok(action) => {
            let response = ClusterActionResponse {
                success: true,
                action: Some(action),
                error: None,
            };
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to create cluster action: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

/// 处理执行集群行动请求
pub async fn handle_execute_cluster_action(
    manager: &ClusterActionManager,
    executor: &ClusterExecutor,
    req: &mut Request,
) -> Result<Response> {
    let body: ExecuteClusterActionRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    // 获取集群行动
    let action = match manager.get_action(&body.action_id).await? {
        Some(action) => action,
        None => {
            let error_response = ErrorResponse {
                error: format!("Cluster action {} not found", body.action_id),
            };
            return json_response_with_status(&error_response, 404);
        }
    };

    // 执行集群行动
    match executor
        .execute_cluster_action(action, body.wallet_address, body.chain)
        .await
    {
        Ok(updated_action) => {
            // 保存更新后的行动
            manager.update_action(&updated_action).await?;

            let response = ClusterActionResponse {
                success: true,
                action: Some(updated_action),
                error: None,
            };
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to execute cluster action: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

/// 处理获取状态请求
pub async fn handle_get_status(
    manager: &ClusterActionManager,
    req: &Request,
) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    
    // 从路径中提取 action_id: /api/cluster-action/{action_id}/status
    let action_id = path
        .split('/')
        .nth(3)
        .ok_or_else(|| AloudError::InvalidInput("Invalid path".to_string()))?;

    match manager.get_action(action_id).await? {
        Some(action) => {
            let response = StatusResponse {
                action_id: action.action_id.clone(),
                status: format!("{:?}", action.status),
                error: action.error.clone(),
            };
            json_response(&response)
        }
        None => {
            let error_response = ErrorResponse {
                error: format!("Cluster action {} not found", action_id),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}

/// 处理获取结果请求
pub async fn handle_get_results(
    manager: &ClusterActionManager,
    req: &Request,
) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    
    // 从路径中提取 action_id: /api/cluster-action/{action_id}/results
    let action_id = path
        .split('/')
        .nth(3)
        .ok_or_else(|| AloudError::InvalidInput("Invalid path".to_string()))?;

    match manager.get_action(action_id).await? {
        Some(action) => {
            let response = ResultsResponse {
                action_id: action.action_id.clone(),
                status: format!("{:?}", action.status),
                final_result: action.final_result.clone(),
                error: action.error.clone(),
            };
            json_response(&response)
        }
        None => {
            let error_response = ErrorResponse {
                error: format!("Cluster action {} not found", action_id),
            };
            json_response_with_status(&error_response, 404)
        }
    }
}

/// 处理取消请求
pub async fn handle_cancel_action(
    manager: &ClusterActionManager,
    req: &mut Request,
) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    
    // 从路径中提取 action_id: /api/cluster-action/{action_id}/cancel
    let action_id = path
        .split('/')
        .nth(3)
        .ok_or_else(|| AloudError::InvalidInput("Invalid path".to_string()))?;

    match manager.cancel_action(action_id).await {
        Ok(_) => {
            let response = json!({
                "success": true,
                "action_id": action_id,
                "message": "Action cancelled",
            });
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to cancel action: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

/// 处理分析任务请求
pub async fn handle_analyze_task(
    manager: &ClusterActionManager,
    req: &mut Request,
) -> Result<Response> {
    let body: AnalyzeTaskRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    match manager
        .analyze_task(&body.task_description, &body.available_agents)
        .await
    {
        Ok(analysis) => {
            let response = AnalyzeResponse { analysis };
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to analyze task: {}", e),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

