//! Bot Gateway HTTP 服务器
//!
//! 提供各平台 Bot 的 webhook 接收和消息处理功能

use crate::bot_gateway::config::BotGatewayConfig;
use crate::bot_gateway::adapters::{TelegramAdapter, FeishuAdapter, DiscordAdapter, QQAdapter};
use crate::bot_gateway::adapters::telegram::TelegramMessage;
use crate::bot_gateway::adapters::feishu::{FeishuWebhookEvent, FeishuChallenge};
use crate::bot_gateway::adapters::discord::DiscordMessage;
use crate::bot_gateway::adapters::qq::OneBotMessageEvent;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub platforms: Vec<String>,
    pub uptime_seconds: u64,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: String,
    pub message: String,
    pub platform: Option<String>,
}

/// API 共享状态
pub struct ApiState {
    pub config: Arc<RwLock<BotGatewayConfig>>,
    pub telegram_adapter: Option<Arc<TelegramAdapter>>,
    pub feishu_adapter: Option<Arc<FeishuAdapter>>,
    pub discord_adapter: Option<Arc<DiscordAdapter>>,
    pub qq_adapter: Option<Arc<QQAdapter>>,
    pub logs: Arc<RwLock<Vec<LogEntry>>>,
    pub start_time: i64,
    pub tool_executor: Arc<tokio::sync::Mutex<crate::tools::executor::ToolExecutionManager>>,
}

/// 工具执行请求
#[derive(Debug, Deserialize)]
pub struct ToolExecuteRequest {
    pub tool_id: String,
    pub args: serde_json::Value,
}

/// 工具执行响应
#[derive(Debug, Serialize)]
pub struct ToolExecuteResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: Option<u64>,
}

/// 创建路由
pub fn create_router(state: Arc<ApiState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        // 健康检查
        .route("/health", get(health_check))
        // 获取状态
        .route("/status", get(get_status))
        // 获取日志
        .route("/logs", get(get_logs))
        // Telegram webhook
        .route("/webhook/telegram", post(telegram_webhook))
        // 飞书 webhook
        .route("/webhook/feishu", post(feishu_webhook))
        // Discord webhook
        .route("/webhook/discord", post(discord_webhook))
        // QQ webhook (OneBot)
        .route("/webhook/qq", post(qq_webhook))
        // 工具执行 API（供 Bot 调用）
        .route("/api/tools/execute", post(execute_tool))
        .layer(cors)
        .with_state(state)
}

/// 健康检查
async fn health_check() -> &'static str {
    "OK"
}

/// 获取服务器状态
async fn get_status(State(state): State<Arc<ApiState>>) -> Json<ServerStatus> {
    let config = state.config.read().await;
    let mut platforms = Vec::new();

    if state.telegram_adapter.is_some() && config.platforms.telegram.as_ref().map(|c| c.enabled).unwrap_or(false) {
        platforms.push("telegram".to_string());
    }
    if state.feishu_adapter.is_some() && config.platforms.feishu.as_ref().map(|c| c.enabled).unwrap_or(false) {
        platforms.push("feishu".to_string());
    }
    if state.discord_adapter.is_some() && config.platforms.discord.as_ref().map(|c| c.enabled).unwrap_or(false) {
        platforms.push("discord".to_string());
    }
    if state.qq_adapter.is_some() && config.platforms.qq.as_ref().map(|c| c.enabled).unwrap_or(false) {
        platforms.push("qq".to_string());
    }

    Json(ServerStatus {
        running: true,
        port: config.port,
        platforms,
        uptime_seconds: (chrono::Utc::now().timestamp() - state.start_time) as u64,
    })
}

/// 获取日志
async fn get_logs(
    State(state): State<Arc<ApiState>>,
) -> axum::Json<Vec<LogEntry>> {
    let logs = state.logs.read().await;
    let recent_logs: Vec<LogEntry> = logs.iter().rev().take(100).cloned().collect();
    axum::Json(recent_logs)
}

