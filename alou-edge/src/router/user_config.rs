use crate::agent::ai_client::{AiClient, AiMessage};
use crate::storage::kv::KvStore;
use crate::utils::crypto::sha256;
use crate::utils::error::{AloudError, Result};
use crate::web3::auth::WalletAuth;
use serde::{Deserialize, Serialize};
use worker::{Request, Response};

use super::{json_response, json_response_with_status, ErrorResponse};

#[derive(Serialize, Deserialize)]
struct UserConfig {
    user_id: String,
    api_key_hash: String, // SHA256 hash of API key (never store plaintext)
    provider: String,
    model: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Deserialize)]
struct SaveConfigRequest {
    api_key: String,
    provider: String,
    model: String,
}

#[derive(Serialize)]
struct ConfigResponse {
    provider: String,
    model: String,
    has_api_key: bool, // Only indicate if API key exists, never return the key
    updated_at: i64,
}

#[derive(Deserialize)]
struct VerifyApiKeyRequest {
    api_key: String,
    provider: String,
    model: String,
}

#[derive(Serialize)]
struct VerifyApiKeyResponse {
    valid: bool,
    error: Option<String>,
}

/// Extract user ID from JWT token (wallet address)
fn extract_user_id(wallet_auth: &WalletAuth, req: &Request) -> Result<String> {
    let token = req
        .headers()
        .get("Authorization")
        .map_err(|e| AloudError::WorkerError(e.to_string()))?
        .ok_or_else(|| AloudError::AuthError("Missing Authorization header".to_string()))?;

    let token = token.strip_prefix("Bearer ").unwrap_or(&token);
    let (wallet_address, _chain) = wallet_auth.parse_token(token)?;
    Ok(wallet_address)
}

/// Get user config
pub(crate) async fn handle_get_user_config(
    wallet_auth: Option<&WalletAuth>,
    kv: &KvStore,
    req: &Request,
) -> Result<Response> {
    let wallet_auth = match wallet_auth {
        Some(auth) => auth,
        None => {
            let error_response = ErrorResponse {
                error: "Wallet authentication not configured".to_string(),
            };
            return Ok(json_response_with_status(&error_response, 500)?);
        }
    };

    // Extract user ID from token
    let user_id = match extract_user_id(wallet_auth, req) {
        Ok(id) => id,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Authentication failed: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 401)?);
        }
    };

    // Get config from KV
    let key = format!("user_config:{}", user_id);
    match kv.get::<UserConfig>(&key).await {
        Ok(Some(config)) => {
            let response = ConfigResponse {
                provider: config.provider,
                model: config.model,
                has_api_key: !config.api_key_hash.is_empty(),
                updated_at: config.updated_at,
            };
            Ok(json_response(&response)?)
        }
        Ok(None) => {
            // No config found, return default
            let response = ConfigResponse {
                provider: "deepseek".to_string(),
                model: "deepseek-chat".to_string(),
                has_api_key: false,
                updated_at: 0,
            };
            Ok(json_response(&response)?)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get config: {}", e),
            };
            Ok(json_response_with_status(&error_response, 500)?)
        }
    }
}

