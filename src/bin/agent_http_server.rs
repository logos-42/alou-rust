use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply, Rejection};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, error};
use std::collections::HashMap;
use rand::{distributions::Alphanumeric, Rng};
use chrono::{Utc, Duration};
use sqlx::PgPool;
use alou::agent::{
    Agent, McpAgent, AgentConfig, DeepSeekConfig, BehaviorConfig, 
    WorkspaceConfig, ToolStrategy
};
use alou::connection_pool::ConnectionPool;

// 🆕 认证相关导入
use alou::auth::google_oauth::GoogleOAuth;
use alou::auth::middleware::with_auth;
use alou::api::{auth as auth_api, invitation_codes};
use alou::api::user as user_api;

#[derive(Debug, Deserialize)]
struct ChatRequest {
    message: String,
    #[allow(dead_code)]
    context: Option<HashMap<String, String>>,
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdatePrivateKeyRequest {
    private_key: String,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    response: String,
    status: String,
    timestamp: u64,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    timestamp: u64,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    agent_ready: bool,
    timestamp: u64,
}

// 智能体服务状态
struct AgentService {
    agents: RwLock<HashMap<String, McpAgent>>,
    default_config: AgentConfig,
    connection_pool: Arc<ConnectionPool>,
}

impl AgentService {
    async fn new() -> Result<Self> {
        let config = load_agent_config().await?;
        let connection_pool = Arc::new(ConnectionPool::new());
        
        Ok(Self {
            agents: RwLock::new(HashMap::new()),
            default_config: config,
            connection_pool,
        })
    }
    
    async fn get_or_create_agent(&self, session_id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;
        
        if !agents.contains_key(session_id) {
            info!("为会话 {} 创建新的智能体实例", session_id);
            
            let mut agent = McpAgent::with_connection_pool(
                self.default_config.clone(), 
                self.connection_pool.clone()
            ).await?;
            
            agent.initialize().await?;
            agents.insert(session_id.to_string(), agent);
        }
        
        Ok(())
    }
    
    async fn process_message(&self, session_id: &str, message: &str) -> Result<String> {
        // 确保智能体存在
        self.get_or_create_agent(session_id).await?;
        
        let mut agents = self.agents.write().await;
        let agent = agents.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("智能体实例不存在"))?;
        
        agent.process_input(message).await.map_err(|e| anyhow::anyhow!("智能体处理错误: {}", e))
    }
}

// 🆕 从环境变量加载认证配置
struct AuthConfig {
    database_url: String,
    jwt_secret: String,
    jwt_expiration_hours: i64,
    refresh_token_expiration_days: i64,
    google_client_id: String,
    google_client_secret: String,
    google_redirect_uri: String,
}

fn load_auth_config() -> Result<AuthConfig> {
    dotenv::dotenv().ok();
    
    Ok(AuthConfig {
        database_url: std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env"),
        jwt_secret: std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in .env"),
        jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()?,
        refresh_token_expiration_days: std::env::var("REFRESH_TOKEN_EXPIRATION_DAYS")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<i64>()?,
        google_client_id: std::env::var("GOOGLE_CLIENT_ID")
            .expect("GOOGLE_CLIENT_ID must be set in .env"),
        google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
            .expect("GOOGLE_CLIENT_SECRET must be set in .env"),
        google_redirect_uri: std::env::var("GOOGLE_REDIRECT_URI")
            .expect("GOOGLE_REDIRECT_URI must be set in .env"),
    })
}

