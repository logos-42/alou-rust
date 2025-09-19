use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply, Rejection};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, error};
use std::collections::HashMap;

use alou::agent::{
    Agent, McpAgent, AgentConfig, DeepSeekConfig, BehaviorConfig, 
    WorkspaceConfig, ToolStrategy
};
use alou::connection_pool::ConnectionPool;

#[derive(Debug, Deserialize)]
struct ChatRequest {
    message: String,
    context: Option<HashMap<String, String>>,
    session_id: Option<String>,
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
    
    // 创建智能体服务
    let service = Arc::new(AgentService::new().await?);
    
    // 为每个路由克隆 service
    let service_health = service.clone();
    let service_chat = service.clone();
    
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
    
    // API 路由
    let api = warp::path("api")
        .and(health.or(chat))
        .with(cors());
    
    // 根路径信息
    let root = warp::path::end()
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "service": "Alou Agent HTTP Server",
                "status": "running",
                "endpoints": {
                    "health": "/api/health",
                    "chat": "/api/chat"
                }
            }))
        });
    
    let routes = root.or(api);
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);
    
    info!("智能体 HTTP 服务器启动在端口 {}", port);
    info!("可用接口:");
    info!("  GET  http://localhost:{}/api/health", port);
    info!("  POST http://localhost:{}/api/chat", port);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
    
    Ok(())
}
