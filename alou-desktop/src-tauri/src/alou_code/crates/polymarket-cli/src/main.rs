use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

use anyhow::Context;

mod analysis;
mod clob;
mod trading;
mod wallet;

use crate::clob::ClobClient;
use crate::wallet::WalletManager;

#[derive(Parser)]
#[command(name = "polymarket-cli")]
#[command(about = "CLI tool for Polymarket trading and analysis", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[arg(long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List available markets
    List {
        #[arg(short, long)]
        category: Option<String>,

        #[arg(short, long, default_value = "20")]
        limit: u32,

        #[arg(short, long)]
        active_only: bool,
    },

    /// Show market details and orderbook
    Market {
        /// Market condition ID
        #[arg(required = true)]
        market_id: String,

        /// Show the orderbook for a specific token ID
        #[arg(short, long)]
        token_id: Option<String>,
    },

    /// Place a trade order
    Trade {
        #[arg(required = true)]
        market_id: String,

        #[arg(required = true)]
        outcome: String,

        #[arg(required = true)]
        amount: f64,

        #[arg(short, long)]
        price: Option<f64>,

        #[arg(short, long, default_value = "buy")]
        side: String,
    },

    /// Show open orders
    Orders {
        #[arg(required = true)]
        market_id: String,
    },

    /// Cancel orders
    Cancel {
        /// Cancel a specific order ID
        #[arg(short, long)]
        order_id: Option<String>,

        /// Cancel all orders for a market
        #[arg(short, long)]
        market: Option<String>,
    },

    /// Wallet operations
    Wallet {
        #[command(subcommand)]
        command: Option<WalletCommands>,
    },

    /// Analyze market data
    Analyze {
        #[arg(required = true)]
        market_id: String,
    },

    /// Automated trading strategies
    Bot {
        #[command(subcommand)]
        command: BotCommands,
    },

    /// Monitor markets and prices
    Monitor {
        /// Market condition IDs to watch (comma separated)
        #[arg(short, long, default_value = "")]
        markets: String,

        /// Watch mode: continuously poll
        #[arg(short, long)]
        watch: bool,

        /// Polling interval in seconds
        #[arg(short, long, default_value = "30")]
        interval: u64,
    },
}

#[derive(Subcommand)]
enum WalletCommands {
    /// Show wallet balance (MATIC + USDC)
    Balance,
    /// Connect wallet with private key or mnemonic
    Connect {
        #[arg(short, long)]
        private_key: Option<String>,
        #[arg(short, long)]
        mnemonic: Option<String>,
    },
}

#[derive(Subcommand)]
enum BotCommands {
    /// Market making: place buy and sell orders around mid price
    MarketMaker {
        #[arg(required = true)]
        market_id: String,

        /// Outcome to make market on
        #[arg(short, long)]
        outcome: String,

        #[arg(short, long, default_value = "0.02")]
        spread: f64,

        #[arg(short, long, default_value = "10.0")]
        size: f64,

        /// Auto-rebalance interval in seconds
        #[arg(short, long, default_value = "60")]
        rebalance_interval: u64,
    },
}

fn load_clob_client() -> ClobClient {
    let key = std::env::var("POLY_API_KEY").ok();
    let secret = std::env::var("POLY_API_SECRET").ok();
    let passphrase = std::env::var("POLY_PASSPHRASE").ok();
    match (key, secret, passphrase) {
        (Some(k), Some(s), Some(p)) => ClobClient::with_credentials(k, s, p),
        _ => ClobClient::new(),
    }
}

