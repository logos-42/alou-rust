use super::common::{
    app_error, invalid_request, missing_field, resolve_environment, respond_encoded,
};
use crate::web3::clients::DiapAgentNetworkClient;
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Request, Response, Result as WorkerResult};

pub async fn handle_agent_request(env: &Env, req: &mut Request) -> WorkerResult<Response> {
    let body: AgentRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => return invalid_request(e.to_string()),
    };

    let (_, contract_env) = match resolve_environment(env, &body.network) {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    let client = DiapAgentNetworkClient::new(&contract_env);

    match body.action {
        AgentAction::GetAgent => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.get_agent(&address).await {
                Ok(agent) => super::super::json_response(&json!({ "agent": agent })),
                Err(e) => app_error(e),
            }
        }
        AgentAction::RegistrationFee => match client.registration_fee().await {
            Ok(fee) => super::super::json_response(&json!({ "registration_fee": fee })),
            Err(e) => app_error(e),
        },
        AgentAction::MinStakeAmount => match client.min_stake_amount().await {
            Ok(min) => super::super::json_response(&json!({ "min_stake_amount": min })),
            Err(e) => app_error(e),
        },
        AgentAction::ReputationThreshold => match client.reputation_threshold().await {
            Ok(threshold) => {
                super::super::json_response(&json!({ "reputation_threshold": threshold }))
            }
            Err(e) => app_error(e),
        },
        AgentAction::IdentifierType => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.get_agent_identifier_type(&address).await {
                Ok(identifier) => super::super::json_response(&json!({
                    "address": address,
                    "identifier_type": identifier
                })),
                Err(e) => app_error(e),
            }
        }
        AgentAction::MetadataCid => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.get_agent_metadata(&address).await {
                Ok(cid) => super::super::json_response(&json!({
                    "address": address,
                    "metadata_cid": cid
                })),
                Err(e) => app_error(e),
            }
        }
        AgentAction::RegisterEncode => {
            let did = match body.did_document {
                Some(did) => did,
                None => return missing_field("did_document"),
            };
            let public_key = match body.public_key {
                Some(pk) => pk,
                None => return missing_field("public_key"),
            };
            let stake_amount = match body.stake_amount {
                Some(amount) => amount,
                None => return missing_field("stake_amount"),
            };
            respond_encoded(client.register_agent_call(&did, &public_key, &stake_amount))
        }
        AgentAction::RegisterAaEncode => {
            let did = match body.did_document {
                Some(did) => did,
                None => return missing_field("did_document"),
            };
            let public_key = match body.public_key {
                Some(pk) => pk,
                None => return missing_field("public_key"),
            };
            let stake_amount = match body.stake_amount {
                Some(amount) => amount,
                None => return missing_field("stake_amount"),
            };
            let salt = body.salt.unwrap_or(0);
            respond_encoded(client.register_agent_with_aa_call(
                &did,
                &public_key,
                &stake_amount,
                salt,
            ))
        }
    }
}

#[derive(Deserialize)]
struct AgentRequest {
    network: String,
    action: AgentAction,
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    did_document: Option<String>,
    #[serde(default)]
    public_key: Option<String>,
    #[serde(default)]
    stake_amount: Option<String>,
    #[serde(default)]
    salt: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AgentAction {
    GetAgent,
    RegistrationFee,
    MinStakeAmount,
    ReputationThreshold,
    IdentifierType,
    MetadataCid,
    RegisterEncode,
    RegisterAaEncode,
}
