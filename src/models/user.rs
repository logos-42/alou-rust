// ============================================
// User Model - User data structure and database operations
// ============================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

/// User data model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub google_id: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    
    // Web3 fields (reserved for future DID implementation)
    pub did: Option<String>,
    pub did_document_cid: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// User creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub google_id: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

/// User update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub did: Option<String>,
    pub did_document_cid: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
}

impl User {
    /// Create a new user in the database
    pub async fn create(pool: &PgPool, user: CreateUser) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, google_id, name, avatar_url)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(&user.email)
        .bind(&user.google_id)
        .bind(&user.name)
        .bind(&user.avatar_url)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by Google ID
    pub async fn find_by_google_id(
        pool: &PgPool,
        google_id: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE google_id = $1
            "#,
        )
        .bind(google_id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Update user information
    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        update: UpdateUser,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET 
                name = COALESCE($2, name),
                avatar_url = COALESCE($3, avatar_url),
                did = COALESCE($4, did),
                did_document_cid = COALESCE($5, did_document_cid),
                wallet_address = COALESCE($6, wallet_address),
                public_key = COALESCE($7, public_key),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&update.name)
        .bind(&update.avatar_url)
        .bind(&update.did)
        .bind(&update.did_document_cid)
        .bind(&update.wallet_address)
        .bind(&update.public_key)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Update last login time
    pub async fn update_last_login(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET last_login = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deactivate user
    pub async fn deactivate(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET is_active = false
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if user exists by email
    pub async fn exists_by_email(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)
            "#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    /// List all active users (for admin)
    pub async fn list_active(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE is_active = true
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

/// Public user info (safe to expose to clients)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub did: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        PublicUser {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            did: user.did,
            created_at: user.created_at,
        }
    }
}

