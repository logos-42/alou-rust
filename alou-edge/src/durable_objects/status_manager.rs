//! 状态管理器 - 用于管理任务状态查询和进度跟踪

use crate::compatibility::models::TaskStatusResponse;
use worker::{Env, Result, console_error};

/// 状态管理器
pub struct StatusManager;

impl StatusManager {
    /// 获取任务状态
    pub async fn get_task_status(env: &Env, task_id: &str) -> Result<TaskStatusResponse> {
        // 获取Durable Object stub
        let namespace = env.durable_object("AI_TASKS")?;
        let id = namespace.id_from_name(task_id)?;
        let stub = id.get_stub()?;
        
        // 调用Durable Object获取状态
        let mut response = stub
            .fetch_with_str("/status")
            .await?;
        
        if response.status_code() != 200 {
            return Ok(TaskStatusResponse::new(
                task_id.to_string(),
                "not_found".to_string(),
            ));
        }
        
        let status: TaskStatusResponse = response.json().await?;
        Ok(status)
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
    pub async fn get_active_tasks(env: &Env) -> Result<Vec<String>> {
        // 注意：Durable Objects没有内置的列表功能
        // 在实际部署中，可能需要使用额外的KV存储来跟踪活跃任务
        // 这里返回空列表作为占位符
        Ok(Vec::new())
    }
    
    /// 清理过期任务
    pub async fn cleanup_expired_tasks(env: &Env, max_age_seconds: u64) -> Result<u64> {
        // 在实际部署中，这里会清理过期的任务
        // 现在返回0作为占位符
        Ok(0)
    }
}
