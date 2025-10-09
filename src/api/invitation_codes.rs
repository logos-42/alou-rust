// ============================================
// Invitation Codes API Endpoints
// ============================================

use sqlx::PgPool;
use warp::{reject, reply, Rejection, Reply};
use chrono::Utc;
use warp::http::StatusCode;
use serde::{Deserialize, Serialize};

// 在handler函数上方添加新的导入
use crate::models::invitation_codes::UseInvitationCodeRequest;
use crate::models::invitation_codes::UseInvitationCodeResponse;
use crate::models::invitation_codes::CheckUserInvitationResponse;

use crate::models::invitation_codes::{InvitationCode, GenerateInvitationCodesRequest, GenerateInvitationCodesResponse};

// ============================================
// Handler Functions
// ============================================

#[derive(Debug, Deserialize)]
pub struct CheckUserInvitationRequest {
    pub user_id: i64,
}
/// 处理生成邀请码的请求
pub async fn handle_generate_invitation_codes(
    req: GenerateInvitationCodesRequest,
    pool: PgPool,
) -> Result<impl Reply, Rejection> {
    // 限制一次最多生成100个邀请码
    if req.num > 100 {
        return Ok(reply::with_status(
            reply::json(&GenerateInvitationCodesResponse {
                status: "error".to_string(),
                message: "Cannot generate more than 100 codes at once".to_string(),
                codes: vec![],
                timestamp: Utc::now().timestamp_millis() as u64,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    match InvitationCode::generate_codes(&pool, req.num).await {
        Ok(codes) => {
            Ok(reply::with_status(
                reply::json(&GenerateInvitationCodesResponse {
                    status: "success".to_string(),
                    message: format!("Successfully generated {} invitation codes", codes.len()),
                    codes,
                    timestamp: Utc::now().timestamp_millis() as u64
                }),
                warp::http::StatusCode::OK,
            ))
        }
        Err(e) => {
            eprintln!("Failed to generate invitation codes: {}", e);
            Ok(reply::with_status(
                reply::json(&GenerateInvitationCodesResponse {
                    status: "error".to_string(),
                    message: "Failed to generate invitation codes".to_string(),
                    codes: vec![],
                    timestamp: Utc::now().timestamp_millis() as u64,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// 处理使用邀请码的请求
pub async fn handle_use_invitation_code(
    req: UseInvitationCodeRequest,
    pool: PgPool,
) -> Result<impl Reply, Rejection> {
    match InvitationCode::use_invitation_code(&pool, req.user_id, &req.code).await {
        Ok(()) => {
            Ok(reply::with_status(
                reply::json(&UseInvitationCodeResponse {
                    status: "success".to_string(),
                    message: "Invitation code used successfully".to_string(),
                    timestamp: Utc::now().timestamp_millis() as u64,
                }),
                StatusCode::OK,
            ))
        }
        Err(sqlx::Error::RowNotFound) => {
            Ok(reply::with_status(
                reply::json(&UseInvitationCodeResponse {
                    status: "error".to_string(),
                    message: "Invitation code not found or already used".to_string(),
                    timestamp: Utc::now().timestamp_millis() as u64,
                }),
                StatusCode::BAD_REQUEST,
            ))
        }
        Err(e) => {
            eprintln!("Failed to use invitation code: {}", e);
            Ok(reply::with_status(
                reply::json(&UseInvitationCodeResponse {
                    status: "error".to_string(),
                    message: "Failed to use invitation code".to_string(),
                    timestamp: Utc::now().timestamp_millis() as u64,
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// 处理检查用户是否有激活邀请码的请求
pub async fn handle_check_user_invitation(
    req: CheckUserInvitationRequest,
    pool: PgPool,
) -> Result<impl Reply, Rejection> {
    match InvitationCode::check_user_has_active_invitation(&pool, req.user_id).await {
        Ok(has_active) => {
            Ok(reply::with_status(
                reply::json(&CheckUserInvitationResponse {
                    status: "success".to_string(),
                    has_active_invitation: has_active,
                    message: if has_active {
                        "User has active invitation code".to_string()
                    } else {
                        "User has no active invitation code".to_string()
                    },
                    timestamp: Utc::now().timestamp_millis() as u64,
                }),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            eprintln!("Failed to check user invitation: {}", e);
            Ok(reply::with_status(
                reply::json(&CheckUserInvitationResponse {
                    status: "error".to_string(),
                    has_active_invitation: false,
                    message: "Failed to check user invitation".to_string(),
                    timestamp: Utc::now().timestamp_millis() as u64,
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
