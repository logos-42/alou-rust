use std::sync::Arc;
use std::time::Duration;
use worker::*;

mod agent;
mod compatibility;
mod durable_objects;
mod mcp;
mod middleware;
mod router;
mod services;
mod storage;
mod utils;
mod web3;

// 导出常用工具函数
pub use utils::time;
pub use utils::error::{AloudError, Result};

// 导出 Durable Objects（必须在根模块导出才能被 wasm-bindgen 识别）
pub use durable_objects::ai_task::AITaskDO;

use agent::{
    ai_client::AiClient,
    batch_create::BatchCreateTask,
    batch_processor::BatchProcessor,
    content_generator::ContentGenerator,
    discovery::{AgentDiscovery, AgentDiscoveryConfig},
    AgentCore, SessionManager,
};
 use mcp::tools::{
    AgentAvatarTool, AgentWalletTool, BroadcastTool, EchoTool, QueryTool, SpecEnhancedTool, TransactionTool,
    WalletAuthTool, WalletManagerTool, WorkflowTool,
};
use mcp::{McpBridge, McpConnectionPool, McpExecutor, McpRegistry};
use middleware::SubscriptionGuard;
use router::Router;
use storage::kv::KvStore;

/// Worker main entry point
///
/// This is the entry point for all incoming requests to the Cloudflare Worker.
/// It initializes all services and routes requests to the appropriate handlers.
///
/// # Architecture
///
/// 1. **Panic Hook**: Set up panic hook for better error messages in WASM
/// 2. **Service Initialization**: Initialize all services (SessionManager, MCP Client, AgentCore)
/// 3. **Request Handling**: Route request through the Router
/// 4. **Error Handling**: Global error handling with proper logging
///
/// # Error Handling
///
/// All errors are caught and logged. The function returns appropriate HTTP responses
/// based on the error type.
#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    // Set panic hook for better error messages in WASM
    // This should be called once at the start of the worker
    console_error_panic_hook::set_once();

    // Log incoming request with timestamp
    console_log!(
        "[{}] {} {}",
        crate::utils::time::now_formatted(),
        req.method().to_string(),
        req.path()
    );

    // Initialize services and handle request with comprehensive error handling
    let result = initialize_and_handle(req, env).await;

    // Log result with appropriate level
    match &result {
        Ok(response) => {
            console_log!(
                "✓ Request completed successfully: {} (status: {})",
                response.status_code(),
                if response.status_code() < 400 {
                    "OK"
                } else {
                    "ERROR"
                }
            );
        }
        Err(e) => {
            console_error!("✗ Request failed with error: {}", e);
        }
    }

    result
}

