//! Polymarket 预测市场工具
//!
//! 提供对 Polymarket CLOB API 的完整访问，支持市场搜索、价格查询、下单交易等功能
//! 让 AI Agent 可以直接从终端查询和交易预测市场

use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolMetadata, ExecutionContext, ToolCategory, ToolPriority, ToolStatus};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

const CLOB_API_URL: &str = "https://clob.polymarket.com";
const GAMMA_API_URL: &str = "https://gamma-api.polymarket.com";

/// Polymarket 工具
#[derive(Clone)]
pub struct PolymarketTool {
    metadata: ToolMetadata,
    client: Arc<reqwest::Client>,
    /// 已认证的 API 凭证 (api_key, api_secret, api_passphrase)
    credentials: Arc<RwLock<Option<PolymarketCredentials>>>,
}

/// Polymarket API 认证凭证
#[derive(Debug, Clone)]
struct PolymarketCredentials {
    api_key: String,
    api_secret: String,
    api_passphrase: String,
}

impl PolymarketTool {
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "polymarket".to_string(),
            name: "Polymarket Prediction Market".to_string(),
            description: "Access Polymarket prediction markets - search markets, get prices, orderbooks, place trades, manage positions. Enables AI agents to interact with prediction markets.".to_string(),
            category: ToolCategory::Web3,
            priority: ToolPriority::High,
            status: ToolStatus::Available,
            version: "1.0.0".to_string(),
            author: "Alou".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            dependencies: vec![],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec!["network".to_string(), "web3".to_string()],
            tags: vec!["prediction-market".to_string(), "trading".to_string(), "polymarket".to_string()],
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            metadata,
            client: Arc::new(client),
            credentials: Arc::new(RwLock::new(None)),
        }
    }

    /// 检查 API 健康状态
    async fn check_health(&self) -> Result<Value, ToolError> {
        let url = format!("{}/ok", CLOB_API_URL);
        let resp = self.client.get(&url).send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to connect to Polymarket: {}", e)))?;

        if resp.status().is_success() {
            Ok(json!({ "status": "ok", "message": "Polymarket CLOB API is reachable" }))
        } else {
            Ok(json!({ "status": "error", "code": resp.status().as_u16() }))
        }
    }

    /// 获取服务器时间
    async fn get_server_time(&self) -> Result<Value, ToolError> {
        let url = format!("{}/time", CLOB_API_URL);
        let resp = self.client.get(&url).send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get server time: {}", e)))?;

        let time: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse time response: {}", e)))?;
        Ok(time)
    }

    /// 搜索市场（通过 Gamma API）
    async fn search_markets(&self, query: &str, limit: Option<u32>, active_only: Option<bool>) -> Result<Value, ToolError> {
        let limit = limit.unwrap_or(10);
        let active_only = active_only.unwrap_or(true);

        let url = format!("{}/events", GAMMA_API_URL);
        let limit_str = limit.to_string();
        let active_str = active_only.to_string();
        let resp = self.client.get(&url)
            .query(&[
                ("q", query),
                ("limit", &limit_str),
                ("active", &active_str),
            ])
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to search markets: {}", e)))?;

        let events: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse search results: {}", e)))?;

        // 精简输出，只保留关键字段
        if let Some(events_arr) = events.as_array() {
            let simplified: Vec<Value> = events_arr.iter().take(limit as usize).map(|event| {
                json!({
                    "id": event.get("id"),
                    "title": event.get("title"),
                    "slug": event.get("slug"),
                    "active": event.get("active"),
                    "closed": event.get("closed"),
                    "end_date_iso": event.get("end_date_iso"),
                    "markets": event.get("markets").and_then(|m| m.as_array()).map(|arr| {
                        arr.iter().map(|market| {
                            json!({
                                "id": market.get("id"),
                                "question": market.get("question"),
                                "outcome_prices": market.get("outcome_prices"),
                                "volume": market.get("volume"),
                                "liquidity": market.get("liquidity"),
                                "clob_token_ids": market.get("clob_token_ids"),
                                "tokens": market.get("tokens"),
                            })
                        }).collect::<Vec<_>>()
                    }),
                })
            }).collect();
            Ok(json!({ "query": query, "count": simplified.len(), "events": simplified }))
        } else {
            Ok(events)
        }
    }

    /// 获取市场列表
    async fn list_markets(&self, limit: Option<u32>, offset: Option<u32>, active_only: Option<bool>) -> Result<Value, ToolError> {
        let limit = limit.unwrap_or(20);
        let offset = offset.unwrap_or(0);
        let active_only = active_only.unwrap_or(true);

        let url = format!("{}/markets", GAMMA_API_URL);
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();
        let active_str = active_only.to_string();
        let order_str = "volume24hr".to_string();
        let asc_str = "false".to_string();
        let resp = self.client.get(&url)
            .query(&[
                ("limit", &limit_str),
                ("offset", &offset_str),
                ("active", &active_str),
                ("order", &order_str),
                ("ascending", &asc_str),
            ])
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to list markets: {}", e)))?;

        let markets: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse markets: {}", e)))?;

        // 精简输出
        if let Some(markets_arr) = markets.as_array() {
            let simplified: Vec<Value> = markets_arr.iter().map(|market| {
                json!({
                    "id": market.get("id"),
                    "question": market.get("question"),
                    "outcome_prices": market.get("outcome_prices"),
                    "outcomes": market.get("outcomes"),
                    "volume": market.get("volume"),
                    "volume24hr": market.get("volume24hr"),
                    "liquidity": market.get("liquidity"),
                    "active": market.get("active"),
                    "clob_token_ids": market.get("clob_token_ids"),
                    "end_date_iso": market.get("end_date_iso"),
                })
            }).collect();
            Ok(json!({ "count": simplified.len(), "markets": simplified }))
        } else {
            Ok(markets)
        }
    }

    /// 获取市场价格（中间价）
    async fn get_price(&self, token_id: &str) -> Result<Value, ToolError> {
        let url = format!("{}/midpoint", CLOB_API_URL);
        let resp = self.client.get(&url)
            .query(&[("token_id", token_id)])
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get price: {}", e)))?;

        let price: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse price: {}", e)))?;
        Ok(price)
    }

    /// 获取最后成交价
    async fn get_last_trade_price(&self, token_id: &str) -> Result<Value, ToolError> {
        let url = format!("{}/last-trade-price", CLOB_API_URL);
        let resp = self.client.get(&url)
            .query(&[("token_id", token_id)])
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get last trade price: {}", e)))?;

        let price: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse last trade price: {}", e)))?;
        Ok(price)
    }

    /// 获取订单簿
    async fn get_orderbook(&self, token_id: &str) -> Result<Value, ToolError> {
        let url = format!("{}/book", CLOB_API_URL);
        let resp = self.client.get(&url)
            .query(&[("token_id", token_id)])
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get orderbook: {}", e)))?;

        let book: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse orderbook: {}", e)))?;
        Ok(book)
    }

    /// 获取市场详情（通过 Gamma API）
    async fn get_market_details(&self, condition_id: &str) -> Result<Value, ToolError> {
        let url = format!("{}/markets/{}", GAMMA_API_URL, condition_id);
        let resp = self.client.get(&url).send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get market details: {}", e)))?;

        let market: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse market details: {}", e)))?;
        Ok(market)
    }

    /// 设置 API 认证凭证
    async fn set_credentials(&self, api_key: &str, api_secret: &str, api_passphrase: &str) -> Result<Value, ToolError> {
        let creds = PolymarketCredentials {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            api_passphrase: api_passphrase.to_string(),
        };
        *self.credentials.write().await = Some(creds);
        Ok(json!({ "success": true, "message": "Polymarket API credentials set successfully" }))
    }

    /// 检查是否已认证
    async fn is_authenticated(&self) -> bool {
        self.credentials.read().await.is_some()
    }

    /// 构建认证请求头
    async fn build_auth_headers(&self) -> Result<reqwest::header::HeaderMap, ToolError> {
        let creds = self.credentials.read().await;
        let creds = creds.as_ref().ok_or_else(|| {
            ToolError::PermissionDenied("Polymarket API credentials not set. Use 'set_credentials' action first.".to_string())
        })?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("POLY_API_KEY", creds.api_key.parse().unwrap());
        headers.insert("POLY_API_SECRET", creds.api_secret.parse().unwrap());
        headers.insert("POLY_PASSPHRASE", creds.api_passphrase.parse().unwrap());

        Ok(headers)
    }

    /// 获取 API Keys 列表（需认证）
    async fn get_api_keys(&self) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/api-keys", CLOB_API_URL);

        let resp = self.client.get(&url)
            .headers(headers)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get API keys: {}", e)))?;

        let keys: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse API keys: {}", e)))?;
        Ok(keys)
    }

    /// 获取持仓（需认证）
    async fn get_positions(&self) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/data/positions", CLOB_API_URL);

        let resp = self.client.get(&url)
            .headers(headers)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get positions: {}", e)))?;

        let positions: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse positions: {}", e)))?;
        Ok(positions)
    }

    /// 获取订单列表（需认证）
    async fn get_orders(&self) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/orders", CLOB_API_URL);

        let resp = self.client.get(&url)
            .headers(headers)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get orders: {}", e)))?;

        let orders: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse orders: {}", e)))?;
        Ok(orders)
    }

    /// 获取交易历史（需认证）
    async fn get_trades(&self) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/trades", CLOB_API_URL);

        let resp = self.client.get(&url)
            .headers(headers)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get trades: {}", e)))?;

        let trades: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse trades: {}", e)))?;
        Ok(trades)
    }

    /// 下单（需认证）- 限价单
    async fn place_limit_order(
        &self,
        token_id: &str,
        price: f64,
        size: f64,
        side: &str,
    ) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/order", CLOB_API_URL);

        let order_body = json!({
            "tokenID": token_id,
            "price": price,
            "size": size,
            "side": side.to_uppercase(),
            "type": "GTC",
        });

        let resp = self.client.post(&url)
            .headers(headers)
            .json(&order_body)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to place order: {}", e)))?;

        let result: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse order response: {}", e)))?;
        Ok(result)
    }

    /// 下单（需认证）- 市价单
    async fn place_market_order(
        &self,
        token_id: &str,
        amount: f64,
        side: &str,
    ) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/order", CLOB_API_URL);

        let order_body = json!({
            "tokenID": token_id,
            "amount": amount,
            "side": side.to_uppercase(),
            "type": "FOK",
        });

        let resp = self.client.post(&url)
            .headers(headers)
            .json(&order_body)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to place market order: {}", e)))?;

        let result: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse market order response: {}", e)))?;
        Ok(result)
    }

    /// 取消订单（需认证）
    async fn cancel_order(&self, order_id: &str) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/order/{}", CLOB_API_URL, order_id);

        let resp = self.client.delete(&url)
            .headers(headers)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to cancel order: {}", e)))?;

        let result: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse cancel response: {}", e)))?;
        Ok(result)
    }

    /// 取消所有订单（需认证）
    async fn cancel_all_orders(&self) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/cancel-all", CLOB_API_URL);

        let resp = self.client.delete(&url)
            .headers(headers)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to cancel all orders: {}", e)))?;

        let result: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse cancel-all response: {}", e)))?;
        Ok(result)
    }

    /// 获取余额（需认证）
    async fn get_balance(&self) -> Result<Value, ToolError> {
        let headers = self.build_auth_headers().await?;
        let url = format!("{}/balance", CLOB_API_URL);

        let resp = self.client.get(&url)
            .headers(headers)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get balance: {}", e)))?;

        let balance: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse balance: {}", e)))?;
        Ok(balance)
    }

    /// 检查地理限制
    async fn check_geoblock(&self) -> Result<Value, ToolError> {
        let url = format!("{}/geoblock", CLOB_API_URL);
        let resp = self.client.get(&url).send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to check geoblock: {}", e)))?;

        let result: Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse geoblock: {}", e)))?;
        Ok(result)
    }
}

