use super::super::{json_response_with_status, ErrorResponse};
use crate::utils::error::AloudError;
use crate::web3::clients::EncodedCall;
use crate::web3::config::load_contract_environment;
use crate::web3::contracts::{network_from_str, Network};
use worker::{Env, Response, Result as WorkerResult};

pub(crate) fn resolve_environment(
    env: &Env,
    network: &str,
) -> std::result::Result<(Network, crate::web3::config::ContractEnvironment), String> {
    let net = network_from_str(network)
        .ok_or_else(|| format!("Unsupported network value: {}", network))?;
    let config = load_contract_environment(env, net);
    Ok((net, config))
}

pub(crate) fn respond_encoded(
    result: crate::utils::error::Result<EncodedCall>,
) -> WorkerResult<Response> {
    match result {
        Ok(call) => super::super::json_response(&call),
        Err(e) => app_error(e),
    }
}

pub(crate) fn app_error(err: AloudError) -> WorkerResult<Response> {
    let status = match err {
        AloudError::InvalidInput(_) => 400,
        AloudError::AuthError(_) => 401,
        AloudError::RpcError(_) => 502,
        _ => 500,
    };
    let payload = ErrorResponse {
        error: err.to_string(),
    };
    json_response_with_status(&payload, status)
}

pub(crate) fn invalid_request(message: String) -> WorkerResult<Response> {
    json_response_with_status(
        &ErrorResponse {
            error: format!("Invalid request: {}", message),
        },
        400,
    )
}

pub(crate) fn missing_field(field: &str) -> WorkerResult<Response> {
    json_response_with_status(
        &ErrorResponse {
            error: format!("Missing required field `{}`", field),
        },
        400,
    )
}

pub(crate) fn method_not_allowed() -> WorkerResult<Response> {
    json_response_with_status(
        &ErrorResponse {
            error: "Action not allowed for selected module".to_string(),
        },
        405,
    )
}
