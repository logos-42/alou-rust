use crate::utils::error::{AloudError, Result};
use crate::web3::abi::{diap_governance_contract, timelock_controller_contract};
use crate::web3::clients::common::{
    token_fixed_bytes32, token_string, token_uint, token_uint_u64, ContractClient, EncodedCall,
};
use crate::web3::config::ContractEnvironment;
use ethabi::Token;
use serde_json::Value;

pub struct DiapGovernanceClient {
    inner: ContractClient,
}

impl DiapGovernanceClient {
    pub fn new(env: &ContractEnvironment) -> Self {
        let address = env.contracts.diap_governance.address;
        let abi = diap_governance_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;

        Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        }
    }

    pub fn address(&self) -> &'static str {
        self.inner.address()
    }

    pub async fn get_proposal_info(&self, proposal_id: &str) -> Result<Value> {
        let result = self
            .inner
            .call("getProposalInfo", vec![token_uint(proposal_id)?])
            .await?;
        Ok(result.json)
    }

    pub async fn proposal_votes(&self, proposal_id: &str) -> Result<Value> {
        let result = self
            .inner
            .call("proposalVotes", vec![token_uint(proposal_id)?])
            .await?;
        Ok(result.json)
    }

    pub async fn proposal_threshold(&self) -> Result<String> {
        let result = self.inner.call("proposalThreshold", vec![]).await?;
        extract_uint(&result.tokens, "proposalThreshold")
    }

    pub async fn voting_period(&self) -> Result<String> {
        let result = self.inner.call("votingPeriod", vec![]).await?;
        extract_uint(&result.tokens, "votingPeriod")
    }

    pub async fn voting_delay(&self) -> Result<String> {
        let result = self.inner.call("votingDelay", vec![]).await?;
        extract_uint(&result.tokens, "votingDelay")
    }

    pub async fn quorum(&self, block_number: &str) -> Result<String> {
        let result = self
            .inner
            .call("quorum", vec![token_uint(block_number)?])
            .await?;
        extract_uint(&result.tokens, "quorum")
    }

    pub fn cast_vote_call(&self, proposal_id: &str, support: u8) -> Result<EncodedCall> {
        self.inner.encode_call(
            "castVote",
            vec![token_uint(proposal_id)?, token_uint_u64(support as u64)],
            None,
        )
    }

    pub fn cast_vote_with_reason_call(
        &self,
        proposal_id: &str,
        support: u8,
        reason: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "castVoteWithReason",
            vec![
                token_uint(proposal_id)?,
                token_uint_u64(support as u64),
                token_string(reason),
            ],
            None,
        )
    }
}

pub struct TimelockControllerClient {
    inner: ContractClient,
}

impl TimelockControllerClient {
    pub fn new(env: &ContractEnvironment) -> Self {
        let address = env.contracts.timelock_controller.address;
        let abi = timelock_controller_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;

        Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        }
    }

    pub fn address(&self) -> &'static str {
        self.inner.address()
    }

    pub async fn min_delay(&self) -> Result<String> {
        let result = self.inner.call("getMinDelay", vec![]).await?;
        extract_uint(&result.tokens, "getMinDelay")
    }

    pub async fn is_operation(&self, operation_id: &str) -> Result<bool> {
        let result = self
            .inner
            .call("isOperation", vec![token_fixed_bytes32(operation_id)?])
            .await?;
        match result.tokens.first() {
            Some(Token::Bool(value)) => Ok(*value),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "isOperation returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "isOperation returned empty result".into(),
            )),
        }
    }

    pub async fn operation_timestamp(&self, operation_id: &str) -> Result<String> {
        let result = self
            .inner
            .call(
                "getTimestamp",
                vec![token_fixed_bytes32(operation_id)?],
            )
            .await?;
        extract_uint(&result.tokens, "getTimestamp")
    }

    pub async fn is_operation_ready(&self, operation_id: &str) -> Result<bool> {
        let result = self
            .inner
            .call(
                "isOperationReady",
                vec![token_fixed_bytes32(operation_id)?],
            )
            .await?;
        match result.tokens.first() {
            Some(Token::Bool(value)) => Ok(*value),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "isOperationReady returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "isOperationReady returned empty result".into(),
            )),
        }
    }
}

fn extract_uint(tokens: &[Token], name: &str) -> Result<String> {
    match tokens.first() {
        Some(Token::Uint(value)) => Ok(value.to_string()),
        Some(other) => Err(AloudError::InvalidInput(format!(
            "`{}` returned unexpected token: {:?}",
            name, other
        ))),
        None => Err(AloudError::InvalidInput(format!(
            "`{}` returned empty result",
            name
        ))),
    }
}