// 加载智能体配置
async fn load_agent_config() -> Result<AgentConfig> {
    // 尝试从配置文件加载
    if let Ok(content) = std::fs::read_to_string("agent_config.json") {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            let api_key = config["deepseek"]["api_key"].as_str()
                .unwrap_or("your-api-key-here")
                .to_string();
            
            // 如果API密钥是占位符，尝试从环境变量获取
            let final_api_key = if api_key == "your-api-key-here" {
                std::env::var("DEEPSEEK_API_KEY")
                    .unwrap_or_else(|_| api_key)
            } else {
                api_key
            };
            
            return Ok(AgentConfig {
                deepseek: DeepSeekConfig {
                    base_url: config["deepseek"]["base_url"].as_str()
                        .unwrap_or("https://api.deepseek.com").to_string(),
                    api_key: final_api_key,
                    model: config["deepseek"]["model"].as_str()
                        .unwrap_or("deepseek-chat").to_string(),
                    // 确保max_tokens在有效范围内 [1, 8192]
                    max_tokens: std::cmp::min(
                        config["deepseek"]["max_tokens"].as_u64().unwrap_or(2000) as u32,
                        8192
                    ),
                    temperature: config["deepseek"]["temperature"].as_f64().unwrap_or(0.7) as f32,
                },
                behavior: BehaviorConfig {
                    max_retries: config["behavior"]["max_retries"].as_u64().unwrap_or(3) as u32,
                    timeout_seconds: config["behavior"]["timeout_seconds"].as_u64().unwrap_or(30) as u64,
                    verbose_logging: config["behavior"]["verbose_logging"].as_bool().unwrap_or(false),
                    tool_strategy: ToolStrategy::Auto,
                },
                workspace: WorkspaceConfig {
                    directories: vec![".".to_string()],
                    smart_detection: true,
                    exclude_patterns: vec!["target".to_string(), "node_modules".to_string()],
                },
            });
        }
    }
    
    // 如果配置文件读取失败，使用环境变量和默认配置
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .unwrap_or_else(|_| "your-api-key".to_string());
    
    Ok(AgentConfig {
        deepseek: DeepSeekConfig {
            base_url: "https://api.deepseek.com".to_string(),
            api_key,
            model: "deepseek-chat".to_string(),
            max_tokens: 2000, // 安全的默认值
            temperature: 0.7,
        },
        behavior: BehaviorConfig {
            max_retries: 3,
            timeout_seconds: 30,
            verbose_logging: false,
            tool_strategy: ToolStrategy::Auto,
        },
        workspace: WorkspaceConfig {
            directories: vec![".".to_string()],
            smart_detection: true,
            exclude_patterns: vec!["target".to_string(), "node_modules".to_string()],
        },
    })
}

// HTTP 处理函数
async fn handle_chat(
    req: ChatRequest,
    service: Arc<AgentService>,
) -> Result<impl Reply, Rejection> {
    let session_id = req.session_id.unwrap_or_else(|| {
        format!("session_{}", chrono::Utc::now().timestamp_millis())
    });
    
    info!("处理聊天请求 - 会话: {}, 消息: {}", session_id, req.message);
    
    match service.process_message(&session_id, &req.message).await {
        Ok(response) => {
            let chat_response = ChatResponse {
                response,
                status: "success".to_string(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                session_id,
            };
            Ok(warp::reply::json(&chat_response))
        }
        Err(e) => {
            error!("处理聊天请求失败: {}", e);
            let error_response = ErrorResponse {
                error: "processing_failed".to_string(),
                message: format!("智能体处理失败: {}", e),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            };
            Ok(warp::reply::json(&error_response))
        }
    }
}

async fn handle_health(service: Arc<AgentService>) -> Result<impl Reply, Rejection> {
    let agents = service.agents.read().await;
    let agent_ready = !agents.is_empty();
    
    let health = HealthResponse {
        status: "healthy".to_string(),
        agent_ready,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };
    
    Ok(warp::reply::json(&health))
}

async fn handle_update_private_key(
    req: UpdatePrivateKeyRequest,
) -> Result<impl Reply, Rejection> {
    info!("更新私钥请求: {}", req.private_key);
    
    // 读取当前的mcp.json文件
    let mcp_config_path = "mcp.json";
    let mcp_content = match std::fs::read_to_string(mcp_config_path) {
        Ok(content) => content,
        Err(e) => {
            error!("无法读取mcp.json文件: {}", e);
            return Ok(warp::reply::json(&ErrorResponse {
                error: "file_read_error".to_string(),
                message: format!("无法读取mcp.json文件: {}", e),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            }));
        }
    };
    
    // 解析JSON
    let mut mcp_config: serde_json::Value = match serde_json::from_str(&mcp_content) {
        Ok(config) => config,
        Err(e) => {
            error!("无法解析mcp.json文件: {}", e);
            return Ok(warp::reply::json(&ErrorResponse {
                error: "json_parse_error".to_string(),
                message: format!("无法解析mcp.json文件: {}", e),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            }));
        }
    };
    
    // 更新私钥
    if let Some(payment_npm) = mcp_config["mcpServers"]["payment-npm"].as_object_mut() {
        if let Some(env) = payment_npm["env"].as_object_mut() {
            env["PRIVATE_KEY"] = serde_json::Value::String(req.private_key.clone());
        }
    }
    
    // 写回文件
    let updated_content = match serde_json::to_string_pretty(&mcp_config) {
        Ok(content) => content,
        Err(e) => {
            error!("无法序列化mcp.json: {}", e);
            return Ok(warp::reply::json(&ErrorResponse {
                error: "json_serialize_error".to_string(),
                message: format!("无法序列化mcp.json: {}", e),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            }));
        }
    };
    
    if let Err(e) = std::fs::write(mcp_config_path, updated_content) {
        error!("无法写入mcp.json文件: {}", e);
        return Ok(warp::reply::json(&ErrorResponse {
            error: "file_write_error".to_string(),
            message: format!("无法写入mcp.json文件: {}", e),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }));
    }
    
    info!("私钥已成功更新到mcp.json");
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "私钥已更新到mcp.json",
        "timestamp": chrono::Utc::now().timestamp_millis()
    })))
}

