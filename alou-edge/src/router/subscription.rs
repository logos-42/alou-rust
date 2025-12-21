// Subscription management routes
use crate::storage::subscription::{SubscriptionStorage, Subscription, TrialPeriod};
#[allow(unused_imports)]
use crate::storage::subscription::SubscriptionPlan;
use crate::utils::error::{AloudError, Result};
use crate::utils::time;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use worker::*;

/// Check trial period status
pub async fn handle_check_trial(
    storage: &SubscriptionStorage,
    req: &mut Request,
) -> Result<Response> {
    let body: CheckTrialRequest = req.json().await.map_err(|e| {
        AloudError::InvalidInput(format!("Failed to parse request body: {}", e))
    })?;

    let trial = storage
        .check_trial(&body.user_id, &body.wallet_address)
        .await?;

    let is_expired = trial
        .as_ref()
        .map(|t| t.expires_at < time::now_timestamp())
        .unwrap_or(false);
    
    let response = CheckTrialResponse {
        has_trial: trial.is_some(),
        trial: trial,
        is_expired,
    };

    json_response(&response)
}

/// Get or create trial period
pub async fn handle_get_or_create_trial(
    storage: &SubscriptionStorage,
    req: &mut Request,
) -> Result<Response> {
    let body: GetTrialRequest = req.json().await.map_err(|e| {
        AloudError::InvalidInput(format!("Failed to parse request body: {}", e))
    })?;

    let trial = storage
        .get_or_create_trial(&body.user_id, &body.wallet_address)
        .await?;

    json_response(&trial)
}

/// Get subscription plans
pub async fn handle_get_plans(storage: &SubscriptionStorage) -> Result<Response> {
    let plans = storage.get_all_plans().await?;
    json_response(&plans)
}

/// Get subscription status
pub async fn handle_get_status(
    storage: &SubscriptionStorage,
    req: &mut Request,
) -> Result<Response> {
    let body: GetStatusRequest = req.json().await.map_err(|e| {
        AloudError::InvalidInput(format!("Failed to parse request body: {}", e))
    })?;

    let subscription = storage
        .get_active_subscription(&body.user_id, &body.wallet_address)
        .await?;

    let is_expired = subscription
        .as_ref()
        .map(|s| s.expires_at < time::now_timestamp())
        .unwrap_or(false);
    
    let response = GetStatusResponse {
        has_subscription: subscription.is_some(),
        subscription: subscription,
        is_expired,
    };

    json_response(&response)
}

/// Create subscription
pub async fn handle_create_subscription(
    storage: &SubscriptionStorage,
    req: &mut Request,
) -> Result<Response> {
    let body: CreateSubscriptionRequest = req.json().await.map_err(|e| {
        AloudError::InvalidInput(format!("Failed to parse request body: {}", e))
    })?;

    // Validate request
    if body.plan_id.is_empty() {
        return Err(AloudError::InvalidInput("plan_id is required".to_string()));
    }
    if body.chain_type.is_empty() {
        return Err(AloudError::InvalidInput("chain_type is required".to_string()));
    }
    if body.token_symbol.is_empty() {
        return Err(AloudError::InvalidInput("token_symbol is required".to_string()));
    }
    if body.amount_paid.is_empty() {
        return Err(AloudError::InvalidInput("amount_paid is required".to_string()));
    }

    let subscription = storage
        .create_subscription(
            &body.user_id,
            &body.wallet_address,
            &body.plan_id,
            &body.chain_type,
            &body.token_symbol,
            &body.amount_paid,
            body.amount_usd,
            body.tx_hash.as_deref(),
        )
        .await?;

    json_response(&subscription)
}

