use crate::agent::tools::{BroadcastTool, QueryTool, TransactionTool};
use crate::utils::error::AloudError;
use crate::web3::tokens::{all_tokens, supported_symbols, tokens_for_chain};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::result::Result as StdResult;
use worker::{Request, Response, Result};

use super::{json_response, json_response_with_status, ErrorResponse};

#[derive(Deserialize)]
struct BalanceRequest {
    address: String,
    chain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_address: Option<String>,
}

#[derive(Serialize)]
struct BalanceResponse {
    address: String,
    chain: String,
    balance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_balance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decimals: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct BuildTxRequest {
    from: String,
    to: String,
    value: f64,
    chain: String,
}

#[derive(Deserialize)]
struct BroadcastRequest {
    signed_tx: String,
    chain: String,
}

#[derive(Serialize)]
struct BroadcastResponse {
    tx_hash: String,
    chain: String,
}

#[derive(Serialize)]
struct TokenListResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<String>,
    tokens: Vec<TokenInfoResponse>,
    supported_symbols: Vec<String>,
}

#[derive(Serialize)]
struct TokenInfoResponse {
    symbol: String,
    name: String,
    decimals: u8,
    chain: String,
    address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_testnet: Option<bool>,
}

pub(crate) async fn handle_blockchain_balance(
    query_tool: Option<&QueryTool>,
    req: &mut Request,
) -> Result<Response> {
    let query_tool = match query_tool {
        Some(tool) => tool,
        None => {
            let error_response = ErrorResponse {
                error: "Blockchain tools not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let body: BalanceRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    let BalanceRequest {
        address,
        chain,
        token_address,
    } = body;

    let chain_lower = chain.to_lowercase();
    let chain_hint = Some(chain.as_str());

    let response: StdResult<BalanceResponse, AloudError> =
        if chain_lower == "eth" || chain_lower == "ethereum" {
            if let Some(token_addr) = token_address.clone() {
                match query_tool
                    .get_erc20_balance(&token_addr, &address, chain_hint)
                    .await
                {
                    Ok(balance) => Ok(BalanceResponse {
                        address,
                        chain,
                        balance: balance
                            .normalized_balance
                            .clone()
                            .unwrap_or_else(|| balance.balance.clone()),
                        raw_balance: Some(balance.balance),
                        token_address: Some(balance.token_address),
                        token_symbol: balance.symbol,
                        token_name: balance.name,
                        decimals: balance.decimals,
                        timestamp: Some(balance.timestamp),
                    }),
                    Err(e) => Err(e),
                }
            } else {
                match query_tool.get_eth_balance(&address, chain_hint).await {
                    Ok(balance) => Ok(BalanceResponse {
                        address,
                        chain,
                        balance,
                        raw_balance: None,
                        token_address: None,
                        token_symbol: None,
                        token_name: None,
                        decimals: None,
                        timestamp: None,
                    }),
                    Err(e) => Err(e),
                }
            }
        } else if chain_lower == "sol" || chain_lower == "solana" {
            match query_tool.get_sol_balance(&address).await {
                Ok(balance) => Ok(BalanceResponse {
                    address,
                    chain,
                    balance,
                    raw_balance: None,
                    token_address: None,
                    token_symbol: None,
                    token_name: None,
                    decimals: None,
                    timestamp: None,
                }),
                Err(e) => Err(e),
            }
        } else {
            Err(AloudError::InvalidInput(format!(
                "Unsupported chain: {}",
                chain
            )))
        };

    match response {
        Ok(balance_response) => json_response(&balance_response),
        Err(AloudError::InvalidInput(msg)) => {
            let error_response = ErrorResponse { error: msg };
            json_response_with_status(&error_response, 400)
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}

pub(crate) async fn handle_blockchain_tokens(req: &Request) -> Result<Response> {
    let url = req.url()?;
    let mut chain: Option<String> = None;

    for (key, value) in url.query_pairs() {
        if key.eq_ignore_ascii_case("chain") {
            let normalized = value.trim().to_string();
            if !normalized.is_empty() {
                chain = Some(normalized);
            }
        }
    }

    let tokens_iter = chain
        .as_deref()
        .map(tokens_for_chain)
        .unwrap_or_else(|| all_tokens().iter().collect());

    let tokens: Vec<TokenInfoResponse> = tokens_iter
        .into_iter()
        .map(|token| TokenInfoResponse {
            symbol: token.symbol.to_string(),
            name: token.name.to_string(),
            decimals: token.decimals,
            chain: token.chain.to_string(),
            address: token.address.to_string(),
            is_testnet: token.is_testnet.then_some(true),
        })
        .collect();

    let supported = supported_symbols(chain.as_deref())
        .into_iter()
        .map(|symbol| symbol.to_string())
        .collect();

    let response = TokenListResponse {
        chain,
        tokens,
        supported_symbols: supported,
    };

    json_response(&response)
}

pub(crate) async fn handle_build_transaction(
    transaction_tool: Option<&TransactionTool>,
    req: &mut Request,
) -> Result<Response> {
    let tx_tool = match transaction_tool {
        Some(tool) => tool,
        None => {
            let error_response = ErrorResponse {
                error: "Transaction tools not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let body: BuildTxRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    let tx_data = match body.chain.to_lowercase().as_str() {
        "eth" | "ethereum" => match tx_tool
            .build_eth_transaction(&body.from, &body.to, body.value, Some(body.chain.as_str()))
            .await
        {
            Ok(tx) => serde_json::to_value(&tx).unwrap_or(Value::Null),
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        },
        "sol" | "solana" => match tx_tool
            .build_sol_transaction(&body.from, &body.to, body.value)
            .await
        {
            Ok(tx) => serde_json::to_value(&tx).unwrap_or(Value::Null),
            Err(e) => {
                let error_response = ErrorResponse {
                    error: e.to_string(),
                };
                return json_response_with_status(&error_response, 500);
            }
        },
        _ => {
            let error_response = ErrorResponse {
                error: format!("Unsupported chain: {}", body.chain),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    json_response(&tx_data)
}

pub(crate) async fn handle_broadcast_transaction(
    broadcast_tool: Option<&BroadcastTool>,
    req: &mut Request,
) -> Result<Response> {
    let broadcast_tool = match broadcast_tool {
        Some(tool) => tool,
        None => {
            let error_response = ErrorResponse {
                error: "Broadcast tools not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let body: BroadcastRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Invalid request body: {}", e),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    let tx_hash = match body.chain.to_lowercase().as_str() {
        "eth" | "ethereum" => {
            broadcast_tool
                .broadcast_eth_transaction(&body.signed_tx, Some(body.chain.as_str()))
                .await
        }
        "sol" | "solana" => {
            broadcast_tool
                .broadcast_sol_transaction(&body.signed_tx)
                .await
        }
        _ => {
            let error_response = ErrorResponse {
                error: format!("Unsupported chain: {}", body.chain),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    match tx_hash {
        Ok(tx_hash) => {
            let response = BroadcastResponse {
                tx_hash,
                chain: body.chain,
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

pub(crate) async fn handle_get_transaction_status(
    broadcast_tool: Option<&BroadcastTool>,
    tx_hash: &str,
    req: &Request,
) -> Result<Response> {
    let broadcast_tool = match broadcast_tool {
        Some(tool) => tool,
        None => {
            let error_response = ErrorResponse {
                error: "Broadcast tools not configured".to_string(),
            };
            return json_response_with_status(&error_response, 500);
        }
    };

    let url = req.url()?;
    let chain = url
        .query_pairs()
        .find(|(k, _)| k == "chain")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "eth".to_string());

    let receipt = match chain.to_lowercase().as_str() {
        "eth" | "ethereum" => {
            broadcast_tool
                .get_eth_transaction_receipt(tx_hash, Some(chain.as_str()))
                .await
        }
        "sol" | "solana" => broadcast_tool.get_sol_transaction_status(tx_hash).await,
        _ => {
            let error_response = ErrorResponse {
                error: format!("Unsupported chain: {}", chain),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    match receipt {
        Ok(receipt) => json_response(&receipt),
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            json_response_with_status(&error_response, 500)
        }
    }
}