/// Initialize all services and handle the request
///
/// This function performs the following steps:
/// 1. Initialize KV stores (SESSIONS, CACHE, NONCES)
/// 2. Load secrets from environment (JWT_SECRET, API keys)
/// 3. Initialize MCP registry and register tools
/// 4. Initialize MCP bridge and connection pool
/// 5. Initialize SessionManager
/// 6. Initialize AgentCore
/// 7. Initialize Router with all services
/// 8. Handle the request
///
/// # Error Handling
///
/// Each initialization step includes proper error handling and logging.
/// If a non-critical service fails to initialize, a warning is logged and
/// the worker continues with degraded functionality.
async fn initialize_and_handle(req: Request, env: Env) -> worker::Result<Response> {
    console_log!("=== Starting service initialization ===");

    // ========================================
    // 1. Initialize KV Stores
    // ========================================
    console_log!("→ Initializing KV stores...");

    let sessions_kv = env.kv("SESSIONS").map_err(|e| {
        console_error!("✗ Failed to initialize SESSIONS KV: {}", e);
        e
    })?;
    console_log!("  ✓ SESSIONS KV initialized");

    let cache_kv = env.kv("CACHE").map_err(|e| {
        console_error!("✗ Failed to initialize CACHE KV: {}", e);
        e
    })?;
    console_log!("  ✓ CACHE KV initialized");

    let nonces_kv = env.kv("NONCES").map_err(|e| {
        console_error!("✗ Failed to initialize NONCES KV: {}", e);
        e
    })?;
    console_log!("  ✓ NONCES KV initialized");

    // Create KV store wrappers
    let sessions_store = KvStore::new(sessions_kv.clone());
    let cache_store = KvStore::new(cache_kv);
    let nonces_store = KvStore::new(nonces_kv);

    // ========================================
    // 2. Load Secrets from Environment
    // ========================================
    console_log!("→ Loading secrets from environment...");

    let jwt_secret = env
        .secret("JWT_SECRET")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| {
            console_warn!("  ⚠ JWT_SECRET not found, using default (NOT SECURE FOR PRODUCTION)");
            "default_jwt_secret_change_in_production".to_string()
        });
    console_log!("  ✓ JWT_SECRET loaded");

    // Load AI provider configuration
    let ai_provider = env
        .var("AI_PROVIDER")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| {
            console_log!("  ℹ AI_PROVIDER not set, defaulting to 'deepseek'");
            "deepseek".to_string()
        });
    console_log!("  ✓ AI Provider: {}", ai_provider);

    let ai_model = env.var("AI_MODEL").map(|v| v.to_string()).ok();
    if let Some(ref model) = ai_model {
        console_log!("  ✓ AI Model: {}", model);
    }

    let api_key = env
        .secret("AI_API_KEY")
        .map(|s| s.to_string())
        .or_else(|_| {
            // Try legacy CLAUDE_API_KEY as fallback
            console_log!("  ℹ AI_API_KEY not found, trying CLAUDE_API_KEY...");
            env.secret("CLAUDE_API_KEY").map(|s| s.to_string())
        })
        .unwrap_or_else(|_| {
            console_error!("  ✗ AI_API_KEY not found! Please set it with: wrangler secret put AI_API_KEY");
            console_error!("  ✗ For local development, add AI_API_KEY to .dev.vars file");
            // 使用空字符串而不是默认值，这样会在 API 调用时立即失败并给出明确的错误
            String::new()
        });
    
    if api_key.is_empty() {
        console_error!("  ✗ Cannot proceed without AI_API_KEY");
        return Err(worker::Error::RustError(
            "AI_API_KEY not configured. Please set it with: wrangler secret put AI_API_KEY".to_string()
        ));
    }
    
    // 记录 API key 预览（前8个字符）用于调试
    let api_key_preview = if api_key.len() > 8 {
        format!("{}...", &api_key[..8])
    } else {
        "***".to_string()
    };
    console_log!("  ✓ API key loaded (preview: {})", api_key_preview);

    // Load RPC URLs (optional, with fallbacks)
    let eth_rpc_url = env
        .secret("ETH_RPC_URL")
        .map(|s| s.to_string())
        .or_else(|_| env.var("ETH_RPC_URL").map(|v| v.to_string()))
        .ok();
    if eth_rpc_url.is_some() {
        console_log!("  ✓ ETH_RPC_URL loaded");
    } else {
        console_log!("  ℹ ETH_RPC_URL not configured (optional)");
    }

    let eth_testnet_rpc_url = env
        .secret("ETH_TESTNET_RPC_URL")
        .map(|s| s.to_string())
        .or_else(|_| env.var("ETH_TESTNET_RPC_URL").map(|v| v.to_string()))
        .ok();
    if eth_testnet_rpc_url.is_some() {
        console_log!("  ✓ ETH_TESTNET_RPC_URL loaded");
    }

    let solana_rpc_url = env
        .secret("SOLANA_RPC_URL")
        .or_else(|_| env.secret("SOL_RPC_URL"))
        .map(|s| s.to_string())
        .ok();
    if solana_rpc_url.is_some() {
        console_log!("  ✓ SOL_RPC_URL loaded");
    } else {
        console_log!("  ℹ SOL_RPC_URL not configured (optional)");
    }

    // Load DIAP/IPFS configuration
    let diap_ipfs_api_url = env.var("DIAP_IPFS_API_URL").map(|v| v.to_string()).ok();
    let diap_ipfs_gateway_url = env.var("DIAP_IPFS_GATEWAY_URL").map(|v| v.to_string()).ok();
    let diap_ipns_key = env.var("DIAP_IPNS_KEY").map(|v| v.to_string()).ok();
    let diap_ipfs_timeout = env
        .var("DIAP_IPFS_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.to_string().parse::<u64>().ok());
    let diap_agent_cache_ttl = env
        .var("DIAP_AGENT_CACHE_TTL_SECS")
        .ok()
        .and_then(|v| v.to_string().parse::<u64>().ok());

    #[allow(unused_mut)]
    let mut agent_discovery = None;

    if let (Some(api_url), Some(gateway_url)) =
        (diap_ipfs_api_url.clone(), diap_ipfs_gateway_url.clone())
    {
        console_log!("  ℹ Configuring agent discovery service (IPFS)");
        let mut config =
            AgentDiscoveryConfig::new(api_url, gateway_url).with_ipns_key(diap_ipns_key.clone());

        if let Some(timeout_secs) = diap_ipfs_timeout {
            config = config.with_request_timeout(Duration::from_secs(timeout_secs.max(1)));
        }

        if let Some(cache_ttl_secs) = diap_agent_cache_ttl {
            config = config.with_cache_ttl(Duration::from_secs(cache_ttl_secs.max(1)));
        }

        match AgentDiscovery::new(config) {
            Ok(discovery) => {
                console_log!("  ✓ Agent discovery configured");
                agent_discovery = Some(discovery);
            }
            Err(e) => {
                console_warn!(
                    "  ⚠ Failed to initialize agent discovery: {} (agent resolution disabled)",
                    e
                );
            }
        }
    } else {
        console_log!("  ℹ Agent discovery not configured (DIAP_IPFS_API_URL or DIAP_IPFS_GATEWAY_URL missing)");
    }

    // ========================================
    // 3. Initialize MCP Registry and Tools
    // ========================================
    console_log!("→ Initializing MCP registry...");

    let mut registry = McpRegistry::new();

    // Register built-in tools
    registry.register(Arc::new(EchoTool));
    console_log!("  ✓ Registered EchoTool");

    // Register workflow tool
    registry.register(Arc::new(WorkflowTool::new()));
    console_log!("  ✓ Registered WorkflowTool");
    
    // Register spec enhanced tool
    registry.register(Arc::new(SpecEnhancedTool::new(sessions_store.clone())));
    console_log!("  ✓ Registered SpecEnhancedTool");
    
    // Register blockchain tools
    registry.register(Arc::new(WalletAuthTool::new(
        nonces_store.clone(),
        jwt_secret.clone(),
    )));
    console_log!("  ✓ Registered WalletAuthTool");

    registry.register(Arc::new(WalletManagerTool::new()));
    console_log!("  ✓ Registered WalletManagerTool");

    registry.register(Arc::new(AgentWalletTool::new(sessions_store.clone())));
    console_log!("  ✓ Registered AgentWalletTool");

    // Register AgentAvatarTool - allows agents to update their own avatar
    let session_manager_for_avatar = SessionManager::new(sessions_store.clone());
    registry.register(Arc::new(AgentAvatarTool::new(session_manager_for_avatar)));
    console_log!("  ✓ Registered AgentAvatarTool");

    if let (Some(ref eth_rpc), Some(ref sol_rpc)) = (&eth_rpc_url, &solana_rpc_url) {
        registry.register(Arc::new(QueryTool::new(
            eth_rpc.clone(),
            eth_testnet_rpc_url.clone(),
            sol_rpc.clone(),
        )));
        console_log!("  ✓ Registered QueryTool");

        registry.register(Arc::new(TransactionTool::new(
            eth_rpc.clone(),
            eth_testnet_rpc_url.clone(),
            sol_rpc.clone(),
        )));
        console_log!("  ✓ Registered TransactionTool");

        registry.register(Arc::new(BroadcastTool::new(
            eth_rpc.clone(),
            eth_testnet_rpc_url.clone(),
            sol_rpc.clone(),
        )));
        console_log!("  ✓ Registered BroadcastTool");
    }

    console_log!(
        "  ✓ MCP registry initialized with {} tools",
        registry.list_tools().len()
    );

    // ========================================
    // 4. Initialize MCP Bridge and Connection Pool
    // ========================================
    console_log!("→ Initializing MCP bridge...");

    let mcp_pool = Arc::new(McpConnectionPool::with_defaults());
    let mcp_bridge = Arc::new(McpBridge::new(mcp_pool.clone()));

    // Attempt to connect to external MCP servers
    // This is optional and non-blocking - if it fails, we continue with local tools only
    let mcp_server_url = env.var("MCP_SERVER_URL").map(|v| v.to_string()).ok();

    if let Some(server_url) = mcp_server_url {
        console_log!("  ℹ Attempting to connect to MCP server: {}", server_url);
        match mcp_bridge.connect_server(&server_url).await {
            Ok(tool_count) => {
                console_log!(
                    "  ✓ Connected to MCP server, registered {} tools",
                    tool_count
                );
            }
            Err(e) => {
                console_warn!(
                    "  ⚠ Failed to connect to MCP server: {} (continuing with local tools)",
                    e
                );
            }
        }
    } else {
        console_log!("  ℹ MCP_SERVER_URL not configured, using local tools only");
    }

    console_log!("  ✓ MCP bridge initialized");

    // ========================================
    // 5. Initialize MCP Executor
    // ========================================
    console_log!("→ Initializing MCP executor...");

    let executor = McpExecutor::new(registry);
    console_log!("  ✓ MCP executor initialized");

    // ========================================
    // 6. Initialize Session Manager
    // ========================================
    console_log!("→ Initializing session manager...");

    let session_manager = SessionManager::new(sessions_store.clone());
    console_log!("  ✓ Session manager initialized");

    // ========================================
    // 7. Initialize Agent Core
    // ========================================
    console_log!("→ Initializing agent core...");

    let agent_core =
        AgentCore::with_provider(&ai_provider, api_key, ai_model, session_manager, executor, sessions_store.clone())
            .map_err(|e| {
                console_error!("✗ Failed to initialize agent core: {}", e);
                worker::Error::RustError(e.to_string())
            })?;
    console_log!("  ✓ Agent core initialized with {} provider", ai_provider);

    // ========================================
    // 8. Initialize Subscription Guard
    // ========================================
    console_log!("→ Initializing subscription guard...");
    
    let subscription_guard = match SubscriptionGuard::new(&env, cache_store.clone()) {
        Ok(guard) => {
            console_log!("  ✓ Subscription guard initialized");
            Some(guard)
        }
        Err(e) => {
            console_warn!("  ⚠ Failed to initialize subscription guard: {} (rate limiting disabled)", e);
            None
        }
    };

    // ========================================
    // 9. Initialize Router
    // ========================================
    console_log!("→ Initializing router...");

    let mut router = Router::new(sessions_store)
        .with_wallet_auth(nonces_store, jwt_secret)
        .with_agent_core(agent_core);

    if let Some(discovery) = agent_discovery {
        router = router.with_agent_discovery(discovery);
    }

    // Add blockchain tools if RPC URLs are configured
    if let (Some(eth_rpc), Some(sol_rpc)) = (eth_rpc_url.clone(), solana_rpc_url.clone()) {
        console_log!("  ℹ Configuring blockchain tools...");
        router = router.with_blockchain_tools(eth_rpc, eth_testnet_rpc_url.clone(), sol_rpc);
        console_log!("  ✓ Blockchain tools configured");
    } else {
        console_log!("  ℹ Blockchain tools not configured (RPC URLs not set)");
    }

    // Add subscription guard if initialized
    if let Some(guard) = subscription_guard {
        router = router.with_subscription_guard(guard);
        console_log!("  ✓ Subscription guard attached to router");
    }

    console_log!("  ✓ Router initialized");

    console_log!("=== All services initialized successfully ===");

    // ========================================
    // 10. Handle Request
    // ========================================
    router.handle(req, env).await
}