/// Save user config
pub(crate) async fn handle_save_user_config(
    wallet_auth: Option<&WalletAuth>,
    kv: &KvStore,
    req: &mut Request,
) -> Result<Response> {
    let wallet_auth = match wallet_auth {
        Some(auth) => auth,
        None => {
            let error_response = ErrorResponse {
                error: "Wallet authentication not configured".to_string(),
            };
            return Ok(json_response_with_status(&error_response, 500)?);
        }
    };

    // Extract user ID from token
    let user_id = match extract_user_id(wallet_auth, req) {
        Ok(id) => id,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Authentication failed: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 401)?);
        }
    };

    // Parse request body
    let body: SaveConfigRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // Validate input
    if body.api_key.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "API key cannot be empty".to_string(),
        };
        return Ok(json_response_with_status(&error_response, 400)?);
    }

    if body.provider.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "Provider cannot be empty".to_string(),
        };
        return Ok(json_response_with_status(&error_response, 400)?);
    }

    if body.model.trim().is_empty() {
        let error_response = ErrorResponse {
            error: "Model cannot be empty".to_string(),
        };
        return Ok(json_response_with_status(&error_response, 400)?);
    }

    // Hash API key (never store plaintext)
    let api_key_hash = sha256(&body.api_key);

    // Get existing config or create new
    let key = format!("user_config:{}", user_id);
    let now = crate::utils::time::now_timestamp();
    let existing_config: Option<UserConfig> = kv.get(&key).await.unwrap_or(None);

    let config = UserConfig {
        user_id: user_id.clone(),
        api_key_hash,
        provider: body.provider.trim().to_string(),
        model: body.model.trim().to_string(),
        created_at: existing_config
            .as_ref()
            .map(|c| c.created_at)
            .unwrap_or(now),
        updated_at: now,
    };

    // Save to KV (no expiration, user config should persist)
    match kv.put(&key, &config, None).await {
        Ok(_) => {
            let response = ConfigResponse {
                provider: config.provider,
                model: config.model,
                has_api_key: true,
                updated_at: config.updated_at,
            };
            Ok(json_response(&response)?)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to save config: {}", e),
            };
            Ok(json_response_with_status(&error_response, 500)?)
        }
    }
}

/// Test API connection by making a simple request
async fn test_api_connection(provider: &str, api_key: &str, model: &str) -> Result<()> {
    // Create AI client
    let client = AiClient::new(provider, api_key.to_string(), Some(model.to_string()))?;
    
    // Create a simple test message
    let test_messages = vec![AiMessage::text(
        "user",
        "Hello! Please respond with just 'OK' to confirm the connection is working.".to_string(),
    )];
    
    // Make a simple request with timeout
    let response = client.send_message(test_messages, None).await?;
    
    // Check if we got a response
    if response.content.trim().is_empty() {
        return Err(AloudError::AgentError("Empty response from AI provider".to_string()));
    }
    
    // Log success for debugging
    worker::console_log!(
        "[API Test] Successfully connected to {} with model {}. Response: {}",
        provider,
        model,
        response.content
    );
    
    Ok(())
}

/// Verify API key (test connection)
pub(crate) async fn handle_verify_api_key(
    _wallet_auth: Option<&WalletAuth>,
    req: &mut Request,
) -> Result<Response> {
    // This endpoint can be called without authentication for testing
    // But if auth is provided, we can optionally log usage

    let body: VerifyApiKeyRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return Ok(json_response_with_status(&error_response, 400)?);
        }
    };

    // Basic validation
    if body.api_key.trim().is_empty() {
        return Ok(json_response(&VerifyApiKeyResponse {
            valid: false,
            error: Some("API key cannot be empty".to_string()),
        })?);
    }

    // Validate provider
    let valid_providers = ["deepseek", "openai", "claude", "qwen", "kimi"];
    if !valid_providers.contains(&body.provider.as_str()) {
        return Ok(json_response(&VerifyApiKeyResponse {
            valid: false,
            error: Some(format!("Invalid provider: {}", body.provider)),
        })?);
    }

    // Basic format validation based on provider
    let is_valid_format = match body.provider.as_str() {
        "openai" => body.api_key.starts_with("sk-") && body.api_key.len() > 20,
        "claude" => body.api_key.starts_with("sk-ant-") && body.api_key.len() > 20,
        "deepseek" => body.api_key.len() > 20,
        "qwen" => body.api_key.len() > 20,
        "kimi" => body.api_key.len() > 20,
        _ => false,
    };

    if !is_valid_format {
        return Ok(json_response(&VerifyApiKeyResponse {
            valid: false,
            error: Some("API key format appears invalid".to_string()),
        })?);
    }

    // Try to create AI client and make a simple test request
    match test_api_connection(&body.provider, &body.api_key, &body.model).await {
        Ok(_) => {
            Ok(json_response(&VerifyApiKeyResponse {
                valid: true,
                error: None,
            })?)
        }
        Err(e) => {
            Ok(json_response(&VerifyApiKeyResponse {
                valid: false,
                error: Some(format!("API connection test failed: {}", e)),
            })?)
        }
    }
}
