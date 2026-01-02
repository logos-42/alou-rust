//! 兼容性路由 - 处理与现有桌面SDK的兼容性

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse};
use crate::durable_objects::task_executor::TaskExecutor;
use worker::{Env, Request, Response, Result};

/// 处理兼容性聊天请求
pub async fn handle_compatible_chat(
    env: &Env,
    req: &mut Request,
) -> Result<Response> {
    // 解析兼容性请求
    let compat_request: CompatibleRequest = match req.json().await {
        Ok(req) => req,
        Err(e) => {
            let error_response = CompatibleResponse::error_response(
                format!("Failed to parse request: {}", e),
            );
            return Response::from_json(&error_response);
        }
    };
    
    // 判断是否应该使用异步处理
    if TaskExecutor::should_use_async(&compat_request) {
        handle_async_chat(env, compat_request).await
    } else {
        handle_sync_chat(env, compat_request).await
    }
}

/// 处理异步聊天请求
async fn handle_async_chat(
    env: &Env,
    request: CompatibleRequest,
) -> Result<Response> {
    // 创建异步任务
    let request_clone = request.clone();
    match TaskExecutor::create_async_task(env, request).await {
        Ok(task_id) => {
            let estimated_time = TaskExecutor::estimate_execution_time(&request_clone);
            let response = CompatibleResponse::async_task(task_id, estimated_time);
            Response::from_json(&response)
        }
        Err(e) => {
            let error_response = CompatibleResponse::error_response(
                format!("Failed to create async task: {}", e),
            );
            Response::from_json(&error_response)
        }
    }
}

/// 处理同步聊天请求
async fn handle_sync_chat(
    env: &Env,
    request: CompatibleRequest,
) -> Result<Response> {
    // 执行同步任务
    match TaskExecutor::execute_sync_task(env, request).await {
        Ok(response) => Response::from_json(&response),
        Err(e) => {
            let error_response = CompatibleResponse::error_response(
                format!("Failed to execute sync task: {}", e),
            );
            Response::from_json(&error_response)
        }
    }
}

/// 处理任务状态查询
pub async fn handle_task_status(
    env: &Env,
    task_id: &str,
) -> Result<Response> {
    use crate::durable_objects::status_manager::StatusManager;
    
    match StatusManager::get_task_status(env, task_id).await {
        Ok(status) => Response::from_json(&status),
        Err(e) => {
            let error_response = CompatibleResponse::error_response(
                format!("Failed to get task status: {}", e),
            );
            Response::from_json(&error_response)
        }
    }
}

/// 处理任务取消
pub async fn handle_task_cancel(
    env: &Env,
    task_id: &str,
) -> Result<Response> {
    match TaskExecutor::cancel_task(env, task_id).await {
        Ok(success) => {
            let response = if success {
                CompatibleResponse::success_response(
                    "Task cancelled successfully".to_string(),
                    None,
                )
            } else {
                CompatibleResponse::error_response(
                    "Failed to cancel task".to_string(),
                )
            };
            Response::from_json(&response)
        }
        Err(e) => {
            let error_response = CompatibleResponse::error_response(
                format!("Failed to cancel task: {}", e),
            );
            Response::from_json(&error_response)
        }
    }
}

/// 处理工具调用结果提交
pub async fn handle_tool_result(
    env: &Env,
    task_id: &str,
    req: &mut Request,
) -> Result<Response> {
    let tool_result: serde_json::Value = match req.json().await {
        Ok(result) => result,
        Err(e) => {
            let error_response = CompatibleResponse::error_response(
                format!("Failed to parse tool result: {}", e),
            );
            return Response::from_json(&error_response);
        }
    };
    
    match TaskExecutor::submit_tool_result(env, task_id, tool_result).await {
        Ok(success) => {
            let response = if success {
                CompatibleResponse::success_response(
                    "Tool result submitted successfully".to_string(),
                    None,
                )
            } else {
                CompatibleResponse::error_response(
                    "Failed to submit tool result".to_string(),
                )
            };
            Response::from_json(&response)
        }
        Err(e) => {
            let error_response = CompatibleResponse::error_response(
                format!("Failed to submit tool result: {}", e),
            );
            Response::from_json(&error_response)
        }
    }
}
