use super::common::{
    app_error, invalid_request, missing_field, resolve_environment, respond_encoded,
};
use crate::web3::clients::DiapTokenClient;
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Request, Response, Result as WorkerResult};

pub async fn handle_token_request(env: &Env, req: &mut Request) -> WorkerResult<Response> {
    let body: TokenRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => return invalid_request(e.to_string()),
    };

    let (network, contract_env) = match resolve_environment(env, &body.network) {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    let client = DiapTokenClient::new(&contract_env);

    match body.action {
        TokenAction::BalanceOf => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.balance_of(&address).await {
                Ok(balance) => super::super::json_response(&json!({
                    "network": network.as_str(),
                    "address": address,
                    "balance": balance
                })),
                Err(e) => app_error(e),
            }
        }
        TokenAction::TotalSupply => match client.total_supply().await {
            Ok(total) => super::super::json_response(&json!({
                "network": network.as_str(),
                "total_supply": total
            })),
            Err(e) => app_error(e),
        },
        TokenAction::TotalStaked => match client.total_staked().await {
            Ok(total) => super::super::json_response(&json!({
                "network": network.as_str(),
                "total_staked": total
            })),
            Err(e) => app_error(e),
        },
        TokenAction::StakingRewardRate => match client.staking_reward_rate().await {
            Ok(rate) => super::super::json_response(&json!({
                "network": network.as_str(),
                "staking_reward_rate": rate
            })),
            Err(e) => app_error(e),
        },
        TokenAction::StakingInfo => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.staking_info(&address).await {
                Ok(info) => super::super::json_response(&json!({
                    "network": network.as_str(),
                    "address": address,
                    "staking": info
                })),
                Err(e) => app_error(e),
            }
        }
        TokenAction::StakeEncode => {
            let amount = match body.amount {
                Some(value) => value,
                None => return missing_field("amount"),
            };
            let tier = body.tier.unwrap_or(0);
            respond_encoded(client.stake_call(&amount, tier))
        }
        TokenAction::ClaimRewardsEncode => respond_encoded(client.claim_rewards_call()),
        TokenAction::UnstakeEncode => respond_encoded(client.unstake_call()),
    }
}

#[derive(Deserialize)]
struct TokenRequest {
    network: String,
    action: TokenAction,
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    amount: Option<String>,
    #[serde(default)]
    tier: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TokenAction {
    BalanceOf,
    TotalSupply,
    TotalStaked,
    StakingRewardRate,
    StakingInfo,
    StakeEncode,
    ClaimRewardsEncode,
    UnstakeEncode,
}