#[derive(Debug, Deserialize)]
struct LogQuery {
    limit: Option<usize>,
}

/// Telegram webhook 处理
async fn telegram_webhook(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<TelegramMessage>,
) -> Result<StatusCode, StatusCode> {
    add_log(&state, "info", "收到 Telegram 消息", Some("telegram")).await;

    // 获取 adapter
    let adapter = match state.telegram_adapter.as_ref() {
        Some(a) => a,
        None => {
            add_log(&state, "error", "Telegram adapter 未初始化", Some("telegram")).await;
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // 处理消息
    if let Some(msg) = payload.message {
        // 验证用户权限
        let user_id_str = msg.from.as_ref().map(|u| u.id.to_string());
        if let Some(from) = &msg.from {
            if !adapter.is_user_allowed(from.id) {
                add_log(&state, "warn", "未授权用户尝试访问", Some("telegram")).await;
                return Err(StatusCode::FORBIDDEN);
            }
        }

        // 解析命令
        if let Some(text) = &msg.text {
            if let Some((command, args)) = adapter.parse_command(text) {
                add_log(&state, "info", &format!("执行命令：{} {:?}", command, args), Some("telegram")).await;

                // 执行命令
                let result = execute_bot_command(
                    &state,
                    "telegram",
                    &command,
                    &args,
                    msg.chat.id,
                    msg.message_id,
                    user_id_str.as_deref(),
                ).await;

                // 发送响应
                if let Err(e) = adapter.send_message(msg.chat.id, &result, Some(msg.message_id)).await {
                    add_log(&state, "error", &format!("发送响应失败：{}", e), Some("telegram")).await;
                }
            }
        }
    }

    Ok(StatusCode::OK)
}

/// 飞书 webhook 处理
async fn feishu_webhook(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    add_log(&state, "info", "收到飞书 webhook", Some("feishu")).await;

    // 获取 adapter
    let adapter = match state.feishu_adapter.as_ref() {
        Some(a) => a,
        None => {
            add_log(&state, "error", "飞书 adapter 未初始化", Some("feishu")).await;
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // 处理挑战验证
    if let Ok(challenge) = serde_json::from_value::<FeishuChallenge>(payload.clone()) {
        if challenge.type_ == "url_verification" {
            if let Some(response) = adapter.verify_challenge(&challenge) {
                add_log(&state, "info", "飞书挑战验证成功", Some("feishu")).await;
                return Ok(Json(serde_json::json!({
                    "challenge": response
                })));
            } else {
                add_log(&state, "error", "飞书挑战验证失败", Some("feishu")).await;
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    // 解析事件
    if let Ok(event) = serde_json::from_value::<FeishuWebhookEvent>(payload.clone()) {
        // 验证签名
        if !adapter.verify_signature(
            &event.header.timestamp,
            &event.header.token,
            &serde_json::to_string(&event.event).unwrap_or_default(),
        ) {
            add_log(&state, "error", "飞书签名验证失败", Some("feishu")).await;
            return Err(StatusCode::FORBIDDEN);
        }

        // 验证租户
        if !adapter.is_tenant_allowed(&event.header.tenant_key) {
            add_log(&state, "warn", "未授权的租户访问", Some("feishu")).await;
            return Err(StatusCode::FORBIDDEN);
        }

        // 处理消息
        if let Some(processed) = adapter.process_message(&event.event) {
            if let Some(command) = processed.command {
                add_log(&state, "info", &format!("执行命令：{} {:?}", command, processed.args), Some("feishu")).await;
                
                let result = execute_bot_command(
                    &state,
                    "feishu",
                    &command,
                    &processed.args,
                    0, // 飞书使用字符串 chat_id
                    0,
                    Some(&processed.user_id),
                ).await;

                // 发送响应
                if let Err(e) = adapter.send_text_message(&processed.chat_id, &result, None).await {
                    add_log(&state, "error", &format!("发送响应失败：{}", e), Some("feishu")).await;
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "status": "success"
    })))
}

/// Discord webhook 处理
async fn discord_webhook(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<DiscordMessage>,
) -> Result<StatusCode, StatusCode> {
    add_log(&state, "info", "收到 Discord 消息", Some("discord")).await;

    // 获取 adapter
    let adapter = match state.discord_adapter.as_ref() {
        Some(a) => a,
        None => {
            add_log(&state, "error", "Discord adapter 未初始化", Some("discord")).await;
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // 处理消息
    if let Some(processed) = adapter.process_message(&payload) {
        if let Some(command) = processed.command {
            add_log(&state, "info", &format!("执行命令：{} {:?}", command, processed.args), Some("discord")).await;
            
            let result = execute_bot_command(
                &state,
                "discord",
                &command,
                &processed.args,
                0,
                0,
                Some(&processed.user_id),
            ).await;

            // 发送响应
            if let Err(e) = adapter.send_message(&processed.channel_id, &result).await {
                add_log(&state, "error", &format!("发送响应失败：{}", e), Some("discord")).await;
            }
        }
    }

    Ok(StatusCode::OK)
}

/// QQ webhook 处理 (OneBot 协议)
async fn qq_webhook(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<OneBotMessageEvent>,
) -> Result<StatusCode, StatusCode> {
    add_log(&state, "info", "收到 QQ 消息", Some("qq")).await;

    // 获取 adapter
    let adapter = match state.qq_adapter.as_ref() {
        Some(a) => a,
        None => {
            add_log(&state, "error", "QQ adapter 未初始化", Some("qq")).await;
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // 处理消息
    if let Some(processed) = adapter.process_message(&payload) {
        if let Some(command) = processed.command {
            add_log(&state, "info", &format!("执行命令：{} {:?}", command, processed.args), Some("qq")).await;
            
            let result = execute_bot_command(
                &state,
                "qq",
                &command,
                &processed.args,
                0,
                0,
                Some(&processed.user_id.to_string()),
            ).await;

            // 发送响应
            let send_result = if processed.is_private {
                adapter.send_private_message(processed.user_id, &result).await
            } else if let Some(group_id) = processed.group_id {
                adapter.send_group_message(group_id, &result).await
            } else {
                Err("无法确定消息类型".to_string())
            };

            if let Err(e) = send_result {
                add_log(&state, "error", &format!("发送响应失败：{}", e), Some("qq")).await;
            }
        }
    }

    Ok(StatusCode::OK)
}

/// 执行 Bot 命令
async fn execute_bot_command(
    state: &ApiState,
    platform: &str,
    command: &str,
    args: &[String],
    _chat_id: i64,
    _message_id: u64,
    _user_id: Option<&str>,
) -> String {
    match command {
        "help" => {
            format!(
                "🤖 Alou Bot 帮助\n\n\
                 可用命令:\n\
                 /help - 显示帮助\n\
                 /status - 查看状态\n\
                 /spec <operation> - 管理规格文档\n\
                 /tools - 列出可用工具\n\n\
                 示例:\n\
                 /spec create product\n\
                 /spec list"
            )
        }
        "status" => {
            let config = state.config.read().await;
            format!(
                "📊 Alou Bot 状态\n\
                 运行中：✅\n\
                 端口：{}\n\
                 平台：{}",
                config.port,
                platform
            )
        }
        "spec" => {
            // 调用 Spec Tool
            if args.is_empty() {
                return "❌ 请指定操作类型：create, get, list, validate".to_string();
            }

            let operation = &args[0];
            let spec_args = serde_json::json!({
                "operation": operation,
                "spec_type": args.get(1).map(|s| s.as_str()).unwrap_or("product"),
            });

            match execute_tool_internal(state, "spec", spec_args).await {
                Ok(result) => {
                    if result.success {
                        format!("✅ Spec 操作 '{}' 执行成功\n{:?}", operation, result.data)
                    } else {
                        format!("❌ Spec 操作失败：{}", result.error.unwrap_or_default())
                    }
                }
                Err(e) => format!("❌ 执行失败：{}", e),
            }
        }
        "tools" => {
            // 获取工具列表
            "🔧 可用工具:\n- spec: 规格文档管理\n- filesystem: 文件操作\n- bash: 终端命令\n- search: 搜索\n...".to_string()
        }
        _ => {
            format!("❌ 未知命令：{}\n使用 /help 查看帮助", command)
        }
    }
}

/// 执行工具（内部方法）
async fn execute_tool_internal(
    state: &ApiState,
    tool_id: &str,
    args: serde_json::Value,
) -> Result<ToolExecuteResponse, String> {
    use crate::tools::ExecutionContext;
    
    let executor_guard = state.tool_executor.lock().await;
    let executor = executor_guard.get_executor(tool_id)
        .ok_or_else(|| format!("工具 '{}' 不存在", tool_id))?;

    let context = ExecutionContext {
        session_id: "bot_session".to_string(),
        user_id: None,
        working_directory: std::env::current_dir().ok().and_then(|p| p.to_str().map(|s| s.to_string())),
        environment: std::env::vars().collect(),
        timeout_seconds: Some(30),
        permissions: vec!["read".to_string(), "write".to_string()],
        timestamp: chrono::Utc::now().timestamp(),
    };

    let start_time = std::time::Instant::now();
    
    match executor.execute(args, &context).await {
        Ok(result) => Ok(ToolExecuteResponse {
            success: result.success,
            data: Some(result.data),
            error: result.error,
            execution_time_ms: Some(result.execution_time_ms),
        }),
        Err(e) => Ok(ToolExecuteResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            execution_time_ms: Some(start_time.elapsed().as_millis() as u64),
        }),
    }
}

/// 工具执行 API
async fn execute_tool(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<ToolExecuteRequest>,
) -> Json<ToolExecuteResponse> {
    match execute_tool_internal(&state, &payload.tool_id, payload.args).await {
        Ok(response) => Json(response),
        Err(e) => Json(ToolExecuteResponse {
            success: false,
            data: None,
            error: Some(e),
            execution_time_ms: None,
        }),
    }
}

/// 添加日志
async fn add_log(state: &ApiState, level: &str, message: &str, platform: Option<&str>) {
    let mut logs = state.logs.write().await;
    logs.push(LogEntry {
        timestamp: chrono::Utc::now().timestamp(),
        level: level.to_string(),
        message: message.to_string(),
        platform: platform.map(|s| s.to_string()),
    });

    // 限制日志数量
    if logs.len() > 1000 {
        logs.remove(0);
    }
}

/// 启动服务器
pub async fn start_server(
    config: BotGatewayConfig,
    tool_executor: Arc<tokio::sync::Mutex<crate::tools::executor::ToolExecutionManager>>,
) -> Result<u16, String> {
    let telegram_adapter = config.platforms.telegram.as_ref().map(|c| {
        Arc::new(TelegramAdapter::new(c.clone()))
    });

    let feishu_adapter = config.platforms.feishu.as_ref().map(|c| {
        Arc::new(FeishuAdapter::new(c.clone()))
    });

    let discord_adapter = config.platforms.discord.as_ref().map(|c| {
        Arc::new(DiscordAdapter::new(c.clone()))
    });

    let qq_adapter = config.platforms.qq.as_ref().map(|c| {
        Arc::new(QQAdapter::new(c.clone()))
    });

    let state = Arc::new(ApiState {
        config: Arc::new(RwLock::new(config.clone())),
        telegram_adapter,
        feishu_adapter,
        discord_adapter,
        qq_adapter,
        logs: Arc::new(RwLock::new(Vec::new())),
        start_time: chrono::Utc::now().timestamp(),
        tool_executor,
    });

    let router = create_router(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("绑定端口失败：{}", e))?;

    let port = config.port;

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("Bot Gateway 服务器错误：{}", e);
        }
    });

    println!("Bot Gateway 服务器启动在端口 {}", port);
    Ok(port)
}
