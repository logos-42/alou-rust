//! AITaskDO HTTP 路由处理器
//!
//! 负责处理所有 HTTP 请求的路由和响应

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, TaskStatusResponse, TaskStatus};
use crate::durable_objects::ai_task_state::{TaskState, get_state_key, get_request_key, get_result_key, get_pending_tool_calls_key, get_conversation_history_key, get_tool_result_key};
use crate::durable_objects::ai_task_persistence::TaskPersistence;
use crate::agent::ai_client::AiMessage;
use crate::utils::time::current_timestamp_secs;
use worker::{Headers, Request, Response, Result, console_error, console_log};

/// AITaskDO HTTP 路由处理器实现
pub struct AITaskHandlers;

impl AITaskHandlers {
    /// 处理 /init 端点
    pub async fn handle_init(
        task_name: &str,
        storage: &worker::Storage,
        request: CompatibleRequest,
    ) -> Result<Response> {
        console_log!("[INIT] Initializing task: {}", task_name);

        Self::save_request_to_storage(storage, task_name, &request).await?;

        let initial_state = TaskState::new_initial();
        Self::save_state_to_storage(storage, task_name, &initial_state).await?;

        let response = TaskStatusResponse::new(
            task_name.to_string(),
            initial_state.status.to_string(),
        );

        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;

        Ok(Response::from_json(&response)?.with_headers(headers))
    }

    /// 处理 /get 端点
    pub async fn handle_get_status(
        task_name: &str,
        storage: &worker::Storage,
    ) -> Result<Response> {
        console_log!("[STATUS] Getting status for task: {}", task_name);

        let state = Self::load_state_from_storage(storage, task_name).await.map_err(|e| {
            console_error!("[STATUS] Failed to load state: {}", e);
            e
        })?;

        console_log!("[STATUS] Task {} state: status={}, progress={}, step='{}', error={:?}, created_at={}, updated_at={}",
                     task_name, state.status, state.progress, state.current_step, state.error, state.created_at, state.updated_at);

        // 如果任务已完成，尝试加载结果
        let result = if state.status == TaskStatus::Completed {
            console_log!("[STATUS] Task completed, loading result");
            match Self::load_result_from_storage(storage, task_name).await {
                Ok(Some(res)) => {
                    console_log!("[STATUS] Result loaded successfully");
                    Some(res)
                }
                Ok(None) => {
                    console_log!("[STATUS] No result found for completed task");
                    None
                }
                Err(e) => {
                    console_error!("[STATUS] Failed to load result: {}", e);
                    None
                }
            }
        } else {
            console_log!("[STATUS] Task not completed, skipping result loading");
            None
        };

        let response = TaskStatusResponse {
            task_id: task_name.to_string(),
            status: state.status.to_string(),
            progress: Some(state.progress),
            current_step: Some(state.current_step),
            result,
            error: state.error,
            created_at: state.created_at,
            updated_at: state.updated_at,
        };

        console_log!("[STATUS] Response prepared: status={}, progress={:.2}, step='{}', has_result={}", 
                     response.status, response.progress.unwrap_or(0.0), response.current_step.as_ref().unwrap_or(&"None".to_string()), response.result.is_some());

        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;

        Ok(Response::from_json(&response)?.with_headers(headers))
    }

    /// 处理 /cancel 端点
    pub async fn handle_cancel(
        task_name: &str,
        storage: &worker::Storage,
        get_current_timestamp_millis: impl Fn() -> u64,
    ) -> Result<Response> {
        let mut state = Self::load_state_from_storage(storage, task_name).await?;

        if state.status != TaskStatus::Running && state.status != TaskStatus::Queued {
            return Response::error("Task cannot be cancelled", 400);
        }

        state.status = TaskStatus::Failed;
        state.error = Some("任务被取消".to_string());
        state.progress = 1.0;
        state.current_step = "已取消".to_string();
        state.updated_at = get_current_timestamp_millis();
        Self::save_state_to_storage(storage, task_name, &state).await?;

        let response = TaskStatusResponse::new(
            task_name.to_string(),
            state.status.to_string(),
        );

        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;

        Ok(Response::from_json(&response)?.with_headers(headers))
    }