fn load_wallet() -> WalletManager {
    let rpc = std::env::var("RPC_URL").unwrap_or_else(|_| "https://polygon.llamarpc.com".into());
    let mut wm = WalletManager::new(&rpc).unwrap_or_else(|e| {
        eprintln!("Warning: cannot create wallet manager: {e}");
        WalletManager::new("https://polygon.llamarpc.com").unwrap()
    });
    if let Ok(pk) = std::env::var("PRIVATE_KEY") {
        println!("🔑 PRIVATE_KEY found, length: {}", pk.len());
        println!("   First 10 chars: {}...", &pk[..10.min(pk.len())]);
        match wm.connect_with_private_key(&pk) {
            Ok(_) => {
                println!("✅ Wallet connected successfully");
                if let Some(addr) = wm.address() {
                    println!("   Address: {:?}", addr);
                }
            }
            Err(e) => eprintln!("❌ Failed to connect wallet: {e}"),
        }
    } else {
        eprintln!("⚠️  PRIVATE_KEY environment variable not set");
    }
    wm
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();
    let clob = load_clob_client();
    let wallet = load_wallet();

    match cli.command {
        // ── List markets ──
        Commands::List {
            category: _,
            limit,
            active_only,
        } => {
            println!("{}", "Fetching markets...".dimmed());
            let markets = clob.get_markets().await?;

            let mut shown = 0u32;
            for m in &markets {
                if active_only {
                    let active = m.active.unwrap_or(false);
                    let accepting = m.accepting_orders.unwrap_or(false);
                    if !active || !accepting {
                        continue;
                    }
                }
                if shown >= limit {
                    break;
                }
                let question = m
                    .question
                    .as_deref()
                    .unwrap_or("(no question)")
                    .chars()
                    .take(70)
                    .collect::<String>();
                let price_str = m
                    .tokens
                    .first()
                    .map(|t| format!("{:.2}¢", t.price * 100.0))
                    .unwrap_or_else(|| "N/A".into());
                let active_label = if m.active.unwrap_or(false) {
                    "✓".green()
                } else {
                    "✗".red()
                };
                println!(
                    "{active_label} {:<10} {:<72} {}",
                    &m.condition_id[..m.condition_id.len().min(10)],
                    question,
                    price_str,
                );
                shown += 1;
            }
            println!("\nShowing {}/{} markets", shown, markets.len());
        }

        // ── Market details + orderbook ──
        Commands::Market {
            market_id,
            token_id,
        } => {
            let markets = clob.get_markets().await?;
            let market = markets
                .iter()
                .find(|m| m.condition_id == market_id)
                .context("Market not found")?;

            println!("{}", "Market Details:".green().bold());
            println!("  Condition ID: {}", market.condition_id);
            println!(
                "  Question:     {}",
                market.question.as_deref().unwrap_or("N/A")
            );
            println!(
                "  Active:       {}",
                if market.active.unwrap_or(false) {
                    "Yes".green()
                } else {
                    "No".red()
                }
            );
            if let Some(end) = &market.end_date_iso {
                println!("  End Date:     {}", end);
            }

            println!("\n{}", "Outcomes:".yellow().bold());
            for token in &market.tokens {
                println!(
                    "  {} ({}): ${:.4}",
                    token.outcome,
                    &token.token_id[..token.token_id.len().min(16)],
                    token.price,
                );
            }

            // Show orderbook for the first or specified token
            let tid = token_id.unwrap_or_else(|| {
                market
                    .tokens
                    .first()
                    .map(|t| t.token_id.clone())
                    .unwrap_or_default()
            });
            if !tid.is_empty() {
                let book = clob.get_orderbook(&tid).await?;
                trading::print_orderbook_summary(market, &book);
            }
        }

        // ── Trade ──
        Commands::Trade {
            market_id,
            outcome,
            amount,
            price,
            side,
        } => {
            let markets = clob.get_markets().await?;
            let market = markets
                .iter()
                .find(|m| m.condition_id == market_id)
                .context("Market not found")?;

            let token_id = trading::find_token_id(market, &outcome).context(format!(
                "Outcome '{}' not found. Available: {}",
                outcome,
                market
                    .tokens
                    .iter()
                    .map(|t| t.outcome.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            ))?;

            trading::place_trade(&clob, &token_id, &outcome, amount, price, &side).await?;
        }

        // ── Orders ──
        Commands::Orders { market_id } => {
            let orders = trading::list_orders(&clob, &market_id).await?;
            trading::print_orders_summary(&orders);
        }

        // ── Cancel ──
        Commands::Cancel { order_id, market } => {
            if let Some(id) = order_id {
                trading::cancel_order(&clob, &id).await?;
            } else if let Some(market_id) = market {
                let count = trading::cancel_all(&clob, &market_id).await?;
                if count == 0 {
                    println!("{}", "No orders to cancel.".dimmed());
                }
            } else {
                eprintln!("Provide --order-id or --market");
            }
        }

        // ── Wallet ──
        Commands::Wallet { command } => match command {
            Some(WalletCommands::Balance) => {
                wallet::print_balances(&wallet).await?;
            }
            Some(WalletCommands::Connect {
                private_key,
                mnemonic,
            }) => {
                let mut wm = wallet;
                if let Some(pk) = private_key {
                    wm.connect_with_private_key(&pk)?;
                    println!("{} Connected with private key", "OK".green().bold());
                } else if let Some(mn) = mnemonic {
                    wm.connect_with_mnemonic(&mn, 0)?;
                    println!("{} Connected with mnemonic", "OK".green().bold());
                } else {
                    eprintln!("Provide --private-key or --mnemonic");
                    return Ok(());
                }
                wallet::print_balances(&wm).await?;
            }
            None => {
                wallet::print_wallet_status(&wallet);
                println!("\nSub-commands:");
                println!("  balance   - Show MATIC + USDC balance");
                println!("  connect   - Connect wallet (private-key or mnemonic)");
            }
        },

        // ── Analyze ──
        Commands::Analyze { market_id } => {
            let analyzer = analysis::MarketAnalyzer::new(clob.clone());

            match analyzer.analyze_market(&market_id).await {
                Ok(analysis) => {
                    println!("{}", "Market Analysis:".green().bold());
                    println!("Question: {}", analysis.market_question);

                    println!("\n{}", "Volume Analysis:".yellow().bold());
                    println!(
                        "  Total Volume:   ${:.2}",
                        analysis.volume_analysis.total_volume
                    );
                    println!(
                        "  Buy Volume:     ${:.2}",
                        analysis.volume_analysis.buy_volume
                    );
                    println!(
                        "  Sell Volume:    ${:.2}",
                        analysis.volume_analysis.sell_volume
                    );
                    println!(
                        "  Volume Ratio:   {:.2}",
                        analysis.volume_analysis.volume_ratio
                    );

                    println!("\n{}", "Price Analysis:".yellow().bold());
                    println!(
                        "  Current Price:  ${:.4}",
                        analysis.price_analysis.current_price
                    );
                    println!(
                        "  Price Change:   {:.2}%",
                        analysis.price_analysis.price_change
                    );
                    println!(
                        "  Volatility:     {:.4}",
                        analysis.price_analysis.volatility
                    );
                    if !analysis.price_analysis.support_levels.is_empty() {
                        println!(
                            "  Supports:       {:?}",
                            analysis.price_analysis.support_levels
                        );
                    }
                    if !analysis.price_analysis.resistance_levels.is_empty() {
                        println!(
                            "  Resistances:    {:?}",
                            analysis.price_analysis.resistance_levels
                        );
                    }

                    println!("\n{}", "Liquidity:".yellow().bold());
                    println!(
                        "  Bid Liquidity:  ${:.2}",
                        analysis.liquidity_analysis.bid_liquidity
                    );
                    println!(
                        "  Ask Liquidity:  ${:.2}",
                        analysis.liquidity_analysis.ask_liquidity
                    );
                    println!(
                        "  Spread:         {:.4}",
                        analysis.liquidity_analysis.spread
                    );
                    println!(
                        "  Depth Ratio:    {:.2}",
                        analysis.liquidity_analysis.depth_ratio
                    );

                    println!("\n{}", "Sentiment:".yellow().bold());
                    println!(
                        "  Score:          {:.2}",
                        analysis.sentiment_analysis.sentiment_score
                    );
                    println!(
                        "  Overall:        {}",
                        analysis.sentiment_analysis.overall_sentiment
                    );

                    let recs = analyzer.generate_recommendations(&analysis);
                    if !recs.is_empty() {
                        println!("\n{}", "Recommendations:".green().bold());
                        for r in &recs {
                            println!(
                                "  {} ({:.0}%) — {}",
                                r.action,
                                r.confidence * 100.0,
                                r.reason
                            );
                        }
                    }
                }
                Err(e) => eprintln!("Analysis failed: {e}"),
            }
        }

        // ── Bot ──
        Commands::Bot {
            command:
                BotCommands::MarketMaker {
                    market_id,
                    outcome,
                    spread,
                    size,
                    rebalance_interval,
                },
        } => {
            run_market_maker(
                &clob,
                &market_id,
                &outcome,
                spread,
                size,
                rebalance_interval,
            )
            .await?;
        }

        // ── Monitor ──
        Commands::Monitor {
            markets,
            watch,
            interval,
        } => {
            run_monitor(&clob, &markets, watch, interval).await?;
        }
    }

    Ok(())
}

// ── Market Maker Bot ──

async fn run_market_maker(
    clob: &ClobClient,
    market_id: &str,
    outcome: &str,
    spread: f64,
    size: f64,
    interval: u64,
) -> anyhow::Result<()> {
    if !clob.is_authenticated() {
        anyhow::bail!(
            "Market maker requires authentication. Set POLY_API_KEY, POLY_API_SECRET, POLY_PASSPHRASE."
        );
    }

    let markets = clob.get_markets().await?;
    let market = markets
        .iter()
        .find(|m| m.condition_id == market_id)
        .context("Market not found")?;
    let token_id =
        trading::find_token_id(market, outcome).context("Outcome not found in market")?;

    println!(
        "{} Market maker started on '{}' ({})",
        "OK".green().bold(),
        market.question.as_deref().unwrap_or("?"),
        outcome,
    );
    println!(
        "  Spread: {:.2}%, Size: ${:.2}, Rebalance: {}s",
        spread * 100.0,
        size,
        interval
    );
    println!("  Press Ctrl+C to stop.\n");

    loop {
        match clob.get_orderbook(&token_id).await {
            Ok(book) => {
                let mid = if !book.bids.is_empty() && !book.asks.is_empty() {
                    (book.bids[0].price + book.asks[0].price) / 2.0
                } else {
                    continue;
                };

                let bid_price = (mid - spread / 2.0).max(0.01);
                let ask_price = (mid + spread / 2.0).min(0.99);
                let now = chrono::Utc::now().format("%H:%M:%S");

                println!(
                    "[{}] mid={:.4}  bid={:.4}  ask={:.4}",
                    now, mid, bid_price, ask_price
                );

                // Cancel existing orders for this market
                if let Err(e) = trading::cancel_all(clob, market_id).await {
                    eprintln!("  Cancel failed: {e}");
                }

                // Place new bid
                let buy_order = PlaceOrderRequest {
                    token_id: token_id.clone(),
                    price: bid_price,
                    size,
                    side: "BUY".to_string(),
                    order_type: "GTC".to_string(),
                    fee_rate_bps: None,
                    nonce: None,
                };
                if let Err(e) = clob.place_order(&buy_order).await {
                    eprintln!("  Buy order failed: {e}");
                }

                // Place new ask
                let sell_order = PlaceOrderRequest {
                    token_id: token_id.clone(),
                    price: ask_price,
                    size,
                    side: "SELL".to_string(),
                    order_type: "GTC".to_string(),
                    fee_rate_bps: None,
                    nonce: None,
                };
                if let Err(e) = clob.place_order(&sell_order).await {
                    eprintln!("  Sell order failed: {e}");
                }
            }
            Err(e) => {
                eprintln!("  Orderbook fetch failed: {e}");
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

use crate::clob::PlaceOrderRequest;

// ── Monitor ──

async fn run_monitor(
    clob: &ClobClient,
    market_ids_csv: &str,
    watch: bool,
    interval: u64,
) -> anyhow::Result<()> {
    let market_ids: Vec<String> = if market_ids_csv.is_empty() {
        // If no markets specified, fetch active markets and show top 5
        let markets = clob.get_markets().await?;
        markets
            .iter()
            .filter(|m| m.active.unwrap_or(false) && m.accepting_orders.unwrap_or(false))
            .take(5)
            .map(|m| m.condition_id.clone())
            .collect()
    } else {
        market_ids_csv
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    };

    let all_markets = clob.get_markets().await?;

    loop {
        println!("{}", "─".repeat(80).dimmed());
        println!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));

        for mid in &market_ids {
            if let Some(market) = all_markets.iter().find(|m| m.condition_id == *mid) {
                let question = market
                    .question
                    .as_deref()
                    .unwrap_or("?")
                    .chars()
                    .take(55)
                    .collect::<String>();

                println!("\n{}", question.bold());
                for token in &market.tokens {
                    let label = format!("  {}", token.outcome);
                    match clob.get_orderbook(&token.token_id).await {
                        Ok(book) => {
                            let best_bid = book.bids.first().map(|b| b.price).unwrap_or(0.0);
                            let best_ask = book.asks.first().map(|a| a.price).unwrap_or(0.0);
                            let mid_price = if best_bid > 0.0 && best_ask > 0.0 {
                                Some((best_bid + best_ask) / 2.0)
                            } else {
                                None
                            };
                            let spread = if best_ask > 0.0 && best_bid > 0.0 {
                                best_ask - best_bid
                            } else {
                                0.0
                            };
                            println!(
                                "{}  bid={:.4}  ask={:.4}  mid={:.4}  spread={:.4}",
                                label,
                                best_bid,
                                best_ask,
                                mid_price.unwrap_or(0.0),
                                spread,
                            );
                        }
                        Err(_e) => {
                            println!("{}  --", label);
                        }
                    }
                }
            } else {
                println!("\n{} (market not found)", mid);
            }
        }

        if !watch {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }

    Ok(())
}
