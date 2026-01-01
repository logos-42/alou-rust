//! AITaskDO - AI任务Durable Object
//! 
//! 负责管理AI任务的执行状态、进度跟踪和结果存储

use crate::compatibility::models::{
    CompatibleRequest, CompatibleResponse, TaskStatus, TaskStatusResponse,
};
use crate::agent::ai_client::{AiClient, AiMessage, AiTool};
use serde_json::Value;
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};
use wasm_bindgen_futures::spawn_local;
use worker::{
    durable_object, Env, Method, Request, Response, Result, console_error, console_log,
};

// 用于捕获 panic 的 hook
#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

/// AI任务Durable Object
#[durable_object]
pub struct AITaskDO {
    /// 状态存储
    state: worker::State,
    /// 环境
    env: Env,
    /// 内存缓存（可选，用于性能优化）
    cache: RefCell<Option<TaskCache>>,
    /// 是否正在执行任务
    is_executing: RefCell<bool>,
}

/// 任务缓存（内存中的状态副本）
struct TaskCache {
    task_id: String,
    status: TaskStatus,
    progress: f32,
    current_step: String,
    created_at: u64,
    updated_at: u64,
}

/// 任务状态
struct TaskState {
    status: TaskStatus,
    progress: f32,
    current_step: String,
    created_at: u64,
    updated_at: u64,
    error: Option<String>,
}

impl DurableObject for AITaskDO {
    fn new(state: worker::State, env: Env) -> Self {
        Self {
            state,
            env,
            cache: RefCell::new(None),
            is_executing: RefCell::new(false),
        }
    }
    
    async fn fetch(&self, req: Request) -> std::result::Result<Response, worker::Error> {
        let url = req.url()?;
        let path = url.path();
        
        match path {
            "/init" if req.method() == Method::Post => {
                self.handle_init(req).await
            }
            "/start" if req.method() == Method::Post => {
                self.handle_start().await
            }
            "/status" if req.method() == Method::Get => {
                self.handle_get_status().await
            }
            "/cancel" if req.method() == Method::Post => {
                self.handle_cancel().await
            }
            "/tool-result" if req.method() == Method::Post => {
                self.handle_tool_result(req).await
            }
            "/execute" if req.method() == Method::Post => {
                self.handle_execute().await
            }
            _ => {
                Response::error("Not found", 404)
            }
        }
    }
    
    async fn alarm(&self) -> std::result::Result<Response, worker::Error> {
        console_log!("AITaskDO alarm triggered for task: {}", self.task_id());
        
        // 设置 panic hook
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
        
        // 检查是否已经在执行
        if *self.is_executing.borrow() {
            console_log!("Task {} is already executing, skipping alarm", self.task_id());
            return Response::ok("Already executing");
        }
        
        // 标记为正在执行
        *self.is_executing.borrow_mut() = true;
        
        // 直接调用处理逻辑，并等待它完成！
        // 不要在这里使用 spawn_local
        let result = self.handle_execute().await;
        
        // 执行完毕后重置标记
        *self.is_executing.borrow_mut() = false;
        
        match result {
            Ok(resp) => {
                console_log!("Task {} executed successfully via alarm", self.task_id());
                Ok(resp)
            }
            Err(e) => {
                console_error!("Task {} execution failed in alarm: {}", self.task_id(), e);
                Err(e)
            }
        }
    }
}

impl AITaskDO {
    /// 处理任务初始化
    async fn handle_init(&self, mut req: Request) -> Result<Response> {
        let request: CompatibleRequest = match req.json().await {
            Ok(req) => req,
            Err(e) => {
                return Response::error(
                    format!("Failed to parse request: {}", e),
                    400,
                );
            }
        };
        
        // 保存请求
        if let Err(e) = self.save_request(&request).await {
            return Response::error(format!("Failed to save request: {}", e), 500);
        }
        
        // 创建初始状态
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let initial_state = TaskState {
            status: TaskStatus::Queued,
            progress: 0.0,
            current_step: "等待执行".to_string(),
            created_at: now,
            updated_at: now,
            error: None,
        };
        
        // 保存状态
        if let Err(e) = self.save_state(&initial_state).await {
            return Response::error(format!("Failed to save initial state: {}", e), 500);
        }
        
        Response::from_json(&TaskStatusResponse::new(
            self.task_id(),
            initial_state.status.to_string(),
        ))
    }
    
    /// 处理任务开始执行
    async fn handle_start(&self) -> Result<Response> {
        let mut state = self.load_state().await?;
        
        if state.status != TaskStatus::Queued {
            return Response::error("Task is not in queued state", 400);
        }
        
        // 更新状态
        state.status = TaskStatus::Running;
        state.progress = 0.1;
        state.current_step = "开始执行".to_string();
        state.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // 保存状态
        if let Err(e) = self.save_state(&state).await {
            console_error!("Failed to save task state: {}", e);
        }
        
        // 设置 Alarm 来触发任务执行（10毫秒后）
        // 这样即使 fetch 返回了，Cloudflare 也会保证 DO 被唤醒并执行 alarm 逻辑
        let storage = self.state.storage();
        if let Err(e) = storage.set_alarm(10).await {
            console_error!("Failed to set alarm: {}", e);
            return Response::error(format!("Failed to schedule task execution: {}", e), 500);
        }
        
        console_log!("Task {} scheduled for execution via alarm", self.task_id());
        
        Response::from_json(&TaskStatusResponse::new(
            self.task_id(),
            state.status.to_string(),
        ))
    }
    
