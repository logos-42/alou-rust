use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use crate::agent::session::SessionManager;
use crate::agent::core::AgentCore;
use crate::agent::tools::{QueryTool, TransactionTool, BroadcastTool};
use crate::web3::auth::WalletAuth;
use crate::storage::kv::KvStore;
use crate::utils::error::AloudError;
use crate::utils::metrics::MetricsCollector;
use crate::agent::context::AgentContext;
use crate::mcp::registry::McpTool;

// Helper function to create JSON response with UTF-8 charset
fn json_response<T: Serialize>(data: &T) -> Result<Response> {
    let json = serde_json::to_string(data)
        .map_err(|e| worker::Error::RustError(format!("JSON serialization error: {}", e)))?;
    
    let mut response = Response::ok(json)?;
    response.headers_mut().set("Content-Type", "application/json; charset=utf-8")?;
    Ok(response)
}

fn json_response_with_status<T: Serialize>(data: &T, status: u16) -> Result<Response> {
    let json = serde_json::to_string(data)
        .map_err(|e| worker::Error::RustError(format!("JSON serialization error: {}", e)))?;
    
    let mut response = Response::ok(json)?.with_status(status);
    response.headers_mut().set("Content-Type", "application/json; charset=utf-8")?;
    Ok(response)
}

#[derive(Deserialize)]
struct CreateSessionRequest {
    wallet_address: Option<String>,
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct NonceResponse {
    nonce: String,
    message: String,
}

#[derive(Deserialize)]
struct VerifySignatureRequest {
    address: String,
    signature: String,
    message: String,
    chain: String,
}

#[derive(Serialize)]
struct VerifySignatureResponse {
    success: bool,
    token: String,
    wallet_address: String,
    chain: String,
}

#[derive(Serialize)]
struct WalletInfoResponse {
    wallet_address: String,
    chain: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    version: String,
    services: ServiceStatus,
    metrics: crate::utils::metrics::MetricsSnapshot,
    timestamp: String,
}

#[derive(Serialize)]
struct ServiceStatus {
    session_manager: String,
    wallet_auth: String,
    agent_core: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    session_id: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wallet_address: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    content: String,
    session_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_calls: Vec<crate::agent::core::ToolCallInfo>,
}

pub struct Router {
    session_manager: SessionManager,
    wallet_auth: Option<WalletAuth>,
    agent_core: Option<AgentCore>,
    query_tool: Option<QueryTool>,
    transaction_tool: Option<TransactionTool>,
    broadcast_tool: Option<BroadcastTool>,
    metrics: MetricsCollector,
}

impl Router {
    pub fn new(kv: KvStore) -> Self {
        Self {
            session_manager: SessionManager::new(kv),
            wallet_auth: None,
            agent_core: None,
            query_tool: None,
            transaction_tool: None,
            broadcast_tool: None,
            metrics: MetricsCollector::new(),
        }
    }
    
    pub fn with_wallet_auth(mut self, kv: KvStore, jwt_secret: String) -> Self {
        self.wallet_auth = Some(WalletAuth::new(kv, jwt_secret));
        self
    }
    
    pub fn with_agent_core(mut self, agent_core: AgentCore) -> Self {
        self.agent_core = Some(agent_core);
        self
    }
    
    pub fn with_blockchain_tools(
        mut self,
        eth_rpc_url: String,
        sol_rpc_url: String,
    ) -> Self {
        self.query_tool = Some(QueryTool::new(eth_rpc_url.clone(), sol_rpc_url.clone()));
        self.transaction_tool = Some(TransactionTool::new(eth_rpc_url.clone(), sol_rpc_url.clone()));
        self.broadcast_tool = Some(BroadcastTool::new(eth_rpc_url, sol_rpc_url));
        self
    }
    
