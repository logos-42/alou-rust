// ============================================
// Session Model - Session/token management
// ============================================

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

/// Session data model for refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    /// Create a new session with refresh token
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        refresh_token: String,
        expires_in_days: i64,
    ) -> Result<Session, sqlx::Error> {
        let expires_at = Utc::now() + Duration::days(expires_in_days);

        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, refresh_token, expires_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&refresh_token)
        .bind(expires_at)
        .fetch_one(pool)
        .await?;

        Ok(session)
    }

    /// Find session by refresh token
    pub async fn find_by_token(
        pool: &PgPool,
        refresh_token: &str,
    ) -> Result<Option<Session>, sqlx::Error> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions
            WHERE refresh_token = $1
            AND expires_at > NOW()
            "#,
        )
        .bind(refresh_token)
        .fetch_optional(pool)
        .await?;

        Ok(session)
    }

    /// Find all sessions for a user
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Session>, sqlx::Error> {
        let sessions = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions
            WHERE user_id = $1
            AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(sessions)
    }

    /// Delete a specific session (logout)
    pub async fn delete(pool: &PgPool, refresh_token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE refresh_token = $1
            "#,
        )
        .bind(refresh_token)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete all sessions for a user (logout all devices)
    pub async fn delete_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Clean up expired sessions (can be run periodically)
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    /// Get remaining validity duration
    pub fn remaining_validity(&self) -> Duration {
        self.expires_at - Utc::now()
    }
}

/// Session creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSession {
    pub user_id: Uuid,
    pub refresh_token: String,
    pub expires_in_days: i64,
}

