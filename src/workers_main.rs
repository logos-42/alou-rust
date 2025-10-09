use worker::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// 导入现有的模块
use crate::agent::{McpAgent, AgentConfig, DeepSeekConfig, BehaviorConfig, WorkspaceConfig, ToolStrategy, Agent};
use crate::connection_pool::ConnectionPool;

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    session_id: Option<String>,
    context: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    status: String,
    timestamp: u64,
    session_id: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    agent_ready: bool,
    timestamp: u64,
    version: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    timestamp: u64,
}

// Workers 入口点
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    
    // 设置 CORS 头部
    let cors_headers = [
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type, Authorization"),
    ];
    
    // 处理 OPTIONS 预检请求
    if req.method() == Method::Options {
        let mut response = Response::empty()?;
        for (key, value) in cors_headers.iter() {
            response.headers_mut().set(key, value)?;
        }
        return Ok(response);
    }
    
    let router = Router::new();
    
    router
        .get_async("/api/health", |_req, _ctx| async move {
            handle_health().await
        })
        .post_async("/api/chat", |req, ctx| async move {
            handle_chat(req, ctx).await
        })
        .get_async("/api/tools", |_req, ctx| async move {
            handle_tools(ctx).await
        })
        .get("/", |_req, _ctx| {
            Response::ok("🦀 Alou Rust Agent - Running on Cloudflare Workers! 🚀")
        })
        .run(req, env)
        .await
}

async fn handle_health() -> Result<Response> {
    let health = HealthResponse {
        status: "healthy".to_string(),
        agent_ready: true,
        timestamp: js_sys::Date::now() as u64,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    let mut response = Response::from_json(&health)?;
    add_cors_headers(&mut response)?;
    Ok(response)
}

async fn handle_chat(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let chat_req: ChatRequest = match req.json().await {
        Ok(req) => req,
        Err(e) => {
            let error = ErrorResponse {
                error: "Invalid JSON".to_string(),
                message: format!("Failed to parse request: {}", e),
                timestamp: js_sys::Date::now() as u64,
            };
            let mut response = Response::from_json(&error)?;
            response.with_status(400);
            add_cors_headers(&mut response)?;
            return Ok(response);
        }
    };
    
    // 获取 DeepSeek API Key
    let api_key = match ctx.env.secret("DEEPSEEK_API_KEY") {
        Ok(secret) => secret.to_string(),
        Err(_) => {
            let error = ErrorResponse {
                error: "Configuration Error".to_string(),
                message: "DEEPSEEK_API_KEY not configured".to_string(),
                timestamp: js_sys::Date::now() as u64,
            };
            let mut response = Response::from_json(&error)?;
            response.with_status(500);
            add_cors_headers(&mut response)?;
            return Ok(response);
        }
    };
    
    // 创建智能体配置 - 保持与原来相同的配置
    let agent_config = AgentConfig {
        deepseek: DeepSeekConfig {
            base_url: "https://api.deepseek.com".to_string(),
            api_key,
            model: "deepseek-chat".to_string(),
            max_tokens: 4000,
            temperature: 0.7,
        },
        behavior: BehaviorConfig {
            max_retries: 3,
            timeout_seconds: 30,
            verbose_logging: false, // Workers 环境下减少日志
            tool_strategy: ToolStrategy::Auto,
        },
        workspace: WorkspaceConfig {
            directories: vec![".".to_string()],
            smart_detection: true,
            exclude_patterns: vec!["target".to_string(), "node_modules".to_string()],
        },
    };

    // 创建智能体 - 使用现有的代码
    let mut agent = match McpAgent::new(agent_config).await {
        Ok(agent) => agent,
        Err(e) => {
            let error = ErrorResponse {
                error: "Agent Creation Failed".to_string(),
                message: format!("Failed to create agent: {}", e),
                timestamp: js_sys::Date::now() as u64,
            };
            let mut response = Response::from_json(&error)?;
            response.with_status(500);
            add_cors_headers(&mut response)?;
            return Ok(response);
        }
    };
    
    // 初始化智能体
    if let Err(e) = agent.initialize().await {
        let error = ErrorResponse {
            error: "Agent Initialization Failed".to_string(),
            message: format!("Failed to initialize agent: {}", e),
            timestamp: js_sys::Date::now() as u64,
        };
        let mut response = Response::from_json(&error)?;
        response.with_status(500);
        add_cors_headers(&mut response)?;
        return Ok(response);
    }
    
    // 处理消息 - 使用现有的逻辑
    let response_text = match agent.process_input(&chat_req.message).await {
        Ok(response) => response,
        Err(e) => {
            let error = ErrorResponse {
                error: "Message Processing Failed".to_string(),
                message: format!("Failed to process message: {}", e),
                timestamp: js_sys::Date::now() as u64,
            };
            let mut response = Response::from_json(&error)?;
            response.with_status(500);
            add_cors_headers(&mut response)?;
            return Ok(response);
        }
    };
    
    let session_id = chat_req.session_id.unwrap_or_else(|| {
        format!("session_{}", (js_sys::Math::random() * 1000000.0) as u32)
    });
    
    let chat_response = ChatResponse {
        response: response_text,
        status: "success".to_string(),
        timestamp: js_sys::Date::now() as u64,
        session_id,
    };
    
    let mut response = Response::from_json(&chat_response)?;
    add_cors_headers(&mut response)?;
    Ok(response)
}

async fn handle_tools(ctx: RouteContext<()>) -> Result<Response> {
    // 返回可用工具列表
    let tools = serde_json::json!({
        "tools": [
            {
                "name": "filesystem",
                "description": "文件系统操作 (通过 R2 存储)",
                "actions": ["read_file", "write_file", "list_directory"]
            },
            {
                "name": "memory", 
                "description": "内存存储操作 (通过 KV 存储)",
                "actions": ["get", "set", "list"]
            },
            {
                "name": "payment",
                "description": "区块链支付操作",
                "actions": ["send", "balance", "history"]
            }
        ],
        "status": "available",
        "timestamp": js_sys::Date::now() as u64
    });
    
    let mut response = Response::from_json(&tools)?;
    add_cors_headers(&mut response)?;
    Ok(response)
}

fn add_cors_headers(response: &mut Response) -> Result<()> {
    response.headers_mut().set("Access-Control-Allow-Origin", "*")?;
    response.headers_mut().set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
    response.headers_mut().set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
    Ok(())
}
