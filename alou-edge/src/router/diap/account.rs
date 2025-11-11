use super::common::{
    app_error, invalid_request, missing_field, method_not_allowed, respond_encoded,
    resolve_environment,
};
use crate::web3::clients::{
    DiapAccountClient, DiapAccountFactoryClient, DiapPaymasterClient, EntryPointClient,
};
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Request, Response, Result as WorkerResult};

pub async fn handle_account_request(env: &Env, req: &mut Request) -> WorkerResult<Response> {
    let body: AccountRequest = match req.json().await {
        Ok(body) => body,
        Err(e) => return invalid_request(e.to_string()),
    };

    let (_, contract_env) = match resolve_environment(env, &body.network) {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    match body.module {
        AccountModule::Factory => {
            let client = match DiapAccountFactoryClient::new(&contract_env) {
                Ok(client) => client,
                Err(e) => return app_error(e),
            };
            handle_account_factory(client, body.action, body).await
        }
        AccountModule::Paymaster => {
            let client = match DiapPaymasterClient::new(&contract_env) {
                Ok(client) => client,
                Err(e) => return app_error(e),
            };
            handle_paymaster(client, body.action, body).await
        }
        AccountModule::EntryPoint => {
            let client = EntryPointClient::new(&contract_env);
            handle_entrypoint(client, body.action, body).await
        }
        AccountModule::Account => {
            let account = match &body.account_address {
                Some(addr) => addr.clone(),
                None => return missing_field("account_address"),
            };
            let client = DiapAccountClient::new(account, &contract_env);
            handle_account_instance(client, body.action, body).await
        }
    }
}

async fn handle_account_factory(
    client: DiapAccountFactoryClient,
    action: AccountAction,
    body: AccountRequest,
) -> WorkerResult<Response> {
    match action {
        AccountAction::FactoryGetAccountByOwner => {
            let owner = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.get_account_by_owner(&owner).await {
                Ok(account) => super::super::json_response(&json!({ "owner": owner, "account": account })),
                Err(e) => app_error(e),
            }
        }
        AccountAction::FactoryGetOwnerByAccount => {
            let account = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.get_owner_by_account(&account).await {
                Ok(owner) => super::super::json_response(&json!({ "account": account, "owner": owner })),
                Err(e) => app_error(e),
            }
        }
        AccountAction::FactoryIsAccount => {
            let account = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.is_account(&account).await {
                Ok(is_account) => super::super::json_response(&json!({ "account": account, "exists": is_account })),
                Err(e) => app_error(e),
            }
        }
        AccountAction::FactoryGetAddress => {
            let owner = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            let salt = match body.salt_value {
                Some(value) => value,
                None => return missing_field("salt"),
            };
            match client.get_address(&owner, &salt).await {
                Ok(predicted) => super::super::json_response(&json!({
                    "owner": owner,
                    "salt": salt,
                    "predicted_account": predicted
                })),
                Err(e) => app_error(e),
            }
        }
        AccountAction::FactoryCreateAccountEncode => {
            let owner = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            let salt = match body.salt_value {
                Some(value) => value,
                None => return missing_field("salt"),
            };
            respond_encoded(client.create_account_call(&owner, &salt))
        }
        AccountAction::FactoryBatchCreateEncode => {
            let owners = body.owners.unwrap_or_default();
            let salts = body.salts.unwrap_or_default();
            respond_encoded(client.batch_create_accounts_call(&owners, &salts))
        }
        _ => method_not_allowed(),
    }
}

async fn handle_paymaster(
    client: DiapPaymasterClient,
    action: AccountAction,
    body: AccountRequest,
) -> WorkerResult<Response> {
    match action {
        AccountAction::PaymasterGasQuotaInfo => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.gas_quota_info(&address).await {
                Ok(info) => super::super::json_response(&json!({ "quota": info })),
                Err(e) => app_error(e),
            }
        }
        AccountAction::PaymasterAccountWhitelisted => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.account_whitelisted(&address).await {
                Ok(whitelisted) => super::super::json_response(&json!({ "address": address, "whitelisted": whitelisted })),
                Err(e) => app_error(e),
            }
        }
        AccountAction::PaymasterTargetWhitelisted => {
            let target = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.target_whitelisted(&target).await {
                Ok(whitelisted) => super::super::json_response(&json!({ "address": target, "whitelisted": whitelisted })),
                Err(e) => app_error(e),
            }
        }
        AccountAction::PaymasterDefaultQuota => match client.default_daily_quota().await {
            Ok(quota) => super::super::json_response(&json!({ "default_daily_quota": quota })),
            Err(e) => app_error(e),
        },
        AccountAction::PaymasterRequireWhitelist => match client.require_whitelist().await {
            Ok(required) => super::super::json_response(&json!({ "require_whitelist": required })),
            Err(e) => app_error(e),
        },
        AccountAction::PaymasterDeposit => match client.get_deposit().await {
            Ok(balance) => super::super::json_response(&json!({ "deposit": balance })),
            Err(e) => app_error(e),
        },
        AccountAction::PaymasterSetGasQuotaEncode => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            let quota = match body.amount {
                Some(value) => value,
                None => return missing_field("amount"),
            };
            respond_encoded(client.set_gas_quota_call(&address, &quota))
        }
        AccountAction::PaymasterSetDefaultQuotaEncode => {
            let quota = match body.amount {
                Some(value) => value,
                None => return missing_field("amount"),
            };
            respond_encoded(client.set_default_daily_quota_call(&quota))
        }
        AccountAction::PaymasterSetRequireWhitelistEncode => {
            let required = body.flag.unwrap_or(true);
            respond_encoded(client.set_require_whitelist_call(required))
        }
        AccountAction::PaymasterAddAccountWhitelistEncode => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            respond_encoded(client.add_account_to_whitelist_call(&address))
        }
        AccountAction::PaymasterRemoveAccountWhitelistEncode => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            respond_encoded(client.remove_account_from_whitelist_call(&address))
        }
        AccountAction::PaymasterAddTargetWhitelistEncode => {
            let target = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            respond_encoded(client.add_target_to_whitelist_call(&target))
        }
        AccountAction::PaymasterRemoveTargetWhitelistEncode => {
            let target = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            respond_encoded(client.remove_target_from_whitelist_call(&target))
        }
        AccountAction::PaymasterAddDepositEncode => {
            let amount = match body.amount {
                Some(value) => value,
                None => return missing_field("amount"),
            };
            respond_encoded(client.add_deposit_call(&amount))
        }
        AccountAction::PaymasterWithdrawDepositEncode => {
            let withdraw = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            let amount = match body.amount {
                Some(value) => value,
                None => return missing_field("amount"),
            };
            respond_encoded(client.withdraw_deposit_call(&withdraw, &amount))
        }
        _ => method_not_allowed(),
    }
}

