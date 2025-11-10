use super::common::{app_error, invalid_request, missing_field, respond_encoded, resolve_environment};
use crate::web3::clients::DiapPaymentChannelClient;
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Request, Response, Result as WorkerResult};

pub async fn handle_payment_channel_request(
    env: &Env,
    req: &mut Request,
) -> WorkerResult<Response> {
    let body: PaymentChannelRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => return invalid_request(e.to_string()),
    };

    let (_, contract_env) = match resolve_environment(env, &body.network) {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    let client = DiapPaymentChannelClient::new(&contract_env);

    match body.action {
        PaymentChannelAction::GetChannel => {
            let channel_id = match body.channel_id {
                Some(id) => id,
                None => return missing_field("channel_id"),
            };
            match client.get_channel(&channel_id).await {
                Ok(channel) => super::super::json_response(&json!({ "channel": channel })),
                Err(e) => app_error(e),
            }
        }
        PaymentChannelAction::ChannelFeeRate => match client.channel_fee_rate().await {
            Ok(rate) => super::super::json_response(&json!({ "channel_fee_rate": rate })),
            Err(e) => app_error(e),
        },
        PaymentChannelAction::OpenChannelEncode => {
            let participant2 = match body.participant {
                Some(addr) => addr,
                None => return missing_field("participant"),
            };
            let deposit = match body.amount {
                Some(amount) => amount,
                None => return missing_field("amount"),
            };
            let channel_id = match body.channel_id {
                Some(id) => id,
                None => return missing_field("channel_id"),
            };
            respond_encoded(client.open_channel_call(&participant2, &deposit, &channel_id))
        }
        PaymentChannelAction::InitiateCloseEncode => {
            let channel_id = match body.channel_id {
                Some(id) => id,
                None => return missing_field("channel_id"),
            };
            let final_balance1 = match body.balance1 {
                Some(value) => value,
                None => return missing_field("balance1"),
            };
            let final_balance2 = match body.balance2 {
                Some(value) => value,
                None => return missing_field("balance2"),
            };
            let nonce = match body.nonce {
                Some(value) => value,
                None => return missing_field("nonce"),
            };
            let signature1 = match body.signature1 {
                Some(sig) => sig,
                None => return missing_field("signature1"),
            };
            let signature2 = match body.signature2 {
                Some(sig) => sig,
                None => return missing_field("signature2"),
            };
            respond_encoded(client.initiate_channel_close_call(
                &channel_id,
                &final_balance1,
                &final_balance2,
                &nonce,
                &signature1,
                &signature2,
            ))
        }
        PaymentChannelAction::ChallengeCloseEncode => {
            let channel_id = match body.channel_id {
                Some(id) => id,
                None => return missing_field("channel_id"),
            };
            let new_balance1 = match body.balance1 {
                Some(value) => value,
                None => return missing_field("balance1"),
            };
            let new_balance2 = match body.balance2 {
                Some(value) => value,
                None => return missing_field("balance2"),
            };
            let new_nonce = match body.nonce {
                Some(value) => value,
                None => return missing_field("nonce"),
            };
            let signature1 = match body.signature1 {
                Some(sig) => sig,
                None => return missing_field("signature1"),
            };
            let signature2 = match body.signature2 {
                Some(sig) => sig,
                None => return missing_field("signature2"),
            };
            respond_encoded(client.challenge_channel_close_call(
                &channel_id,
                &new_balance1,
                &new_balance2,
                &new_nonce,
                &signature1,
                &signature2,
            ))
        }
        PaymentChannelAction::FinalizeCloseEncode => {
            let channel_id = match body.channel_id {
                Some(id) => id,
                None => return missing_field("channel_id"),
            };
            respond_encoded(client.finalize_channel_close_call(&channel_id))
        }
    }
}

#[derive(Deserialize)]
struct PaymentChannelRequest {
    network: String,
    action: PaymentChannelAction,
    #[serde(default)]
    channel_id: Option<String>,
    #[serde(default)]
    participant: Option<String>,
    #[serde(default)]
    amount: Option<String>,
    #[serde(default)]
    balance1: Option<String>,
    #[serde(default)]
    balance2: Option<String>,
    #[serde(default)]
    nonce: Option<String>,
    #[serde(default)]
    signature1: Option<String>,
    #[serde(default)]
    signature2: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum PaymentChannelAction {
    GetChannel,
    ChannelFeeRate,
    OpenChannelEncode,
    InitiateCloseEncode,
    ChallengeCloseEncode,
    FinalizeCloseEncode,
}

