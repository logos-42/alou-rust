use crate::agent::context::AgentContext;
use crate::agent::session::SessionManager;
use crate::mcp::registry::McpTool;
use crate::mcp::tools::AgentWalletTool;
use crate::utils::error::AloudError;
use crate::web3::auth::WalletAuth;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::{Request, Response, Result};

use super::{json_response, json_response_with_status, ErrorResponse};

#[derive(Serialize)]
struct NonceResponse {
    nonce: String,
    message: String,
}

#[derive(Deserialize)]
struct VerifySignatureRequest {
    address: String,
    signature: String,
    message: String,
    chain: String,
}

#[derive(Serialize)]
struct VerifySignatureResponse {
    success: bool,
    token: String,
    wallet_address: String,
    chain: String,
}

#[derive(Serialize)]
struct WalletInfoResponse {
    wallet_address: String,
    chain: String,
}

#[derive(Deserialize)]
struct AgentWalletRequest {
    session_id: String,
    action: String,
    chain: Option<String>,
    transaction: Option<Value>,
    balance: Option<String>,
}

pub(crate) async fn handle_get_nonce(
    wallet_auth: Option<&WalletAuth>,
    address: &str,
) -> Result<Response> {
    let wallet_auth = match wallet_auth {
        Some(auth) => auth,
        None => {
            let error_response = ErrorResponse {
                error: "Wallet authentication not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    match wallet_auth.generate_nonce_for_address(address).await {
        Ok(nonce) => {
            let response = NonceResponse {
                nonce: nonce.clone(),
                message: format!("Sign this message to authenticate: {}", nonce),
            };
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

pub(crate) async fn handle_verify_signature(
    wallet_auth: Option<&WalletAuth>,
    req: &mut Request,
) -> Result<Response> {
    let wallet_auth = match wallet_auth {
        Some(auth) => auth,
        None => {
            let error_response = ErrorResponse {
                error: "Wallet authentication not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let body: VerifySignatureRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    let chain = match crate::utils::crypto::ChainType::from_str(&body.chain) {
        Ok(chain) => chain,
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    match wallet_auth
        .verify_and_create_token(&body.address, &body.signature, &body.message, chain)
        .await
    {
        Ok(token) => {
            let response = VerifySignatureResponse {
                success: true,
                token,
                wallet_address: body.address,
                chain: body.chain,
            };
            json_response(&response)
        }
        Err(AloudError::InvalidSignature) => {
            let error_response = ErrorResponse {
                error: "Invalid signature".to_string(),
            };
            json_response_with_status(&error_response, 401)
        }
        Err(AloudError::NonceExpired) => {
            let error_response = ErrorResponse {
                error: "Nonce expired".to_string(),
            };
            json_response_with_status(&error_response, 401)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

pub(crate) async fn handle_get_wallet_info(
    wallet_auth: Option<&WalletAuth>,
    req: &Request,
) -> Result<Response> {
    let wallet_auth = match wallet_auth {
        Some(auth) => auth,
        None => {
            let error_response = ErrorResponse {
                error: "Wallet authentication not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let token = match req.headers().get("Authorization")? {
        Some(auth_header) => auth_header
            .strip_prefix("Bearer ")
            .unwrap_or(&auth_header)
            .to_string(),
        None => {
            let error_response = ErrorResponse {
                error: "Missing Authorization header".to_string(),
            };
            return json_response_with_status(&error_response, 401);
        }
    };

    match wallet_auth.parse_token(&token) {
        Ok((wallet_address, chain)) => {
            let response = WalletInfoResponse {
                wallet_address,
                chain: chain.as_str().to_string(),
            };
            json_response(&response)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid token: {}", e),
            };
            json_response_with_status(&error_response, 401)
        }
    }
}

pub(crate) async fn handle_agent_wallet(
    session_manager: &SessionManager,
    agent_wallet_tool: &AgentWalletTool,
    req: &mut Request,
) -> Result<Response> {
    let body: AgentWalletRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    let mut args_map = serde_json::Map::new();
    args_map.insert("action".to_string(), json!(body.action));

    if let Some(chain) = body.chain.as_ref() {
        args_map.insert("chain".to_string(), json!(chain));
    }

    if let Some(transaction) = body.transaction {
        args_map.insert("transaction".to_string(), transaction);
    }

    if let Some(balance) = body.balance {
        args_map.insert("balance".to_string(), json!(balance));
    }

    let args = Value::Object(args_map);
    let mut context = AgentContext::new(body.session_id.clone());
    context.chain = body.chain;

    if context.chain.is_none() {
        if let Ok(session) = session_manager.get_session(&body.session_id).await {
            context.chain = session.chain;
            if context.wallet_address.is_none() {
                context.wallet_address = session.wallet_address;
            }
        }
    }

    match agent_wallet_tool.execute(args, &context).await {
        Ok(result) => json_response(&result),
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

pub(crate) fn extract_wallet_from_token(
    wallet_auth: &WalletAuth,
    req: &Request,
) -> std::result::Result<String, AloudError> {
    let token = req
        .headers()
        .get("Authorization")
        .map_err(|e| AloudError::WorkerError(e.to_string()))?
        .ok_or_else(|| AloudError::AuthError("Missing Authorization header".to_string()))?;

    let token = token.strip_prefix("Bearer ").unwrap_or(&token);

    let (wallet_address, _chain) = wallet_auth.parse_token(token)?;
    Ok(wallet_address)
}