/// Renew subscription
pub async fn handle_renew_subscription(
    storage: &SubscriptionStorage,
    req: &mut Request,
) -> Result<Response> {
    let body: RenewSubscriptionRequest = req.json().await.map_err(|e| {
        AloudError::InvalidInput(format!("Failed to parse request body: {}", e))
    })?;

    // Get current subscription
    let current = storage
        .get_active_subscription(&body.user_id, &body.wallet_address)
        .await?
        .ok_or_else(|| AloudError::InvalidInput("No active subscription found".to_string()))?;

    // Create new subscription with same plan (renewal)
    let subscription = storage
        .create_subscription(
            &body.user_id,
            &body.wallet_address,
            &current.plan_id,
            &body.chain_type,
            &body.token_symbol,
            &body.amount_paid,
            body.amount_usd,
            body.tx_hash.as_deref(),
        )
        .await?;

    json_response(&subscription)
}

/// Verify payment transaction
pub async fn handle_verify_payment(
    _storage: &SubscriptionStorage,
    req: &mut Request,
) -> Result<Response> {
    let body: VerifyPaymentRequest = req.json().await.map_err(|e| {
        AloudError::InvalidInput(format!("Failed to parse request body: {}", e))
    })?;

    // This would typically verify the transaction on-chain
    // For now, we'll just return success if tx_hash is provided
    let response = VerifyPaymentResponse {
        verified: !body.tx_hash.is_empty(),
        tx_hash: body.tx_hash.clone(),
        message: if body.tx_hash.is_empty() {
            "Transaction hash is required".to_string()
        } else {
            "Payment verified".to_string()
        },
    };

    json_response(&response)
}

/// Get notifications (expiring subscriptions and trials)
pub async fn handle_get_notifications(
    storage: &SubscriptionStorage,
    req: &Request,
) -> Result<Response> {
    let url = req.url()?;
    let days_before: i64 = url
        .query_pairs()
        .find(|(key, _)| key == "days_before")
        .and_then(|(_, value)| value.parse().ok())
        .unwrap_or(7);

    let expiring_subscriptions = storage.get_expiring_subscriptions(days_before).await?;
    let expiring_trials = storage.get_expiring_trials(days_before).await?;

    let response = NotificationsResponse {
        expiring_subscriptions,
        expiring_trials,
        days_before,
    };

    json_response(&response)
}

// ============ Request/Response Types ============

#[derive(Deserialize)]
struct CheckTrialRequest {
    user_id: String,
    wallet_address: String,
}

#[derive(Serialize)]
struct CheckTrialResponse {
    has_trial: bool,
    trial: Option<TrialPeriod>,
    is_expired: bool,
}

#[derive(Deserialize)]
struct GetTrialRequest {
    user_id: String,
    wallet_address: String,
}

#[derive(Deserialize)]
struct GetStatusRequest {
    user_id: String,
    wallet_address: String,
}

#[derive(Serialize)]
struct GetStatusResponse {
    has_subscription: bool,
    subscription: Option<Subscription>,
    is_expired: bool,
}

#[derive(Deserialize)]
struct CreateSubscriptionRequest {
    user_id: String,
    wallet_address: String,
    plan_id: String,
    chain_type: String,
    token_symbol: String,
    amount_paid: String,
    amount_usd: f64,
    tx_hash: Option<String>,
}

#[derive(Deserialize)]
struct RenewSubscriptionRequest {
    user_id: String,
    wallet_address: String,
    chain_type: String,
    token_symbol: String,
    amount_paid: String,
    amount_usd: f64,
    tx_hash: Option<String>,
}

#[derive(Deserialize)]
struct VerifyPaymentRequest {
    tx_hash: String,
    #[allow(dead_code)]
    chain_type: String,
}

#[derive(Serialize)]
struct VerifyPaymentResponse {
    verified: bool,
    tx_hash: String,
    message: String,
}

#[derive(Serialize)]
struct NotificationsResponse {
    expiring_subscriptions: Vec<Subscription>,
    expiring_trials: Vec<TrialPeriod>,
    days_before: i64,
}

// Helper function for JSON responses
fn json_response<T: Serialize>(data: &T) -> crate::utils::error::Result<Response> {
    use crate::router::json_response as router_json_response;
    router_json_response(data)
        .map_err(|e| crate::utils::error::AloudError::AgentError(e.to_string()))
}

