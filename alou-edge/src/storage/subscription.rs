// Subscription storage layer for D1 database
use crate::storage::d1::D1Database;
use crate::utils::error::{AloudError, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[allow(unused_imports)]
use serde_json::Value;
use worker::Env;

/// Subscription plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub price_usd: f64,
    pub duration_days: i64,
    pub chain_type: String,
    pub supported_tokens: Vec<String>,
    pub is_active: bool,
}

/// Trial period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialPeriod {
    pub id: String,
    pub user_id: String,
    pub wallet_address: String,
    pub started_at: i64,
    pub expires_at: i64,
    pub is_used: bool,
}

/// Subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub user_id: String,
    pub wallet_address: String,
    pub plan_id: String,
    pub chain_type: String,
    pub token_symbol: String,
    pub amount_paid: String,
    pub amount_usd: f64,
    pub started_at: i64,
    pub expires_at: i64,
    pub status: String,
    pub tx_hash: Option<String>,
}

/// Subscription payment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SubscriptionPayment {
    pub id: String,
    pub subscription_id: String,
    pub user_id: String,
    pub wallet_address: String,
    pub chain_type: String,
    pub token_symbol: String,
    pub amount: String,
    pub amount_usd: f64,
    pub tx_hash: String,
    pub from_address: Option<String>,
    pub to_address: String,
    pub status: String,
    pub block_number: Option<i64>,
    pub confirmation_count: i64,
    pub created_at: i64,
    pub confirmed_at: Option<i64>,
}

/// Subscription storage service
pub struct SubscriptionStorage {
    db: D1Database,
}

impl SubscriptionStorage {
    pub fn new(env: &Env) -> Result<Self> {
        let db = D1Database::from_env(env)?;
        Ok(Self { db })
    }

