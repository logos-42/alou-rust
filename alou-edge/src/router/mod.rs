use crate::agent::core::AgentCore;
use crate::agent::discovery::AgentDiscovery;
use crate::agent::session::SessionManager;
use crate::agent::stream::get_events;
use crate::agent::tools::{BroadcastTool, QueryTool, TransactionTool};
use crate::mcp::tools::AgentWalletTool;
use crate::mcp::UiResourceBuilder;
use crate::storage::kv::KvStore;
use crate::utils::error::AloudError;
use crate::utils::metrics::MetricsCollector;
use crate::web3::auth::WalletAuth;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

mod agent;
mod blockchain;
mod diap;
mod pubsub;
mod session;
mod wallet;

pub struct Router {
    session_manager: SessionManager,
    agent_wallet_tool: AgentWalletTool,
    wallet_auth: Option<WalletAuth>,
    agent_core: Option<AgentCore>,
    agent_discovery: Option<AgentDiscovery>,
    query_tool: Option<QueryTool>,
    transaction_tool: Option<TransactionTool>,
    broadcast_tool: Option<BroadcastTool>,
    pubsub_manager: pubsub::PubSubManager,
    metrics: MetricsCollector,
}

impl Router {
    pub fn new(kv: KvStore) -> Self {
        let session_store = kv.clone();
        let wallet_store = kv.clone();
        let pubsub_store = kv;
        Self {
            session_manager: SessionManager::new(session_store),
            agent_wallet_tool: AgentWalletTool::new(wallet_store),
            wallet_auth: None,
            agent_core: None,
            agent_discovery: None,
            query_tool: None,
            transaction_tool: None,
            broadcast_tool: None,
            pubsub_manager: pubsub::PubSubManager::new(pubsub_store),
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

    pub fn with_agent_discovery(mut self, agent_discovery: AgentDiscovery) -> Self {
        self.agent_discovery = Some(agent_discovery);
        self
    }

    pub fn with_blockchain_tools(
        mut self,
        eth_rpc_url: String,
        eth_testnet_rpc_url: Option<String>,
        sol_rpc_url: String,
    ) -> Self {
        self.query_tool = Some(QueryTool::new(
            eth_rpc_url.clone(),
            eth_testnet_rpc_url.clone(),
            sol_rpc_url.clone(),
        ));
        self.transaction_tool = Some(TransactionTool::new(
            eth_rpc_url.clone(),
            eth_testnet_rpc_url.clone(),
            sol_rpc_url.clone(),
        ));
        self.broadcast_tool = Some(BroadcastTool::new(
            eth_rpc_url,
            eth_testnet_rpc_url,
            sol_rpc_url,
        ));
        self
    }

    pub async fn handle(&self, mut req: Request, env: Env) -> Result<Response> {
        let start_time = crate::utils::time::now_timestamp_millis();
        let path = req.path();
        let method = req.method();

        console_log!("→ {} {}", method.to_string(), path);

        let headers = Headers::new();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")?;
        headers.set(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )?;

        if method == Method::Options {
            console_log!("← OPTIONS {} (preflight)", path);
            return Response::empty().map(|r| r.with_headers(headers));
        }

        let result = self.route_request(&mut req, &method, &path, &env).await;

        let end_time = crate::utils::time::now_timestamp_millis();
        let duration_us = ((end_time - start_time) * 1000) as u64;

        let success = result
            .as_ref()
            .map(|r| r.status_code() < 400)
            .unwrap_or(false);
        self.metrics.record_request(&path, success, duration_us);

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

        match result {
            Ok(r) => {
                let mut response = r.with_headers(headers);
                let response_headers = response.headers_mut();
                let _ = response_headers.set("X-Response-Time", &format!("{:.2}ms", duration_ms));
                let _ = response_headers.set("X-Request-ID", &uuid::Uuid::new_v4().to_string());
                Ok(response)
            }
            Err(e) => {
                // Create error response with CORS headers
                let error_response = ErrorResponse {
                    error: format!("Internal server error: {}", e),
                };
                let json = serde_json::to_string(&error_response)
                    .unwrap_or_else(|_| r#"{"error":"Failed to serialize error"}"#.to_string());
                
                let mut response = Response::ok(json)
                    .map_err(|_| worker::Error::RustError("Failed to create error response".to_string()))?
                    .with_status(500)
                    .with_headers(headers);
                
                let response_headers = response.headers_mut();
                let _ = response_headers.set("Content-Type", "application/json; charset=utf-8");
                let _ = response_headers.set("X-Response-Time", &format!("{:.2}ms", duration_ms));
                let _ = response_headers.set("X-Request-ID", &uuid::Uuid::new_v4().to_string());
                Ok(response)
            }
        }
    }

    async fn route_request(
        &self,
        req: &mut Request,
        method: &Method,
        path: &str,
        env: &Env,
    ) -> Result<Response> {
        match (method, path) {
            (Method::Get, "/") => self.handle_root().await,
            (Method::Get, "/api/health") => self.handle_health().await,
            (Method::Get, "/api/status") => self.handle_status().await,

            (Method::Post, "/api/session") => {
                session::handle_create_session(&self.session_manager, req).await
            }
            (Method::Get, path) if path.starts_with("/api/session/") => {
                let session_id = path.trim_start_matches("/api/session/");
                session::handle_get_session(&self.session_manager, session_id).await
            }
            (Method::Delete, path) if path.starts_with("/api/session/") => {
                let session_id = path.trim_start_matches("/api/session/");
                session::handle_delete_session(&self.session_manager, session_id).await
            }

            (Method::Get, path) if path.starts_with("/api/wallet/nonce/") => {
                let address = path.trim_start_matches("/api/wallet/nonce/");
                wallet::handle_get_nonce(self.wallet_auth.as_ref(), address).await
            }
            (Method::Post, "/api/wallet/verify") => {
                wallet::handle_verify_signature(self.wallet_auth.as_ref(), req).await
            }
            (Method::Get, "/api/wallet/me") => {
                wallet::handle_get_wallet_info(self.wallet_auth.as_ref(), req).await
            }
            (Method::Post, "/api/agent/wallet") => {
                wallet::handle_agent_wallet(&self.session_manager, &self.agent_wallet_tool, req)
                    .await
            }

            (Method::Post, "/api/agent/chat") => {
                session::handle_agent_chat(
                    &self.session_manager,
                    self.agent_core.as_ref(),
                    self.wallet_auth.as_ref(),
                    req,
                )
                .await
            }
            (Method::Post, "/api/agent/resolve") => {
                agent::handle_resolve_agent(
                    self.agent_discovery.as_ref(),
                    &self.session_manager,
                    req,
                )
                .await
            }
            (Method::Post, "/api/agent/search") => {
                agent::handle_search_agents(self.agent_discovery.as_ref(), req).await
            }
            (Method::Post, "/api/agent/diap/get-identity") => {
                agent::handle_get_diap_identity(&self.session_manager, &env, req).await
            }
            (Method::Post, "/api/agent/diap/get-identity-by-session") => {
                agent::handle_get_diap_identity_by_session(&self.session_manager, req).await
            }
            (Method::Post, "/api/agent/create-claude") => {
                agent::handle_create_claude_agent(&self.session_manager, req).await
            }
            (Method::Post, "/api/agent/diap/register-onchain") => {
                agent::handle_register_agent_onchain(&self.session_manager, &env, req).await
            }
            (Method::Get, "/api/agent/progress") => self.handle_agent_progress(req).await,

            (Method::Post, "/api/mcp/ui-resource") => self.handle_mcp_ui_resource(req).await,

            (Method::Post, "/api/blockchain/balance") => {
                blockchain::handle_blockchain_balance(self.query_tool.as_ref(), req).await
            }
            (Method::Get, "/api/blockchain/tokens") => {
                blockchain::handle_blockchain_tokens(req).await
            }
            (Method::Post, "/api/blockchain/transaction/build") => {
                blockchain::handle_build_transaction(self.transaction_tool.as_ref(), req).await
            }
            (Method::Post, "/api/blockchain/transaction/broadcast") => {
                blockchain::handle_broadcast_transaction(self.broadcast_tool.as_ref(), req).await
            }
            (Method::Get, path) if path.starts_with("/api/blockchain/transaction/") => {
                let tx_hash = path.trim_start_matches("/api/blockchain/transaction/");
                blockchain::handle_get_transaction_status(
                    self.broadcast_tool.as_ref(),
                    tx_hash,
                    req,
                )
                .await
            }

            (Method::Post, "/api/diap/token") => diap::handle_token_request(env, req).await,
            (Method::Post, "/api/diap/agent") => diap::handle_agent_request(env, req).await,
            (Method::Post, "/api/diap/payment/core") => {
                diap::handle_payment_core_request(env, req).await
            }
            (Method::Post, "/api/diap/payment/channel") => {
                diap::handle_payment_channel_request(env, req).await
            }
            (Method::Post, "/api/diap/payment/privacy") => {
                diap::handle_payment_privacy_request(env, req).await
            }
            (Method::Post, "/api/diap/governance") => {
                diap::handle_governance_request(env, req).await
            }
            (Method::Post, "/api/diap/timelock") => diap::handle_timelock_request(env, req).await,
            (Method::Post, "/api/diap/account") => diap::handle_account_request(env, req).await,

            // PubSub endpoints for group chat and agent communication
            (Method::Post, "/api/pubsub/publish") => {
                pubsub::handle_publish(&self.pubsub_manager, req).await
            }
            (Method::Get, "/api/pubsub/messages") => {
                pubsub::handle_get_messages(&self.pubsub_manager, req).await
            }

            _ => {
                console_log!("Route not found: {} {}", method.to_string(), path);
                let error_response = ErrorResponse {
                    error: format!("Route not found: {} {}", method, path),
                };
                json_response_with_status(&error_response, 404)
            }
        }
    }

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

    async fn handle_health(&self) -> Result<Response> {
        let response = HealthResponse {
            status: "healthy".to_string(),
            timestamp: crate::utils::time::now_rfc3339(),
        };
        json_response(&response)
    }

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

    async fn handle_agent_progress(&self, req: &Request) -> Result<Response> {
        let url = req.url()?;
        let mut session_id: Option<String> = None;
        let mut since: Option<i64> = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "session_id" | "sessionId" => session_id = Some(value.into_owned()),
                "since" | "cursor" => {
                    if let Ok(parsed) = value.parse::<i64>() {
                        since = Some(parsed);
                    }
                }
                _ => {}
            }
        }

        let session_id = match session_id {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                let error_response = ErrorResponse {
                    error: "Missing session_id query parameter".to_string(),
                };
                return json_response_with_status(&error_response, 400);
            }
        };

        let events = get_events(&session_id, since).await;
        let response = json!({
            "session_id": session_id,
            "events": events,
        });
        json_response(&response)
    }

