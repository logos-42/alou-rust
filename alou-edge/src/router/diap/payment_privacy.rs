use super::common::{app_error, invalid_request, missing_field, respond_encoded, resolve_environment};
use crate::web3::clients::DiapPaymentPrivacyClient;
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Request, Response, Result as WorkerResult};

pub async fn handle_payment_privacy_request(
    env: &Env,
    req: &mut Request,
) -> WorkerResult<Response> {
    let body: PaymentPrivacyRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => return invalid_request(e.to_string()),
    };

    let (_, contract_env) = match resolve_environment(env, &body.network) {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    let client = DiapPaymentPrivacyClient::new(&contract_env);

    match body.action {
        PaymentPrivacyAction::GetCommitmentInfo => {
            let commitment = match body.commitment {
                Some(value) => value,
                None => return missing_field("commitment"),
            };
            match client.get_commitment_info(&commitment).await {
                Ok(info) => super::super::json_response(&json!({ "commitment": info })),
                Err(e) => app_error(e),
            }
        }
        PaymentPrivacyAction::LockFundsEncode => {
            let commitment = match body.commitment {
                Some(value) => value,
                None => return missing_field("commitment"),
            };
            let amount = match body.amount {
                Some(amount) => amount,
                None => return missing_field("amount"),
            };
            respond_encoded(client.lock_funds_call(&commitment, &amount))
        }
        PaymentPrivacyAction::ExecutePaymentEncode => {
            let commitment = match body.commitment {
                Some(value) => value,
                None => return missing_field("commitment"),
            };
            let nullifier = match body.nullifier {
                Some(value) => value,
                None => return missing_field("nullifier"),
            };
            let proof_values = body.proof.unwrap_or_default();
            let recipient = match body.to {
                Some(addr) => addr,
                None => return missing_field("to"),
            };
            let amount = match body.amount {
                Some(amount) => amount,
                None => return missing_field("amount"),
            };
            let encoded = client.execute_privacy_payment_call(
                &commitment,
                &nullifier,
                proof_values.as_slice(),
                &recipient,
                &amount,
            );
            respond_encoded(encoded)
        }
        PaymentPrivacyAction::WithdrawFundsEncode => {
            let commitment = match body.commitment {
                Some(value) => value,
                None => return missing_field("commitment"),
            };
            respond_encoded(client.withdraw_locked_funds_call(&commitment))
        }
        PaymentPrivacyAction::RefundCommitmentEncode => {
            let commitment = match body.commitment {
                Some(value) => value,
                None => return missing_field("commitment"),
            };
            respond_encoded(client.refund_expired_commitment_call(&commitment))
        }
    }
}

#[derive(Deserialize)]
struct PaymentPrivacyRequest {
    network: String,
    action: PaymentPrivacyAction,
    #[serde(default)]
    commitment: Option<String>,
    #[serde(default)]
    nullifier: Option<String>,
    #[serde(default)]
    amount: Option<String>,
    #[serde(default)]
    proof: Option<Vec<String>>,
    #[serde(default)]
    to: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum PaymentPrivacyAction {
    GetCommitmentInfo,
    LockFundsEncode,
    ExecutePaymentEncode,
    WithdrawFundsEncode,
    RefundCommitmentEncode,
}

