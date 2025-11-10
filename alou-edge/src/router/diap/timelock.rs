use super::common::{app_error, invalid_request, missing_field, resolve_environment};
use crate::web3::clients::TimelockControllerClient;
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Request, Response, Result as WorkerResult};

pub async fn handle_timelock_request(env: &Env, req: &mut Request) -> WorkerResult<Response> {
    let body: TimelockRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => return invalid_request(e.to_string()),
    };

    let (_, contract_env) = match resolve_environment(env, &body.network) {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    let client = TimelockControllerClient::new(&contract_env);

    match body.action {
        TimelockAction::MinDelay => match client.min_delay().await {
            Ok(delay) => super::super::json_response(&json!({ "min_delay": delay })),
            Err(e) => app_error(e),
        },
        TimelockAction::IsOperation => {
            let operation_id = match body.operation_id {
                Some(id) => id,
                None => return missing_field("operation_id"),
            };
            match client.is_operation(&operation_id).await {
                Ok(is_operation) => super::super::json_response(&json!({
                    "operation_id": operation_id,
                    "exists": is_operation
                })),
                Err(e) => app_error(e),
            }
        }
        TimelockAction::OperationTimestamp => {
            let operation_id = match body.operation_id {
                Some(id) => id,
                None => return missing_field("operation_id"),
            };
            match client.operation_timestamp(&operation_id).await {
                Ok(timestamp) => super::super::json_response(&json!({
                    "operation_id": operation_id,
                    "timestamp": timestamp
                })),
                Err(e) => app_error(e),
            }
        }
        TimelockAction::IsOperationReady => {
            let operation_id = match body.operation_id {
                Some(id) => id,
                None => return missing_field("operation_id"),
            };
            match client.is_operation_ready(&operation_id).await {
                Ok(ready) => super::super::json_response(&json!({
                    "operation_id": operation_id,
                    "ready": ready
                })),
                Err(e) => app_error(e),
            }
        }
    }
}

#[derive(Deserialize)]
struct TimelockRequest {
    network: String,
    action: TimelockAction,
    #[serde(default)]
    operation_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TimelockAction {
    MinDelay,
    IsOperation,
    OperationTimestamp,
    IsOperationReady,
}