    async fn handle_mcp_ui_resource(&self, req: &mut Request) -> Result<Response> {
        let body: McpUiResourceRequest = match req.json().await {
            Ok(body) => body,
            Err(e) => {
                console_log!("Invalid MCP UI resource request body: {}", e);
                let error_response = ErrorResponse {
                    error: format!("Invalid request body: {}", e),
                };
                return json_response_with_status(&error_response, 400);
            }
        };

        console_log!(
            "MCP UI resource request - target: {}, params: {:?}",
            body.target,
            body.params
        );

        let params = body.params.unwrap_or_else(|| json!({}));
        let builder = UiResourceBuilder::new(&self.session_manager, &self.agent_wallet_tool);

        match builder.build_resource(&body.target, params).await {
            Ok(resource) => {
                console_log!("MCP UI resource built successfully for target: {}", body.target);
                json_response(&resource)
            }
            Err(AloudError::InvalidInput(msg)) => {
                console_log!(
                    "Invalid input for MCP UI resource (target: {}): {}",
                    body.target,
                    msg
                );
                let error_response = ErrorResponse {
                    error: format!("Invalid request: {}", msg),
                };
                json_response_with_status(&error_response, 400)
            }
            Err(e) => {
                let error_msg = e.to_string();
                console_log!(
                    "MCP UI resource build error (target: {}): {}",
                    body.target,
                    error_msg
                );
                let error_response = ErrorResponse {
                    error: format!("Failed to build resource: {}", error_msg),
                };
                json_response_with_status(&error_response, 500)
            }
        }
    }
}

pub(crate) fn json_response<T: Serialize>(data: &T) -> Result<Response> {
    let json = serde_json::to_string(data)
        .map_err(|e| worker::Error::RustError(format!("JSON serialization error: {}", e)))?;

    let mut response = Response::ok(json)?;
    response
        .headers_mut()
        .set("Content-Type", "application/json; charset=utf-8")?;
    Ok(response)
}

pub(crate) fn json_response_with_status<T: Serialize>(data: &T, status: u16) -> Result<Response> {
    let json = serde_json::to_string(data)
        .map_err(|e| worker::Error::RustError(format!("JSON serialization error: {}", e)))?;

    let mut response = Response::ok(json)?.with_status(status);
    response
        .headers_mut()
        .set("Content-Type", "application/json; charset=utf-8")?;
    Ok(response)
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    pub error: String,
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
pub(crate) struct McpUiResourceRequest {
    pub target: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}
