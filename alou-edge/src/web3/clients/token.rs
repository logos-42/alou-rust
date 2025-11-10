use crate::utils::error::{AloudError, Result};
use crate::web3::abi::diap_token_contract;
use crate::web3::clients::common::{
    token_address, token_uint, token_uint_u64, ContractClient, EncodedCall,
};
use crate::web3::config::ContractEnvironment;
use ethabi::Token;
use serde_json::Value;

pub struct DiapTokenClient {
    inner: ContractClient,
}

impl DiapTokenClient {
    pub fn new(env: &ContractEnvironment) -> Self {
        let address = env.contracts.diap_token.proxy;
        let abi = diap_token_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;

        Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        }
    }

    pub fn address(&self) -> &'static str {
        self.inner.address()
    }

    pub async fn balance_of(&self, wallet: &str) -> Result<String> {
        let result = self
            .inner
            .call("balanceOf", vec![token_address(wallet)?])
            .await?;
        extract_uint(&result.tokens, "balanceOf")
    }

    pub async fn total_supply(&self) -> Result<String> {
        let result = self.inner.call("totalSupply", vec![]).await?;
        extract_uint(&result.tokens, "totalSupply")
    }

    pub async fn total_staked(&self) -> Result<String> {
        let result = self.inner.call("totalStaked", vec![]).await?;
        extract_uint(&result.tokens, "totalStaked")
    }

    pub async fn staking_reward_rate(&self) -> Result<String> {
        let result = self.inner.call("stakingRewardRate", vec![]).await?;
        extract_uint(&result.tokens, "stakingRewardRate")
    }

    pub async fn staking_info(&self, wallet: &str) -> Result<Value> {
        let result = self
            .inner
            .call("stakingInfo", vec![token_address(wallet)?])
            .await?;
        Ok(result.json)
    }

    pub fn stake_call(&self, amount_wei: &str, tier: u64) -> Result<EncodedCall> {
        self.inner
            .encode_call(
                "stake",
                vec![token_uint(amount_wei)?, token_uint_u64(tier)],
                None,
            )
    }

    pub fn claim_rewards_call(&self) -> Result<EncodedCall> {
        self.inner.encode_call("claimRewards", vec![], None)
    }

    pub fn unstake_call(&self) -> Result<EncodedCall> {
        self.inner.encode_call("unstake", vec![], None)
    }
}

fn extract_uint(tokens: &[Token], fn_name: &str) -> Result<String> {
    match tokens.first() {
        Some(Token::Uint(value)) => Ok(value.to_string()),
        Some(other) => Err(AloudError::InvalidInput(format!(
            "`{}` returned unexpected token: {:?}",
            fn_name, other
        ))),
        None => Err(AloudError::InvalidInput(format!(
            "`{}` returned empty result",
            fn_name
        ))),
    }
}