impl Default for PolymarketTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for PolymarketTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let start = std::time::Instant::now();

        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field. Use 'help' action to see available actions.".to_string()))?;

        let result: Result<Value, ToolError> = match action {
            // ===== 只读操作（无需认证）=====
            "health" => self.check_health().await,
            "server_time" => self.get_server_time().await,
            "search" => {
                let query = args.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'query' parameter".to_string()))?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32);
                let active_only = args.get("active_only").and_then(|v| v.as_bool());
                self.search_markets(query, limit, active_only).await
            }
            "markets" => {
                let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32);
                let offset = args.get("offset").and_then(|v| v.as_u64()).map(|v| v as u32);
                let active_only = args.get("active_only").and_then(|v| v.as_bool());
                self.list_markets(limit, offset, active_only).await
            }
            "price" => {
                let token_id = args.get("token_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'token_id' parameter".to_string()))?;
                self.get_price(token_id).await
            }
            "last_price" => {
                let token_id = args.get("token_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'token_id' parameter".to_string()))?;
                self.get_last_trade_price(token_id).await
            }
            "orderbook" => {
                let token_id = args.get("token_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'token_id' parameter".to_string()))?;
                self.get_orderbook(token_id).await
            }
            "market_details" => {
                let condition_id = args.get("condition_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'condition_id' parameter".to_string()))?;
                self.get_market_details(condition_id).await
            }
            "geoblock" => self.check_geoblock().await,

            // ===== 认证操作 =====
            "set_credentials" => {
                let api_key = args.get("api_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'api_key' parameter".to_string()))?;
                let api_secret = args.get("api_secret")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'api_secret' parameter".to_string()))?;
                let api_passphrase = args.get("api_passphrase")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'api_passphrase' parameter".to_string()))?;
                self.set_credentials(api_key, api_secret, api_passphrase).await
            }
            "auth_status" => {
                let is_auth = self.is_authenticated().await;
                Ok(json!({ "authenticated": is_auth }))
            }
            "api_keys" => self.get_api_keys().await,

            // ===== 交易操作（需认证）=====
            "positions" => self.get_positions().await,
            "orders" => self.get_orders().await,
            "trades" => self.get_trades().await,
            "balance" => self.get_balance().await,

            "buy_limit" => {
                let token_id = args.get("token_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'token_id' parameter".to_string()))?;
                let price = args.get("price")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'price' parameter (0.01-0.99)".to_string()))?;
                let size = args.get("size")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'size' parameter (number of shares)".to_string()))?;
                self.place_limit_order(token_id, price, size, "BUY").await
            }
            "sell_limit" => {
                let token_id = args.get("token_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'token_id' parameter".to_string()))?;
                let price = args.get("price")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'price' parameter (0.01-0.99)".to_string()))?;
                let size = args.get("size")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'size' parameter (number of shares)".to_string()))?;
                self.place_limit_order(token_id, price, size, "SELL").await
            }
            "buy_market" => {
                let token_id = args.get("token_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'token_id' parameter".to_string()))?;
                let amount = args.get("amount")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'amount' parameter (USD amount)".to_string()))?;
                self.place_market_order(token_id, amount, "BUY").await
            }
            "sell_market" => {
                let token_id = args.get("token_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'token_id' parameter".to_string()))?;
                let amount = args.get("amount")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'amount' parameter (USD amount)".to_string()))?;
                self.place_market_order(token_id, amount, "SELL").await
            }
            "cancel_order" => {
                let order_id = args.get("order_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'order_id' parameter".to_string()))?;
                self.cancel_order(order_id).await
            }
            "cancel_all" => self.cancel_all_orders().await,

            "help" => {
                Ok(json!({
                    "tool": "polymarket",
                    "description": "Polymarket prediction market tool for AI agents",
                    "actions": {
                        "read_only": {
                            "health": "Check API health status",
                            "server_time": "Get CLOB server time",
                            "search": "Search markets by keyword (params: query, limit?, active_only?)",
                            "markets": "List markets sorted by volume (params: limit?, offset?, active_only?)",
                            "price": "Get midpoint price for a token (params: token_id)",
                            "last_price": "Get last trade price (params: token_id)",
                            "orderbook": "Get order book depth (params: token_id)",
                            "market_details": "Get full market details (params: condition_id)",
                            "geoblock": "Check if trading is available in your region"
                        },
                        "authentication": {
                            "set_credentials": "Set API credentials (params: api_key, api_secret, api_passphrase)",
                            "auth_status": "Check if credentials are set",
                            "api_keys": "List your API keys (requires auth)"
                        },
                        "trading": {
                            "positions": "Get your open positions (requires auth)",
                            "orders": "Get your open orders (requires auth)",
                            "trades": "Get your trade history (requires auth)",
                            "balance": "Get your account balance (requires auth)",
                            "buy_limit": "Place a limit buy order (params: token_id, price, size)",
                            "sell_limit": "Place a limit sell order (params: token_id, price, size)",
                            "buy_market": "Place a market buy order (params: token_id, amount)",
                            "sell_market": "Place a market sell order (params: token_id, amount)",
                            "cancel_order": "Cancel an order (params: order_id)",
                            "cancel_all": "Cancel all open orders"
                        }
                    },
                    "note": "Prices range from 0.01 to 0.99 (representing probability). Token IDs can be found via 'search' or 'markets' actions. Trading requires Polygon chain wallet + API credentials."
                }))
            }

            _ => Err(ToolError::InvalidArguments(format!(
                "Unknown action '{}'. Use 'help' to see available actions.", action
            ))),
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(data) => Ok(ToolResult {
                success: true,
                data,
                error: None,
                execution_time_ms,
                output: None,
                warnings: vec![],
                context: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                data: json!({}),
                error: Some(e.to_string()),
                execution_time_ms,
                output: None,
                warnings: vec![],
                context: None,
            }),
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Args must be an object".to_string()));
        }

        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        // 验证必要参数
        match action {
            "search" => {
                if args.get("query").is_none() {
                    return Err(ToolError::InvalidArguments("'search' requires 'query' parameter".to_string()));
                }
            }
            "price" | "last_price" | "orderbook" => {
                if args.get("token_id").is_none() {
                    return Err(ToolError::InvalidArguments(format!("'{}' requires 'token_id' parameter", action)));
                }
            }
            "market_details" => {
                if args.get("condition_id").is_none() {
                    return Err(ToolError::InvalidArguments("'market_details' requires 'condition_id' parameter".to_string()));
                }
            }
            "set_credentials" => {
                if args.get("api_key").is_none() || args.get("api_secret").is_none() || args.get("api_passphrase").is_none() {
                    return Err(ToolError::InvalidArguments("'set_credentials' requires api_key, api_secret, and api_passphrase".to_string()));
                }
            }
            "buy_limit" | "sell_limit" => {
                if args.get("token_id").is_none() || args.get("price").is_none() || args.get("size").is_none() {
                    return Err(ToolError::InvalidArguments(format!("'{}' requires token_id, price, and size parameters", action)));
                }
            }
            "buy_market" | "sell_market" => {
                if args.get("token_id").is_none() || args.get("amount").is_none() {
                    return Err(ToolError::InvalidArguments(format!("'{}' requires token_id and amount parameters", action)));
                }
            }
            "cancel_order" => {
                if args.get("order_id").is_none() {
                    return Err(ToolError::InvalidArguments("'cancel_order' requires 'order_id' parameter".to_string()));
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Polymarket Prediction Market Tool - Access prediction markets for AI agents

READ-ONLY ACTIONS (no auth required):
  health           - Check API health status
  server_time      - Get CLOB server time
  search           - Search markets by keyword
                     {"action":"search","query":"bitcoin","limit":10}
  markets          - List markets sorted by volume
                     {"action":"markets","limit":20,"active_only":true}
  price            - Get midpoint price for a token
                     {"action":"price","token_id":"<token_id>"}
  last_price       - Get last trade price
                     {"action":"last_price","token_id":"<token_id>"}
  orderbook        - Get order book depth
                     {"action":"orderbook","token_id":"<token_id>"}
  market_details   - Get full market details
                     {"action":"market_details","condition_id":"<id>"}
  geoblock         - Check trading availability in your region

AUTHENTICATION:
  set_credentials  - Set API credentials
                     {"action":"set_credentials","api_key":"...","api_secret":"...","api_passphrase":"..."}
  auth_status      - Check authentication status
  api_keys         - List your API keys (auth required)

TRADING ACTIONS (auth required):
  positions        - Get your open positions
  orders           - Get your open orders
  trades           - Get your trade history
  balance          - Get your account balance
  buy_limit        - Place limit buy order
                     {"action":"buy_limit","token_id":"...","price":0.65,"size":10}
  sell_limit       - Place limit sell order
                     {"action":"sell_limit","token_id":"...","price":0.35,"size":5}
  buy_market       - Place market buy order
                     {"action":"buy_market","token_id":"...","amount":25}
  sell_market      - Place market sell order
                     {"action":"sell_market","token_id":"...","amount":25}
  cancel_order     - Cancel a specific order
                     {"action":"cancel_order","order_id":"..."}
  cancel_all       - Cancel all open orders

NOTE: Prices range 0.01-0.99 (probability). Get token_ids via search/markets."#.to_string()
    }

    async fn is_available(&self) -> bool {
        // 检查 Polymarket API 是否可达
        self.check_health().await.is_ok()
    }
}
