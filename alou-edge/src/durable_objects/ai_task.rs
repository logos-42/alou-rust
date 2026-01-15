//! AITaskDO - AI任务Durable Object
//! 
//! 负责管理AI任务的执行状态、进度跟踪和结果存储

use crate::compatibility::models::{
    CompatibleRequest, CompatibleResponse, TaskStatus, TaskStatusResponse,
};
use crate::durable_objects::{
    ai_task_state::{TaskState, TaskCache, get_state_key, get_request_key, get_result_key, get_tool_result_key},
    ai_task_execution::{TaskExecutor, handle_alarm},
    ai_task_ai::DefaultAiCaller,
};
use crate::agent::ai_client::AiClient;
use serde_json::Value;
use std::cell::RefCell;
use crate::utils::time::current_timestamp_secs;

use worker::{
    durable_object, Env, Headers, Method, Request, Response, Result, State, console_error, console_log,
};

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use futures::future::{select, Either};
#[cfg(target_arch = "wasm32")]
use std::pin::Pin;

/// AI任务Durable Object
#[durable_object]
pub struct AITaskDO {
    state: State,
    env: Env,
    cache: RefCell<Option<TaskCache>>,
    is_executing: RefCell<bool>,
    debug_logs: RefCell<Vec<String>>,
    ai_caller: DefaultAiCaller,
}

impl DurableObject for AITaskDO {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
            cache: RefCell::new(None),
            is_executing: RefCell::new(false),
            debug_logs: RefCell::new(Vec::new()),
            ai_caller: DefaultAiCaller,
        }
    }
    
    async fn fetch(&self, req: Request) -> Result<Response> {
        console_log!("[AITaskDO] fetch called, path: {}", req.path());
        
        let path = req.path();
        let method = req.method();
        
        match (&method, path.as_str()) {
            (Method::Get, "/status") => self.handle_get_status().await,
            (Method::Post, "/init") => self.handle_init(req).await,
            (Method::Post, "/start") => self.handle_start().await,
            (Method::Post, "/init-and-start") => self.handle_init_and_start(req).await,
            (Method::Post, "/cancel") => self.handle_cancel().await,
            (Method::Post, "/tool-result") => self.handle_tool_result(req).await,
            _ => {
                console_log!("[AITaskDO] Unknown route: {:?} {}", method, path);
                Response::error("Not found", 404)
            }
        }
    }
    
    async fn alarm(&self) -> Result<Response> {
        console_error!("!!! ALARM ACTIVE !!! Task: {}", self.task_name());
        console_log!("🚨 AITaskDO ALARM STARTED");
        console_log!("[DEBUG] Current DO ID: {}", self.state.id().to_string());
        
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
        
        handle_alarm(self).await
    }
}