/// Initialize MCP bridge connection to external MCP servers
///
/// This function establishes connections to external MCP servers and registers
/// their tools in the MCP registry. It's called during worker initialization.
///
/// # Arguments
///
/// * `env` - The Cloudflare Worker environment
/// * `bridge` - The MCP bridge to register tools with
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if initialization fails.
///
/// # Note
///
/// This is currently a placeholder. In production, this would:
/// 1. Read MCP server configuration from environment
/// 2. Connect to each configured MCP server
/// 3. Perform handshake and list available tools
/// 4. Register tools in the bridge
#[allow(dead_code)]
async fn initialize_mcp_servers(env: &Env, bridge: Arc<McpBridge>) -> Result<()> {
    console_log!("→ Initializing MCP server connections...");

    // Get MCP server URLs from environment
    // Format: MCP_SERVERS=server1_url,server2_url,server3_url
    let mcp_servers = env.var("MCP_SERVERS").map(|v| v.to_string()).ok();

    if let Some(servers) = mcp_servers {
        let server_list: Vec<&str> = servers.split(',').collect();
        console_log!("  ℹ Found {} MCP server(s) to connect", server_list.len());

        for server_url in server_list {
            let server_url = server_url.trim();
            if server_url.is_empty() {
                continue;
            }

            console_log!("  → Connecting to MCP server: {}", server_url);

            match bridge.connect_server(server_url).await {
                Ok(tool_count) => {
                    console_log!(
                        "    ✓ Connected successfully, registered {} tools",
                        tool_count
                    );
                }
                Err(e) => {
                    console_warn!("    ⚠ Failed to connect: {} (skipping)", e);
                    // Continue with other servers even if one fails
                }
            }
        }

        console_log!("  ✓ MCP server initialization complete");
    } else {
        console_log!("  ℹ No MCP servers configured (MCP_SERVERS not set)");
    }

    Ok(())
}

