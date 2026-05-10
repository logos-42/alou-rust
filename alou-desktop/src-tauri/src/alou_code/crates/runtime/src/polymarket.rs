//! Polymarket CLOB API integration for runtime.
//!
//! This module provides Polymarket trading functionality for tools using the
//! polymarket-client-sdk.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

// Re-export SDK types for convenience
pub use polymarket_client_sdk::auth;
pub use polymarket_client_sdk::clob;
pub use polymarket_client_sdk::gamma;

// ── Polymarket execute functions (using official SDK) ──

use polymarket_client_sdk::auth::Signer as _; // for .with_chain_id()

/// Bridge synchronous tool calls to the async SDK
fn poly_block_on<F: std::future::Future>(f: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(f)),
        Err(_) => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(f)
        }
    }
}

/// Create an authenticated CLOB client. Returns human-readable error if credentials are missing.
pub fn poly_clob_auth_client() -> Result<
    polymarket_client_sdk::clob::Client<
        polymarket_client_sdk::auth::state::Authenticated<polymarket_client_sdk::auth::Normal>,
    >,
    String,
> {
    dotenv::dotenv().ok();
    let private_key =
        std::env::var("POLYMARKET_PRIVATE_KEY").or_else(|_| std::env::var("PRIVATE_KEY"));
    let private_key = private_key.map_err(|_| {
        "POLYMARKET_PRIVATE_KEY (or PRIVATE_KEY) not set. Add it to your .env file.".to_string()
    })?;

    let sig_type = match std::env::var("POLY_SIGNATURE_TYPE").as_deref() {
        Ok("0" | "EOA") => polymarket_client_sdk::clob::types::SignatureType::Eoa,
        Ok("1" | "PROXY") => polymarket_client_sdk::clob::types::SignatureType::Proxy,
        _ => polymarket_client_sdk::clob::types::SignatureType::GnosisSafe,
    };

    let signer = polymarket_client_sdk::auth::LocalSigner::from_str(&private_key)
        .map_err(|e| format!("Invalid private key: {e}"))?
        .with_chain_id(Some(polymarket_client_sdk::POLYGON));

    let builder = polymarket_client_sdk::clob::Client::new(
        "https://clob.polymarket.com",
        polymarket_client_sdk::clob::Config::default(),
    )
    .map_err(|e| format!("Failed to create CLOB client: {e}"))?
    .authentication_builder(&signer)
    .signature_type(sig_type);

    let builder = if let Ok(funder) = std::env::var("POLY_ADDRESS") {
        let addr = polymarket_client_sdk::types::Address::from_str(&funder)
            .map_err(|e| format!("Invalid POLY_ADDRESS: {e}"))?;
        builder.funder(addr)
    } else {
        builder
    };

    poly_block_on(builder.authenticate()).map_err(|e| {
        format!("Authentication failed. Check your private key and network. Error: {e}")
    })
}

pub fn parse_asset_id(s: &str) -> Result<polymarket_client_sdk::types::U256, String> {
    let clean = s.trim().trim_matches('"').trim_matches('\'');
    if clean.starts_with("0x") || clean.starts_with("0X") {
        let hex_str = &clean[2..];
        let val = u128::from_str_radix(hex_str, 16)
            .map_err(|e| format!("Invalid asset_id (hex): {}", e))?;
        Ok(polymarket_client_sdk::types::U256::from(val))
    } else {
        let val = clean
            .parse::<num_bigint::BigUint>()
            .map_err(|e| format!("Invalid asset_id (decimal): {}", e))?;
        let bytes = val.to_bytes_be();
        let mut padded = vec![0u8; 32usize.saturating_sub(bytes.len())];
        padded.extend_from_slice(&bytes);
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&padded[..32]);
        Ok(polymarket_client_sdk::types::U256::from_be_bytes(arr))
    }
}

// ── Input/Output structs ──