    /// 处理 /pending-tools 端点 - 获取待处理的工具调用
    pub async fn handle_get_pending_tools(
        task_name: &str,
        storage: &worker::Storage,
    ) -> Result<Response> {
        console_log!("[PENDING-TOOLS] Getting pending tools for task: {}", task_name);

        // 加载待处理的工具调用
        let pending_tools = TaskPersistence::load_pending_tool_calls(storage, task_name).await?;

        let response = serde_json::json!({
            "task_id": task_name,
            "toolCalls": pending_tools.unwrap_or_default(),
        });

        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;

        Ok(Response::from_json(&response)?.with_headers(headers))
    }

    /// 处理 /tool-result 端点
    pub async fn handle_tool_result(
        task_name: &str,
        storage: &worker::Storage,
        mut req: Request,
        get_current_timestamp_millis: impl Fn() -> u64,
    ) -> Result<Response> {
        console_log!("[TOOL-RESULT] === START PROCESSING TOOL RESULTS ===");
        console_log!("[TOOL-RESULT] Processing tool results for task: {}", task_name);

        let mut state = Self::load_state_from_storage(storage, task_name).await?;
        console_log!("[TOOL-RESULT] Current task state: status={:?}, progress={}", state.status, state.progress);

        if state.status != TaskStatus::Processing && state.status != TaskStatus::Running {
            console_error!("[TOOL-RESULT] Task is not in a state to accept tool results: {:?}", state.status);
            return Response::error("Task is not in a state to accept tool results", 400);
        }

        console_log!("[TOOL-RESULT] Task state is valid, proceeding to parse tool results");

        // 解析工具结果 - 期望格式: { results: [{ tool, success, result, error, arguments, timestamp }] }
        let tool_results_data: serde_json::Value = req.json().await?;
        console_log!("[TOOL-RESULT] Raw tool results data: {}", serde_json::to_string_pretty(&tool_results_data).unwrap_or_default());
        
        let cloned_data = tool_results_data.clone();
        let tool_results = cloned_data.get("results")
            .and_then(|r| r.as_array())
            .ok_or_else(|| {
                console_error!("[TOOL-RESULT] Invalid tool results format - missing 'results' array");
                worker::Error::RustError("Invalid tool results format".to_string())
            })?;

        console_log!("[TOOL-RESULT] Received {} tool results", tool_results.len());

        // 加载待处理的工具调用，用于获取正确的tool_call_id
        let pending_tool_calls = TaskPersistence::load_pending_tool_calls(storage, task_name).await?
            .unwrap_or_default();
        console_log!("[TOOL-RESULT] Loaded {} pending tool calls", pending_tool_calls.len());
        for (i, tc) in pending_tool_calls.iter().enumerate() {
            console_log!("[TOOL-RESULT] Pending tool call {}: tool={}, id={:?}", i, tc.tool, tc.id);
        }

        // 加载对话历史
        let mut conversation_history = Self::load_conversation_history_from_storage(storage, task_name)
            .await
            .unwrap_or_else(|_| Some(Vec::new()))
            .unwrap_or_default();
        console_log!("[TOOL-RESULT] Loaded conversation history: {} messages", conversation_history.len());

        // 为每个工具结果添加消息
        for (result_index, tool_result) in tool_results.iter().enumerate() {
            console_log!("[TOOL-RESULT] Processing tool result {}: {:?}", result_index, tool_result);
            
            let tool_name = tool_result.get("tool")
                .and_then(|t| t.as_str())
                .ok_or_else(|| worker::Error::RustError("Tool name missing".to_string()))?;

            let success = tool_result.get("success")
                .and_then(|s| s.as_bool())
                .unwrap_or(false);

            let result_content = if success {
                tool_result.get("result")
                    .and_then(|r| r.as_str())
                    .unwrap_or("工具执行成功")
            } else {
                tool_result.get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("工具执行失败")
            };

            // 查找对应的待处理工具调用ID
            let tool_call_id = pending_tool_calls.iter()
                .find(|tc| tc.tool == tool_name)
                .and_then(|tc| tc.id.clone())
                .unwrap_or_else(|| {
                    // 如果找不到匹配的ID，使用工具名和时间戳作为fallback
                    let timestamp = tool_result.get("timestamp")
                        .and_then(|t| t.as_u64())
                        .unwrap_or_else(|| get_current_timestamp_millis());
                    format!("{}_{}", tool_name, timestamp)
                });

            console_log!("[TOOL-RESULT] Adding tool result message for {} (success: {}, tool_call_id: {})", tool_name, success, tool_call_id);

            // 添加工具结果消息到对话历史
            conversation_history.push(AiMessage {
                role: "tool".to_string(),
                content: result_content.to_string(),
                tool_call_id: Some(tool_call_id),
                tool_calls: None,
            });
            
            console_log!("[TOOL-RESULT] Added tool response message, conversation history now has {} messages", conversation_history.len());
        }

        Self::save_conversation_history_to_storage(storage, task_name, &conversation_history).await?;
        console_log!("[TOOL-RESULT] Saved conversation history with {} messages", conversation_history.len());

        // 清除待处理的工具调用
        let pending_key = get_pending_tool_calls_key(task_name);
        storage.delete(&pending_key).await?;
        console_log!("[TOOL-RESULT] Cleared pending tool calls");

        // 保存工具结果
        let tool_key = get_tool_result_key(task_name);
        storage.put(&tool_key, tool_results_data).await?;
        console_log!("[TOOL-RESULT] Saved tool results");

        // 更新状态
        state.progress = (state.progress + 0.2).min(0.9);
        state.current_step = format!("已处理 {} 个工具结果，继续执行", tool_results.len());
        state.updated_at = get_current_timestamp_millis();
        Self::save_state_to_storage(storage, task_name, &state).await?;
        console_log!("[TOOL-RESULT] Updated task state: progress={}, step='{}'", state.progress, state.current_step);

        console_log!("[TOOL-RESULT] Tool results processed, setting alarm to continue");

        // 设置alarm来继续执行（500ms后，提高响应速度）
        let now_ms = get_current_timestamp_millis();
        storage.set_alarm((now_ms + 500) as i64).await?;

        let response = TaskStatusResponse::new(
            task_name.to_string(),
            state.status.to_string(),
        );

        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;

        Ok(Response::from_json(&response)?.with_headers(headers))
    }

