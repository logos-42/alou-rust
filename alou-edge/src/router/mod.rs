use crate::agent::cluster_action::ClusterActionManager;
use crate::agent::cluster_executor::ClusterExecutor;
use crate::agent::core::AgentCore;
use crate::agent::discovery::AgentDiscovery;
use crate::agent::session::SessionManager;
use crate::agent::stream::get_events;
use crate::agent::tools::{BroadcastTool, QueryTool, TransactionTool};
use crate::mcp::tools::AgentWalletTool;
use crate::mcp::UiResourceBuilder;
use crate::middleware::SubscriptionGuard;
use crate::storage::kv::KvStore;
use crate::utils::error::AloudError;
use crate::utils::metrics::MetricsCollector;
use crate::web3::auth::WalletAuth;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use worker::*;

pub mod agent;
mod blockchain;
mod cluster_action;
mod diap;
mod mcp;
pub mod pubsub;
mod session;
mod subscription;
mod user_config;
mod wallet;

pub struct Router {
    kv: KvStore,
    session_manager: SessionManager,
    agent_wallet_tool: AgentWalletTool,
    wallet_auth: Option<WalletAuth>,
    agent_core: Option<Arc<AgentCore>>,
    agent_discovery: Option<AgentDiscovery>,
    query_tool: Option<QueryTool>,
    transaction_tool: Option<TransactionTool>,
    broadcast_tool: Option<BroadcastTool>,
    pubsub_manager: pubsub::PubSubManager,
    cluster_action_manager: Option<ClusterActionManager>,
    cluster_executor: Option<ClusterExecutor>,
    subscription_guard: Option<SubscriptionGuard>,
    metrics: MetricsCollector,
}

impl Router {
    pub fn new(kv: KvStore) -> Self {
        let session_store = kv.clone();
        let wallet_store = kv.clone();
        let pubsub_store = kv.clone();
        let cluster_action_store = kv.clone();
        Self {
            kv,
            session_manager: SessionManager::new(session_store),
            agent_wallet_tool: AgentWalletTool::new(wallet_store),
            wallet_auth: None,
            agent_core: None,
            agent_discovery: None,
            query_tool: None,
            transaction_tool: None,
            broadcast_tool: None,
            pubsub_manager: pubsub::PubSubManager::new(pubsub_store),
            cluster_action_manager: Some(ClusterActionManager::new(cluster_action_store)),
            cluster_executor: None,
            subscription_guard: None,
            metrics: MetricsCollector::new(),
        }
    }

    pub fn with_wallet_auth(mut self, kv: KvStore, jwt_secret: String) -> Self {
        self.wallet_auth = Some(WalletAuth::new(kv, jwt_secret));
        self
    }