impl AITaskDO {
    // HTTP路由处理器
    async fn handle_init(&self, mut req: Request) -> Result<Response> {
        let request: CompatibleRequest = req.json().await?;
        self.save_request(&request).await?;
        
        let initial_state = TaskState::new_initial();
        self.save_state(&initial_state).await?;
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            initial_state.status.to_string(),
        );
        
        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    async fn handle_get_status(&self) -> Result<Response> {
        console_log!("[STATUS] Getting status for task: {}", self.task_name());
        
        let state = self.load_state().await.map_err(|e| {
            console_error!("[STATUS] Failed to load state: {}", e);
            e
        })?;
        
        let response = TaskStatusResponse {
            task_id: self.task_name(),
            status: state.status.to_string(),
            progress: Some(state.progress),
            current_step: Some(state.current_step),
            result: None,
            error: state.error,
            created_at: state.created_at,
            updated_at: state.updated_at,
        };
        
        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    async fn handle_cancel(&self) -> Result<Response> {
        let mut state = self.load_state().await?;
        
        if state.status != TaskStatus::Running && state.status != TaskStatus::Queued {
            return Response::error("Task cannot be cancelled", 400);
        }
        
        state.status = TaskStatus::Failed;
        state.error = Some("任务被取消".to_string());
        state.progress = 1.0;
        state.current_step = "已取消".to_string();
        state.updated_at = current_timestamp_secs();
        self.save_state(&state).await?;
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            state.status.to_string(),
        );
        
        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    async fn handle_tool_result(&self, mut req: Request) -> Result<Response> {
        let mut state = self.load_state().await?;
        
        if state.status != TaskStatus::Running {
            return Response::error("Task is not running", 400);
        }
        
        let tool_result: Value = req.json().await?;
        
        state.progress = (state.progress + 0.2).min(0.9);
        state.current_step = "处理工具结果".to_string();
        state.updated_at = current_timestamp_secs();
        
        let tool_key = get_tool_result_key(&self.task_name());
        self.state.storage().put(&tool_key, tool_result).await?;
        self.save_state(&state).await?;
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            state.status.to_string(),
        );
        
        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    // 内部辅助方法
    fn task_name(&self) -> String {
        match self.state.id().name() {
            Some(name) => name.to_string(),
            None => self.state.id().to_string()
        }
    }
    
    async fn load_state(&self) -> Result<TaskState> {
        console_log!("[AITaskDO] load_state called");
        
        let storage = self.state.storage();
        let state_key = get_state_key(&self.task_name());
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
    
    async fn save_state(&self, state: &TaskState) -> Result<()> {
        let storage = self.state.storage();
        
        let state_data = serde_json::json!({
            "status": state.status.to_string(),
            "progress": state.progress,
            "current_step": state.current_step,
            "created_at": state.created_at,
            "updated_at": state.updated_at,
            "error": state.error,
        });
        
        console_log!("[DEBUG] Saving state");
        let state_key = get_state_key(&self.task_name());
        storage.put(&state_key, state_data).await?;
        Ok(())
    }
    
    async fn get_request(&self) -> Result<Option<CompatibleRequest>> {
        let storage = self.state.storage();
        let request_key = get_request_key(&self.task_name());
        Ok(storage.get::<CompatibleRequest>(&request_key).await.ok())
    }
    
    async fn save_request(&self, request: &CompatibleRequest) -> Result<()> {
        let storage = self.state.storage();
        let request_key = get_request_key(&self.task_name());
        storage.put(&request_key, request).await?;
        Ok(())
    }
    
    async fn save_result(&self, result: &CompatibleResponse) -> Result<()> {
        let storage = self.state.storage();
        let result_key = get_result_key(&self.task_name());
        storage.put(&result_key, result).await?;
        Ok(())
    }
    
    fn get_current_timestamp(&self) -> u64 {
        current_timestamp_secs()
    }
    
    fn get_current_timestamp_millis(&self) -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::Date;
            Date::now() as u64
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        }
    }
}