    /// Get subscription plan by ID
    pub async fn get_plan(&self, plan_id: &str) -> Result<Option<SubscriptionPlan>> {
        let sql = "SELECT * FROM subscription_plans WHERE id = ?";
        let result = self.db.query(sql, &[json!(plan_id)]).await?;
        
        if let Some(row) = result.value() {
            let plan: SubscriptionPlan = serde_json::from_value(row.clone())
                .map_err(|e| AloudError::DatabaseError(format!("Failed to deserialize plan: {}", e)))?;
            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    /// Get all active subscription plans
    pub async fn get_all_plans(&self) -> Result<Vec<SubscriptionPlan>> {
        let sql = "SELECT * FROM subscription_plans WHERE is_active = 1 ORDER BY price_usd ASC";
        let rows = self.db.query_all(sql, &[]).await?;
        
        let mut plans = Vec::new();
        for row in rows {
            let plan: SubscriptionPlan = serde_json::from_value(row)
                .map_err(|e| AloudError::DatabaseError(format!("Failed to deserialize plan: {}", e)))?;
            plans.push(plan);
        }
        Ok(plans)
    }

    /// Get or create trial period for user
    pub async fn get_or_create_trial(&self, user_id: &str, wallet_address: &str) -> Result<TrialPeriod> {
        // Check if trial already exists
        let sql = "SELECT * FROM trial_periods WHERE user_id = ? OR wallet_address = ?";
        let result = self.db.query(sql, &[json!(user_id), json!(wallet_address)]).await?;
        
        if let Some(row) = result.value() {
            let trial: TrialPeriod = serde_json::from_value(row.clone())
                .map_err(|e| AloudError::DatabaseError(format!("Failed to deserialize trial: {}", e)))?;
            return Ok(trial);
        }

        // Create new trial period (12 days)
        let now = crate::utils::time::now_timestamp();
        let expires_at = now + (12 * 24 * 60 * 60); // 12 days in seconds
        let trial_id = format!("trial_{}_{}", user_id, now);

        let sql = "INSERT INTO trial_periods (id, user_id, wallet_address, started_at, expires_at, is_used, created_at) VALUES (?, ?, ?, ?, ?, 1, ?)";
        self.db.execute(sql, &[
            json!(trial_id),
            json!(user_id),
            json!(wallet_address),
            json!(now),
            json!(expires_at),
            json!(now),
        ]).await?;

        Ok(TrialPeriod {
            id: trial_id,
            user_id: user_id.to_string(),
            wallet_address: wallet_address.to_string(),
            started_at: now,
            expires_at,
            is_used: true,
        })
    }

    /// Check trial period status
    pub async fn check_trial(&self, user_id: &str, wallet_address: &str) -> Result<Option<TrialPeriod>> {
        let sql = "SELECT * FROM trial_periods WHERE (user_id = ? OR wallet_address = ?) AND is_used = 1";
        let result = self.db.query(sql, &[json!(user_id), json!(wallet_address)]).await?;
        
        if let Some(row) = result.value() {
            let trial: TrialPeriod = serde_json::from_value(row.clone())
                .map_err(|e| AloudError::DatabaseError(format!("Failed to deserialize trial: {}", e)))?;
            Ok(Some(trial))
        } else {
            Ok(None)
        }
    }

    /// Create subscription
    pub async fn create_subscription(
        &self,
        user_id: &str,
        wallet_address: &str,
        plan_id: &str,
        chain_type: &str,
        token_symbol: &str,
        amount_paid: &str,
        amount_usd: f64,
        tx_hash: Option<&str>,
    ) -> Result<Subscription> {
        let now = crate::utils::time::now_timestamp();
        
        // Get plan to calculate expiration
        let plan = self.get_plan(plan_id).await?
            .ok_or_else(|| AloudError::InvalidInput(format!("Plan not found: {}", plan_id)))?;
        
        let expires_at = now + (plan.duration_days * 24 * 60 * 60);
        let subscription_id = format!("sub_{}_{}", user_id, now);

        let sql = "INSERT INTO subscriptions (id, user_id, wallet_address, plan_id, chain_type, token_symbol, amount_paid, amount_usd, started_at, expires_at, status, tx_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?, ?, ?)";
        
        self.db.execute(sql, &[
            json!(subscription_id),
            json!(user_id),
            json!(wallet_address),
            json!(plan_id),
            json!(chain_type),
            json!(token_symbol),
            json!(amount_paid),
            json!(amount_usd),
            json!(now),
            json!(expires_at),
            json!(tx_hash),
            json!(now),
            json!(now),
        ]).await?;

        Ok(Subscription {
            id: subscription_id,
            user_id: user_id.to_string(),
            wallet_address: wallet_address.to_string(),
            plan_id: plan_id.to_string(),
            chain_type: chain_type.to_string(),
            token_symbol: token_symbol.to_string(),
            amount_paid: amount_paid.to_string(),
            amount_usd,
            started_at: now,
            expires_at,
            status: "active".to_string(),
            tx_hash: tx_hash.map(|s| s.to_string()),
        })
    }

    /// Get active subscription for user
    pub async fn get_active_subscription(&self, user_id: &str, wallet_address: &str) -> Result<Option<Subscription>> {
        let now = crate::utils::time::now_timestamp();
        let sql = "SELECT * FROM subscriptions WHERE (user_id = ? OR wallet_address = ?) AND status = 'active' AND expires_at > ? ORDER BY expires_at DESC LIMIT 1";
        
        let result = self.db.query(sql, &[json!(user_id), json!(wallet_address), json!(now)]).await?;
        
        if let Some(row) = result.value() {
            let subscription: Subscription = serde_json::from_value(row.clone())
                .map_err(|e| AloudError::DatabaseError(format!("Failed to deserialize subscription: {}", e)))?;
            Ok(Some(subscription))
        } else {
            Ok(None)
        }
    }

    /// Update subscription status
    #[allow(dead_code)]
    pub async fn update_subscription_status(&self, subscription_id: &str, status: &str) -> Result<()> {
        let now = crate::utils::time::now_timestamp();
        let sql = "UPDATE subscriptions SET status = ?, updated_at = ? WHERE id = ?";
        self.db.execute(sql, &[json!(status), json!(now), json!(subscription_id)]).await?;
        Ok(())
    }

    /// Create payment record
    #[allow(dead_code)]
    pub async fn create_payment(&self, payment: &SubscriptionPayment) -> Result<()> {
        let sql = "INSERT INTO subscription_payments (id, subscription_id, user_id, wallet_address, chain_type, token_symbol, amount, amount_usd, tx_hash, from_address, to_address, status, block_number, confirmation_count, created_at, confirmed_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
        
        self.db.execute(sql, &[
            json!(payment.id),
            json!(payment.subscription_id),
            json!(payment.user_id),
            json!(payment.wallet_address),
            json!(payment.chain_type),
            json!(payment.token_symbol),
            json!(payment.amount),
            json!(payment.amount_usd),
            json!(payment.tx_hash),
            json!(payment.from_address),
            json!(payment.to_address),
            json!(payment.status),
            json!(payment.block_number),
            json!(payment.confirmation_count),
            json!(payment.created_at),
            json!(payment.confirmed_at),
        ]).await?;
        
        Ok(())
    }

    /// Update payment status
    #[allow(dead_code)]
    pub async fn update_payment_status(&self, payment_id: &str, status: &str, confirmed_at: Option<i64>) -> Result<()> {
        let sql = "UPDATE subscription_payments SET status = ?, confirmed_at = ? WHERE id = ?";
        self.db.execute(sql, &[json!(status), json!(confirmed_at), json!(payment_id)]).await?;
        Ok(())
    }

    /// Get subscriptions expiring soon (for notifications)
    pub async fn get_expiring_subscriptions(&self, days_before: i64) -> Result<Vec<Subscription>> {
        let now = crate::utils::time::now_timestamp();
        let threshold = now + (days_before * 24 * 60 * 60);
        
        let sql = "SELECT * FROM subscriptions WHERE status = 'active' AND expires_at > ? AND expires_at <= ?";
        let rows = self.db.query_all(sql, &[json!(now), json!(threshold)]).await?;
        
        let mut subscriptions = Vec::new();
        for row in rows {
            let subscription: Subscription = serde_json::from_value(row)
                .map_err(|e| AloudError::DatabaseError(format!("Failed to deserialize subscription: {}", e)))?;
            subscriptions.push(subscription);
        }
        Ok(subscriptions)
    }

    /// Get expiring trials (for notifications)
    pub async fn get_expiring_trials(&self, days_before: i64) -> Result<Vec<TrialPeriod>> {
        let now = crate::utils::time::now_timestamp();
        let threshold = now + (days_before * 24 * 60 * 60);
        
        let sql = "SELECT * FROM trial_periods WHERE is_used = 1 AND expires_at > ? AND expires_at <= ?";
        let rows = self.db.query_all(sql, &[json!(now), json!(threshold)]).await?;
        
        let mut trials = Vec::new();
        for row in rows {
            let trial: TrialPeriod = serde_json::from_value(row)
                .map_err(|e| AloudError::DatabaseError(format!("Failed to deserialize trial: {}", e)))?;
            trials.push(trial);
        }
        Ok(trials)
    }
}

