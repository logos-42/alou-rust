use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CLOB_API_BASE: &str = "https://clob.polymarket.com";
const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";

/// Client for Polymarket CLOB (Central Limit Order Book) REST API.
/// This is the primary way to place and manage orders on Polymarket.
#[derive(Clone)]
pub struct ClobClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    api_secret: Option<String>,
    api_passphrase: Option<String>,
}

impl ClobClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: CLOB_API_BASE.to_string(),
            api_key: None,
            api_secret: None,
            api_passphrase: None,
        }
    }

    pub fn with_credentials(api_key: String, api_secret: String, api_passphrase: String) -> Self {
        Self {
            client: Client::new(),
            base_url: CLOB_API_BASE.to_string(),
            api_key: Some(api_key),
            api_secret: Some(api_secret),
            api_passphrase: Some(api_passphrase),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    fn auth_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        if let (Some(key), Some(secret), Some(passphrase)) =
            (&self.api_key, &self.api_secret, &self.api_passphrase)
        {
            headers.insert("POLY_API_KEY".to_string(), key.clone());
            headers.insert("POLY_API_SECRET".to_string(), secret.clone());
            headers.insert("POLY_PASSPHRASE".to_string(), passphrase.clone());
        }
        headers
    }

    /// Get the server's current time in epoch millis (for L1 signing).
    #[allow(dead_code)]
    pub async fn get_server_time(&self) -> Result<u64> {
        let resp = self
            .client
            .get(format!("{}/time", self.base_url))
            .send()
            .await?
            .error_for_status()?;
        let time: ServerTime = resp.json().await?;
        Ok(time.epoch_millis)
    }

    /// Get all markets with their token IDs and prices.
    pub async fn get_markets(&self) -> Result<Vec<ClobMarket>> {
        // Try Gamma API first for current markets
        match self.get_gamma_markets().await {
            Ok(markets) => {
                if !markets.is_empty() {
                    return Ok(markets);
                }
            }
            Err(e) => {
                eprintln!("Gamma API failed: {}. Falling back to CLOB API.", e);
            }
        }

        // Fallback to CLOB API
        let resp = self
            .client
            .get(format!("{}/markets", self.base_url))
            .send()
            .await?
            .error_for_status()?;
        let body: serde_json::Value = resp.json().await?;

        // API returns {"data": [...]}
        let markets: Vec<ClobMarket> = if body.is_array() {
            serde_json::from_value(body)?
        } else {
            let data = body.get("data").context("Response missing 'data' field")?;
            serde_json::from_value(data.clone())?
        };
        // Filter out archived markets and those with empty condition_id
        Ok(markets
            .into_iter()
            .filter(|m| !m.condition_id.is_empty() && m.archived != Some(true))
            .collect())
    }

    /// Get markets from Gamma API (returns current active markets)
    pub async fn get_gamma_markets(&self) -> Result<Vec<ClobMarket>> {
        let resp = self
            .client
            .get(format!("{}/markets", GAMMA_API_BASE))
            .send()
            .await?
            .error_for_status()?;
        let body: serde_json::Value = resp.json().await?;

        // Gamma API returns an array of markets
        let gamma_markets: Vec<GammaMarket> = if body.is_array() {
            serde_json::from_value(body)?
        } else {
            let data = body
                .get("data")
                .context("Gamma response missing 'data' field")?;
            serde_json::from_value(data.clone())?
        };

        // Convert Gamma markets to CLOB markets
        let mut markets = Vec::new();
        for gm in gamma_markets {
            // Parse outcomes, outcomePrices, and clobTokenIds from JSON strings
            let outcomes: Vec<String> = serde_json::from_str(&gm.outcomes).unwrap_or_default();
            let outcome_prices: Vec<f64> =
                serde_json::from_str(&gm.outcome_prices).unwrap_or_default();
            let clob_token_ids: Vec<String> =
                serde_json::from_str(&gm.clob_token_ids).unwrap_or_default();

            // Create tokens
            let mut tokens = Vec::new();
            for i in 0..outcomes.len() {
                let outcome = outcomes.get(i).cloned().unwrap_or_default();
                let price = outcome_prices.get(i).copied().unwrap_or(0.0);
                let token_id = clob_token_ids.get(i).cloned().unwrap_or_default();

                if !token_id.is_empty() {
                    tokens.push(MarketToken {
                        token_id,
                        outcome,
                        price,
                        winner: None,
                    });
                }
            }

            if !tokens.is_empty() {
                markets.push(ClobMarket {
                    condition_id: gm.condition_id,
                    question_id: None,
                    tokens,
                    question: Some(gm.question),
                    description: None,
                    market_slug: Some(gm.slug),
                    end_date_iso: None,
                    game_start_time: None,
                    active: Some(gm.active),
                    closed: None,
                    archived: Some(gm.archived),
                    accepting_orders: Some(gm.accepting_orders),
                    enable_order_book: None,
                    neg_risk: None,
                    tags: Some(gm.tags),
                });
            }
        }

        // Filter out archived markets and those with empty condition_id
        Ok(markets
            .into_iter()
            .filter(|m| !m.condition_id.is_empty() && m.archived != Some(true))
            .collect())
    }

    /// Get the order book for a specific token ID.
    pub async fn get_orderbook(&self, token_id: &str) -> Result<ClobOrderbook> {
        let resp = self
            .client
            .get(format!("{}/book", self.base_url))
            .query(&[("token_id", token_id)])
            .send()
            .await?
            .error_for_status()?;
        let book: ClobOrderbook = resp.json().await?;
        Ok(book)
    }

    /// Get open orders for the authenticated user.
    pub async fn get_orders(&self, market: &str) -> Result<Vec<ClobOrder>> {
        let headers = self.auth_headers();
        let mut req = self
            .client
            .get(format!("{}/orders", self.base_url))
            .query(&[("market", market)]);
        for (k, v) in &headers {
            req = req.header(k, v);
        }
        let resp = req.send().await?.error_for_status()?;
        let orders: Vec<ClobOrder> = resp.json().await?;
        Ok(orders)
    }

    /// Place a limit order. Returns the order ID on success.
    pub async fn place_order(&self, order: &PlaceOrderRequest) -> Result<String> {
        anyhow::ensure!(
            self.is_authenticated(),
            "CLOB client is not authenticated. Set API credentials."
        );

        let headers = self.auth_headers();
        let mut req = self
            .client
            .post(format!("{}/order", self.base_url))
            .json(order);
        for (k, v) in &headers {
            req = req.header(k, v);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            let msg = body["error"].as_str().unwrap_or("Unknown error");
            anyhow::bail!("Order failed ({}): {}", status, msg);
        }

        body["orderID"]
            .as_str()
            .map(|s| s.to_string())
            .context("No orderID in response")
    }

    /// Cancel an order by its ID.
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        anyhow::ensure!(self.is_authenticated(), "Not authenticated");
        let headers = self.auth_headers();
        let mut req = self
            .client
            .delete(format!("{}/orders/{}", self.base_url, order_id));
        for (k, v) in &headers {
            req = req.header(k, v);
        }
        req.send().await?.error_for_status()?;
        Ok(())
    }

    /// Cancel all orders for a market.
    pub async fn cancel_all_orders(&self, market: &str) -> Result<Vec<String>> {
        anyhow::ensure!(self.is_authenticated(), "Not authenticated");
        let headers = self.auth_headers();
        let mut req = self
            .client
            .delete(format!("{}/orders", self.base_url))
            .query(&[("market", market)]);
        for (k, v) in &headers {
            req = req.header(k, v);
        }
        let resp = req.send().await?.error_for_status()?;
        let body: serde_json::Value = resp.json().await?;
        let ids: Vec<String> = body
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v["orderID"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Ok(ids)
    }

    /// Get trades for a market.
    pub async fn get_trades(&self, market: &str) -> Result<Vec<ClobTrade>> {
        let resp = self
            .client
            .get(format!("{}/trades", self.base_url))
            .query(&[("market", market)])
            .send()
            .await?
            .error_for_status()?;
        let trades: Vec<ClobTrade> = resp.json().await?;
        Ok(trades)
    }
}