async fn handle_restart_mcp() -> Result<impl Reply, Rejection> {
    info!("收到MCP服务重启请求");
    
    // 这里可以添加重启MCP服务的逻辑
    // 由于MCP服务是在ConnectionPool中管理的，我们可以通过重新初始化来重启
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "MCP服务重启请求已处理",
        "timestamp": chrono::Utc::now().timestamp_millis()
    })))
}





// CORS 过滤器
fn cors() -> warp::filters::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("启动智能体 HTTP 服务器...");
    
    // 🆕 加载认证配置
    let auth_config = load_auth_config()?;
    info!("认证配置加载成功");
    
    // 🆕 初始化数据库连接池
    let pool = alou::db::init_pool(&auth_config.database_url).await?;
    info!("数据库连接池初始化成功");
    
    // 🆕 创建Google OAuth客户端
    let oauth_client = GoogleOAuth::new(
        auth_config.google_client_id.clone(),
        auth_config.google_client_secret.clone(),
        auth_config.google_redirect_uri.clone(),
    ).map_err(|e| anyhow::anyhow!("Failed to create OAuth client: {}", e))?;
    info!("Google OAuth客户端初始化成功");
    
    // 创建智能体服务
    let service = Arc::new(AgentService::new().await?);
    
    // 为每个路由克隆 service 和配置
    let service_health = service.clone();
    let service_chat = service.clone();
    let pool_clone1 = pool.clone();
    let pool_clone2 = pool.clone();
    let pool_clone3 = pool.clone();
    let pool_clone4 = pool.clone();
    let pool_clone5 = pool.clone();
    let pool_clone6 = pool.clone();
    let oauth_clone1 = oauth_client.clone();
    let oauth_clone2 = oauth_client.clone();
    let jwt_secret1 = auth_config.jwt_secret.clone();
    let jwt_secret2 = auth_config.jwt_secret.clone();
    let jwt_secret3 = auth_config.jwt_secret.clone();
    let jwt_secret4 = auth_config.jwt_secret.clone();
    let jwt_exp = auth_config.jwt_expiration_hours;
    let refresh_exp = auth_config.refresh_token_expiration_days;
    let pool_clone7 = pool.clone();
    let pool_clone8 = pool.clone();
    let pool_clone9 = pool.clone();
    
    // ========== 原有智能体路由 ==========
    
    // 健康检查接口
    let health = warp::path("health")
        .and(warp::get())
        .and(warp::any().map(move || service_health.clone()))
        .and_then(handle_health);
    
    // 聊天接口
    let chat = warp::path("chat")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || service_chat.clone()))
        .and_then(handle_chat);
    
    // 更新私钥接口
    let update_private_key = warp::path("update-private-key")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_update_private_key);
    
    // 重启MCP接口
    let restart_mcp = warp::path("restart-mcp")
        .and(warp::post())
        .and_then(handle_restart_mcp);
    
    // ========== 🆕 认证路由 ==========
    
    // Google 登录 - 获取授权URL
    let auth_google_login = warp::path!("auth" / "google" / "login")
        .and(warp::get())
        .and(warp::any().map(move || oauth_clone1.clone()))
        .and_then(|oauth: GoogleOAuth| async move {
            auth_api::google_login_handler(oauth).await
        });
    
    // Google 回调 - 处理OAuth回调
    let auth_google_callback = warp::path!("auth" / "google" / "callback")
        .and(warp::get())
        .and(warp::query::<auth_api::GoogleCallbackQuery>())
        .and(warp::any().map(move || oauth_clone2.clone()))
        .and(warp::any().map(move || pool_clone1.clone()))
        .and(warp::any().map(move || jwt_secret1.clone()))
        .and(warp::any().map(move || jwt_exp))
        .and(warp::any().map(move || refresh_exp))
        .and_then(|query, oauth, pool, secret, exp, refresh_exp| async move {
            auth_api::google_callback_handler(query, oauth, pool, secret, exp, refresh_exp).await
        });
    
    // 验证Token
    let auth_verify = warp::path!("auth" / "verify")
        .and(warp::post())
        .and(with_auth(jwt_secret2))
        .and(warp::any().map(move || pool_clone2.clone()))
        .and_then(|user_id, pool| async move {
            auth_api::verify_token_handler(user_id, pool).await
        });
    
    // 刷新Token
    let auth_refresh = warp::path!("auth" / "refresh")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone3.clone()))
        .and(warp::any().map(move || jwt_secret3.clone()))
        .and(warp::any().map(move || jwt_exp))
        .and_then(|body, pool, secret, exp| async move {
            auth_api::refresh_token_handler(body, pool, secret, exp).await
        });
    
    // 登出
    let auth_logout = warp::path!("auth" / "logout")
        .and(warp::post())
        .and(with_auth(jwt_secret4))
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone4.clone()))
        .and_then(|user_id, body, pool| async move {
            auth_api::logout_handler(user_id, body, pool).await
        });
    
    // ========== 🆕 用户路由 ==========
    
    // 获取当前用户信息
    let user_me = warp::path!("user" / "me")
        .and(warp::get())
        .and(with_auth(auth_config.jwt_secret.clone()))
        .and(warp::any().map(move || pool_clone5.clone()))
        .and_then(|user_id, pool| async move {
            user_api::get_me_handler(user_id, pool).await
        });
    
    // 更新用户资料
    let user_update = warp::path!("user" / "profile")
        .and(warp::put())
        .and(with_auth(auth_config.jwt_secret.clone()))
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone6.clone()))
        .and_then(|user_id, body, pool| async move {
            user_api::update_profile_handler(user_id, body, pool).await
        });
    
    // 组合所有认证和用户路由
    let auth_routes = auth_google_login
        .or(auth_google_callback)
        .or(auth_verify)
        .or(auth_refresh)
        .or(auth_logout);

    let user_routes = user_me.or(user_update);
    
    // 生成邀请码接口
    let generate_invitation_codes = warp::path!("invitation" / "generate")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone7.clone()))
        .and_then(|req, pool| async move {
            invitation_codes::handle_generate_invitation_codes(req, pool).await
        });
    // 在invitation_routes上方添加新的路由
    let use_invitation_code = warp::path!("invitation" / "use")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone8.clone()))
        .and_then(|req, pool| async move {
            invitation_codes::handle_use_invitation_code(req, pool).await
        });

    let check_user_invitation = warp::path!("invitation" / "check")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone9.clone()))
        .and_then(|req, pool| async move {
            invitation_codes::handle_check_user_invitation(req, pool).await
        });

    let invitation_routes = generate_invitation_codes
        .or(use_invitation_code)
        .or(check_user_invitation);
    // API 路由 - 组合原有路由和新路由
    let api = warp::path("api")
        .and(
            health
                .or(chat)
                .or(update_private_key)
                .or(restart_mcp)
                .or(auth_routes)
                .or(user_routes)
                .or(invitation_routes)
        )
        .with(cors());




    // 根路径信息
    let root = warp::path::end()
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "service": "Alou Agent HTTP Server with Authentication",
                "status": "running",
                "version": "0.1.4",
                "endpoints": {
                    "agent": {
                        "health": "/api/health",
                        "chat": "/api/chat",
                        "update_private_key": "/api/update-private-key",
                        "restart_mcp": "/api/restart-mcp"
                    },
                    "auth": {
                        "google_login": "/api/auth/google/login",
                        "google_callback": "/api/auth/google/callback",
                        "verify": "/api/auth/verify",
                        "refresh": "/api/auth/refresh",
                        "logout": "/api/auth/logout"
                    },
                    "user": {
                        "me": "/api/user/me",
                        "profile": "/api/user/profile"
                    },
                    "invitation": {
                        "generate": "/api/invitation/generate",
                        "use": "/api/invitation/use",
                        "check": "/api/invitation/check"
                    }
                }
            }))
        });
    
    let routes = root.or(api);
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);
    
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🚀 智能体 HTTP 服务器启动成功");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🌐 服务地址: http://localhost:{}", port);
    info!("");
    info!("📡 智能体接口:");
    info!("   GET  /api/health - 健康检查");
    info!("   POST /api/chat - 智能体聊天");
    info!("");
    info!("🔐 认证接口:");
    info!("   GET  /api/auth/google/login - Google登录");
    info!("   GET  /api/auth/google/callback - OAuth回调");
    info!("   POST /api/auth/verify - 验证Token");
    info!("   POST /api/auth/refresh - 刷新Token");
    info!("   POST /api/auth/logout - 登出");
    info!("");
    info!("👤 用户接口:");
    info!("   GET  /api/user/me - 获取用户信息");
    info!("   PUT  /api/user/profile - 更新用户资料");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
    
    Ok(())
}
