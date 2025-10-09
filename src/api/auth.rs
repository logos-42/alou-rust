// ============================================
// Authentication API Endpoints
// ============================================

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use warp::{reject, reply, Rejection, Reply};

use crate::auth::google_oauth::GoogleOAuth;
use crate::auth::jwt::{generate_token, verify_token};
use crate::models::session::Session;
use crate::models::user::{CreateUser, PublicUser, User};

// ============================================
// Request/Response Types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleLoginResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: PublicUser,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub user: Option<PublicUser>,
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

/// GET /api/auth/google/login
/// Generate Google OAuth authorization URL
pub async fn google_login_handler(
    oauth_client: GoogleOAuth,
) -> Result<impl Reply, Rejection> {
    let (auth_url, csrf_token) = oauth_client.get_auth_url();

    let response = GoogleLoginResponse {
        auth_url,
        state: csrf_token.secret().clone(),
    };

    Ok(reply::json(&response))
}

/// GET /api/auth/google/callback
/// Handle Google OAuth callback
pub async fn google_callback_handler(
    query: GoogleCallbackQuery,
    oauth_client: GoogleOAuth,
    pool: PgPool,
    jwt_secret: String,
    jwt_expiration_hours: i64,
    refresh_token_expiration_days: i64,
) -> Result<impl Reply, Rejection> {
    // Exchange authorization code for access token and user info
    let (_google_access_token, google_user) = oauth_client
        .exchange_code(query.code)
        .await
        .map_err(|e| {
            reject::custom(ApiError {
                message: format!("Failed to exchange code: {}", e),
            })
        })?;

    // Find or create user
    let user = match User::find_by_google_id(&pool, &google_user.id).await {
        Ok(Some(existing_user)) => {
            // Update last login
            User::update_last_login(&pool, existing_user.id)
                .await
                .map_err(|e| {
                    reject::custom(ApiError {
                        message: format!("Failed to update last login: {}", e),
                    })
                })?;
            existing_user
        }
        Ok(None) => {
            // Create new user
            let new_user = CreateUser {
                email: google_user.email.clone(),
                google_id: Some(google_user.id.clone()),
                name: google_user.name.clone(),
                avatar_url: google_user.picture.clone(),
            };

            User::create(&pool, new_user).await.map_err(|e| {
                reject::custom(ApiError {
                    message: format!("Failed to create user: {}", e),
                })
            })?
        }
        Err(e) => {
            return Err(reject::custom(ApiError {
                message: format!("Database error: {}", e),
            }));
        }
    };

    // Generate JWT tokens
    let access_token = generate_token(user.id, &user.email, &jwt_secret, jwt_expiration_hours)
        .map_err(|e| {
            reject::custom(ApiError {
                message: format!("Failed to generate token: {}", e),
            })
        })?;

    let refresh_token =
        generate_token(user.id, &user.email, &jwt_secret, refresh_token_expiration_days * 24)
            .map_err(|e| {
                reject::custom(ApiError {
                    message: format!("Failed to generate refresh token: {}", e),
                })
            })?;

    // Store refresh token in database
    Session::create(&pool, user.id, refresh_token.clone(), refresh_token_expiration_days)
        .await
        .map_err(|e| {
            reject::custom(ApiError {
                message: format!("Failed to store session: {}", e),
            })
        })?;

    // Return tokens and user info
    let response = AuthResponse {
        access_token,
        refresh_token,
        expires_in: jwt_expiration_hours * 3600,
        user: user.into(),
    };

    Ok(reply::json(&response))
}

/// POST /api/auth/verify
/// Verify JWT token validity
pub async fn verify_token_handler(
    user_id: Uuid,
    pool: PgPool,
) -> Result<impl Reply, Rejection> {
    // User ID is already verified by middleware
    // Fetch user info
    let user = User::find_by_id(&pool, user_id).await.map_err(|e| {
        reject::custom(ApiError {
            message: format!("Database error: {}", e),
        })
    })?;

    let response = VerifyResponse {
        valid: true,
        user: user.map(|u| u.into()),
    };

    Ok(reply::json(&response))
}

/// POST /api/auth/refresh
/// Refresh access token using refresh token
pub async fn refresh_token_handler(
    body: RefreshTokenRequest,
    pool: PgPool,
    jwt_secret: String,
    jwt_expiration_hours: i64,
) -> Result<impl Reply, Rejection> {
    // Verify refresh token
    let claims = verify_token(&body.refresh_token, &jwt_secret).map_err(|_| {
        reject::custom(ApiError {
            message: "Invalid refresh token".to_string(),
        })
    })?;

    // Check if refresh token exists in database
    let session = Session::find_by_token(&pool, &body.refresh_token)
        .await
        .map_err(|e| {
            reject::custom(ApiError {
                message: format!("Database error: {}", e),
            })
        })?
        .ok_or_else(|| {
            reject::custom(ApiError {
                message: "Session not found".to_string(),
            })
        })?;

    // Check if session is expired
    if session.is_expired() {
        return Err(reject::custom(ApiError {
            message: "Refresh token expired".to_string(),
        }));
    }

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        reject::custom(ApiError {
            message: "Invalid user ID in token".to_string(),
        })
    })?;

    // Generate new access token
    let new_access_token =
        generate_token(user_id, &claims.email, &jwt_secret, jwt_expiration_hours).map_err(|e| {
            reject::custom(ApiError {
                message: format!("Failed to generate token: {}", e),
            })
        })?;

    // Fetch user info
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

    let response = AuthResponse {
        access_token: new_access_token,
        refresh_token: body.refresh_token, // Return same refresh token
        expires_in: jwt_expiration_hours * 3600,
        user: user.into(),
    };

    Ok(reply::json(&response))
}

/// POST /api/auth/logout
/// Logout user by deleting refresh token
pub async fn logout_handler(
    _user_id: Uuid,
    body: RefreshTokenRequest,
    pool: PgPool,
) -> Result<impl Reply, Rejection> {
    // Delete the specific session
    Session::delete(&pool, &body.refresh_token)
        .await
        .map_err(|e| {
            reject::custom(ApiError {
                message: format!("Failed to delete session: {}", e),
            })
        })?;

    let response = MessageResponse {
        message: "Logged out successfully".to_string(),
    };

    Ok(reply::json(&response))
}

/// POST /api/auth/logout-all
/// Logout user from all devices
pub async fn logout_all_handler(user_id: Uuid, pool: PgPool) -> Result<impl Reply, Rejection> {
    // Delete all sessions for this user
    Session::delete_all_for_user(&pool, user_id)
        .await
        .map_err(|e| {
            reject::custom(ApiError {
                message: format!("Failed to delete sessions: {}", e),
            })
        })?;

    let response = MessageResponse {
        message: "Logged out from all devices".to_string(),
    };

    Ok(reply::json(&response))
}

