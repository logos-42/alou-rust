use crate::utils::error::{AloudError, Result};
use crate::web3::abi::{
    diap_account_contract, diap_account_factory_contract, diap_paymaster_contract,
    entry_point_contract,
};
use crate::web3::clients::common::{
    parse_u256, token_address, token_bytes_from_hex, token_uint,
    ContractClient, EncodedCall,
};
use crate::web3::config::ContractEnvironment;
use ethabi::Token;
use serde_json::Value;

pub struct DiapAccountFactoryClient {
    inner: ContractClient,
}

impl DiapAccountFactoryClient {
    pub fn new(env: &ContractEnvironment) -> Result<Self> {
        let address = env
            .contracts
            .account_factory
            .as_ref()
            .map(|c| c.factory)
            .ok_or_else(|| {
                AloudError::InvalidInput("Account factory not configured for selected network".into())
            })?;
        let abi = diap_account_factory_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;
        Ok(Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        })
    }

    #[allow(dead_code)]
    pub fn address(&self) -> &str {
        self.inner.address()
    }

    pub async fn get_account_by_owner(&self, owner: &str) -> Result<String> {
        let result = self
            .inner
            .call("getAccountByOwner", vec![token_address(owner)?])
            .await?;
        match result.tokens.first() {
            Some(Token::Address(addr)) => Ok(format!("0x{:040x}", addr)),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "getAccountByOwner returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "getAccountByOwner returned empty result".into(),
            )),
        }
    }

    pub async fn get_owner_by_account(&self, account: &str) -> Result<String> {
        let result = self
            .inner
            .call("getOwnerByAccount", vec![token_address(account)?])
            .await?;
        match result.tokens.first() {
            Some(Token::Address(addr)) => Ok(format!("0x{:040x}", addr)),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "getOwnerByAccount returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "getOwnerByAccount returned empty result".into(),
            )),
        }
    }

    pub async fn is_account(&self, account: &str) -> Result<bool> {
        let result = self
            .inner
            .call("isAccount", vec![token_address(account)?])
            .await?;
        match result.tokens.first() {
            Some(Token::Bool(value)) => Ok(*value),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "isAccount returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "isAccount returned empty result".into(),
            )),
        }
    }

    pub async fn get_address(&self, owner: &str, salt: &str) -> Result<String> {
        let result = self
            .inner
            .call(
                "getAddress",
                vec![token_address(owner)?, token_uint(salt)?],
            )
            .await?;
        match result.tokens.first() {
            Some(Token::Address(addr)) => Ok(format!("0x{:040x}", addr)),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "getAddress returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "getAddress returned empty result".into(),
            )),
        }
    }

    pub fn create_account_call(&self, owner: &str, salt: &str) -> Result<EncodedCall> {
        self.inner.encode_call(
            "createAccount",
            vec![token_address(owner)?, token_uint(salt)?],
            None,
        )
    }

    pub fn batch_create_accounts_call(
        &self,
        owners: &[String],
        salts: &[String],
    ) -> Result<EncodedCall> {
        if owners.len() != salts.len() {
            return Err(AloudError::InvalidInput(
                "Owners and salts length mismatch".into(),
            ));
        }

        let owner_tokens = owners
            .iter()
            .map(|owner| token_address(owner))
            .collect::<Result<Vec<_>>>()?;
        let salt_tokens = salts
            .iter()
            .map(|salt| token_uint(salt))
            .collect::<Result<Vec<_>>>()?;

        self.inner.encode_call(
            "batchCreateAccounts",
            vec![Token::Array(owner_tokens), Token::Array(salt_tokens)],
            None,
        )
    }
}

pub struct DiapPaymasterClient {
    inner: ContractClient,
}

impl DiapPaymasterClient {
    pub fn new(env: &ContractEnvironment) -> Result<Self> {
        let address = env
            .contracts
            .paymaster
            .as_ref()
            .map(|c| c.address)
            .ok_or_else(|| {
                AloudError::InvalidInput("Paymaster not configured for selected network".into())
            })?;
        let abi = diap_paymaster_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;
        Ok(Self {
            inner: ContractClient::new(address, abi, rpc_url, network),
        })
    }

    #[allow(dead_code)]
    pub fn address(&self) -> &str {
        self.inner.address()
    }

    pub async fn gas_quota_info(&self, account: &str) -> Result<Value> {
        let result = self
            .inner
            .call("getGasQuotaInfo", vec![token_address(account)?])
            .await?;
        Ok(result.json)
    }