    pub async fn handle(&self, mut req: Request, _env: Env) -> Result<Response> {
        let start_time = crate::utils::time::now_timestamp_millis();
        let path = req.path();
        let method = req.method();
        
        // Log request
        console_log!("→ {} {}", method.to_string(), path);
        
        // Add CORS headers
        let headers = Headers::new();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")?;
        headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
        
        // Handle preflight requests
        if method == Method::Options {
            console_log!("← OPTIONS {} (preflight)", path);
            return Response::empty()
                .map(|r| r.with_headers(headers));
        }
        
        // Route matching with proper routing table
        let result = self.route_request(&mut req, &method, &path).await;
        
        // Calculate request duration
        let end_time = crate::utils::time::now_timestamp_millis();
        let duration_us = ((end_time - start_time) * 1000) as u64; // Convert ms to microseconds
        
        // Record metrics
        let success = result.as_ref().map(|r| r.status_code() < 400).unwrap_or(false);
        self.metrics.record_request(&path, success, duration_us);
        
        // Log response
        let duration_ms = (end_time - start_time) as f64;
        match &result {
            Ok(response) => {
                console_log!(
                    "← {} {} - {} ({:.2}ms)",
                    method.to_string(),
                    path,
                    response.status_code(),
                    duration_ms
                );
            }
            Err(e) => {
                console_error!(
                    "← {} {} - ERROR: {} ({:.2}ms)",
                    method.to_string(),
                    path,
                    e,
                    duration_ms
                );
            }
        }
        
        // Add CORS headers and performance metrics to response
        result.map(|mut r| {
            r = r.with_headers(headers);
            // Add custom headers for performance monitoring
            let response_headers = r.headers_mut();
            let _ = response_headers.set(
                "X-Response-Time",
                &format!("{:.2}ms", duration_ms)
            );
            let _ = response_headers.set(
                "X-Request-ID",
                &uuid::Uuid::new_v4().to_string()
            );
            r
        })
    }
    
    /// Route request to appropriate handler
    async fn route_request(&self, req: &mut Request, method: &Method, path: &str) -> Result<Response> {
        match (method, path) {
            // Root endpoint
            (Method::Get, "/") => {
                self.handle_root().await
            }
            
            // Health and status endpoints
            (Method::Get, "/api/health") => {
                self.handle_health().await
            }
            
            (Method::Get, "/api/status") => {
                self.handle_status().await
            }
            
            // Session endpoints
            (Method::Post, "/api/session") => {
                self.handle_create_session(req).await
            }
            
            (Method::Get, path) if path.starts_with("/api/session/") => {
                let session_id = path.trim_start_matches("/api/session/");
                self.handle_get_session(session_id).await
            }
            
            (Method::Delete, path) if path.starts_with("/api/session/") => {
                let session_id = path.trim_start_matches("/api/session/");
                self.handle_delete_session(session_id).await
            }
            
            // Wallet authentication endpoints
            (Method::Get, path) if path.starts_with("/api/wallet/nonce/") => {
                let address = path.trim_start_matches("/api/wallet/nonce/");
                self.handle_get_nonce(address).await
            }
            
            (Method::Post, "/api/wallet/verify") => {
                self.handle_verify_signature(req).await
            }
            
            (Method::Get, "/api/wallet/me") => {
                self.handle_get_wallet_info(req).await
            }
            
            // Agent endpoints
            (Method::Post, "/api/agent/chat") => {
                self.handle_agent_chat(req).await
            }
            
            (Method::Post, "/api/agent/stream") => {
                self.handle_agent_stream(req).await
            }
            
            (Method::Post, "/api/agent/wallet") => {
                self.handle_agent_wallet(req).await
            }
            
            // Blockchain query endpoints
            (Method::Post, "/api/blockchain/balance") => {
                self.handle_blockchain_balance(req).await
            }
            
            (Method::Post, "/api/blockchain/transaction/build") => {
                self.handle_build_transaction(req).await
            }
            
            (Method::Post, "/api/blockchain/transaction/broadcast") => {
                self.handle_broadcast_transaction(req).await
            }
            
            (Method::Get, path) if path.starts_with("/api/blockchain/transaction/") => {
                let tx_hash = path.trim_start_matches("/api/blockchain/transaction/");
                self.handle_get_transaction_status(tx_hash, req).await
            }
            
            // 404 Not Found
            _ => {
                console_log!("Route not found: {} {}", method.to_string(), path);
                let error_response = ErrorResponse {
                    error: format!("Route not found: {} {}", method, path),
                };
                json_response_with_status(&error_response, 404)
            }
        }
    }
    
    /// GET / - Root endpoint
    async fn handle_root(&self) -> Result<Response> {
        let response = serde_json::json!({
            "name": "Alou Edge",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Web3 AI Agent on Cloudflare Workers",
            "endpoints": {
                "health": "/api/health",
                "status": "/api/status",
                "session": "/api/session",
                "agent": "/api/agent/chat",
                "blockchain": "/api/blockchain/*"
            }
        });
        json_response(&response)
    }
    
    /// GET /api/health - Health check endpoint
    async fn handle_health(&self) -> Result<Response> {
        let response = HealthResponse {
            status: "healthy".to_string(),
            timestamp: crate::utils::time::now_rfc3339(),
        };
        json_response(&response)
    }
    