    pub fn with_agent_core(mut self, agent_core: AgentCore) -> Self {
        let agent_core_arc = Arc::new(agent_core);
        let session_manager = self.session_manager.clone();
        let pubsub_manager = self.pubsub_manager.clone();
        
        // 初始化 ClusterExecutor
        self.cluster_executor = Some(ClusterExecutor::new(
            agent_core_arc.clone(),
            session_manager,
            pubsub_manager,
        ));
        
        self.agent_core = Some(agent_core_arc);
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

    pub fn with_subscription_guard(mut self, subscription_guard: SubscriptionGuard) -> Self {
        self.subscription_guard = Some(subscription_guard);
        self
    }

    pub async fn handle(&mut self, mut req: Request, env: Env) -> Result<Response> {
        let start_time = crate::utils::time::now_timestamp_millis();
        let path = req.path();
        let method = req.method();

        console_log!("→ {} {}", method.to_string(), path);
        
        // 确保集群执行器已初始化（如果需要）
        // Note: AgentCore doesn't implement Clone, so we skip executor initialization here
        // The executor should be initialized elsewhere if needed

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
                session::handle_delete_session(&self.session_manager, session_id, &self.kv).await
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

            // User config endpoints
            (Method::Get, "/api/user/config") => {
                match env.kv("CACHE") {
                    Ok(kv_binding) => {
                        let kv = KvStore::new(kv_binding);
                        user_config::handle_get_user_config(self.wallet_auth.as_ref(), &kv, req).await
                            .map_err(|e| worker::Error::RustError(e.to_string()))
                    }
                    Err(e) => {
                        let error_response = ErrorResponse {
                            error: format!("Failed to access KV store: {}", e),
                        };
                        json_response_with_status(&error_response, 500)
                    }
                }
            }
            (Method::Post, "/api/user/config") => {
                match env.kv("CACHE") {
                    Ok(kv_binding) => {
                        let kv = KvStore::new(kv_binding);
                        user_config::handle_save_user_config(self.wallet_auth.as_ref(), &kv, req).await
                            .map_err(|e| worker::Error::RustError(e.to_string()))
                    }
                    Err(e) => {
                        let error_response = ErrorResponse {
                            error: format!("Failed to access KV store: {}", e),
                        };
                        json_response_with_status(&error_response, 500)
                    }
                }
            }
            (Method::Post, "/api/user/config/verify") => {
                user_config::handle_verify_api_key(self.wallet_auth.as_ref(), req).await
                    .map_err(|e| worker::Error::RustError(e.to_string()))
            }

            // AI 任务路由 - 异步任务创建和执行
            (Method::Post, "/api/ai-task/init-and-start") => {
                // 直接调用兼容性处理（使用AITaskDO）
                self.handle_compatible_chat(req, env).await
            }

            (Method::Get, path) if path.starts_with("/api/ai-task/") && path.ends_with("/status") => {
                let task_id = path.trim_start_matches("/api/ai-task/").trim_end_matches("/status");
                self.handle_task_status(env, task_id).await
            }

            (Method::Get, path) if path.starts_with("/api/ai-task/") && path.ends_with("/pending-tools") => {
                let task_id = path.trim_start_matches("/api/ai-task/").trim_end_matches("/pending-tools");
                self.handle_pending_tools(env, task_id).await
            }

            (Method::Get, path) if path.starts_with("/api/ai-task/") && path.ends_with("/result") => {
                let task_id = path.trim_start_matches("/api/ai-task/").trim_end_matches("/result");
                // result 接口返回完整结果（兼容前端AsyncTaskService）
                use crate::compatibility::router::handle_task_status as handle_status;
                match handle_status(env, task_id).await {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        let error_response = ErrorResponse {
                            error: format!("Failed to get task result: {}", e),
                        };
                        json_response_with_status(&error_response, 500)
                    }
                }
            }

            (Method::Post, path) if path.starts_with("/api/ai-task/") && path.ends_with("/tool-result") => {
                let task_id = path.trim_start_matches("/api/ai-task/").trim_end_matches("/tool-result");
                self.handle_tool_result(env, task_id, req).await
            }

            (Method::Post, "/api/agent/chat") => {
                // 首先尝试兼容性处理
                match self.handle_compatible_chat(req, env).await {
                    Ok(response) => Ok(response),
                    Err(_) => {
                        // 如果兼容性处理失败，回退到原有逻辑
                        session::handle_agent_chat(
                            &self.session_manager,
                            self.agent_core.as_ref().map(|arc| arc.as_ref()),
                            self.wallet_auth.as_ref(),
                            self.subscription_guard.as_ref(),
                            &self.kv,
                            req,
                        )
                        .await
                    }
                }
            }
            (Method::Post, "/api/agent/resolve") => {
                agent::handle_resolve_agent(
                    self.agent_discovery.as_ref(),
                    &self.session_manager,
                    req,
                )
                .await
                .map_err(|e| worker::Error::RustError(e.to_string()))
                .map_err(|e| worker::Error::RustError(e.to_string()))
            }
            (Method::Post, "/api/agent/search") => {
                agent::handle_search_agents(self.agent_discovery.as_ref(), req).await
            }
            (Method::Post, "/api/agent/parse-creation-command") => {
                agent::handle_parse_creation_command(&self.session_manager, req).await
            }
            (Method::Post, "/api/agent/create-from-command") => {
                agent::handle_create_agent_from_command(&self.session_manager, &env, req).await
            }
            (Method::Post, "/api/agent/create") | (Method::Post, "/api/agent/create_agent") => {
                agent::handle_create_agent(&self.session_manager, req).await
            }
            (Method::Post, "/api/agent/update") => {
                agent::handle_update_agent(&self.session_manager, req).await
            }
            (Method::Post, "/api/agent/batch-create") => {
                agent::handle_batch_create_agent(&self.session_manager, req, &env).await
            }
            (Method::Get, path) if path.starts_with("/api/agent/batch-create/") => {
                agent::handle_get_batch_create_task(&self.session_manager, req, &env).await
            }
            (Method::Get, path) if path.starts_with("/api/agent/batch-create/sessions/") => {
                agent::handle_get_batch_agent_sessions(&self.session_manager, req, &env).await
            }
            (Method::Get, "/api/agent/progress") => self.handle_agent_progress(req).await,

            (Method::Post, "/api/mcp/ui-resource") => self.handle_mcp_ui_resource(req).await,
            (Method::Post, "/api/mcp/execute-tool") => {
                if let Some(agent_core) = self.agent_core.as_ref() {
                    mcp::handle_execute_tool(agent_core.as_ref().get_executor(), req).await
                } else {
                    Response::error("Agent core not initialized", 500)
                }
            }
            (Method::Get, "/api/mcp/tools") => {
                if let Some(agent_core) = self.agent_core.as_ref() {
                    mcp::handle_list_tools(agent_core.as_ref().get_executor()).await
                } else {
                    Response::error("Agent core not initialized", 500)
                }
            }

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

            // 集群行动路由
            (Method::Post, "/api/cluster-action/create") => {
                if let Some(ref manager) = self.cluster_action_manager {
                    cluster_action::handle_create_cluster_action(manager, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string()))
                } else {
                    Response::error("Cluster action manager not initialized", 500)
                }
            }
            (Method::Post, "/api/cluster-action/execute") => {
                if let (Some(ref manager), Some(ref executor)) = 
                    (self.cluster_action_manager.as_ref(), self.cluster_executor.as_ref()) {
                    cluster_action::handle_execute_cluster_action(manager, executor, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string()))
                } else {
                    Response::error("Cluster action manager or executor not initialized", 500)
                }
            }
            (Method::Get, path) if path.starts_with("/api/cluster-action/") && path.ends_with("/status") => {
                if let Some(ref manager) = self.cluster_action_manager {
                    cluster_action::handle_get_status(manager, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string()))
                } else {
                    Response::error("Cluster action manager not initialized", 500)
                }
            }
            (Method::Get, path) if path.starts_with("/api/cluster-action/") && path.ends_with("/results") => {
                if let Some(ref manager) = self.cluster_action_manager {
                    cluster_action::handle_get_results(manager, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string()))
                } else {
                    Response::error("Cluster action manager not initialized", 500)
                }
            }
            (Method::Post, path) if path.starts_with("/api/cluster-action/") && path.ends_with("/cancel") => {
                if let Some(ref manager) = self.cluster_action_manager {
                    cluster_action::handle_cancel_action(manager, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string()))
                } else {
                    Response::error("Cluster action manager not initialized", 500)
                }
            }
            (Method::Post, "/api/cluster-action/analyze") => {
                if let Some(ref manager) = self.cluster_action_manager {
                    cluster_action::handle_analyze_task(manager, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string()))
                } else {
                    Response::error("Cluster action manager not initialized", 500)
                }
            }
            (Method::Post, "/api/diap/timelock") => diap::handle_timelock_request(env, req).await,
            (Method::Post, "/api/diap/account") => diap::handle_account_request(env, req).await,

            // DIAP身份管理路由（基于SDK架构）
            (Method::Post, "/api/agent/diap/get-identity-by-session") => {
                crate::router::agent::agent_diap::handle_get_diap_identity_by_session(&self.session_manager, env, req).await
                    .map_err(|e| worker::Error::RustError(e.to_string()))
            }
            (Method::Post, "/api/agent/diap/save-complete-identity") => {
                crate::router::agent::agent_diap::handle_save_complete_diap_identity(&self.session_manager, env, req).await
                    .map_err(|e| worker::Error::RustError(e.to_string()))
            }
            (Method::Post, "/api/agent/diap/create-identity") => {
                crate::router::agent::agent_diap::handle_create_diap_identity(&self.session_manager, env, req).await
                    .map_err(|e| worker::Error::RustError(e.to_string()))
            }
            (Method::Post, "/api/agent/diap/register-onchain") => {
                crate::router::agent::agent_diap::handle_register_agent_onchain(&self.session_manager, env, req).await
                    .map_err(|e| worker::Error::RustError(e.to_string()))
            }

            // PubSub endpoints for group chat and agent communication
            (Method::Post, "/api/pubsub/publish") => {
                pubsub::handle_publish(&self.pubsub_manager, req).await
            }
            (Method::Get, "/api/pubsub/messages") => {
                pubsub::handle_get_messages(&self.pubsub_manager, req).await
            }

            // Subscription endpoints
            (Method::Post, "/api/subscription/check-trial") => {
                match crate::storage::subscription::SubscriptionStorage::new(&env) {
                    Ok(storage) => subscription::handle_check_trial(&storage, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string())),
                    Err(e) => Err(worker::Error::RustError(format!("Failed to initialize subscription storage: {}", e))),
                }
            }
            (Method::Post, "/api/subscription/get-trial") => {
                match crate::storage::subscription::SubscriptionStorage::new(&env) {
                    Ok(storage) => subscription::handle_get_or_create_trial(&storage, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string())),
                    Err(e) => Err(worker::Error::RustError(format!("Failed to initialize subscription storage: {}", e))),
                }
            }
            (Method::Get, "/api/subscription/plans") => {
                match crate::storage::subscription::SubscriptionStorage::new(&env) {
                    Ok(storage) => subscription::handle_get_plans(&storage).await
                        .map_err(|e| worker::Error::RustError(e.to_string())),
                    Err(e) => Err(worker::Error::RustError(format!("Failed to initialize subscription storage: {}", e))),
                }
            }
            (Method::Get, "/api/subscription/status") => {
                match crate::storage::subscription::SubscriptionStorage::new(&env) {
                    Ok(storage) => subscription::handle_get_status(&storage, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string())),
                    Err(e) => Err(worker::Error::RustError(format!("Failed to initialize subscription storage: {}", e))),
                }
            }
            (Method::Post, "/api/subscription/create") => {
                match crate::storage::subscription::SubscriptionStorage::new(&env) {
                    Ok(storage) => subscription::handle_create_subscription(&storage, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string())),
                    Err(e) => Err(worker::Error::RustError(format!("Failed to initialize subscription storage: {}", e))),
                }
            }
            (Method::Post, "/api/subscription/renew") => {
                match crate::storage::subscription::SubscriptionStorage::new(&env) {
                    Ok(storage) => subscription::handle_renew_subscription(&storage, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string())),
                    Err(e) => Err(worker::Error::RustError(format!("Failed to initialize subscription storage: {}", e))),
                }
            }
            (Method::Post, "/api/subscription/verify-payment") => {
                match crate::storage::subscription::SubscriptionStorage::new(&env) {
                    Ok(storage) => subscription::handle_verify_payment(&storage, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string())),
                    Err(e) => Err(worker::Error::RustError(format!("Failed to initialize subscription storage: {}", e))),
                }
            }
            (Method::Get, "/api/subscription/notifications") => {
                match crate::storage::subscription::SubscriptionStorage::new(&env) {
                    Ok(storage) => subscription::handle_get_notifications(&storage, req).await
                        .map_err(|e| worker::Error::RustError(e.to_string())),
                    Err(e) => Err(worker::Error::RustError(format!("Failed to initialize subscription storage: {}", e))),
                }
            }

            // 任务状态查询已迁移到 /api/ai-task/{task_id}/status

            // KV 存储管理端点
            (Method::Get, "/api/kv/keys") => {
                self.handle_kv_list_keys(req, env).await
            }
            (Method::Get, "/api/kv/get") => {
                self.handle_kv_get(req, env).await
            }
            (Method::Post, "/api/kv/put") => {
                self.handle_kv_put(req, env).await
            }
            (Method::Delete, "/api/kv/delete") => {
                self.handle_kv_delete(req, env).await
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
                "blockchain": "/api/blockchain/*",
                "kv": {
                    "keys": "/api/kv/keys",
                    "get": "/api/kv/get",
                    "put": "/api/kv/put",
                    "delete": "/api/kv/delete"
                },
                "diap": {
                    "get_identity": "/api/agent/diap/get-identity",
                    "get_identity_by_session": "/api/agent/diap/get-identity-by-session",
                    "create_identity": "/api/agent/diap/create-identity",
                    "save_complete_identity": "/api/agent/diap/save-complete-identity",
                    "update_identity": "/api/agent/diap/update-identity",
                    "register_onchain": "/api/agent/diap/register-onchain"
                }
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

        let events = get_events(&session_id, since, &self.kv).await;
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
    let headers = response.headers_mut();
    headers.set("Content-Type", "application/json; charset=utf-8")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
    Ok(response)
}