    /// 处理 /init-and-start 端点
    pub async fn handle_init_and_start(
        task_name: &str,
        storage: &worker::Storage,
        req: Request,
        get_current_timestamp_millis: impl Fn() -> u64,
    ) -> Result<Response> {
        console_log!("[INIT-START] Handling init-and-start for task: {}", task_name);

        let mut req = req;
        let request: CompatibleRequest = req.json().await.map_err(|e| {
            console_error!("[AITaskDO] Failed to parse request: {}", e);
            worker::Error::RustError(format!("Failed to parse request: {}", e))
        })?;

        console_log!("[INIT-START] Request parsed successfully, prompt length: {}, tools: {}", request.prompt.len(), request.tools.len());

        // 保存请求和初始状态
        Self::save_request_to_storage(storage, task_name, &request).await?;
        console_log!("[INIT-START] Request saved to storage");

        // 清除旧的对话历史，确保从干净的状态开始
        let conversation_history_key = get_conversation_history_key(task_name);
        storage.delete(&conversation_history_key).await?;
        console_log!("[INIT-START] Cleared old conversation history");

        let now = current_timestamp_secs();
        let initial_state = TaskState {
            status: TaskStatus::Queued,
            progress: 0.0,
            current_step: "等待执行".to_string(),
            created_at: now,
            updated_at: now,
            error: None,
        };
        Self::save_state_to_storage(storage, task_name, &initial_state).await?;
        console_log!("[INIT-START] Initial state saved: status={}, created_at={}", initial_state.status, initial_state.created_at);

        // 设置alarm - 采用"Fire and Forget"模式，只设置alarm，不直接执行
        let now_ms = get_current_timestamp_millis();
        console_log!("[INIT-START] Current timestamp: {}ms", now_ms);
        
        // 确保alarm时间至少是未来100ms，避免立即触发或过去时间
        let min_scheduled_time = now_ms + 100;
        let scheduled_time = std::cmp::max(now_ms + 1000, min_scheduled_time);
        
        console_log!("[INIT-START] Setting alarm for execution at {}ms (current: {}ms, delay: {}ms)", 
                     scheduled_time, now_ms, scheduled_time - now_ms);
        
        // 记录详细的alarm设置信息
        console_log!("[INIT-START] Alarm target timestamp: {} (i64: {})", scheduled_time, scheduled_time as i64);
        
        // 设置alarm并检查结果
        match storage.set_alarm(scheduled_time as i64).await {
            Ok(_) => console_log!("[INIT-START] Alarm set successfully!"),
            Err(e) => {
                console_error!("[INIT-START] FAILED to set alarm: {}", e);
                return Err(e);
            }
        }

        let response = TaskStatusResponse::new(
            task_name.to_string(),
            "scheduled".to_string(),
        );

        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;

        console_log!("[INIT-START] Init-and-start completed successfully for task: {}", task_name);
        Ok(Response::from_json(&response)?.with_headers(headers))
    }

