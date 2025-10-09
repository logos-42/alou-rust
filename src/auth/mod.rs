// ============================================
// Authentication Module
// ============================================

pub mod jwt;
pub mod google_oauth;
pub mod middleware;

pub use jwt::{generate_token, verify_token, Claims};
pub use google_oauth::GoogleOAuth;
pub use middleware::with_auth;

