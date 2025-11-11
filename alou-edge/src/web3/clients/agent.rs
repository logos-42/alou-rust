use crate::utils::error::{AloudError, Result};
use crate::web3::abi::diap_agent_network_contract;
use crate::web3::clients::common::{
    token_address, token_string, token_uint, token_uint_u64, ContractClient, EncodedCall,
};
use crate::web3::config::ContractEnvironment;
use ethabi::Token;
use serde_json::Value;

pub struct DiapAgentNetworkClient {
    inner: ContractClient,
}

impl DiapAgentNetworkClient {
    pub fn new(env: &ContractEnvironment) -> Self {
        let address = env.contracts.diap_agent_network.proxy;
        let abi = diap_agent_network_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;
        Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        }
    }

    #[allow(dead_code)]
    pub fn address(&self) -> &str {
        self.inner.address()
    }

    pub async fn get_agent(&self, agent: &str) -> Result<Value> {
        let result = self
            .inner
            .call("getAgent", vec![token_address(agent)?])
            .await?;
        Ok(result.json)
    }

    pub async fn registration_fee(&self) -> Result<String> {
        let result = self.inner.call("registrationFee", vec![]).await?;
        extract_uint(&result.tokens, "registrationFee")
    }

    pub async fn min_stake_amount(&self) -> Result<String> {
        let result = self.inner.call("minStakeAmount", vec![]).await?;
        extract_uint(&result.tokens, "minStakeAmount")
    }

    pub async fn reputation_threshold(&self) -> Result<String> {
        let result = self.inner.call("reputationThreshold", vec![]).await?;
        extract_uint(&result.tokens, "reputationThreshold")
    }

    pub async fn get_agent_identifier_type(&self, agent: &str) -> Result<String> {
        let result = self
            .inner
            .call("getAgentIdentifierType", vec![token_address(agent)?])
            .await?;
        match result.tokens.first() {
            Some(Token::Uint(value)) => Ok(value.to_string()),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "getAgentIdentifierType returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "getAgentIdentifierType returned empty result".into(),
            )),
        }
    }

    pub async fn get_agent_metadata(&self, agent: &str) -> Result<String> {
        let result = self
            .inner
            .call("agentMetadataCID", vec![token_address(agent)?])
            .await?;
        match result.tokens.first() {
            Some(Token::String(value)) => Ok(value.clone()),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "agentMetadataCID returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "agentMetadataCID returned empty result".into(),
            )),
        }
    }

    pub fn register_agent_call(
        &self,
        did_document: &str,
        public_key: &str,
        stake_amount: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "registerAgent",
            vec![
                token_string(did_document),
                token_string(public_key),
                token_uint(stake_amount)?,
            ],
            None,
        )
    }

    pub fn register_agent_with_aa_call(
        &self,
        did_document: &str,
        public_key: &str,
        stake_amount: &str,
        salt: u64,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "registerAgentWithAA",
            vec![
                token_string(did_document),
                token_string(public_key),
                token_uint(stake_amount)?,
                token_uint_u64(salt),
            ],
            None,
        )
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