    pub async fn account_whitelisted(&self, account: &str) -> Result<bool> {
        let result = self
            .inner
            .call("accountWhitelist", vec![token_address(account)?])
            .await?;
        match result.tokens.first() {
            Some(Token::Bool(value)) => Ok(*value),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "accountWhitelist returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "accountWhitelist returned empty result".into(),
            )),
        }
    }

    pub async fn target_whitelisted(&self, target: &str) -> Result<bool> {
        let result = self
            .inner
            .call("targetWhitelist", vec![token_address(target)?])
            .await?;
        match result.tokens.first() {
            Some(Token::Bool(value)) => Ok(*value),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "targetWhitelist returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "targetWhitelist returned empty result".into(),
            )),
        }
    }

    pub async fn default_daily_quota(&self) -> Result<String> {
        let result = self.inner.call("defaultDailyQuota", vec![]).await?;
        extract_uint(&result.tokens, "defaultDailyQuota")
    }

    pub async fn require_whitelist(&self) -> Result<bool> {
        let result = self.inner.call("requireWhitelist", vec![]).await?;
        match result.tokens.first() {
            Some(Token::Bool(value)) => Ok(*value),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "requireWhitelist returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput(
                "requireWhitelist returned empty result".into(),
            )),
        }
    }

    pub async fn get_deposit(&self) -> Result<String> {
        let result = self.inner.call("getDeposit", vec![]).await?;
        extract_uint(&result.tokens, "getDeposit")
    }

    pub fn set_gas_quota_call(&self, account: &str, quota: &str) -> Result<EncodedCall> {
        self.inner.encode_call(
            "setGasQuota",
            vec![token_address(account)?, token_uint(quota)?],
            None,
        )
    }

    pub fn set_default_daily_quota_call(&self, quota: &str) -> Result<EncodedCall> {
        self.inner
            .encode_call("setDefaultDailyQuota", vec![token_uint(quota)?], None)
    }

    pub fn set_require_whitelist_call(&self, required: bool) -> Result<EncodedCall> {
        self.inner
            .encode_call("setRequireWhitelist", vec![Token::Bool(required)], None)
    }

    pub fn add_account_to_whitelist_call(&self, account: &str) -> Result<EncodedCall> {
        self.inner
            .encode_call("addAccountToWhitelist", vec![token_address(account)?], None)
    }

    pub fn remove_account_from_whitelist_call(&self, account: &str) -> Result<EncodedCall> {
        self.inner.encode_call(
            "removeAccountFromWhitelist",
            vec![token_address(account)?],
            None,
        )
    }

    pub fn add_target_to_whitelist_call(&self, target: &str) -> Result<EncodedCall> {
        self.inner
            .encode_call("addTargetToWhitelist", vec![token_address(target)?], None)
    }

    pub fn remove_target_from_whitelist_call(&self, target: &str) -> Result<EncodedCall> {
        self.inner.encode_call(
            "removeTargetFromWhitelist",
            vec![token_address(target)?],
            None,
        )
    }

    pub fn add_deposit_call(&self, amount_eth: &str) -> Result<EncodedCall> {
        let value = parse_u256(amount_eth)?;
        self.inner.encode_call("addDeposit", vec![], Some(value))
    }

    pub fn withdraw_deposit_call(
        &self,
        withdraw_address: &str,
        amount: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "withdrawDeposit",
            vec![token_address(withdraw_address)?, token_uint(amount)?],
            None,
        )
    }
}

pub struct EntryPointClient {
    inner: ContractClient,
}

impl EntryPointClient {
    pub fn new(env: &ContractEnvironment) -> Self {
        let address = env.contracts.entry_point;
        let abi = entry_point_contract();
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

    pub async fn balance_of(&self, account: &str) -> Result<String> {
        let result = self
            .inner
            .call("balanceOf", vec![token_address(account)?])
            .await?;
        extract_uint(&result.tokens, "balanceOf")
    }

    pub fn deposit_to_call(&self, account: &str, amount: &str) -> Result<EncodedCall> {
        let value = parse_u256(amount)?;
        self.inner
            .encode_call("depositTo", vec![token_address(account)?], Some(value))
    }

    pub fn withdraw_to_call(
        &self,
        withdraw_address: &str,
        amount: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "withdrawTo",
            vec![token_address(withdraw_address)?, token_uint(amount)?],
            None,
        )
    }
}

pub struct DiapAccountClient {
    inner: ContractClient,
}

impl DiapAccountClient {
    pub fn new(account_address: impl Into<String>, env: &ContractEnvironment) -> Self {
        let abi = diap_account_contract();
        let rpc_url = env.rpc.url.clone();
        let network = env.rpc.network;
        Self {
            inner: ContractClient::new(account_address, abi, rpc_url, network),
        }
    }

    #[allow(dead_code)]
    pub fn address(&self) -> &str {
        self.inner.address()
    }

    pub async fn owner(&self) -> Result<String> {
        let result = self.inner.call("owner", vec![]).await?;
        match result.tokens.first() {
            Some(Token::Address(addr)) => Ok(format!("0x{:040x}", addr)),
            Some(other) => Err(AloudError::InvalidInput(format!(
                "owner returned unexpected token: {:?}",
                other
            ))),
            None => Err(AloudError::InvalidInput("owner returned empty result".into())),
        }
    }

    pub fn execute_call(
        &self,
        target: &str,
        value: &str,
        data_hex: &str,
    ) -> Result<EncodedCall> {
        self.inner.encode_call(
            "execute",
            vec![
                token_address(target)?,
                token_uint(value)?,
                token_bytes_from_hex(data_hex)?,
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

