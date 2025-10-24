use worker::*;
use std::sync::Arc;

mod router;
mod agent;
mod mcp;
mod web3;
mod storage;
mod utils;

use router::Router;
use storage::kv::KvStore;
use agent::{AgentCore, SessionManager};
use mcp::{McpRegistry, McpExecutor, McpBridge, McpConnectionPool};
use mcp::tools::EchoTool;

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
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Set panic hook for better error messages in WASM
    // This should be called once at the start of the worker
    console_error_panic_hook::set_once();
    
    // Log incoming request with timestamp
    console_log!(
        "[{}] {} {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
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
                if response.status_code() < 400 { "OK" } else { "ERROR" }
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
async fn initialize_and_handle(req: Request, env: Env) -> Result<Response> {
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
    let _cache_store = KvStore::new(cache_kv);
    let nonces_store = KvStore::new(nonces_kv);
    
    // ========================================
    // 2. Load Secrets from Environment
    // ========================================
    console_log!("→ Loading secrets from environment...");
    
    let jwt_secret = env.secret("JWT_SECRET")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| {
            console_warn!("  ⚠ JWT_SECRET not found, using default (NOT SECURE FOR PRODUCTION)");
            "default_jwt_secret_change_in_production".to_string()
        });
    console_log!("  ✓ JWT_SECRET loaded");
    
    // Load AI provider configuration
    let ai_provider = env.var("AI_PROVIDER")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| {
            console_log!("  ℹ AI_PROVIDER not set, defaulting to 'deepseek'");
            "deepseek".to_string()
        });
    console_log!("  ✓ AI Provider: {}", ai_provider);
    
    let ai_model = env.var("AI_MODEL")
        .map(|v| v.to_string())
        .ok();
    if let Some(ref model) = ai_model {
        console_log!("  ✓ AI Model: {}", model);
    }
    
    let api_key = env.secret("AI_API_KEY")
        .map(|s| s.to_string())
        .or_else(|_| {
            // Try legacy CLAUDE_API_KEY as fallback
            console_log!("  ℹ AI_API_KEY not found, trying CLAUDE_API_KEY...");
            env.secret("CLAUDE_API_KEY").map(|s| s.to_string())
        })
        .unwrap_or_else(|_| {
            console_warn!("  ⚠ No API key found, using default (NOT SECURE FOR PRODUCTION)");
            "default_api_key_change_in_production".to_string()
        });
    console_log!("  ✓ API key loaded");
    
    // Load RPC URLs (optional, with fallbacks)
    let eth_rpc_url = env.secret("ETH_RPC_URL")
        .map(|s| s.to_string())
        .ok();
    if eth_rpc_url.is_some() {
        console_log!("  ✓ ETH_RPC_URL loaded");
    } else {
        console_log!("  ℹ ETH_RPC_URL not configured (optional)");
    }
    
    let solana_rpc_url = env.secret("SOLANA_RPC_URL")
        .map(|s| s.to_string())
        .ok();
    if solana_rpc_url.is_some() {
        console_log!("  ✓ SOLANA_RPC_URL loaded");
    } else {
        console_log!("  ℹ SOLANA_RPC_URL not configured (optional)");
    }
    
    // ========================================
    // 3. Initialize MCP Registry and Tools
    // ========================================
    console_log!("→ Initializing MCP registry...");
    
    let mut registry = McpRegistry::new();
    
    // Register built-in tools
    registry.register(Arc::new(EchoTool));
    console_log!("  ✓ Registered EchoTool");
    
    // TODO: Register additional tools as they are implemented
    // registry.register(Arc::new(WalletAuthTool::new(nonces_store.clone())));
    // registry.register(Arc::new(QueryTool::new(cache_store.clone(), eth_rpc_url, solana_rpc_url)));
    // registry.register(Arc::new(TransactionTool::new(eth_rpc_url, solana_rpc_url)));
    // registry.register(Arc::new(BroadcastTool::new(eth_rpc_url, solana_rpc_url)));
    // registry.register(Arc::new(ContractTool::new(cache_store.clone(), eth_rpc_url)));
    
    console_log!("  ✓ MCP registry initialized with {} tools", registry.list_tools().len());
    
    // ========================================
    // 4. Initialize MCP Bridge and Connection Pool
    // ========================================
    console_log!("→ Initializing MCP bridge...");
    
    let mcp_pool = Arc::new(McpConnectionPool::with_defaults());
    let mcp_bridge = Arc::new(McpBridge::new(mcp_pool.clone()));
    
    // Attempt to connect to external MCP servers
    // This is optional and non-blocking - if it fails, we continue with local tools only
    let mcp_server_url = env.var("MCP_SERVER_URL")
        .map(|v| v.to_string())
        .ok();
    
    if let Some(server_url) = mcp_server_url {
        console_log!("  ℹ Attempting to connect to MCP server: {}", server_url);
        match mcp_bridge.connect_server(&server_url).await {
            Ok(tool_count) => {
                console_log!("  ✓ Connected to MCP server, registered {} tools", tool_count);
            }
            Err(e) => {
                console_warn!("  ⚠ Failed to connect to MCP server: {} (continuing with local tools)", e);
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
    
    let agent_core = AgentCore::with_provider(
        &ai_provider,
        api_key,
        ai_model,
        session_manager,
        executor,
    ).map_err(|e| {
        console_error!("✗ Failed to initialize agent core: {}", e);
        worker::Error::RustError(e.to_string())
    })?;
    console_log!("  ✓ Agent core initialized with {} provider", ai_provider);
    
    // ========================================
    // 8. Initialize Router
    // ========================================
    console_log!("→ Initializing router...");
    
    let mut router = Router::new(sessions_store)
        .with_wallet_auth(nonces_store, jwt_secret)
        .with_agent_core(agent_core);
    
    // Add blockchain tools if RPC URLs are configured
    if let (Some(eth_rpc), Some(sol_rpc)) = (eth_rpc_url, solana_rpc_url) {
        console_log!("  ℹ Configuring blockchain tools...");
        router = router.with_blockchain_tools(eth_rpc, sol_rpc);
        console_log!("  ✓ Blockchain tools configured");
    } else {
        console_log!("  ℹ Blockchain tools not configured (RPC URLs not set)");
    }
    
    console_log!("  ✓ Router initialized");
    
    console_log!("=== All services initialized successfully ===");
    
    // ========================================
    // 9. Handle Request
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
    let mcp_servers = env.var("MCP_SERVERS")
        .map(|v| v.to_string())
        .ok();
    
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
                    console_log!("    ✓ Connected successfully, registered {} tools", tool_count);
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