// 实现TaskExecutor trait
impl TaskExecutor for AITaskDO {
    async fn handle_init_and_start(&self, req: Request) -> Result<Response> {
        console_log!("[INIT-START] Handling init-and-start for task: {}", self.task_name());
        
        let mut req = req;
        let request: CompatibleRequest = req.json().await.map_err(|e| {
            console_error!("[AITaskDO] Failed to parse request: {}", e);
            worker::Error::RustError(format!("Failed to parse request: {}", e))
        })?;
        
        // 保存请求和初始状态
        self.save_request(&request).await?;
        
        let now = current_timestamp_secs();
        let initial_state = TaskState {
            status: TaskStatus::Queued,
            progress: 0.0,
            current_step: "等待执行".to_string(),
            created_at: now,
            updated_at: now,
            error: None,
        };
        self.save_state(&initial_state).await?;
        
        // 设置Alarm
        let now_ms = self.get_current_timestamp_millis();
        let scheduled_time = now_ms + 5000;
        self.state.storage().set_alarm(scheduled_time as i64).await?;
        
        // 直接执行任务（测试）
        console_log!("[TEST] Direct task execution");
        let _ = self.execute_task().await;
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            "scheduled".to_string(),
        );
        
        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }
    
    async fn handle_start(&self) -> Result<Response> {
        let state = self.load_state().await?;
        
        match state.status {
            TaskStatus::Completed => return Response::error("Task already completed", 400),
            TaskStatus::Failed => console_log!("Task is in failed state, attempting to restart"),
            TaskStatus::Running => console_log!("Task is already running"),
            TaskStatus::Queued => {
                let mut new_state = state;
                new_state.status = TaskStatus::Running;
                new_state.progress = 0.1;
                new_state.current_step = "开始执行".to_string();
                new_state.updated_at = current_timestamp_secs();
                self.save_state(&new_state).await?;
            }
            TaskStatus::Processing => console_log!("Task is processing"),
        }
        
        // 设置Alarm
        let now_ms = self.get_current_timestamp_millis();
        let alarm_at = now_ms + 100;
        self.state.storage().set_alarm(alarm_at as i64).await?;
        
        let response = TaskStatusResponse::new(
            self.task_name(),
            "scheduled".to_string(),
        );
        
        let headers = Headers::new();
        headers.set("Content-Type", "application/json; charset=utf-8")?;
        
        Ok(Response::from_json(&response)?.with_headers(headers))
    }

    async fn execute_task(&self) -> Result<()> {
        console_log!("[EXECUTE] Starting task execution");
        
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
        
        // 1. 更新状态
        let mut state = self.load_state().await?;
        state.progress = 0.2;
        state.current_step = "加载请求数据".to_string();
        state.updated_at = self.get_current_timestamp();
        self.save_state(&state).await?;
        
        // 2. 获取请求数据
        let request = match self.get_request().await? {
            Some(req) => req,
            None => {
                state.status = TaskStatus::Failed;
                state.error = Some("请求数据不存在".to_string());
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(worker::Error::RustError("Request data not found".to_string()));
            }
        };
        
        // 3. 初始化AI客户端
        state.progress = 0.3;
        state.current_step = "初始化AI客户端".to_string();
        self.save_state(&state).await?;
        
        let ai_api_key = match self.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    state.status = TaskStatus::Failed;
                    state.error = Some("AI_API_KEY 或 DEEPSEEK_API_KEY 未配置".to_string());
                    state.progress = 1.0;
                    self.save_state(&state).await?;
                    return Err(worker::Error::RustError("AI key not configured".to_string()));
                }
            }
        };
        
        let ai_client = AiClient::new("deepseek", ai_api_key, Some(request.model.clone()))
            .map_err(|e| {
                state.status = TaskStatus::Failed;
                state.error = Some(format!("创建AI客户端失败: {}", e));
                state.progress = 1.0;
                worker::Error::RustError(format!("Failed to create AI client: {}", e))
            })?;
        
        // 4. 准备消息和工具
        let messages = self.ai_caller.convert_to_ai_messages(&request);
        let tools = self.ai_caller.convert_to_ai_tools(&request.tools);
        
        // 5. 调用AI服务
        state.progress = 0.5;
        state.current_step = "调用AI服务".to_string();
        self.save_state(&state).await?;
        
        match self.ai_caller.call_ai_with_timeout(ai_client, messages, tools, &self.task_name()).await {
            Ok(ai_response) => {
                // 保存结果
                let response = self.ai_caller.convert_from_ai_response(ai_response, &self.task_name());
                let _ = self.save_result(&response).await;
                
                // 更新状态为完成
                state.status = TaskStatus::Completed;
                state.progress = 1.0;
                state.current_step = "任务完成".to_string();
                state.updated_at = self.get_current_timestamp();
                self.save_state(&state).await?;
                
                console_log!("✅ [EXECUTE] Task completed");
                Ok(())
            }
            Err(e) => {
                // 更新状态为失败
                state.status = TaskStatus::Failed;
                state.error = Some(format!("AI服务错误: {}", e));
                state.progress = 1.0;
                state.current_step = "任务失败".to_string();
                state.updated_at = self.get_current_timestamp();
                self.save_state(&state).await?;
                
                console_error!("❌ [EXECUTE] Task failed: {}", e);
                Err(e)
            }
        }
    }
    
    async fn execute_task_simplified(&self) -> Result<()> {
        console_log!("=== SIMPLIFIED TASK EXECUTION START ===");
        
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
        
        // 1. 快速更新状态
        let now = self.get_current_timestamp();
        let running_state = serde_json::json!({
            "status": "running",
            "progress": 0.3,
            "current_step": "简化执行中",
            "created_at": now,
            "updated_at": now,
            "error": serde_json::Value::Null
        });
        
        let storage = self.state.storage();
        let state_key = get_state_key(&self.task_name());
        storage.put(&state_key, running_state).await?;
        
        // 2. 获取请求数据
        let request_key = get_request_key(&self.task_name());
        let request = storage.get::<CompatibleRequest>(&request_key).await?;
        
        // 3. 简单的AI调用
        let ai_api_key = match self.env.secret("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => match self.env.secret("DEEPSEEK_API_KEY") {
                Ok(key) => key.to_string(),
                Err(_) => {
                    return Err(worker::Error::RustError("No API key configured".to_string()));
                }
            }
        };
        
        let ai_client = AiClient::new("deepseek", ai_api_key, Some(request.model.clone()))?;
        
        let messages = vec![
            crate::agent::ai_client::AiMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
                tool_call_id: None,
                tool_calls: None,
            }
        ];
        
        match self.ai_caller.call_ai_simplified(ai_client, messages, &self.task_name()).await {
            Ok(ai_response) => {
                let response = CompatibleResponse {
                    success: true,
                    response: Some(ai_response.content),
                    tool_calls: None,
                    metadata: None,
                    task_id: Some(self.task_name()),
                    status: Some("completed".to_string()),
                    progress: Some(1.0),
                    estimated_time: None,
                    error: None,
                };
                
                let result_key = get_result_key(&self.task_name());
                let _ = storage.put(&result_key, &response).await;
                
                let completed_state = serde_json::json!({
                    "status": "completed",
                    "progress": 1.0,
                    "current_step": "任务完成",
                    "created_at": now,
                    "updated_at": self.get_current_timestamp(),
                    "error": serde_json::Value::Null
                });
                
                let state_key = get_state_key(&self.task_name());
                let _ = storage.put(&state_key, completed_state).await;
                
                console_log!("✅ [SIMPLIFIED] Task completed");
                Ok(())
            }
            Err(e) => {
                console_error!("❌ [SIMPLIFIED] Task failed: {}", e);
                Err(e)
            }
        }
    }
    
    async fn execute_task_with_timeout(&self) -> Result<()> {
        self.execute_task().await
    }
    
    async fn load_state_with_timeout(&self) -> Result<TaskState> {
        #[cfg(target_arch = "wasm32")]
        {
            let state_future = self.load_state();
            let timeout_future = TimeoutFuture::new(5_000);
            
            match select(Pin::from(Box::pin(state_future)), Pin::from(Box::pin(timeout_future))).await {
                Either::Left((Ok(state), _)) => Ok(state),
                Either::Left((Err(e), _)) => Err(e),
                Either::Right((_, _)) => {
                    Err(worker::Error::RustError("State loading timeout".to_string()))
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.load_state().await
        }
    }
}

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_state_creation() {
        let state = TaskState::new_initial();
        assert_eq!(state.status, TaskStatus::Queued);
        assert_eq!(state.progress, 0.0);
    }
    
    #[test]
    fn test_task_state_failed() {
        let state = TaskState::new_failed("test error".to_string());
        assert_eq!(state.status, TaskStatus::Failed);
        assert_eq!(state.progress, 1.0);
        assert_eq!(state.error, Some("test error".to_string()));
    }
}
