use crate::agent::tools::{BroadcastTool, QueryTool, TransactionTool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

    let chain_hint = Some(body.chain.as_str());
    let balance = match body.chain.to_lowercase().as_str() {
        "eth" | "ethereum" => {
            if let Some(token_addr) = body.token_address {
                query_tool
                    .get_erc20_balance(&token_addr, &body.address, chain_hint)
                    .await
            } else {
                query_tool.get_eth_balance(&body.address, chain_hint).await
            }
        }
        "sol" | "solana" => query_tool.get_sol_balance(&body.address).await,
        _ => {
            let error_response = ErrorResponse {
                error: format!("Unsupported chain: {}", body.chain),
            };
            return json_response_with_status(&error_response, 400);
        }
    };

    match balance {
        Ok(balance) => {
            let response = BalanceResponse {
                address: body.address,
                chain: body.chain,
                balance,
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
