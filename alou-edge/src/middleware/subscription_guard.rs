// Subscription guard middleware for rate limiting and feature access control
use crate::storage::kv::KvStore;
use crate::storage::subscription::SubscriptionStorage;
#[allow(unused_imports)]
use crate::utils::error::AloudError;
use crate::utils::error::Result;
use crate::utils::time;
use serde::{Deserialize, Serialize};
use worker::Env;

/// Daily request limit constants
const FREE_USER_DAILY_LIMIT: u64 = 20;
const TRIAL_USER_DAILY_LIMIT: u64 = 50;
const DAILY_LIMIT_KV_TTL_SECONDS: u64 = 2 * 24 * 60 * 60; // 2 days to handle timezone edge cases

/// Daily request counter stored in KV
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DailyRequestCounter {
    date: i64,
    count: u64,
}

/// Subscription guard for checking user access
pub struct SubscriptionGuard {
    storage: SubscriptionStorage,
    cache_store: KvStore,
}

impl SubscriptionGuard {
    pub fn new(env: &Env, cache_store: KvStore) -> Result<Self> {
        let storage = SubscriptionStorage::new(env)?;
        Ok(Self {
            storage,
            cache_store,
        })
    }

    /// Check if user can make a request
    /// Returns (allowed, remaining_requests, is_premium)
    pub async fn check_request_access(
        &self,
        user_id: &str,
        wallet_address: &str,
    ) -> Result<(bool, u64, bool)> {
        // Check if user has active subscription
        let subscription = self
            .storage
            .get_active_subscription(user_id, wallet_address)
            .await?;

        if let Some(sub) = subscription {
            // Premium user - unlimited requests
            if sub.status == "active" && sub.expires_at > time::now_timestamp() {
                return Ok((true, u64::MAX, true));
            }
        }

        // Check if user is in trial period
        let trial = self.storage.check_trial(user_id, wallet_address).await?;
        let is_trial = trial
            .as_ref()
            .map(|t| t.is_used && t.expires_at > time::now_timestamp())
            .unwrap_or(false);

        // Determine daily limit based on user type
        let daily_limit = if is_trial {
            TRIAL_USER_DAILY_LIMIT
        } else {
            FREE_USER_DAILY_LIMIT
        };

        // Use wallet_address if available, otherwise use user_id as identifier
        let identifier = if !wallet_address.is_empty() {
            wallet_address.to_string()
        } else {
            user_id.to_string()
        };

        // Get today's date (UTC days since epoch)
        let today = time::now_timestamp() / 86400;
        let kv_key = format!("daily_limit:{}:{}", identifier, today);

        // Read current counter from KV
        let mut counter = self
            .cache_store
            .get::<DailyRequestCounter>(&kv_key)
            .await?
            .unwrap_or_else(|| DailyRequestCounter {
                date: today,
                count: 0,
            });

        // Reset counter if it's a new day (safety check, though key already includes date)
        if counter.date != today {
            counter.date = today;
            counter.count = 0;
        }

        // Check if limit is exceeded
        let remaining = daily_limit.saturating_sub(counter.count);
        let allowed = counter.count < daily_limit;

        // Increment counter if allowed
        if allowed {
            counter.count += 1;
            // Store updated counter in KV with TTL
            self.cache_store
                .put(&kv_key, &counter, Some(DAILY_LIMIT_KV_TTL_SECONDS))
                .await?;
        }

        Ok((allowed, remaining, false))
    }

    /// Check if user has premium subscription
    #[allow(dead_code)]
    pub async fn is_premium(&self, user_id: &str, wallet_address: &str) -> Result<bool> {
        let subscription = self
            .storage
            .get_active_subscription(user_id, wallet_address)
            .await?;

        if let Some(sub) = subscription {
            return Ok(sub.status == "active" && sub.expires_at > time::now_timestamp());
        }

        Ok(false)
    }

    /// Check if user is in trial period
    #[allow(dead_code)]
    pub async fn is_in_trial(&self, user_id: &str, wallet_address: &str) -> Result<bool> {
        let trial = self.storage.check_trial(user_id, wallet_address).await?;

        if let Some(t) = trial {
            return Ok(t.is_used && t.expires_at > time::now_timestamp());
        }

        Ok(false)
    }

    /// Get user's subscription status
    #[allow(dead_code)]
    pub async fn get_user_status(
        &self,
        user_id: &str,
        wallet_address: &str,
    ) -> Result<UserStatus> {
        let subscription = self
            .storage
            .get_active_subscription(user_id, wallet_address)
            .await?;
        let trial = self.storage.check_trial(user_id, wallet_address).await?;

        let is_premium = subscription
            .as_ref()
            .map(|s| s.status == "active" && s.expires_at > time::now_timestamp())
            .unwrap_or(false);

        let is_trial = trial
            .as_ref()
            .map(|t| t.is_used && t.expires_at > time::now_timestamp())
            .unwrap_or(false);

        Ok(UserStatus {
            is_premium,
            is_trial,
            subscription,
            trial,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct UserStatus {
    pub is_premium: bool,
    pub is_trial: bool,
    pub subscription: Option<crate::storage::subscription::Subscription>,
    pub trial: Option<crate::storage::subscription::TrialPeriod>,
}

