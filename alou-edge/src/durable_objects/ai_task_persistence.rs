//! AITaskDO 持久化辅助模块
//!
//! 负责处理所有与存储相关的操作

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, ToolCall};
use crate::durable_objects::ai_task_state::{TaskState, get_request_key, get_result_key, get_pending_tool_calls_key, get_workflow_key, get_state_key};
use crate::compatibility::models::TaskStatus;
use crate::agent::ai_client::AiMessage;
use crate::mcp::tools::workflow::Workflow;
use crate::utils::time::current_timestamp_secs;
use worker::{Result, console_log};
use serde_json::Value;

/// 持久化操作集合
pub struct TaskPersistence;

impl TaskPersistence {
    /// 获取请求数据
    pub async fn get_request(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<Option<CompatibleRequest>> {
        let request_key = get_request_key(task_name);
        Ok(storage.get::<CompatibleRequest>(&request_key).await.ok())
    }

    /// 保存结果
    pub async fn save_result(
        storage: &worker::Storage,
        task_name: &str,
        result: &CompatibleResponse,
    ) -> Result<()> {
        let result_key = get_result_key(task_name);
        storage.put(&result_key, result).await?;
        Ok(())
    }

    /// 加载结果
    pub async fn load_result(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<Option<CompatibleResponse>> {
        let result_key = get_result_key(task_name);
        Ok(storage.get::<CompatibleResponse>(&result_key).await.ok())
    }

    /// 保存待处理的工具调用
    pub async fn save_pending_tool_calls(
        storage: &worker::Storage,
        task_name: &str,
        tool_calls: &[ToolCall],
    ) -> Result<()> {
        let key = get_pending_tool_calls_key(task_name);
        storage.put(&key, tool_calls).await?;
        Ok(())
    }

    /// 加载待处理的工具调用
    pub async fn load_pending_tool_calls(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<Option<Vec<ToolCall>>> {
        let key = get_pending_tool_calls_key(task_name);
        Ok(storage.get::<Vec<ToolCall>>(&key).await.ok())
    }

    /// 加载工作流
    pub async fn load_workflow(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<Option<Workflow>> {
        let key = get_workflow_key(task_name);
        let result = storage.get::<Workflow>(&key).await;

        match result {
            Ok(wf) => {
                console_log!("[PERSISTENCE] Loaded workflow with {} steps", wf.steps.len());
                Ok(Some(wf))
            }
            Err(e) => {
                console_log!("[PERSISTENCE] Failed to load workflow: {}", e);
                Ok(None)
            }
        }
    }

    /// 保存工作流
    pub async fn save_workflow(
        storage: &worker::Storage,
        task_name: &str,
        workflow: &Workflow,
    ) -> Result<()> {
        let key = get_workflow_key(task_name);
        storage.put(&key, workflow).await?;
        console_log!("[PERSISTENCE] Saved workflow with {} steps", workflow.steps.len());
        Ok(())
    }

    /// 检查是否为工作流任务
    pub async fn is_workflow_task(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<bool> {
        let request_key = get_request_key(task_name);
        match storage.get::<CompatibleRequest>(&request_key).await {
            Ok(req) => Ok(req.workflow_id.is_some()),
            Err(_) => Ok(false),
        }
    }

    /// 加载状态
    pub async fn load_state(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<TaskState> {
        console_log!("[PERSISTENCE] load_state called");

        let state_key = get_state_key(task_name);
        let state_data = storage.get::<Value>(&state_key).await?;

        let status = state_data.get("status")
            .and_then(|s| s.as_str())
            .map(|s| match s {
                "queued" => TaskStatus::Queued,
                "running" => TaskStatus::Running,
                "processing" => TaskStatus::Processing,
                "completed" => TaskStatus::Completed,
                "failed" => TaskStatus::Failed,
                _ => TaskStatus::Queued,
            })
            .unwrap_or(TaskStatus::Queued);

        let progress = state_data.get("progress")
            .and_then(|p| p.as_f64())
            .map(|p| p as f32)
            .unwrap_or(0.0);

        let current_step = state_data.get("current_step")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "初始化".to_string());

        let created_at = state_data.get("created_at")
            .and_then(|c| c.as_u64())
            .unwrap_or_else(current_timestamp_secs);

        let updated_at = state_data.get("updated_at")
            .and_then(|u| u.as_u64())
            .unwrap_or(created_at);

        let error = state_data.get("error")
            .and_then(|e| e.as_str())
            .map(|e| e.to_string());

        console_log!("[PERSISTENCE] Successfully loaded state");
        Ok(TaskState {
            status,
            progress,
            current_step,
            created_at,
            updated_at,
            error,
        })
    }

    /// 保存状态
    pub async fn save_state(
        storage: &worker::Storage,
        task_name: &str,
        state: &TaskState,
    ) -> Result<()> {
        let state_data = serde_json::json!({
            "status": state.status.to_string(),
            "progress": state.progress,
            "current_step": state.current_step,
            "created_at": state.created_at,
            "updated_at": state.updated_at,
            "error": state.error,
        });

        let state_key = get_state_key(task_name);
        storage.put(&state_key, state_data).await?;
        Ok(())
    }
}
