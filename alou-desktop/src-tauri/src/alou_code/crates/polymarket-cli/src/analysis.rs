use crate::clob::{ClobClient, ClobMarket, ClobTrade};
use anyhow::Result;
use chrono::Timelike;
use std::collections::HashMap;

pub struct MarketAnalyzer {
    clob: ClobClient,
}

impl MarketAnalyzer {
    pub fn new(clob: ClobClient) -> Self {
        Self { clob }
    }

    pub async fn analyze_market(&self, market_id: &str) -> Result<MarketAnalysis> {
        // Get all markets to find the one we want
        let markets = self.clob.get_markets().await?;
        let market = markets
            .iter()
            .find(|m| m.condition_id == market_id)
            .ok_or_else(|| anyhow::anyhow!("Market not found: {}", market_id))?;

        // Get trades for this market
        let trades = self.clob.get_trades(market_id).await?;

        // Get orderbook for the first token in the market
        let token_id = market
            .tokens
            .first()
            .map(|t| t.token_id.clone())
            .ok_or_else(|| anyhow::anyhow!("Market has no tokens"))?;
        let orderbook = self.clob.get_orderbook(&token_id).await?;

        let volume_analysis = self.analyze_volume(&trades);
        let price_analysis = self.analyze_price(&trades, &orderbook);
        let liquidity_analysis = self.analyze_liquidity(&orderbook);
        let sentiment_analysis = self.analyze_sentiment(market, &trades);

        Ok(MarketAnalysis {
            market_id: market_id.to_string(),
            market_question: market
                .question
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            volume_analysis,
            price_analysis,
            liquidity_analysis,
            sentiment_analysis,
            recommendations: vec![],
        })
    }

    fn analyze_volume(&self, trades: &[ClobTrade]) -> VolumeAnalysis {
        let mut total_volume = 0.0;
        let mut buy_volume = 0.0;
        let mut sell_volume = 0.0;
        let mut volume_by_hour: HashMap<u32, f64> = HashMap::new();

        for trade in trades {
            total_volume += trade.size;
            if trade.side.to_lowercase() == "buy" {
                buy_volume += trade.size;
            } else {
                sell_volume += trade.size;
            }

            // Parse hour from timestamp
            if let Some(match_time) = trade.match_time {
                let timestamp = chrono::DateTime::from_timestamp(match_time as i64, 0);
                if let Some(dt) = timestamp {
                    let hour = dt.hour();
                    *volume_by_hour.entry(hour).or_insert(0.0) += trade.size;
                }
            }
        }

        let volume_ratio = if sell_volume > 0.0 {
            buy_volume / sell_volume
        } else {
            1.0
        };

        VolumeAnalysis {
            total_volume,
            buy_volume,
            sell_volume,
            volume_ratio,
            volume_by_hour,
        }
    }

    fn analyze_price(
        &self,
        trades: &[ClobTrade],
        orderbook: &crate::clob::ClobOrderbook,
    ) -> PriceAnalysis {
        let mut current_price = 0.0;
        let mut price_change = 0.0;
        let mut volatility = 0.0;

        if !trades.is_empty() {
            let prices: Vec<f64> = trades.iter().map(|t| t.price).collect();
            current_price = *prices.last().unwrap();
            let first_price = *prices.first().unwrap();
            price_change = if first_price > 0.0 {
                ((current_price - first_price) / first_price) * 100.0
            } else {
                0.0
            };

            // Calculate volatility (standard deviation)
            let mean = prices.iter().sum::<f64>() / prices.len() as f64;
            let variance =
                prices.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / prices.len() as f64;
            volatility = variance.sqrt();
        }

        // Identify support and resistance from orderbook
        let mut support_levels: Vec<f64> = orderbook.bids.iter().take(5).map(|b| b.price).collect();
        support_levels.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let mut resistance_levels: Vec<f64> =
            orderbook.asks.iter().take(5).map(|a| a.price).collect();
        resistance_levels.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        PriceAnalysis {
            current_price,
            price_change,
            volatility,
            support_levels,
            resistance_levels,
        }
    }

    fn analyze_liquidity(&self, orderbook: &crate::clob::ClobOrderbook) -> LiquidityAnalysis {
        let bid_liquidity: f64 = orderbook.bids.iter().map(|b| b.size).sum();
        let ask_liquidity: f64 = orderbook.asks.iter().map(|a| a.size).sum();
        let total_liquidity = bid_liquidity + ask_liquidity;

        let spread = if !orderbook.asks.is_empty() && !orderbook.bids.is_empty() {
            orderbook.asks[0].price - orderbook.bids[0].price
        } else {
            0.0
        };

        let depth_ratio = if ask_liquidity > 0.0 {
            bid_liquidity / ask_liquidity
        } else {
            1.0
        };

        LiquidityAnalysis {
            bid_liquidity,
            ask_liquidity,
            total_liquidity,
            spread,
            depth_ratio,
        }
    }