/// Queue handler for batch agent creation
/// 
/// This handler processes messages from the agent creation queue.
/// It runs in a separate worker context and does not block HTTP requests.
#[event(queue)]
async fn queue_handler(batch: MessageBatch<BatchCreateTask>, env: Env, _ctx: Context) -> worker::Result<()> {
    console_log!("=== Processing batch agent creation queue ===");

    // Initialize services
    let sessions_kv = match env.kv("SESSIONS") {
        Ok(kv) => kv,
        Err(e) => {
            console_error!("Failed to access SESSIONS KV: {}", e);
            return Err(worker::Error::RustError(format!(
                "Failed to access SESSIONS KV: {}",
                e
            )));
        }
    };
    let kv_store = KvStore::new(sessions_kv.clone());
    let session_manager = SessionManager::new(kv_store.clone());

    // Create AI client (needed for ContentGenerator)
    let ai_provider = env
        .var("AI_PROVIDER")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| {
            console_log!("AI_PROVIDER not set, defaulting to 'deepseek'");
            "deepseek".to_string()
        });

    let api_key = match env.secret("AI_API_KEY") {
        Ok(key) => key.to_string(),
        Err(_) => {
            console_error!("AI_API_KEY not found in environment");
            return Err(worker::Error::RustError(
                "AI_API_KEY not configured".to_string(),
            ));
        }
    };

    let ai_model = env.var("AI_MODEL").map(|v| v.to_string()).ok();
    let ai_client = match AiClient::new(&ai_provider, api_key, ai_model) {
        Ok(client) => client,
        Err(e) => {
            console_error!("Failed to create AI client: {}", e);
            return Err(worker::Error::RustError(format!(
                "Failed to create AI client: {}",
                e
            )));
        }
    };

    let content_generator = ContentGenerator::new(ai_client);
    let processor = BatchProcessor::new(session_manager, content_generator, kv_store);

    // Process each message in the batch
    let messages = batch.messages()?;
    console_log!("Batch size: {}", messages.len());
    
    for message in messages {
        let task = message.body();
        console_log!("Processing task: {}", task.task_id);
        match processor.process_batch(task.clone(), &env).await {
            Ok(()) => {
                console_log!("Task {} processed successfully", message.id());
                message.ack();
            }
            Err(e) => {
                console_error!("Failed to process task {}: {}", message.id(), e);
                message.retry();
            }
        }
    }

    console_log!("=== Batch processing completed ===");
    Ok(())
}