    /// 处理 /start 端点
    pub async fn handle_start(
        task_name: &str,
        storage: &worker::Storage,
        get_current_timestamp_millis: impl Fn() -> u64,
    ) -> Result<Response> {
        let state = Self::load_state_from_storage(storage, task_name).await?;

        match state.status {
            TaskStatus::Completed => return Response::error("Task already completed", 400),
            TaskStatus::Failed => console_log!("Task is in failed state, attempting to restart"),
            TaskStatus::Running => console_log!("Task is already running"),
            TaskStatus::Queued => {
                let mut new_state = state;
                new_state.status = TaskStatus::Running;
                new_state.progress = 0.1;
                new_state.current_step = "开始执行".to_string();
                new_state.updated_at = get_current_timestamp_millis();
                Self::save_state_to_storage(storage, task_name, &new_state).await?;
            }
            TaskStatus::Processing => console_log!("Task is processing"),
        }

        // 设置Alarm - 采用"Fire and Forget"模式，5秒后开始执行
        let now_ms = get_current_timestamp_millis();
        console_log!("[START] Current timestamp: {}ms", now_ms);
        
        // 确保alarm时间至少是未来100ms，避免立即触发或过去时间
        let min_scheduled_time = now_ms + 100;
        let alarm_at = std::cmp::max(now_ms + 5000, min_scheduled_time);
        
        console_log!("[START] Setting alarm for execution at {}ms (current: {}ms, delay: {}ms)", 
                     alarm_at, now_ms, alarm_at - now_ms);
        
        // 记录详细的alarm设置信息
        console_log!("[START] Alarm target timestamp: {} (i64: {})", alarm_at, alarm_at as i64);
        
        // 设置alarm并检查结果
        match storage.set_alarm(alarm_at as i64).await {
            Ok(_) => console_log!("[START] Alarm set successfully!"),
            Err(e) => {
                console_error!("[START] FAILED to set alarm: {}", e);
                return Err(e);
            }
        }

        let response = TaskStatusResponse::new(
            task_name.to_string(),
            "scheduled".to_string(),
        );

        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;

        Ok(Response::from_json(&response)?.with_headers(headers))
    }

    // ===== 存储辅助方法 =====

    /// 保存请求到存储
    async fn save_request_to_storage(
        storage: &worker::Storage,
        task_name: &str,
        request: &CompatibleRequest,
    ) -> Result<()> {
        let request_key = get_request_key(task_name);
        storage.put(&request_key, request).await?;
        Ok(())
    }

    /// 从存储加载状态
    async fn load_state_from_storage(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<TaskState> {
        console_log!("[AITaskDO] load_state called");

        let state_key = get_state_key(task_name);
        let state_data = storage.get::<serde_json::Value>(&state_key).await?;

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

        console_log!("[AITaskDO] Successfully loaded state");
        Ok(TaskState {
            status,
            progress,
            current_step,
            created_at,
            updated_at,
            error,
        })
    }

    /// 保存状态到存储
    async fn save_state_to_storage(
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

        console_log!("[DEBUG] Saving state");
        let state_key = get_state_key(task_name);
        storage.put(&state_key, state_data).await?;
        Ok(())
    }

    /// 从存储加载结果
    async fn load_result_from_storage(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<Option<CompatibleResponse>> {
        let result_key = get_result_key(task_name);
        Ok(storage.get::<CompatibleResponse>(&result_key).await.ok())
    }

    /// 从存储加载对话历史
    async fn load_conversation_history_from_storage(
        storage: &worker::Storage,
        task_name: &str,
    ) -> Result<Option<Vec<AiMessage>>> {
        let key = get_conversation_history_key(task_name);
        Ok(storage.get::<Vec<AiMessage>>(&key).await.ok())
    }

    /// 保存对话历史到存储
    async fn save_conversation_history_to_storage(
        storage: &worker::Storage,
        task_name: &str,
        history: &[AiMessage],
    ) -> Result<()> {
        let key = get_conversation_history_key(task_name);
        storage.put(&key, history).await?;
        Ok(())
    }
}