    /// 处理获取状态
    async fn handle_get_status(&self) -> Result<Response> {
        let state = self.load_state().await?;
        let result = self.get_result().await?;
        
        let response = TaskStatusResponse {
            task_id: self.task_id(),
            status: state.status.to_string(),
            progress: Some(state.progress),
            current_step: Some(state.current_step),
            result,
            error: state.error,
            created_at: state.created_at,
            updated_at: state.updated_at,
        };
        
        Response::from_json(&response)
    }
    
    /// 处理任务取消
    async fn handle_cancel(&self) -> Result<Response> {
        let mut state = self.load_state().await?;
        
        if state.status != TaskStatus::Running && state.status != TaskStatus::Queued {
            return Response::error("Task cannot be cancelled in current state", 400);
        }
        
        state.status = TaskStatus::Failed;
        state.error = Some("任务被取消".to_string());
        state.progress = 1.0;
        state.current_step = "已取消".to_string();
        state.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // 保存状态
        if let Err(e) = self.save_state(&state).await {
            console_error!("Failed to save task state: {}", e);
        }
        
        Response::from_json(&TaskStatusResponse::new(
            self.task_id(),
            state.status.to_string(),
        ))
    }
    
    /// 处理任务执行（由 alarm 触发）
    async fn handle_execute(&self) -> Result<Response> {
        console_log!("Task execution started for: {}", self.task_id());
        
        // 执行任务
        match self.execute_task().await {
            Ok(_) => {
                console_log!("Task {} executed successfully", self.task_id());
                Response::ok("Task executed successfully")
            }
            Err(e) => {
                console_error!("Task {} execution failed: {}", self.task_id(), e);
                Response::error(format!("Task execution failed: {}", e), 500)
            }
        }
    }
    
    /// 处理工具调用结果
    async fn handle_tool_result(&self, mut req: Request) -> Result<Response> {
        let mut state = self.load_state().await?;
        
        if state.status != TaskStatus::Running {
            return Response::error("Task is not running", 400);
        }
        
        let tool_result: Value = match req.json().await {
            Ok(result) => result,
            Err(e) => {
                return Response::error(
                    format!("Failed to parse tool result: {}", e),
                    400,
                );
            }
        };
        
        // 更新进度
        state.progress = (state.progress + 0.2).min(0.9);
        state.current_step = "处理工具结果".to_string();
        state.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // 保存工具结果
        let storage = self.state.storage();
        if let Err(e) = storage.put("last_tool_result", tool_result).await {
            console_error!("Failed to save tool result: {}", e);
        }
        
        // 保存状态
        if let Err(e) = self.save_state(&state).await {
            console_error!("Failed to save task state: {}", e);
        }
        
        Response::from_json(&TaskStatusResponse::new(
            self.task_id(),
            state.status.to_string(),
        ))
    }
    
    /// 获取任务ID
    fn task_id(&self) -> String {
        self.state.id().to_string()
    }
    
    /// 从存储加载状态
    async fn load_state(&self) -> Result<TaskState> {
        let storage = self.state.storage();
        
        if let Ok(state_data) = storage.get::<Value>("state").await {
            let status = if let Some(status_str) = state_data.get("status").and_then(|s| s.as_str()) {
                match status_str {
                    "queued" => TaskStatus::Queued,
                    "running" => TaskStatus::Running,
                    "completed" => TaskStatus::Completed,
                    "failed" => TaskStatus::Failed,
                    _ => TaskStatus::Queued,
                }
            } else {
                TaskStatus::Queued
            };
            
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
                .unwrap_or_else(|| {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                });
            
            let updated_at = state_data.get("updated_at")
                .and_then(|u| u.as_u64())
                .unwrap_or(created_at);
            
            let error = state_data.get("error")
                .and_then(|e| e.as_str())
                .map(|e| e.to_string());
            
            Ok(TaskState {
                status,
                progress,
                current_step,
                created_at,
                updated_at,
                error,
            })
        } else {
            // 初始状态
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            Ok(TaskState {
                status: TaskStatus::Queued,
                progress: 0.0,
                current_step: "初始化".to_string(),
                created_at: now,
                updated_at: now,
                error: None,
            })
        }
    }
    
    /// 保存状态到存储
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
        
