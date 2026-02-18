// Tool API server module - 为 CLI 提供 HTTP API 来执行工具
use tauri::{AppHandle, Manager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    extract::State as AxumState,
    http::Method,
    response::Json,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::bridges::BridgeManager;

// 工具执行请求
#[derive(Deserialize)]
pub struct ToolExecuteRequest {
    pub tool_id: String,
    pub args: serde_json::Value,
    pub working_directory: Option<String>,
    pub timeout_seconds: Option<u64>,
}

// 工具执行响应
#[derive(Serialize)]
pub struct ToolExecuteResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: Option<u64>,
}

// 工具列表项
#[derive(Serialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

// API 共享状态
pub struct ApiState {
    pub app_handle: AppHandle,
}

// 添加工具执行路由
pub async fn add_tool_routes(
    router: Router<Arc<Mutex<Option<ApiState>>>>,
) -> Router<Arc<Mutex<Option<ApiState>>>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    router
        .route("/api/tools/execute", post(execute_tool))
        .route("/api/tools/list", get(list_tools))
        .route("/api/tools/:tool_id/execute", post(execute_tool_by_id))
        .layer(cors)
}

// 执行工具
async fn execute_tool(
    AxumState(state): AxumState<Arc<Mutex<Option<ApiState>>>>,
    Json(payload): Json<ToolExecuteRequest>,
) -> Json<ToolExecuteResponse> {
    let state_guard = state.lock().await;
    let api_state = match state_guard.as_ref() {
        Some(s) => s,
        None => {
            return Json(ToolExecuteResponse {
                success: false,
                data: None,
                error: Some("API server not initialized".to_string()),
                execution_time_ms: None,
            });
        }
    };

    let start_time = std::time::Instant::now();

    // 获取 BridgeManager - 使用 try_state 安全获取
    let bridge_manager: Arc<tokio::sync::Mutex<BridgeManager>> = match api_state
        .app_handle
        .try_state::<Arc<tokio::sync::Mutex<BridgeManager>>>()
    {
        Some(s) => s.inner().clone(),
        None => {
            return Json(ToolExecuteResponse {
                success: false,
                data: None,
                error: Some("BridgeManager not available".to_string()),
                execution_time_ms: None,
            });
        }
    };

    let manager = bridge_manager.lock().await;
    let tool_bridge = manager.tool_bridge();

    let request = crate::bridges::ToolCallRequest {
        session_id: "cli_session".to_string(),
        user_id: None,
        tool_id: payload.tool_id.clone(),
        args: payload.args,
        working_directory: payload.working_directory,
        environment: std::env::vars().collect(),
        timeout_seconds: payload.timeout_seconds,
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    match tool_bridge.handle_request(request).await {
        Ok(response) => {
            if response.success {
                Json(ToolExecuteResponse {
                    success: true,
                    data: response.result.map(|r| r.data),
                    error: None,
                    execution_time_ms: Some(execution_time_ms),
                })
            } else {
                Json(ToolExecuteResponse {
                    success: false,
                    data: None,
                    error: response.error,
                    execution_time_ms: Some(execution_time_ms),
                })
            }
        }
        Err(e) => Json(ToolExecuteResponse {
            success: false,
            data: None,
            error: Some(format!("Tool bridge error: {}", e)),
            execution_time_ms: Some(execution_time_ms),
        }),
    }
}

// 通过 ID 执行工具
async fn execute_tool_by_id(
    AxumState(state): AxumState<Arc<Mutex<Option<ApiState>>>>,
    axum::extract::Path(tool_id): axum::extract::Path<String>,
    Json(payload): Json<ToolExecuteRequest>,
) -> Json<ToolExecuteResponse> {
    // 复用 execute_tool 的逻辑，但使用路径中的 tool_id
    let mut req = payload;
    req.tool_id = tool_id;
    execute_tool(AxumState(state), Json(req)).await
}

// 获取工具列表
async fn list_tools(
    AxumState(state): AxumState<Arc<Mutex<Option<ApiState>>>>,
) -> Json<Vec<ToolInfo>> {
    let state_guard = state.lock().await;
    let api_state = match state_guard.as_ref() {
        Some(s) => s,
        None => {
            return Json(vec![]);
        }
    };

    // 获取 BridgeManager - 使用 try_state 安全获取
    let bridge_manager: Arc<tokio::sync::Mutex<BridgeManager>> = match api_state
        .app_handle
        .try_state::<Arc<tokio::sync::Mutex<BridgeManager>>>()
    {
        Some(s) => s.inner().clone(),
        None => {
            return Json(vec![]);
        }
    };

    let manager = bridge_manager.lock().await;
    let tool_bridge = manager.tool_bridge();

    // 获取工具列表（list_tools 返回 Vec<serde_json::Value>）
    let tools: Vec<serde_json::Value> = tool_bridge.list_tools().await;

    Json(tools.into_iter().map(|t| ToolInfo {
        id: t["id"].as_str().unwrap_or("").to_string(),
        name: t["name"].as_str().unwrap_or("").to_string(),
        description: t["description"].as_str().map(|s| s.to_string()),
        category: t["category"].as_str().map(|s| s.to_string()),
    }).collect())
}

// 启动工具 API 服务器
pub async fn start_tool_api_server(app: AppHandle) -> Result<u16, String> {
    let api_state = ApiState { app_handle: app };
    let state: Arc<Mutex<Option<ApiState>>> = Arc::new(Mutex::new(Some(api_state)));

    // add_tool_routes 是 async fn，需要 .await 后再调用 .with_state()
    let router = add_tool_routes(Router::new()).await.with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind server: {}", e))?;

    let port = listener.local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("Tool API server error: {}", e);
        }
    });

    println!("Tool API server started on port {}", port);
    Ok(port)
}
