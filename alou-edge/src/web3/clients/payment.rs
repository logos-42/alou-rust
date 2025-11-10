use crate::utils::error::{AloudError, Result};
use crate::web3::abi::{
    diap_payment_channel_contract, diap_payment_core_contract, diap_payment_privacy_contract,
};
use crate::web3::clients::common::{
    token_address, token_bytes_from_hex, token_fixed_bytes32, token_string, token_uint,
    ContractClient, EncodedCall,
};
use crate::web3::config::ContractEnvironment;
use ethabi::Token;
use serde_json::Value;

pub struct DiapPaymentCoreClient {
    inner: ContractClient,
}

impl DiapPaymentCoreClient {
    pub fn new(env: &ContractEnvironment) -> Self {
        let address = env.contracts.diap_payment_core.proxy;
        let abi = diap_payment_core_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;

        Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        }
    }

    pub fn address(&self) -> &'static str {
        self.inner.address()
    }

    pub async fn get_payment(&self, payment_id: &str) -> Result<Value> {
        let result = self
            .inner
            .call("getPayment", vec![token_string(payment_id)])
            .await?;
        Ok(result.json)
    }

    pub async fn get_service_order(&self, service_id: &str) -> Result<Value> {
        let result = self
            .inner
            .call("getServiceOrder", vec![token_uint(service_id)?])
            .await?;
        Ok(result.json)
    }

    pub async fn payment_stats(&self) -> Result<Value> {
        let result = self.inner.call("getPaymentStats", vec![]).await?;
        Ok(result.json)
    }

    pub fn create_payment_call(
        &self,
        to: &str,
        amount: &str,
        payment_id: &str,
        description: &str,
        metadata: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "createPayment",
            vec![
                token_address(to)?,
                token_uint(amount)?,
                token_string(payment_id),
                token_string(description),
                token_string(metadata),
            ],
            None,
        )
    }

    pub fn confirm_payment_call(&self, payment_id: &str) -> Result<EncodedCall> {
        self.inner
            .encode_call("confirmPayment", vec![token_string(payment_id)], None)
    }

    pub fn cancel_payment_call(&self, payment_id: &str) -> Result<EncodedCall> {
        self.inner
            .encode_call("cancelPayment", vec![token_string(payment_id)], None)
    }

    pub fn create_service_order_call(
        &self,
        provider: &str,
        service_type_cid: &str,
        price: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "createServiceOrder",
            vec![
                token_address(provider)?,
                token_string(service_type_cid),
                token_uint(price)?,
            ],
            None,
        )
    }
}

pub struct DiapPaymentChannelClient {
    inner: ContractClient,
}

impl DiapPaymentChannelClient {
    pub fn new(env: &ContractEnvironment) -> Self {
        let address = env.contracts.diap_payment_channel.proxy;
        let abi = diap_payment_channel_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;

        Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        }
    }

    pub fn address(&self) -> &'static str {
        self.inner.address()
    }

    pub async fn get_channel(&self, channel_id: &str) -> Result<Value> {
        let result = self
            .inner
            .call("getPaymentChannel", vec![token_string(channel_id)])
            .await?;
        Ok(result.json)
    }

    pub async fn channel_fee_rate(&self) -> Result<String> {
        let result = self.inner.call("channelFeeRate", vec![]).await?;
        match result.tokens.first() {
            Some(Token::Uint(value)) => Ok(value.to_string()),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "channelFeeRate returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "channelFeeRate returned empty result".into(),
            )),
        }
    }

    pub fn open_channel_call(
        &self,
        participant2: &str,
        deposit: &str,
        channel_id: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "openPaymentChannel",
            vec![
                token_address(participant2)?,
                token_uint(deposit)?,
                token_string(channel_id),
            ],
            None,
        )
    }

    pub fn initiate_channel_close_call(
        &self,
        channel_id: &str,
        final_balance1: &str,
        final_balance2: &str,
        nonce: &str,
        signature1_hex: &str,
        signature2_hex: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "initiateChannelClose",
            vec![
                token_string(channel_id),
                token_uint(final_balance1)?,
                token_uint(final_balance2)?,
                token_uint(nonce)?,
                token_bytes_from_hex(signature1_hex)?,
                token_bytes_from_hex(signature2_hex)?,
            ],
            None,
        )
    }

    pub fn challenge_channel_close_call(
        &self,
        channel_id: &str,
        new_balance1: &str,
        new_balance2: &str,
        new_nonce: &str,
        signature1_hex: &str,
        signature2_hex: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "challengeChannelClose",
            vec![
                token_string(channel_id),
                token_uint(new_balance1)?,
                token_uint(new_balance2)?,
                token_uint(new_nonce)?,
                token_bytes_from_hex(signature1_hex)?,
                token_bytes_from_hex(signature2_hex)?,
            ],
            None,
        )
    }

    pub fn finalize_channel_close_call(&self, channel_id: &str) -> Result<EncodedCall> {
        self.inner
            .encode_call("finalizeChannelClose", vec![token_string(channel_id)], None)
    }
}

pub struct DiapPaymentPrivacyClient {
    inner: ContractClient,
}

impl DiapPaymentPrivacyClient {
    pub fn new(env: &ContractEnvironment) -> Self {
        let address = env.contracts.diap_payment_privacy.proxy;
        let abi = diap_payment_privacy_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;

        Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        }
    }

    pub fn address(&self) -> &'static str {
        self.inner.address()
    }

    pub async fn get_commitment_info(&self, commitment: &str) -> Result<Value> {
        let result = self
            .inner
            .call("getCommitmentInfo", vec![token_fixed_bytes32(commitment)?])
            .await?;
        Ok(result.json)
    }

    pub fn lock_funds_call(&self, commitment: &str, amount: &str) -> Result<EncodedCall> {
        self.inner.encode_call(
            "lockFundsForPrivacy",
            vec![
                token_fixed_bytes32(commitment)?,
                token_uint(amount)?,
            ],
            None,
        )
    }

    pub fn execute_privacy_payment_call(
        &self,
        commitment: &str,
        nullifier: &str,
        proof: &[String],
        recipient: &str,
        amount: &str,
    ) -> Result<EncodedCall> {
        let proof_tokens = proof
            .iter()
            .map(|value| token_uint(value))
            .collect::<Result<Vec<_>>>()?;

        if proof_tokens.len() != 8 {
            return Err(AloudError::InvalidInput(
                "Privacy proof must contain 8 elements".into(),
            ));
        }

        self.inner.encode_call(
            "executePrivacyPayment",
            vec![
                token_fixed_bytes32(commitment)?,
                token_fixed_bytes32(nullifier)?,
                Token::FixedArray(proof_tokens),
                token_address(recipient)?,
                token_uint(amount)?,
            ],
            None,
        )
    }

    pub fn withdraw_locked_funds_call(&self, commitment: &str) -> Result<EncodedCall> {
        self.inner.encode_call(
            "withdrawLockedFunds",
            vec![token_fixed_bytes32(commitment)?],
            None,
        )
    }

    pub fn refund_expired_commitment_call(&self, commitment: &str) -> Result<EncodedCall> {
        self.inner.encode_call(
            "refundExpiredCommitment",
            vec![token_fixed_bytes32(commitment)?],
            None,
        )
    }
}