    fn analyze_sentiment(&self, market: &ClobMarket, trades: &[ClobTrade]) -> SentimentAnalysis {
        let mut sentiment_score = 0.0;
        let mut total_trades = 0;

        for trade in trades {
            if trade.side.to_lowercase() == "buy" {
                sentiment_score += 1.0;
            } else {
                sentiment_score -= 1.0;
            }
            total_trades += 1;
        }

        let normalized_sentiment = if total_trades > 0 {
            sentiment_score / total_trades as f64
        } else {
            0.0
        };

        // Analyze market description for keywords
        let description = market.description.as_deref().unwrap_or("").to_lowercase();
        let positive_keywords = vec!["win", "success", "positive", "bullish", "up", "gain"];
        let negative_keywords = vec!["lose", "failure", "negative", "bearish", "down", "loss"];

        let mut keyword_score = 0.0;
        for keyword in &positive_keywords {
            if description.contains(keyword) {
                keyword_score += 1.0;
            }
        }
        for keyword in &negative_keywords {
            if description.contains(keyword) {
                keyword_score -= 1.0;
            }
        }

        SentimentAnalysis {
            sentiment_score: normalized_sentiment,
            keyword_score,
            total_trades,
            overall_sentiment: if normalized_sentiment > 0.1 {
                "Bullish".to_string()
            } else if normalized_sentiment < -0.1 {
                "Bearish".to_string()
            } else {
                "Neutral".to_string()
            },
        }
    }

    pub fn generate_recommendations(&self, analysis: &MarketAnalysis) -> Vec<Recommendation> {
        let mut recommendations = vec![];

        // Volume-based recommendation
        if analysis.volume_analysis.volume_ratio > 1.5 {
            recommendations.push(Recommendation {
                action: "Consider buying".to_string(),
                reason: "Strong buy volume relative to sell volume".to_string(),
                confidence: 0.7,
            });
        } else if analysis.volume_analysis.volume_ratio < 0.67 {
            recommendations.push(Recommendation {
                action: "Consider selling".to_string(),
                reason: "Strong sell volume relative to buy volume".to_string(),
                confidence: 0.7,
            });
        }

        // Price trend recommendation
        if analysis.price_analysis.price_change > 5.0 {
            recommendations.push(Recommendation {
                action: "Watch for profit taking".to_string(),
                reason: "Significant price increase may lead to correction".to_string(),
                confidence: 0.6,
            });
        } else if analysis.price_analysis.price_change < -5.0 {
            recommendations.push(Recommendation {
                action: "Consider buying dip".to_string(),
                reason: "Significant price drop may present buying opportunity".to_string(),
                confidence: 0.6,
            });
        }

        // Liquidity recommendation
        if analysis.liquidity_analysis.spread < 0.01 {
            recommendations.push(Recommendation {
                action: "Good trading conditions".to_string(),
                reason: "Tight bid-ask spread indicates good liquidity".to_string(),
                confidence: 0.8,
            });
        }

        // Sentiment recommendation
        if analysis.sentiment_analysis.overall_sentiment == "Bullish" {
            recommendations.push(Recommendation {
                action: "Align with sentiment".to_string(),
                reason: "Market sentiment is bullish".to_string(),
                confidence: 0.65,
            });
        }

        recommendations
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarketAnalysis {
    pub market_id: String,
    pub market_question: String,
    pub volume_analysis: VolumeAnalysis,
    pub price_analysis: PriceAnalysis,
    pub liquidity_analysis: LiquidityAnalysis,
    pub sentiment_analysis: SentimentAnalysis,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VolumeAnalysis {
    pub total_volume: f64,
    pub buy_volume: f64,
    pub sell_volume: f64,
    pub volume_ratio: f64,
    pub volume_by_hour: HashMap<u32, f64>,
}

#[derive(Debug, Clone)]
pub struct PriceAnalysis {
    pub current_price: f64,
    pub price_change: f64,
    pub volatility: f64,
    pub support_levels: Vec<f64>,
    pub resistance_levels: Vec<f64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LiquidityAnalysis {
    pub bid_liquidity: f64,
    pub ask_liquidity: f64,
    pub total_liquidity: f64,
    pub spread: f64,
    pub depth_ratio: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SentimentAnalysis {
    pub sentiment_score: f64,
    pub keyword_score: f64,
    pub total_trades: i32,
    pub overall_sentiment: String,
}

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub action: String,
    pub reason: String,
    pub confidence: f64,
}
