// ============================================
// Google OAuth 2.0 Implementation
// ============================================

use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    TokenUrl, basic::BasicClient, reqwest::async_http_client, TokenResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUser {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Clone)]
pub struct GoogleOAuth {
    client: BasicClient,
}

impl GoogleOAuth {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Result<Self, Box<dyn std::error::Error>> {
        let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
        let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?;

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        Ok(Self { client })
    }

    /// Generate authorization URL
    pub fn get_auth_url(&self) -> (String, CsrfToken) {
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(oauth2::Scope::new("openid".to_string()))
            .add_scope(oauth2::Scope::new("email".to_string()))
            .add_scope(oauth2::Scope::new("profile".to_string()))
            .url();

        (auth_url.to_string(), csrf_token)
    }

    /// Exchange authorization code for access token and fetch user info
    pub async fn exchange_code(&self, code: String) -> Result<(String, GoogleUser), Box<dyn std::error::Error>> {
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await?;

        let access_token = token.access_token().secret().clone();

        // Fetch user info from Google
        let user_info = self.fetch_user_info(&access_token).await?;

        Ok((access_token, user_info))
    }

    /// Fetch user information from Google
    async fn fetch_user_info(&self, access_token: &str) -> Result<GoogleUser, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await?;

        let user_info: GoogleUser = response.json().await?;
        Ok(user_info)
    }
}