async fn handle_entrypoint(
    client: EntryPointClient,
    action: AccountAction,
    body: AccountRequest,
) -> WorkerResult<Response> {
    match action {
        AccountAction::EntryPointBalanceOf => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            match client.balance_of(&address).await {
                Ok(balance) => super::super::json_response(&json!({ "address": address, "balance": balance })),
                Err(e) => app_error(e),
            }
        }
        AccountAction::EntryPointDepositToEncode => {
            let address = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            let amount = match body.amount {
                Some(value) => value,
                None => return missing_field("amount"),
            };
            respond_encoded(client.deposit_to_call(&address, &amount))
        }
        AccountAction::EntryPointWithdrawToEncode => {
            let withdraw = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            let amount = match body.amount {
                Some(value) => value,
                None => return missing_field("amount"),
            };
            respond_encoded(client.withdraw_to_call(&withdraw, &amount))
        }
        _ => method_not_allowed(),
    }
}

async fn handle_account_instance(
    client: DiapAccountClient,
    action: AccountAction,
    body: AccountRequest,
) -> WorkerResult<Response> {
    match action {
        AccountAction::AccountOwner => match client.owner().await {
            Ok(owner) => super::super::json_response(&json!({ "owner": owner })),
            Err(e) => app_error(e),
        },
        AccountAction::AccountExecuteEncode => {
            let target = match body.address {
                Some(addr) => addr,
                None => return missing_field("address"),
            };
            let value = body.amount.unwrap_or_else(|| "0".to_string());
            let data = match body.data {
                Some(data) => data,
                None => return missing_field("data"),
            };
            respond_encoded(client.execute_call(&target, &value, &data))
        }
        _ => method_not_allowed(),
    }
}

#[derive(Deserialize)]
struct AccountRequest {
    network: String,
    module: AccountModule,
    action: AccountAction,
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    account_address: Option<String>,
    #[serde(default)]
    amount: Option<String>,
    #[serde(default)]
    salt_value: Option<String>,
    #[serde(default)]
    owners: Option<Vec<String>>,
    #[serde(default)]
    salts: Option<Vec<String>>,
    #[serde(default)]
    flag: Option<bool>,
    #[serde(default)]
    data: Option<String>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum AccountModule {
    Factory,
    Paymaster,
    EntryPoint,
    Account,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum AccountAction {
    // Factory
    FactoryGetAccountByOwner,
    FactoryGetOwnerByAccount,
    FactoryIsAccount,
    FactoryGetAddress,
    FactoryCreateAccountEncode,
    FactoryBatchCreateEncode,

    // Paymaster
    PaymasterGasQuotaInfo,
    PaymasterAccountWhitelisted,
    PaymasterTargetWhitelisted,
    PaymasterDefaultQuota,
    PaymasterRequireWhitelist,
    PaymasterDeposit,
    PaymasterSetGasQuotaEncode,
    PaymasterSetDefaultQuotaEncode,
    PaymasterSetRequireWhitelistEncode,
    PaymasterAddAccountWhitelistEncode,
    PaymasterRemoveAccountWhitelistEncode,
    PaymasterAddTargetWhitelistEncode,
    PaymasterRemoveTargetWhitelistEncode,
    PaymasterAddDepositEncode,
    PaymasterWithdrawDepositEncode,

    // EntryPoint
    EntryPointBalanceOf,
    EntryPointDepositToEncode,
    EntryPointWithdrawToEncode,

    // Account
    AccountOwner,
    AccountExecuteEncode,
}

