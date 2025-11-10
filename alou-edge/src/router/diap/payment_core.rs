use super::common::{app_error, invalid_request, missing_field, respond_encoded, resolve_environment};
use crate::web3::clients::DiapPaymentCoreClient;
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Request, Response, Result as WorkerResult};

pub async fn handle_payment_core_request(env: &Env, req: &mut Request) -> WorkerResult<Response> {
    let body: PaymentCoreRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => return invalid_request(e.to_string()),
    };

    let (_, contract_env) = match resolve_environment(env, &body.network) {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    let client = DiapPaymentCoreClient::new(&contract_env);

    match body.action {
        PaymentCoreAction::GetPayment => {
            let payment_id = match body.payment_id {
                Some(id) => id,
                None => return missing_field("payment_id"),
            };
            match client.get_payment(&payment_id).await {
                Ok(payment) => super::super::json_response(&json!({ "payment": payment })),
                Err(e) => app_error(e),
            }
        }
        PaymentCoreAction::GetServiceOrder => {
            let service_id = match body.service_id {
                Some(id) => id,
                None => return missing_field("service_id"),
            };
            match client.get_service_order(&service_id).await {
                Ok(service) => super::super::json_response(&json!({ "service": service })),
                Err(e) => app_error(e),
            }
        }
        PaymentCoreAction::PaymentStats => match client.payment_stats().await {
            Ok(stats) => super::super::json_response(&json!({ "stats": stats })),
            Err(e) => app_error(e),
        },
        PaymentCoreAction::CreatePaymentEncode => {
            let to = match body.to {
                Some(addr) => addr,
                None => return missing_field("to"),
            };
            let amount = match body.amount {
                Some(value) => value,
                None => return missing_field("amount"),
            };
            let payment_id = match body.payment_id {
                Some(id) => id,
                None => return missing_field("payment_id"),
            };
            let description = body.description.unwrap_or_default();
            let metadata = body.metadata.unwrap_or_default();
            respond_encoded(client.create_payment_call(
                &to,
                &amount,
                &payment_id,
                &description,
                &metadata,
            ))
        }
        PaymentCoreAction::ConfirmPaymentEncode => {
            let payment_id = match body.payment_id {
                Some(id) => id,
                None => return missing_field("payment_id"),
            };
            respond_encoded(client.confirm_payment_call(&payment_id))
        }
        PaymentCoreAction::CancelPaymentEncode => {
            let payment_id = match body.payment_id {
                Some(id) => id,
                None => return missing_field("payment_id"),
            };
            respond_encoded(client.cancel_payment_call(&payment_id))
        }
        PaymentCoreAction::CreateServiceOrderEncode => {
            let provider = match body.to {
                Some(addr) => addr,
                None => return missing_field("to"),
            };
            let service_cid = match body.service_type {
                Some(cid) => cid,
                None => return missing_field("service_type"),
            };
            let price = match body.amount {
                Some(price) => price,
                None => return missing_field("amount"),
            };
            respond_encoded(client.create_service_order_call(&provider, &service_cid, &price))
        }
    }
}

#[derive(Deserialize)]
struct PaymentCoreRequest {
    network: String,
    action: PaymentCoreAction,
    #[serde(default)]
    payment_id: Option<String>,
    #[serde(default)]
    service_id: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    amount: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    metadata: Option<String>,
    #[serde(default)]
    service_type: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum PaymentCoreAction {
    GetPayment,
    GetServiceOrder,
    PaymentStats,
    CreatePaymentEncode,
    ConfirmPaymentEncode,
    CancelPaymentEncode,
    CreateServiceOrderEncode,
}

