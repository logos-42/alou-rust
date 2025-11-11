use super::common::{
    app_error, invalid_request, missing_field, resolve_environment, respond_encoded,
};
use crate::web3::clients::DiapGovernanceClient;
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Request, Response, Result as WorkerResult};

pub async fn handle_governance_request(env: &Env, req: &mut Request) -> WorkerResult<Response> {
    let body: GovernanceRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => return invalid_request(e.to_string()),
    };

    let (_, contract_env) = match resolve_environment(env, &body.network) {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    let client = DiapGovernanceClient::new(&contract_env);

    match body.action {
        GovernanceAction::GetProposalInfo => {
            let proposal_id = match body.proposal_id {
                Some(id) => id,
                None => return missing_field("proposal_id"),
            };
            match client.get_proposal_info(&proposal_id).await {
                Ok(info) => super::super::json_response(&json!({ "proposal": info })),
                Err(e) => app_error(e),
            }
        }
        GovernanceAction::ProposalVotes => {
            let proposal_id = match body.proposal_id {
                Some(id) => id,
                None => return missing_field("proposal_id"),
            };
            match client.proposal_votes(&proposal_id).await {
                Ok(votes) => super::super::json_response(&json!({ "votes": votes })),
                Err(e) => app_error(e),
            }
        }
        GovernanceAction::ProposalThreshold => match client.proposal_threshold().await {
            Ok(value) => super::super::json_response(&json!({ "proposal_threshold": value })),
            Err(e) => app_error(e),
        },
        GovernanceAction::VotingPeriod => match client.voting_period().await {
            Ok(value) => super::super::json_response(&json!({ "voting_period": value })),
            Err(e) => app_error(e),
        },
        GovernanceAction::VotingDelay => match client.voting_delay().await {
            Ok(value) => super::super::json_response(&json!({ "voting_delay": value })),
            Err(e) => app_error(e),
        },
        GovernanceAction::Quorum => {
            let block_number = match body.block_number {
                Some(value) => value,
                None => return missing_field("block_number"),
            };
            match client.quorum(&block_number).await {
                Ok(value) => super::super::json_response(&json!({
                    "block_number": block_number,
                    "quorum": value
                })),
                Err(e) => app_error(e),
            }
        }
        GovernanceAction::CastVoteEncode => {
            let proposal_id = match body.proposal_id {
                Some(id) => id,
                None => return missing_field("proposal_id"),
            };
            let support = body.support.unwrap_or(0) as u8;
            respond_encoded(client.cast_vote_call(&proposal_id, support))
        }
        GovernanceAction::CastVoteWithReasonEncode => {
            let proposal_id = match body.proposal_id {
                Some(id) => id,
                None => return missing_field("proposal_id"),
            };
            let support = body.support.unwrap_or(0) as u8;
            let reason = body.reason.unwrap_or_default();
            respond_encoded(client.cast_vote_with_reason_call(&proposal_id, support, &reason))
        }
    }
}

#[derive(Deserialize)]
struct GovernanceRequest {
    network: String,
    action: GovernanceAction,
    #[serde(default)]
    proposal_id: Option<String>,
    #[serde(default)]
    block_number: Option<String>,
    #[serde(default)]
    support: Option<u64>,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum GovernanceAction {
    GetProposalInfo,
    ProposalVotes,
    ProposalThreshold,
    VotingPeriod,
    VotingDelay,
    Quorum,
    CastVoteEncode,
    CastVoteWithReasonEncode,
}
