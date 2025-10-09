// ============================================
// User API Endpoints
// ============================================

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use warp::{reject, reply, Rejection, Reply};

use crate::models::user::{PublicUser, UpdateUser, User};

// ============================================
// Request/Response Types
// ============================================

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user: PublicUser,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// ============================================
// Error Types
// ============================================

#[derive(Debug)]
pub struct ApiError {
    pub message: String,
}

impl reject::Reject for ApiError {}

// ============================================
// Handler Functions
// ============================================

/// GET /api/user/me
/// Get current user information
pub async fn get_me_handler(user_id: Uuid, pool: PgPool) -> Result<impl Reply, Rejection> {
    let user = User::find_by_id(&pool, user_id)
        .await
        .map_err(|e| {
            reject::custom(ApiError {
                message: format!("Database error: {}", e),
            })
        })?
        .ok_or_else(|| {
            reject::custom(ApiError {
                message: "User not found".to_string(),
            })
        })?;

    let response = UserResponse {
        user: user.into(),
    };

    Ok(reply::json(&response))
}

/// PUT /api/user/profile
/// Update user profile
pub async fn update_profile_handler(
    user_id: Uuid,
    body: UpdateProfileRequest,
    pool: PgPool,
) -> Result<impl Reply, Rejection> {
    let update = UpdateUser {
        name: body.name,
        avatar_url: body.avatar_url,
        did: None,
        did_document_cid: None,
        wallet_address: None,
        public_key: None,
    };

    let user = User::update(&pool, user_id, update).await.map_err(|e| {
        reject::custom(ApiError {
            message: format!("Failed to update profile: {}", e),
        })
    })?;

    let response = UserResponse {
        user: user.into(),
    };

    Ok(reply::json(&response))
}

/// PUT /api/user/web3
/// Update Web3/DID information
#[derive(Debug, Deserialize)]
pub struct UpdateWeb3Request {
    pub did: Option<String>,
    pub did_document_cid: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
}

pub async fn update_web3_handler(
    user_id: Uuid,
    body: UpdateWeb3Request,
    pool: PgPool,
) -> Result<impl Reply, Rejection> {
    let update = UpdateUser {
        name: None,
        avatar_url: None,
        did: body.did,
        did_document_cid: body.did_document_cid,
        wallet_address: body.wallet_address,
        public_key: body.public_key,
    };

    let user = User::update(&pool, user_id, update).await.map_err(|e| {
        reject::custom(ApiError {
            message: format!("Failed to update Web3 info: {}", e),
        })
    })?;

    let response = UserResponse {
        user: user.into(),
    };

    Ok(reply::json(&response))
}

/// DELETE /api/user/account
/// Deactivate user account
pub async fn deactivate_account_handler(
    user_id: Uuid,
    pool: PgPool,
) -> Result<impl Reply, Rejection> {
    User::deactivate(&pool, user_id).await.map_err(|e| {
        reject::custom(ApiError {
            message: format!("Failed to deactivate account: {}", e),
        })
    })?;

    let response = MessageResponse {
        message: "Account deactivated successfully".to_string(),
    };

    Ok(reply::json(&response))
}

