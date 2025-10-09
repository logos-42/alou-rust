// ============================================
// Warp Authentication Middleware
// ============================================

use warp::{Filter, Rejection, reject};
use uuid::Uuid;

use super::jwt::verify_token;

/// Custom rejection for authentication errors
#[derive(Debug)]
pub struct Unauthorized;
impl reject::Reject for Unauthorized {}

/// Extract JWT from Authorization header and verify
pub fn with_auth(
    jwt_secret: String,
) -> impl Filter<Extract = (Uuid,), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and_then(move |auth_header: Option<String>| {
            let secret = jwt_secret.clone();
            async move {
                match auth_header {
                    Some(header) => {
                        // Extract token from "Bearer <token>"
                        let token = header
                            .strip_prefix("Bearer ")
                            .ok_or_else(|| reject::custom(Unauthorized))?;

                        // Verify token
                        let claims = verify_token(token, &secret)
                            .map_err(|_| reject::custom(Unauthorized))?;

                        // Parse user ID from claims
                        let user_id = Uuid::parse_str(&claims.sub)
                            .map_err(|_| reject::custom(Unauthorized))?;

                        Ok(user_id)
                    }
                    None => Err(reject::custom(Unauthorized)),
                }
            }
        })
}