        storage.put("state", state_data).await?;
        Ok(())
    }
    
    /// 获取请求数据
    async fn get_request(&self) -> Result<Option<CompatibleRequest>> {
        let storage = self.state.storage();
        
        if let Ok(request) = storage.get::<CompatibleRequest>("request").await {
            Ok(Some(request))
        } else {
            Ok(None)
        }
    }
    
    /// 保存请求数据
    async fn save_request(&self, request: &CompatibleRequest) -> Result<()> {
        let storage = self.state.storage();
        storage.put("request", request).await?;
        Ok(())
    }
    
    /// 获取结果数据
    async fn get_result(&self) -> Result<Option<CompatibleResponse>> {
        let storage = self.state.storage();
        
        if let Ok(result) = storage.get::<CompatibleResponse>("result").await {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
    
    /// 保存结果数据
    async fn save_result(&self, result: &CompatibleResponse) -> Result<()> {
        let storage = self.state.storage();
        storage.put("result", result).await?;
        Ok(())
    }
    
    /// 执行AI任务
    async fn execute_task(&self) -> Result<()> {
        console_log!("Starting task execution for: {}", self.task_id());
        
        // 设置 panic hook 来捕获 panic
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
        
        // 更新状态为执行中
        let mut state = self.load_state().await?;
        state.progress = 0.2;
        state.current_step = "加载请求数据".to_string();
        state.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.save_state(&state).await?;
        
        // 获取请求数据
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
        
        // 更新状态
        state.progress = 0.3;
        state.current_step = "初始化AI客户端".to_string();
        self.save_state(&state).await?;
        
        // 创建AI客户端
        let ai_api_key = match self.env.var("AI_API_KEY") {
            Ok(key) => key.to_string(),
            Err(_) => {
                state.status = TaskStatus::Failed;
                state.error = Some("AI_API_KEY 未配置".to_string());
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(worker::Error::RustError("AI_API_KEY not configured".to_string()));
            }
        };
        
        let ai_client = match AiClient::new("deepseek", ai_api_key, Some(request.model.clone())) {
            Ok(client) => client,
            Err(e) => {
                state.status = TaskStatus::Failed;
                state.error = Some(format!("创建AI客户端失败: {}", e));
                state.progress = 1.0;
                self.save_state(&state).await?;
                return Err(worker::Error::RustError(format!("Failed to create AI client: {}", e)));
            }
        };
        
        // 转换消息格式
        let messages = self.convert_to_ai_messages(&request);
        let tools = self.convert_to_ai_tools(&request.tools);
        
        // 更新状态
        state.progress = 0.5;
        state.current_step = "调用AI服务".to_string();
        self.save_state(&state).await?;
        
        // 调用AI服务
        match ai_client.send_message(messages, Some(tools)).await {
            Ok(ai_response) => {
                // 转换响应格式
                let response = self.convert_from_ai_response(ai_response);
                
                // 保存结果
                if let Err(e) = self.save_result(&response).await {
                    console_error!("Failed to save result: {}", e);
                }
                
                // 更新状态为完成
                state.status = TaskStatus::Completed;
                state.progress = 1.0;
                state.current_step = "任务完成".to_string();
                state.updated_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                self.save_state(&state).await?;
                
                console_log!("Task {} completed successfully", self.task_id());
                Ok(())
            }
            Err(e) => {
                state.status = TaskStatus::Failed;
                state.error = Some(format!("AI服务错误: {}", e));
                state.progress = 1.0;
                state.current_step = "任务失败".to_string();
                self.save_state(&state).await?;
                
                console_error!("Task {} failed: {}", self.task_id(), e);
                Err(worker::Error::RustError(format!("AI service error: {}", e)))
            }
        }
    }
    
    /// 转换到AI消息格式
    fn convert_to_ai_messages(&self, request: &CompatibleRequest) -> Vec<AiMessage> {
        let mut messages = Vec::new();
        
        // 添加系统提示
        if let Some(system_prompt) = &request.system_prompt {
            messages.push(AiMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        
        // 添加历史消息
        for history_msg in &request.history {
            messages.push(AiMessage {
                role: history_msg.role.clone(),
                content: history_msg.content.clone(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        
        // 添加当前提示
        messages.push(AiMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
            tool_call_id: None,
            tool_calls: None,
        });
        
        messages
    }
    
    /// 转换到AI工具格式
    fn convert_to_ai_tools(&self, tools: &[crate::compatibility::models::Tool]) -> Vec<AiTool> {
        tools.iter().map(|tool| {
            AiTool {
                name: tool.name.clone(),
                description: tool.description.clone().unwrap_or_default(),
                parameters: tool.parameters.clone().unwrap_or_default(),
            }
        }).collect()
    }
    
    /// 从AI响应转换
    fn convert_from_ai_response(&self, ai_response: crate::agent::ai_client::AiResponse) -> CompatibleResponse {
        CompatibleResponse {
            success: true,
            response: Some(ai_response.content),
            tool_calls: Some(
                ai_response.tool_calls.into_iter().map(|tc| {
                    crate::compatibility::models::ToolCall {
                        tool: tc.name,
                        arguments: tc.arguments,
                    }
                }).collect()
            ),
            metadata: None,
            task_id: Some(self.task_id()),
            status: Some("completed".to_string()),
            progress: Some(1.0),
            estimated_time: None,
            error: None,
        }
    }
    
    
}