    /// GET /api/status - Service status endpoint
    async fn handle_status(&self) -> Result<Response> {
        let response = StatusResponse {
            status: "operational".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            services: ServiceStatus {
                session_manager: "operational".to_string(),
                wallet_auth: if self.wallet_auth.is_some() {
                    "operational".to_string()
                } else {
                    "not_configured".to_string()
                },
                agent_core: if self.agent_core.is_some() {
                    "operational".to_string()
                } else {
                    "not_configured".to_string()
                },
            },
            metrics: self.metrics.snapshot(),
            timestamp: crate::utils::time::now_rfc3339(),
        };
        json_response(&response)
    }
    
    async fn handle_create_session(&self, req: &mut Request) -> Result<Response> {
        let body: CreateSessionRequest = match req.json().await {
            Ok(body) => body,
            Err(_) => {
                // If no body provided, create session without wallet address
                CreateSessionRequest {
                    wallet_address: None,
                }
            }
        };
        
        console_log!("Creating session with wallet_address: {:?}", body.wallet_address);
        
        match self.session_manager.create_session(body.wallet_address).await {
            Ok(session_id) => {
                console_log!("Session created: {}", session_id);
                let response = CreateSessionResponse { session_id };
                json_response(&response)
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
    
    async fn handle_get_session(&self, session_id: &str) -> Result<Response> {
        match self.session_manager.get_session(session_id).await {
            Ok(session) => {
                json_response(&session)
            }
            Err(AloudError::InvalidInput(_)) => {
                let error_response = ErrorResponse {
                    error: format!("Session not found: {}", session_id),
                };
                json_response_with_status(&error_response, 404)
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
    
    async fn handle_delete_session(&self, session_id: &str) -> Result<Response> {
        match self.session_manager.clear_session(session_id).await {
            Ok(_) => {
                Response::ok("Session deleted")
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
    
    async fn handle_get_nonce(&self, address: &str) -> Result<Response> {
        let wallet_auth = match &self.wallet_auth {
            Some(auth) => auth,
            None => {
                let error_response = ErrorResponse {
                    error: "Wallet authentication not configured".to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        };
        
        match wallet_auth.generate_nonce_for_address(address).await {
            Ok(nonce) => {
                let response = NonceResponse {
                    nonce: nonce.clone(),
                    message: format!("Sign this message to authenticate: {}", nonce),
                };
                json_response(&response)
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
    
    async fn handle_verify_signature(&self, req: &mut Request) -> Result<Response> {
        let wallet_auth = match &self.wallet_auth {
            Some(auth) => auth,
            None => {
                let error_response = ErrorResponse {
                    error: "Wallet authentication not configured".to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        };
        
        let body: VerifySignatureRequest = match req.json().await {
            Ok(body) => body,
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        let chain = match crate::utils::crypto::ChainType::from_str(&body.chain) {
            Ok(chain) => chain,
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        match wallet_auth.verify_and_create_token(
            &body.address,
            &body.signature,
            &body.message,
            chain,
        ).await {
            Ok(token) => {
                let response = VerifySignatureResponse {
                    success: true,
                    token,
                    wallet_address: body.address,
                    chain: body.chain,
                };
                json_response(&response)
            }
            Err(AloudError::InvalidSignature) => {
                let error_response = ErrorResponse {
                    error: "Invalid signature".to_string(),
                };
                json_response_with_status(&error_response, 401)
            }
            Err(AloudError::NonceExpired) => {
                let error_response = ErrorResponse {
                    error: "Nonce expired".to_string(),
                };
                json_response_with_status(&error_response, 401)
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
    
    async fn handle_get_wallet_info(&self, req: &Request) -> Result<Response> {
        let wallet_auth = match &self.wallet_auth {
            Some(auth) => auth,
            None => {
                let error_response = ErrorResponse {
                    error: "Wallet authentication not configured".to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        };
        
        // Extract token from Authorization header
        let token = match req.headers().get("Authorization")? {
            Some(auth_header) => {
                auth_header.strip_prefix("Bearer ").unwrap_or(&auth_header).to_string()
            }
            None => {
                let error_response = ErrorResponse {
                    error: "Missing Authorization header".to_string(),
                };
                return json_response_with_status(&error_response, 401);
            }
        };
        
        match wallet_auth.parse_token(&token) {
            Ok((wallet_address, chain)) => {
                let response = WalletInfoResponse {
                    wallet_address,
                    chain: chain.as_str().to_string(),
                };
                json_response(&response)
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("Invalid token: {}", e),
                };
                json_response_with_status(&error_response, 401)
            }
        }
    }
    
    async fn handle_agent_chat(&self, req: &mut Request) -> Result<Response> {
        let agent_core = match &self.agent_core {
            Some(core) => core,
            None => {
                let error_response = ErrorResponse {
                    error: "Agent not configured".to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        };
        
        // Parse request body
        let body: ChatRequest = match req.json().await {
            Ok(body) => body,
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        // Priority: request body > JWT token > session binding
        // First, try to get wallet from request body or JWT
        console_log!("Chat request - session_id: {}, wallet_address from body: {:?}", body.session_id, body.wallet_address);
        
        let wallet_address = body.wallet_address
            .or_else(|| self.extract_wallet_from_token(req).ok());
        
        console_log!("Wallet address after body/jwt: {:?}", wallet_address);
        
        // If still no wallet address, try to get from session
        let wallet_address = match wallet_address {
            Some(addr) => {
                console_log!("Using wallet from body/jwt: {}", addr);
                Some(addr)
            },
            None => {
                // Try to get from session
                match self.session_manager.get_session(&body.session_id).await {
                    Ok(session) => {
                        console_log!("Wallet from session: {:?}", session.wallet_address);
                        session.wallet_address
                    },
                    Err(_) => {
                        console_log!("No wallet found anywhere");
                        None
                    },
                }
            }
        };
        
        console_log!("Final wallet_address being passed to agent: {:?}", wallet_address);
        
        // Handle message
        match agent_core
            .handle_message(&body.session_id, &body.message, wallet_address)
            .await
        {
            Ok(response) => {
                let chat_response = ChatResponse {
                    content: response.content,
                    session_id: response.session_id,
                    tool_calls: response.tool_calls,
                };
                json_response(&chat_response)
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
    
    async fn handle_agent_stream(&self, _req: &mut Request) -> Result<Response> {
        // For now, streaming is not implemented
        // In production, this would use Server-Sent Events (SSE)
        let error_response = ErrorResponse {
            error: "Streaming not yet implemented. Use /api/agent/chat instead.".to_string(),
        };
        json_response_with_status(&error_response, 501)
    }
    
    async fn handle_agent_wallet(&self, req: &mut Request) -> Result<Response> {
        #[derive(Deserialize)]
        struct AgentWalletRequest {
            session_id: String,
            action: String,
            chain: Option<String>,
            transaction: Option<Value>,
            balance: Option<String>,
        }
        
        let body: AgentWalletRequest = match req.json().await {
            Ok(body) => body,
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        // Build tool args
        let mut args = json!({
            "action": body.action
        });
        
        if let Some(chain) = body.chain {
            args["chain"] = json!(chain);
        }
        
        if let Some(transaction) = body.transaction {
            args["transaction"] = transaction;
        }
        
        if let Some(balance) = body.balance {
            args["balance"] = json!(balance);
        }
        
        // Create context
        let context = AgentContext {
            session_id: body.session_id.clone(),
            wallet_address: None,
            chain: None,
        };
        
        // Execute tool directly - we'll need to get KV from environment
        // For now, let's return an error since we need proper KV setup
        let error_response = ErrorResponse {
            error: "Agent wallet tool requires KV store setup".to_string(),
        };
        json_response_with_status(&error_response, 500)
    }
    
    /// Extract wallet address from Authorization token (optional)
    fn extract_wallet_from_token(&self, req: &Request) -> std::result::Result<String, AloudError> {
        let wallet_auth = self
            .wallet_auth
            .as_ref()
            .ok_or_else(|| AloudError::AuthError("Wallet auth not configured".to_string()))?;
        
        let token = req
            .headers()
            .get("Authorization")
            .map_err(|e| AloudError::WorkerError(e.to_string()))?
            .ok_or_else(|| AloudError::AuthError("Missing Authorization header".to_string()))?;
        
        let token = token.strip_prefix("Bearer ").unwrap_or(&token);
        
        let (wallet_address, _chain) = wallet_auth.parse_token(token)?;
        Ok(wallet_address)
    }
    
    // ========================================
    // Blockchain Tool Endpoints
    // ========================================
    
    async fn handle_blockchain_balance(&self, req: &mut Request) -> Result<Response> {
        let query_tool = match &self.query_tool {
            Some(tool) => tool,
            None => {
                let error_response = ErrorResponse {
                    error: "Blockchain tools not configured".to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        };
        
        #[derive(Deserialize)]
        struct BalanceRequest {
            address: String,
            chain: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            token_address: Option<String>,
        }
        
        #[derive(Serialize)]
        struct BalanceResponse {
            address: String,
            chain: String,
            balance: String,
        }
        
        let body: BalanceRequest = match req.json().await {
            Ok(body) => body,
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        let balance = match body.chain.to_lowercase().as_str() {
            "eth" | "ethereum" => {
                if let Some(token_addr) = body.token_address {
                    query_tool.get_erc20_balance(&token_addr, &body.address).await
                } else {
                    query_tool.get_eth_balance(&body.address).await
                }
            }
            "sol" | "solana" => query_tool.get_sol_balance(&body.address).await,
            _ => {
                let error_response = ErrorResponse {
                    error: format!("Unsupported chain: {}", body.chain),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        match balance {
            Ok(balance) => {
                let response = BalanceResponse {
                    address: body.address,
                    chain: body.chain,
                    balance,
                };
                json_response(&response)
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
    
    async fn handle_build_transaction(&self, req: &mut Request) -> Result<Response> {
        let tx_tool = match &self.transaction_tool {
            Some(tool) => tool,
            None => {
                let error_response = ErrorResponse {
                    error: "Transaction tools not configured".to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        };
        
        #[derive(Deserialize)]
        struct BuildTxRequest {
            from: String,
            to: String,
            value: f64,
            chain: String,
        }
        
        let body: BuildTxRequest = match req.json().await {
            Ok(body) => body,
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        let tx_data = match body.chain.to_lowercase().as_str() {
            "eth" | "ethereum" => {
                match tx_tool.build_eth_transaction(&body.from, &body.to, body.value).await {
                    Ok(tx) => serde_json::to_value(&tx).unwrap_or(Value::Null),
                    Err(e) => {
                        let error_response = ErrorResponse {
                            error: e.to_string(),
                        };
                        return json_response_with_status(&error_response, 500);
                    }
                }
            }
            "sol" | "solana" => {
                match tx_tool.build_sol_transaction(&body.from, &body.to, body.value).await {
                    Ok(tx) => serde_json::to_value(&tx).unwrap_or(Value::Null),
                    Err(e) => {
                        let error_response = ErrorResponse {
                            error: e.to_string(),
                        };
                        return json_response_with_status(&error_response, 500);
                    }
                }
            }
            _ => {
                let error_response = ErrorResponse {
                    error: format!("Unsupported chain: {}", body.chain),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        json_response(&tx_data)
    }
    
    async fn handle_broadcast_transaction(&self, req: &mut Request) -> Result<Response> {
        let broadcast_tool = match &self.broadcast_tool {
            Some(tool) => tool,
            None => {
                let error_response = ErrorResponse {
                    error: "Broadcast tools not configured".to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        };
        
        #[derive(Deserialize)]
        struct BroadcastRequest {
            signed_tx: String,
            chain: String,
        }
        
        #[derive(Serialize)]
        struct BroadcastResponse {
            tx_hash: String,
            chain: String,
        }
        
        let body: BroadcastRequest = match req.json().await {
            Ok(body) => body,
            Err(e) => {
                let error_response = ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        let tx_hash = match body.chain.to_lowercase().as_str() {
            "eth" | "ethereum" => {
                broadcast_tool.broadcast_eth_transaction(&body.signed_tx).await
            }
            "sol" | "solana" => {
                broadcast_tool.broadcast_sol_transaction(&body.signed_tx).await
            }
            _ => {
                let error_response = ErrorResponse {
                    error: format!("Unsupported chain: {}", body.chain),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        match tx_hash {
            Ok(tx_hash) => {
                let response = BroadcastResponse {
                    tx_hash,
                    chain: body.chain,
                };
                json_response(&response)
            }
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
    
    async fn handle_get_transaction_status(&self, tx_hash: &str, req: &Request) -> Result<Response> {
        let broadcast_tool = match &self.broadcast_tool {
            Some(tool) => tool,
            None => {
                let error_response = ErrorResponse {
                    error: "Broadcast tools not configured".to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        };
        
        // Get chain from query parameter
        let url = req.url()?;
        let chain = url
            .query_pairs()
            .find(|(k, _)| k == "chain")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "eth".to_string());
        
        let receipt = match chain.to_lowercase().as_str() {
            "eth" | "ethereum" => {
                broadcast_tool.get_eth_transaction_receipt(tx_hash).await
            }
            "sol" | "solana" => {
                broadcast_tool.get_sol_transaction_status(tx_hash).await
            }
            _ => {
                let error_response = ErrorResponse {
                    error: format!("Unsupported chain: {}", chain),
                };
                return json_response_with_status(&error_response, 400);
            }
        };
        
        match receipt {
            Ok(receipt) => json_response(&receipt),
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
}


