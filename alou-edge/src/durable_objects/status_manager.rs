//! 状态管理器 - 用于管理任务状态查询和进度跟踪

use crate::compatibility::models::TaskStatusResponse;
use worker::{Env, Result, console_error, console_log};

/// 状态管理器
pub struct StatusManager;

impl StatusManager {
    /// 获取任务状态
    pub async fn get_task_status(env: &Env, task_id: &str) -> Result<TaskStatusResponse> {
        console_log!("[StatusManager] Getting status for task: {}", task_id);
        
        // 获取Durable Object stub
        let namespace = match env.durable_object("AI_TASKS") {
            Ok(ns) => {
                console_log!("[StatusManager] Got AI_TASKS namespace");
                ns
            }
            Err(e) => {
                console_error!("[StatusManager] Failed to get AI_TASKS namespace: {}", e);
                return Ok(TaskStatusResponse::new(
                    task_id.to_string(),
                    "error".to_string(),
                ));
            }
        };
        
        let id = match namespace.id_from_name(task_id) {
            Ok(id) => {
                console_log!("[StatusManager] Got DO id from name");
                id
            }
            Err(e) => {
                console_error!("[StatusManager] Failed to get DO id from name: {}", e);
                return Ok(TaskStatusResponse::new(
                    task_id.to_string(),
                    "not_found".to_string(),
                ));
            }
        };
        
        let stub = match id.get_stub() {
            Ok(stub) => {
                console_log!("[StatusManager] Got DO stub");
                stub
            }
            Err(e) => {
                console_error!("[StatusManager] Failed to get DO stub: {}", e);
                return Ok(TaskStatusResponse::new(
                    task_id.to_string(),
                    "error".to_string(),
                ));
            }
        };
        
        // 调用Durable Object获取状态
        console_log!("[StatusManager] Calling DO /status endpoint");
        match stub.fetch_with_str("/status").await {
            Ok(mut response) => {
                console_log!("[StatusManager] Got DO response, status: {}", response.status_code());
                
                if response.status_code() != 200 {
                    console_error!("[StatusManager] DO returned non-200 status: {}", response.status_code());
                    return Ok(TaskStatusResponse::new(
                        task_id.to_string(),
                        "not_found".to_string(),
                    ));
                }
                
                match response.json().await {
                    Ok(status) => {
                        console_log!("[StatusManager] Successfully parsed status response");
                        Ok(status)
                    }
                    Err(e) => {
                        console_error!("[StatusManager] Failed to parse status response: {}", e);
                        Ok(TaskStatusResponse::new(
                            task_id.to_string(),
                            "error".to_string(),
                        ))
                    }
                }
            }
            Err(e) => {
                console_error!("[StatusManager] Failed to call DO: {}", e);
                Ok(TaskStatusResponse::new(
                    task_id.to_string(),
                    "error".to_string(),
                ))
            }
        }
    }
    
    /// 批量获取任务状态
    pub async fn batch_get_task_status(
        env: &Env,
        task_ids: &[String],
    ) -> Result<Vec<TaskStatusResponse>> {
        let mut results = Vec::new();
        
        for task_id in task_ids {
            match Self::get_task_status(env, task_id).await {
                Ok(status) => results.push(status),
                Err(e) => {
                    console_error!("Failed to get status for task {}: {}", task_id, e);
                    results.push(TaskStatusResponse::new(
                        task_id.clone(),
                        "error".to_string(),
                    ));
                }
            }
        }
        
        Ok(results)
    }
    
    /// 检查任务是否存在
    pub async fn task_exists(env: &Env, task_id: &str) -> Result<bool> {
        let status = Self::get_task_status(env, task_id).await?;
        Ok(status.status != "not_found")
    }
    
    /// 获取活跃任务列表
    pub async fn get_active_tasks(_env: &Env) -> Result<Vec<String>> {
        // 注意：Durable Objects没有内置的列表功能
        // 在实际部署中，可能需要使用额外的KV存储来跟踪活跃任务
        // 这里返回空列表作为占位符
        Ok(Vec::new())
    }
    
    /// 清理过期任务
    pub async fn cleanup_expired_tasks(_env: &Env, _max_age_seconds: u64) -> Result<u64> {
        // 在实际部署中，这里会清理过期的任务
        // 现在返回0作为占位符
        Ok(0)
    }
}
