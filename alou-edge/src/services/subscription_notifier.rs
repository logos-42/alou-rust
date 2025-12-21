// Subscription notification service
use crate::storage::subscription::SubscriptionStorage;
use crate::utils::error::Result;
use serde::{Deserialize, Serialize};

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum NotificationType {
    TrialExpiringSoon,      // 3 days before trial expires
    SubscriptionExpiringSoon, // 7 days before subscription expires
    TrialExpired,
    SubscriptionExpired,
}

/// Notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub wallet_address: String,
    pub notification_type: NotificationType,
    pub message: String,
    pub created_at: i64,
    pub read: bool,
}

/// Subscription notifier service
#[allow(dead_code)]
pub struct SubscriptionNotifier {
    storage: SubscriptionStorage,
}

#[allow(dead_code)]
impl SubscriptionNotifier {
    pub fn new(storage: SubscriptionStorage) -> Self {
        Self { storage }
    }

    /// Get notifications for expiring trials (3 days before)
    pub async fn get_trial_notifications(&self) -> Result<Vec<Notification>> {
        let expiring_trials = self.storage.get_expiring_trials(3).await?;
        
        let mut notifications = Vec::new();
        for trial in expiring_trials {
            notifications.push(Notification {
                id: format!("trial_notif_{}", trial.id),
                user_id: trial.user_id.clone(),
                wallet_address: trial.wallet_address.clone(),
                notification_type: NotificationType::TrialExpiringSoon,
                message: format!(
                    "Your trial period expires in {} days. Subscribe now to continue using premium features.",
                    (trial.expires_at - crate::utils::time::now_timestamp()) / 86400
                ),
                created_at: crate::utils::time::now_timestamp(),
                read: false,
            });
        }

        Ok(notifications)
    }

    /// Get notifications for expiring subscriptions (7 days before)
    pub async fn get_subscription_notifications(&self) -> Result<Vec<Notification>> {
        let expiring_subscriptions = self.storage.get_expiring_subscriptions(7).await?;
        
        let mut notifications = Vec::new();
        for subscription in expiring_subscriptions {
            notifications.push(Notification {
                id: format!("sub_notif_{}", subscription.id),
                user_id: subscription.user_id.clone(),
                wallet_address: subscription.wallet_address.clone(),
                notification_type: NotificationType::SubscriptionExpiringSoon,
                message: format!(
                    "Your subscription expires in {} days. Renew now to continue enjoying premium features.",
                    (subscription.expires_at - crate::utils::time::now_timestamp()) / 86400
                ),
                created_at: crate::utils::time::now_timestamp(),
                read: false,
            });
        }

        Ok(notifications)
    }

    /// Get all pending notifications
    pub async fn get_all_notifications(&self) -> Result<Vec<Notification>> {
        let mut all_notifications = Vec::new();
        
        all_notifications.extend(self.get_trial_notifications().await?);
        all_notifications.extend(self.get_subscription_notifications().await?);

        Ok(all_notifications)
    }
}

