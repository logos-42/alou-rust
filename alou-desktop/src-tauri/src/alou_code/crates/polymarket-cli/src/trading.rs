use anyhow::{Context, Result};
use colored::*;

use crate::clob::{ClobClient, ClobMarket, ClobOrder, ClobOrderbook, PlaceOrderRequest};

/// Execute a trade via the CLOB API.
pub async fn place_trade(
    clob: &ClobClient,
    token_id: &str,
    outcome: &str,
    amount: f64,
    price: Option<f64>,
    side: &str,
) -> Result<String> {
    let book = clob
        .get_orderbook(token_id)
        .await
        .context("Failed to fetch orderbook")?;

    let effective_price = price.unwrap_or_else(|| match side {
        "buy" => {
            if !book.asks.is_empty() {
                book.asks[0].price
            } else {
                0.0
            }
        }
        _ => {
            if !book.bids.is_empty() {
                book.bids[0].price
            } else {
                0.0
            }
        }
    });

    if effective_price <= 0.0 {
        anyhow::bail!(
            "Cannot determine price for {} — {} side orderbook is empty",
            side,
            if side == "buy" { "ask" } else { "bid" }
        );
    }

    println!(
        "{} {} ${:.2} of '{}' @ ${:.4} ({} side)",
        "Placing order:".yellow(),
        side.to_uppercase(),
        amount,
        outcome,
        effective_price,
        side,
    );

    let order = PlaceOrderRequest {
        token_id: token_id.to_string(),
        price: effective_price,
        size: amount,
        side: side.to_string(),
        order_type: "GTC".to_string(),
        fee_rate_bps: None,
        nonce: None,
    };

    let order_id = clob
        .place_order(&order)
        .await
        .context("Failed to place order")?;

    println!("{} Order placed. ID: {}", "OK".green().bold(), order_id);
    Ok(order_id)
}

/// List open orders for a market.
pub async fn list_orders(clob: &ClobClient, market: &str) -> Result<Vec<ClobOrder>> {
    let orders = clob.get_orders(market).await?;
    Ok(orders)
}

/// Cancel an order by ID.
pub async fn cancel_order(clob: &ClobClient, order_id: &str) -> Result<()> {
    clob.cancel_order(order_id)
        .await
        .context("Failed to cancel order")?;
    println!("{} Order {} cancelled.", "OK".green().bold(), order_id);
    Ok(())
}

/// Cancel all open orders for a market.
pub async fn cancel_all(clob: &ClobClient, market: &str) -> Result<usize> {
    let ids = clob.cancel_all_orders(market).await?;
    let count = ids.len();
    println!(
        "{} Cancelled {} order(s) for market {}",
        "OK".green().bold(),
        count,
        market
    );
    Ok(count)
}

/// Print a summary of the orderbook.
pub fn print_orderbook_summary(market: &ClobMarket, book: &ClobOrderbook) {
    println!("{}", "Orderbook:".green().bold());
    println!(
        "  {}",
        market.question.as_deref().unwrap_or("(no question)")
    );
    println!(
        "{:>10} {:>10}   {:>10} {:>10}",
        "BID", "SIZE", "ASK", "SIZE"
    );
    println!("{}", "-".repeat(48));

    let max_depth = book.bids.len().max(book.asks.len());
    for i in 0..max_depth {
        let bid = book.bids.get(i);
        let ask = book.asks.get(i);

        let bid_str = bid.map(|b| format!("{:.4}", b.price)).unwrap_or_default();
        let bid_size = bid.map(|b| format!("{:.2}", b.size)).unwrap_or_default();
        let ask_str = ask.map(|a| format!("{:.4}", a.price)).unwrap_or_default();
        let ask_size = ask.map(|a| format!("{:.2}", a.size)).unwrap_or_default();

        println!(
            "{:>10} {:>10}   {:>10} {:>10}",
            bid_str, bid_size, ask_str, ask_size
        );
    }

    if !book.bids.is_empty() || !book.asks.is_empty() {
        let spread = if !book.asks.is_empty() && !book.bids.is_empty() {
            book.asks[0].price - book.bids[0].price
        } else {
            0.0
        };
        println!("  Spread: {:.4}", spread);
    }
}

/// Find the token ID for a given outcome name in a market.
pub fn find_token_id(market: &ClobMarket, outcome: &str) -> Option<String> {
    market
        .tokens
        .iter()
        .find(|t| t.outcome.eq_ignore_ascii_case(outcome))
        .map(|t| t.token_id.clone())
}

/// Print open orders summary.
pub fn print_orders_summary(orders: &[ClobOrder]) {
    if orders.is_empty() {
        println!("{}", "No open orders.".dimmed());
        return;
    }
    println!("{}", "Open Orders:".green().bold());
    println!(
        "{:<42} {:<8} {:>8} {:>8} {:>8}  {}",
        "ORDER ID", "SIDE", "PRICE", "SIZE", "STATUS", "TOKEN"
    );
    println!("{}", "-".repeat(90));
    for o in orders {
        println!(
            "{:<42} {:<8} {:>8.4} {:>8.2} {:>8}  {}",
            &o.id[..o.id.len().min(42)],
            o.side,
            o.price,
            o.size,
            o.status,
            &o.asset_id[..o.asset_id.len().min(16)],
        );
    }
}