// ── Request / Response types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ServerTime {
    pub epoch_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobMarket {
    pub condition_id: String,
    pub question_id: Option<String>,
    pub tokens: Vec<MarketToken>,
    pub question: Option<String>,
    pub description: Option<String>,
    pub market_slug: Option<String>,
    pub end_date_iso: Option<String>,
    pub game_start_time: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub accepting_orders: Option<bool>,
    pub enable_order_book: Option<bool>,
    pub neg_risk: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketToken {
    pub token_id: String,
    pub outcome: String,
    pub price: f64,
    pub winner: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobOrderbook {
    pub bids: Vec<OrderbookEntry>,
    pub asks: Vec<OrderbookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookEntry {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobOrder {
    pub id: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub status: String,
    pub created_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobTrade {
    pub id: String,
    pub market: String,
    pub asset_id: String,
    pub price: f64,
    pub size: f64,
    pub side: String,
    pub match_time: Option<f64>,
    pub maker: Option<String>,
    pub taker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
    pub token_id: String,
    pub price: f64,
    pub size: f64,
    pub side: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    #[serde(rename = "feeRateBps")]
    pub fee_rate_bps: Option<u32>,
    pub nonce: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaMarket {
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    pub question: String,
    pub slug: String,
    pub active: bool,
    #[serde(rename = "acceptingOrders")]
    pub accepting_orders: bool,
    pub archived: bool,
    pub outcomes: String, // JSON string like "[\"Yes\",\"No\"]"
    #[serde(rename = "outcomePrices")]
    pub outcome_prices: String, // JSON string like "[0.535,0.465]"
    #[serde(rename = "clobTokenIds")]
    pub clob_token_ids: String, // JSON string like "[\"token1\",\"token2\"]"
    pub tags: Vec<String>,
}
