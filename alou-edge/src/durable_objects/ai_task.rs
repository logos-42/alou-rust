//! AITaskDO - AI任务Durable Object
//! 
//! 负责管理AI任务的执行状态、进度跟踪和结果存储

use crate::compatibility::models::{
    CompatibleRequest, CompatibleResponse, TaskStatus, TaskStatusResponse,
};
use serde_json::Value;
use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};
use wasm_bindgen_futures::spawn_local;
use worker::{
    durable_object, Env, Method, Request, Response, Result, console_error, console_log,
};

/// AI任务Durable Object
#[durable_object]
pub struct AITaskDO {
    /// 状态存储
    state: worker::State,
    /// 环境
    env: Env,
    /// 内存缓存（可选，用于性能优化）
    cache: RefCell<Option<TaskCache>>,
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
            _ => {
                Response::error("Not found", 404)
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
        
        // 异步开始执行（不阻塞响应）
        let task_id = self.task_id();
        
        // 在实际实现中，这里会调用任务执行器
        // 现在先记录日志
        console_log!("Task {} started execution", task_id);
        
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
    
}