pub(crate) fn json_response_with_status<T: Serialize>(data: &T, status: u16) -> Result<Response> {
    let json = serde_json::to_string(data)
        .map_err(|e| worker::Error::RustError(format!("JSON serialization error: {}", e)))?;
 
    let mut response = Response::ok(json)?.with_status(status);
    let headers = response.headers_mut();
    headers.set("Content-Type", "application/json; charset=utf-8")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
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

// 兼容性处理方法实现
impl Router {
    /// 处理兼容性聊天请求
    async fn handle_compatible_chat(&self, req: &mut Request, env: &Env) -> Result<Response> {
        use crate::compatibility::router::handle_compatible_chat as handle_chat;
        handle_chat(env, req).await
    }
    
    /// 处理任务状态查询
    async fn handle_task_status(&self, env: &Env, task_id: &str) -> Result<Response> {
        use crate::compatibility::router::handle_task_status as handle_status;
        handle_status(env, task_id).await
    }

    /// 处理待处理工具查询
    async fn handle_pending_tools(&self, env: &Env, task_id: &str) -> Result<Response> {
        console_log!("[Router] 🔍 Forwarding pending-tools request to DO for task: {}", task_id);
        
        // 获取 Durable Object stub
        let namespace = env.durable_object("AI_TASKS")?;
        let id = namespace.id_from_name(task_id)?;
        let stub = id.get_stub()?;
        
        // 创建请求
        let mut init = worker::RequestInit::new();
        init.with_method(Method::Get);
        
        let request = worker::Request::new_with_init(
            "http://dummy/pending-tools",
            &init
        )?;
        
        // 调用 Durable Object
        match stub.fetch_with_request(request).await {
            Ok(response) => {
                console_log!("[Router] ✅ Got pending tools response from DO");
                Ok(response)
            }
            Err(e) => {
                console_error!("[Router] ❌ Failed to get pending tools: {}", e);
                // 返回空数组作为fallback
                let response = serde_json::json!({
                    "task_id": task_id,
                    "toolCalls": []
                });
                Response::from_json(&response)
            }
        }
    }
    
    /// 处理任务取消
    async fn handle_task_cancel(&self, env: &Env, task_id: &str) -> Result<Response> {
        use crate::compatibility::router::handle_task_cancel as handle_cancel;
        handle_cancel(env, task_id).await
    }
    
    /// 处理工具结果提交
    async fn handle_tool_result(&self, env: &Env, task_id: &str, req: &mut Request) -> Result<Response> {
        use crate::compatibility::router::handle_tool_result as handle_tool;
        handle_tool(env, task_id, req).await
    }

    /// KV 存储管理方法
    async fn handle_kv_list_keys(&self, req: &mut Request, _env: &Env) -> Result<Response> {
        let url = req.url()?;
        let prefix = url
            .query_pairs()
            .find(|(key, _)| key == "prefix")
            .map(|(_, value)| value.to_string())
            .unwrap_or_default();
        
        console_log!("[KV] Listing keys with prefix: {}", prefix);
        
        match self.kv.list(&prefix, Some(1000)).await {
            Ok(keys) => {
                let response = serde_json::json!({
                    "success": true,
                    "keys": keys,
                    "count": keys.len()
                });
                json_response(&response)
            }
            Err(e) => {
                console_error!("[KV] Failed to list keys: {}", e);
                let response = serde_json::json!({
                    "success": false,
                    "error": format!("Failed to list keys: {}", e)
                });
                json_response_with_status(&response, 500)
            }
        }
    }

    async fn handle_kv_get(&self, req: &mut Request, _env: &Env) -> Result<Response> {
        let url = req.url()?;
        let key = url
            .query_pairs()
            .find(|(key, _)| key == "key")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| {
                worker::Error::RustError("Missing 'key' parameter".to_string())
            })?;
        
        console_log!("[KV] Getting key: {}", key);
        
        match self.kv.get::<String>(&key).await {
            Ok(Some(value)) => {
                let response = serde_json::json!({
                    "success": true,
                    "value": value,
                    "key": key
                });
                json_response(&response)
            }
            Ok(None) => {
                let response = serde_json::json!({
                    "success": false,
                    "error": "Key not found",
                    "key": key
                });
                json_response_with_status(&response, 404)
            }
            Err(e) => {
                console_error!("[KV] Failed to get key {}: {}", key, e);
                let response = serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get key: {}", e),
                    "key": key
                });
                json_response_with_status(&response, 500)
            }
        }
    }

    async fn handle_kv_put(&self, req: &mut Request, _env: &Env) -> Result<Response> {
        let url = req.url()?;
        let key = url
            .query_pairs()
            .find(|(key, _)| key == "key")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| {
                worker::Error::RustError("Missing 'key' parameter".to_string())
            })?;
        
        let body: serde_json::Value = req.json().await.map_err(|e| {
            worker::Error::RustError(format!("Failed to parse request body: {}", e))
        })?;
        
        let value = body.get("value").ok_or_else(|| {
            worker::Error::RustError("Missing 'value' field in request body".to_string())
        })?;
        
        let ttl = url
            .query_pairs()
            .find(|(key, _)| key == "ttl")
            .and_then(|(_, value)| value.parse::<u64>().ok());
        
        console_log!("[KV] Putting key: {} (TTL: {:?})", key, ttl);
        
        match self.kv.put(&key, &value, ttl).await {
            Ok(()) => {
                let response = serde_json::json!({
                    "success": true,
                    "message": "Key stored successfully",
                    "key": key
                });
                json_response(&response)
            }
            Err(e) => {
                console_error!("[KV] Failed to put key {}: {}", key, e);
                let response = serde_json::json!({
                    "success": false,
                    "error": format!("Failed to store key: {}", e),
                    "key": key
                });
                json_response_with_status(&response, 500)
            }
        }
    }

    async fn handle_kv_delete(&self, req: &mut Request, _env: &Env) -> Result<Response> {
        let url = req.url()?;
        let key = url
            .query_pairs()
            .find(|(key, _)| key == "key")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| {
                worker::Error::RustError("Missing 'key' parameter".to_string())
            })?;
        
        console_log!("[KV] Deleting key: {}", key);
        
        match self.kv.delete(&key).await {
            Ok(()) => {
                let response = serde_json::json!({
                    "success": true,
                    "message": "Key deleted successfully",
                    "key": key
                });
                json_response(&response)
            }
            Err(e) => {
                console_error!("[KV] Failed to delete key {}: {}", key, e);
                let response = serde_json::json!({
                    "success": false,
                    "error": format!("Failed to delete key: {}", e),
                    "key": key
                });
                json_response_with_status(&response, 500)
            }
        }
    }
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