#[derive(Debug, Deserialize)]
pub struct PolymarketListMarketsInput {
    pub active_only: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct PolymarketGetOrderbookInput {
    pub token_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PolymarketPlaceOrderInput {
    pub token_id: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Deserialize)]
pub struct PolymarketGetOrdersInput {
    /// Filter by market condition ID (hex string, e.g. "0x...")
    pub market: Option<String>,
    /// Filter by asset/token ID (hex string)
    pub asset_id: Option<String>,
    /// Maximum number of orders to return per page (default 50)
    #[allow(dead_code)]
    pub limit: Option<u64>,
    /// Cursor for pagination (from previous response next_cursor)
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PolymarketGetTradesInput {
    /// Filter by market condition ID (hex string, e.g. "0x...")
    pub market: Option<String>,
    /// Filter by asset/token ID (hex string)
    pub asset_id: Option<String>,
    /// Filter by taker address (hex string)
    pub taker_address: Option<String>,
    /// Filter by maker address (hex string)
    pub maker_address: Option<String>,
    /// Filter trades before this Unix timestamp
    pub before: Option<i64>,
    /// Filter trades after this Unix timestamp
    pub after: Option<i64>,
    /// Maximum number of trades to return per page (default 50)
    #[allow(dead_code)]
    pub limit: Option<u64>,
    /// Cursor for pagination (from previous response next_cursor)
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PolymarketMarketSummary {
    pub condition_id: String,
    pub question: String,
    pub active: bool,
    pub accepting_orders: bool,
    pub outcomes: Vec<PolymarketOutcomeSummary>,
}

#[derive(Debug, Serialize)]
pub struct PolymarketOutcomeSummary {
    pub outcome: String,
    pub token_id: String,
    pub price: f64,
}

#[derive(Debug, Serialize)]
pub struct PolymarketOrderbookEntry {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Serialize)]
pub struct PolymarketOrderbookResult {
    pub token_id: String,
    pub bids: Vec<PolymarketOrderbookEntry>,
    pub asks: Vec<PolymarketOrderbookEntry>,
    pub spread: f64,
}

#[derive(Debug, Serialize)]
pub struct PolymarketWalletResult {
    pub address: String,
    pub connected: bool,
    pub matic_balance: Option<f64>,
    pub usdc_balance: Option<f64>,
    pub chain_usdc_balance: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct PolymarketOrderResult {
    pub success: bool,
    pub order_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PolymarketOrdersResult {
    pub orders: Vec<PolymarketOrderItem>,
    pub next_cursor: Option<String>,
    pub limit: u64,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct PolymarketOrderItem {
    pub id: String,
    pub status: String,
    pub owner: String,
    pub maker_address: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub original_size: String,
    pub size_matched: String,
    pub price: String,
    pub outcome: String,
    pub created_at: String,
    pub expiration: String,
    pub order_type: String,
}

#[derive(Debug, Serialize)]
pub struct PolymarketTradesResult {
    pub trades: Vec<PolymarketTradeItem>,
    pub next_cursor: Option<String>,
    pub limit: u64,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct PolymarketTradeItem {
    pub id: String,
    pub taker_order_id: String,
    pub maker_order_id: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub size: String,
    pub fee_rate_bps: String,
    pub price: String,
    pub taker_address: String,
    pub maker_address: String,
    pub created_at: String,
    pub trade_slippage_bps: Option<String>,
}

// ── Execute functions ──

pub fn execute_polymarket_list_markets(
    input: &PolymarketListMarketsInput,
) -> Result<Vec<PolymarketMarketSummary>, String> {
    let client = polymarket_client_sdk::gamma::Client::default();

    let request = polymarket_client_sdk::gamma::types::request::MarketsRequest::builder()
        .closed(false)
        .limit(input.limit.unwrap_or(20) as i32)
        .build();

    let markets = poly_block_on(client.markets(&request)).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for m in &markets {
        let condition_id = m
            .condition_id
            .map(|c| format!("{c:#x}"))
            .unwrap_or_default();
        if condition_id.is_empty() {
            continue;
        }

        let question = m
            .question
            .as_deref()
            .unwrap_or("(no question)")
            .chars()
            .take(120)
            .collect::<String>();

        let active = m.active.unwrap_or(false);
        let accepting = m.accepting_orders.unwrap_or(false);
        if input.active_only.unwrap_or(true) && (!active || !accepting) {
            continue;
        }

        let outcomes: Vec<PolymarketOutcomeSummary> = m
            .clob_token_ids
            .as_deref()
            .unwrap_or_default()
            .iter()
            .zip(m.outcome_prices.as_deref().unwrap_or_default().iter())
            .zip(m.outcomes.as_deref().unwrap_or_default().iter())
            .map(|((token_id, price), outcome)| PolymarketOutcomeSummary {
                outcome: outcome.clone(),
                token_id: format!("{token_id}"),
                price: price.to_string().parse::<f64>().unwrap_or(0.0),
            })
            .collect();

        results.push(PolymarketMarketSummary {
            condition_id,
            question,
            active,
            accepting_orders: accepting,
            outcomes,
        });
    }

    Ok(results)
}

pub fn execute_polymarket_get_orderbook(
    input: &PolymarketGetOrderbookInput,
) -> Result<PolymarketOrderbookResult, String> {
    // Clean token_id: strip surrounding quotes and whitespace if present
    let clean_token_id = input.token_id.trim().trim_matches('"').trim_matches('\'');

    // Parse token_id - Polymarket may return decimal or hex (0x...) format
    let token_id = if clean_token_id.starts_with("0x") || clean_token_id.starts_with("0X") {
        let hex_str = &clean_token_id[2..];
        let val = u128::from_str_radix(hex_str, 16)
            .map_err(|e| format!("Invalid token_id (hex): {}", e))?;
        polymarket_client_sdk::types::U256::from(val)
    } else {
        let val = clean_token_id
            .parse::<num_bigint::BigUint>()
            .map_err(|e| format!("Invalid token_id (decimal): {}", e))?;
        let bytes = val.to_bytes_be();
        let mut padded = vec![0u8; 32usize.saturating_sub(bytes.len())];
        padded.extend_from_slice(&bytes);
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&padded[..32]);
        polymarket_client_sdk::types::U256::from_be_bytes(arr)
    };

    let request = polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest::builder()
        .token_id(token_id)
        .build();

    let client = polymarket_client_sdk::clob::Client::default();
    let book = poly_block_on(client.order_book(&request)).map_err(|e| e.to_string())?;

    let bids: Vec<PolymarketOrderbookEntry> = book
        .bids
        .iter()
        .map(|b| PolymarketOrderbookEntry {
            price: b.price.to_string().parse::<f64>().unwrap_or(0.0),
            size: b.size.to_string().parse::<f64>().unwrap_or(0.0),
        })
        .collect();

    let asks: Vec<PolymarketOrderbookEntry> = book
        .asks
        .iter()
        .map(|a| PolymarketOrderbookEntry {
            price: a.price.to_string().parse::<f64>().unwrap_or(0.0),
            size: a.size.to_string().parse::<f64>().unwrap_or(0.0),
        })
        .collect();

    let spread = if !bids.is_empty() && !asks.is_empty() {
        asks[0].price - bids[0].price
    } else {
        0.0
    };

    Ok(PolymarketOrderbookResult {
        token_id: input.token_id.clone(),
        bids,
        asks,
        spread,
    })
}

pub fn execute_polymarket_wallet_balance() -> Result<PolymarketWalletResult, String> {
    dotenv::dotenv().ok();
    let private_key =
        std::env::var("POLYMARKET_PRIVATE_KEY").or_else(|_| std::env::var("PRIVATE_KEY"));

    let has_key = private_key.is_ok();

    if !has_key {
        return Ok(PolymarketWalletResult {
            address: String::new(),
            connected: false,
            matic_balance: None,
            usdc_balance: None,
            chain_usdc_balance: None,
        });
    }

    let private_key = private_key.unwrap();

    // Derive address from private key for on-chain queries
    let signer = polymarket_client_sdk::auth::LocalSigner::from_str(&private_key)
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let address = format!("0x{:x}", signer.address());

    // Get RPC URL for on-chain queries
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://polygon-mainnet.g.alchemy.com/v2/demo".to_string());

    // USDC.e (Bridged) contract on Polygon - Polymarket accepts ONLY USDC.e
    let usdc_address = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";

    // Query chain directly using alloy
    let chain_usdc = query_chain_usdc_balance(&rpc_url, usdc_address, &address);
    let chain_matic = query_chain_matic_balance(&rpc_url, &address);

    // Also get CLOB balance (funds deposited in Polymarket contracts)
    let client = match poly_clob_auth_client() {
        Ok(c) => c,
        Err(_) => {
            return Ok(PolymarketWalletResult {
                address,
                connected: false,
                matic_balance: chain_matic,
                usdc_balance: chain_usdc,
                chain_usdc_balance: chain_usdc,
            });
        }
    };

    // Refresh balance cache from on-chain data
    let update_req =
        polymarket_client_sdk::clob::types::request::UpdateBalanceAllowanceRequest::builder()
            .asset_type(polymarket_client_sdk::clob::types::AssetType::Collateral)
            .build();
    let _ = poly_block_on(client.update_balance_allowance(update_req));

    let balance_req =
        polymarket_client_sdk::clob::types::request::BalanceAllowanceRequest::builder()
            .asset_type(polymarket_client_sdk::clob::types::AssetType::Collateral)
            .build();

    let balance =
        poly_block_on(client.balance_allowance(balance_req)).map_err(|e| e.to_string())?;

    // Parse CLOB balance - SDK returns raw value with USDC decimals
    let clob_balance_str = balance.balance.to_string();
    let clob_balance: f64 = clob_balance_str.parse().unwrap_or(0.0);

    Ok(PolymarketWalletResult {
        address,
        connected: true,
        matic_balance: chain_matic,
        usdc_balance: Some(clob_balance), // CLOB balance
        chain_usdc_balance: chain_usdc,   // Direct chain balance
    })
}

/// Query USDC balance directly from Polygon chain using JSON-RPC
fn query_chain_usdc_balance(rpc_url: &str, usdc_contract: &str, wallet: &str) -> Option<f64> {
    let client = reqwest::blocking::Client::new();

    let wallet_hex = if wallet.starts_with("0x") {
        &wallet[2..]
    } else {
        wallet
    };
    let padded_wallet = format!("{:0>64}", wallet_hex);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{
            "to": usdc_contract,
            "data": format!("0x70a08231000000000000000000000000{}", padded_wallet)
        }, "latest"],
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .ok()?
        .json::<serde_json::Value>()
        .ok()?;

    let result = response.get("result")?.as_str()?;
    let balance_hex = result.strip_prefix("0x")?;
    let balance_u128 = u128::from_str_radix(balance_hex, 16).ok()?;

    // USDC has 6 decimals
    let whole = balance_u128 / 1_000_000;
    let fraction = balance_u128 % 1_000_000;
    let result_str = format!("{}.{:06}", whole, fraction);
    result_str.parse::<f64>().ok()
}

/// Query MATIC balance directly from Polygon chain using JSON-RPC
fn query_chain_matic_balance(rpc_url: &str, wallet: &str) -> Option<f64> {
    let client = reqwest::blocking::Client::new();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": [wallet, "latest"],
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .ok()?
        .json::<serde_json::Value>()
        .ok()?;

    let result = response.get("result")?.as_str()?;
    let balance_hex = result.strip_prefix("0x")?;
    let balance_u128 = u128::from_str_radix(balance_hex, 16).ok()?;

    // MATIC has 18 decimals
    let whole = balance_u128 / 1_000_000_000_000_000_000u128;
    let fraction = balance_u128 % 1_000_000_000_000_000_000u128;
    let result_str = format!("{}.{:018}", whole, fraction);
    result_str.parse::<f64>().ok()
}

pub fn execute_polymarket_place_order(
    input: &PolymarketPlaceOrderInput,
) -> Result<PolymarketOrderResult, String> {
    let client = match poly_clob_auth_client() {
        Ok(c) => c,
        Err(e) => {
            return Ok(PolymarketOrderResult {
                success: false,
                order_id: None,
                message: e,
            });
        }
    };

    let token_id_str = input.token_id.trim().trim_matches('"').trim_matches('\'');
    let token_id = if token_id_str.starts_with("0x") || token_id_str.starts_with("0X") {
        let hex_str = &token_id_str[2..];
        let val = u128::from_str_radix(hex_str, 16)
            .map_err(|e| format!("Invalid token_id (hex): {}", e))?;
        polymarket_client_sdk::types::U256::from(val)
    } else {
        let val = token_id_str
            .parse::<num_bigint::BigUint>()
            .map_err(|e| format!("Invalid token_id (decimal): {}", e))?;
        let bytes = val.to_bytes_be();
        let mut padded = vec![0u8; 32usize.saturating_sub(bytes.len())];
        padded.extend_from_slice(&bytes);
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&padded[..32]);
        polymarket_client_sdk::types::U256::from_be_bytes(arr)
    };

    let side = match input.side.to_uppercase().as_str() {
        "BUY" => polymarket_client_sdk::clob::types::Side::Buy,
        _ => polymarket_client_sdk::clob::types::Side::Sell,
    };

    let price = polymarket_client_sdk::types::Decimal::from_str(&input.price.to_string())
        .map_err(|e| format!("Invalid price: {e}"))?;
    let size_val = polymarket_client_sdk::types::Decimal::from_str(&input.size.to_string())
        .map_err(|e| format!("Invalid size: {e}"))?;

    // Refresh balance and allowance cache from chain before placing order
    let update_req =
        polymarket_client_sdk::clob::types::request::UpdateBalanceAllowanceRequest::builder()
            .asset_type(polymarket_client_sdk::clob::types::AssetType::Collateral)
            .build();
    let _ = poly_block_on(client.update_balance_allowance(update_req));

    // Build the order (async, fetches tick_size + fee_rate from server)
    let signable_order = poly_block_on(
        client
            .limit_order()
            .token_id(token_id)
            .price(price)
            .size(size_val)
            .side(side)
            .build(),
    )
    .map_err(|e| format!("Failed to build order: {e}"))?;

    // Sign with private key
    let private_key = std::env::var("POLYMARKET_PRIVATE_KEY")
        .or_else(|_| std::env::var("PRIVATE_KEY"))
        .unwrap();
    let signer = polymarket_client_sdk::auth::LocalSigner::from_str(&private_key)
        .map_err(|e| format!("Invalid private key for signing: {e}"))?
        .with_chain_id(Some(polymarket_client_sdk::POLYGON));

    let signed_order = poly_block_on(client.sign(&signer, signable_order))
        .map_err(|e| format!("Failed to sign order: {e}"))?;

    let response = poly_block_on(client.post_order(signed_order))
        .map_err(|e| format!("Failed to post order: {e}"))?;

    if response.success {
        Ok(PolymarketOrderResult {
            success: true,
            order_id: Some(response.order_id),
            message: format!("Order placed successfully. Status: {:?}", response.status),
        })
    } else {
        Ok(PolymarketOrderResult {
            success: false,
            order_id: Some(response.order_id),
            message: response
                .error_msg
                .unwrap_or_else(|| format!("Order failed. Status: {:?}", response.status)),
        })
    }
}

pub fn execute_polymarket_get_orders(
    input: &PolymarketGetOrdersInput,
) -> Result<PolymarketOrdersResult, String> {
    let client = poly_clob_auth_client()?;

    let market = input.market.as_ref().map(|m| {
        let clean = m.trim().trim_matches('"').trim_matches('\'');
        clean
            .parse::<polymarket_client_sdk::types::B256>()
            .map_err(|e| format!("Invalid market: {}", e))
    });

    let asset_id = input.asset_id.as_ref().map(|a| parse_asset_id(a));

    // Handle optional fields using typestate builder pattern
    let request = match (market, asset_id) {
        (Some(Ok(m)), Some(Ok(a))) => {
            polymarket_client_sdk::clob::types::request::OrdersRequest::builder()
                .market(m)
                .asset_id(a)
                .build()
        }
        (Some(Ok(m)), None) => {
            polymarket_client_sdk::clob::types::request::OrdersRequest::builder()
                .market(m)
                .build()
        }
        (None, Some(Ok(a))) => {
            polymarket_client_sdk::clob::types::request::OrdersRequest::builder()
                .asset_id(a)
                .build()
        }
        _ => polymarket_client_sdk::clob::types::request::OrdersRequest::builder().build(),
    };

    let response = poly_block_on(client.orders(&request, input.next_cursor.clone()))
        .map_err(|e| format!("Failed to get orders: {}", e))?;

    let orders: Vec<PolymarketOrderItem> = response
        .data
        .into_iter()
        .map(|o| PolymarketOrderItem {
            id: o.id,
            status: format!("{:?}", o.status),
            owner: format!("{}", o.owner),
            maker_address: format!("{}", o.maker_address),
            market: format!("{:#x}", o.market),
            asset_id: format!("{}", o.asset_id),
            side: format!("{:?}", o.side),
            original_size: o.original_size.to_string(),
            size_matched: o.size_matched.to_string(),
            price: o.price.to_string(),
            outcome: o.outcome,
            created_at: format!("{}", o.created_at.timestamp()),
            expiration: format!("{}", o.expiration.timestamp()),
            order_type: format!("{:?}", o.order_type),
        })
        .collect();

    Ok(PolymarketOrdersResult {
        orders,
        next_cursor: if response.next_cursor.is_empty() {
            None
        } else {
            Some(response.next_cursor)
        },
        limit: response.limit,
        count: response.count as usize,
    })
}

pub fn execute_polymarket_get_trades(
    input: &PolymarketGetTradesInput,
) -> Result<PolymarketTradesResult, String> {
    let client = poly_clob_auth_client()?;

    let market = input.market.as_ref().map(|m| {
        let clean = m.trim().trim_matches('"').trim_matches('\'');
        clean
            .parse::<polymarket_client_sdk::types::B256>()
            .map_err(|e| format!("Invalid market: {}", e))
    });

    let asset_id = input.asset_id.as_ref().map(|a| parse_asset_id(a));

    let taker_address = input.taker_address.as_ref().map(|t| {
        let clean = t.trim().trim_matches('"').trim_matches('\'');
        polymarket_client_sdk::types::Address::from_str(clean)
            .map_err(|e| format!("Invalid taker_address: {}", e))
    });

    let maker_address = input.maker_address.as_ref().map(|m| {
        let clean = m.trim().trim_matches('"').trim_matches('\'');
        polymarket_client_sdk::types::Address::from_str(clean)
            .map_err(|e| format!("Invalid maker_address: {}", e))
    });

    // Build trades request with optional fields using conditional construction
    let request = match (market, asset_id, taker_address, maker_address) {
        (Some(Ok(m)), Some(Ok(a)), Some(Ok(t)), Some(Ok(ma))) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .market(m)
                .asset_id(a)
                .taker_address(t)
                .maker_address(ma)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (Some(Ok(m)), Some(Ok(a)), Some(Ok(t)), None) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .market(m)
                .asset_id(a)
                .taker_address(t)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (Some(Ok(m)), Some(Ok(a)), None, Some(Ok(ma))) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .market(m)
                .asset_id(a)
                .maker_address(ma)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (Some(Ok(m)), Some(Ok(a)), None, None) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .market(m)
                .asset_id(a)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (Some(Ok(m)), None, Some(Ok(t)), Some(Ok(ma))) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .market(m)
                .taker_address(t)
                .maker_address(ma)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (Some(Ok(m)), None, Some(Ok(t)), None) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .market(m)
                .taker_address(t)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (Some(Ok(m)), None, None, Some(Ok(ma))) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .market(m)
                .maker_address(ma)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (Some(Ok(m)), None, None, None) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .market(m)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (None, Some(Ok(a)), Some(Ok(t)), Some(Ok(ma))) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .asset_id(a)
                .taker_address(t)
                .maker_address(ma)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (None, Some(Ok(a)), Some(Ok(t)), None) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .asset_id(a)
                .taker_address(t)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (None, Some(Ok(a)), None, Some(Ok(ma))) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .asset_id(a)
                .maker_address(ma)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (None, Some(Ok(a)), None, None) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .asset_id(a)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (None, None, Some(Ok(t)), Some(Ok(ma))) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .taker_address(t)
                .maker_address(ma)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (None, None, Some(Ok(t)), None) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .taker_address(t)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        (None, None, None, Some(Ok(ma))) => {
            polymarket_client_sdk::clob::types::request::TradesRequest::builder()
                .maker_address(ma)
                .before(input.before.unwrap_or_default())
                .after(input.after.unwrap_or_default())
                .build()
        }
        _ => polymarket_client_sdk::clob::types::request::TradesRequest::builder()
            .before(input.before.unwrap_or_default())
            .after(input.after.unwrap_or_default())
            .build(),
    };

    let response = poly_block_on(client.trades(&request, input.next_cursor.clone()))
        .map_err(|e| format!("Failed to get trades: {}", e))?;

    let trades: Vec<PolymarketTradeItem> = response
        .data
        .into_iter()
        .map(|t| PolymarketTradeItem {
            id: t.id,
            taker_order_id: t.taker_order_id,
            maker_order_id: t
                .maker_orders
                .first()
                .map(|m| m.order_id.clone())
                .unwrap_or_default(),
            market: format!("{:#x}", t.market),
            asset_id: format!("{}", t.asset_id),
            side: format!("{:?}", t.side),
            size: t.size.to_string(),
            fee_rate_bps: t.fee_rate_bps.to_string(),
            price: t.price.to_string(),
            taker_address: format!("{}", t.maker_address),
            maker_address: format!("{}", t.maker_address),
            created_at: format!("{}", t.match_time.timestamp()),
            trade_slippage_bps: None,
        })
        .collect();

    Ok(PolymarketTradesResult {
        trades,
        next_cursor: if response.next_cursor.is_empty() {
            None
        } else {
            Some(response.next_cursor)
        },
        limit: response.limit,
        count: response.count as usize,
    })
}
