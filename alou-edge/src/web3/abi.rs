//! DIAP 合约 ABI 资源加载与缓存
//!
//! 该模块通过 `include_str!` 将 Hardhat 生成的 ABI JSON 静态打包进二进制，
//! 并提供解析后的 `ethabi::Contract` 引用，便于在 Cloudflare Workers 环境
//! 中直接使用，而无需运行时读取文件系统。

use ethabi::Contract;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::io::Cursor;
use std::sync::Arc;

fn parse_contract(json: &str) -> Contract {
    let value: Value = serde_json::from_str(json)
        .expect("Failed to parse Hardhat artifact JSON");
    let abi_value = value
        .get("abi")
        .expect("ABI field missing in Hardhat artifact");
    let abi_bytes =
        serde_json::to_vec(abi_value).expect("Failed to serialize ABI portion of artifact");
    let mut cursor = Cursor::new(abi_bytes);
    Contract::load(&mut cursor).expect("Failed to load ABI definition")
}

macro_rules! lazy_contract {
    ($name:ident, $artifact:ident) => {
        pub static $name: Lazy<Arc<Contract>> =
            Lazy::new(|| Arc::new(parse_contract($artifact)));
    };
}

/// DIAPToken 合约 ABI（包含完整 Hardhat artifact JSON）
pub const DIAP_TOKEN: &str = include_str!("../../abi/diap/DIAPToken.json");
lazy_contract!(DIAP_TOKEN_CONTRACT, DIAP_TOKEN);

/// DIAPAgentNetwork 合约 ABI
pub const DIAP_AGENT_NETWORK: &str = include_str!("../../abi/diap/DIAPAgentNetwork.json");
lazy_contract!(DIAP_AGENT_NETWORK_CONTRACT, DIAP_AGENT_NETWORK);

/// DIAPPaymentCore 合约 ABI
pub const DIAP_PAYMENT_CORE: &str = include_str!("../../abi/diap/DIAPPaymentCore.json");
lazy_contract!(DIAP_PAYMENT_CORE_CONTRACT, DIAP_PAYMENT_CORE);

/// DIAPPaymentChannel 合约 ABI
pub const DIAP_PAYMENT_CHANNEL: &str =
    include_str!("../../abi/diap/DIAPPaymentChannel.json");
lazy_contract!(
    DIAP_PAYMENT_CHANNEL_CONTRACT,
    DIAP_PAYMENT_CHANNEL
);

/// DIAPPaymentPrivacy 合约 ABI
pub const DIAP_PAYMENT_PRIVACY: &str =
    include_str!("../../abi/diap/DIAPPaymentPrivacy.json");
lazy_contract!(
    DIAP_PAYMENT_PRIVACY_CONTRACT,
    DIAP_PAYMENT_PRIVACY
);

/// DIAPVerification 合约 ABI（ZKP 入口，当前仅作占位）
#[allow(dead_code)]
pub const DIAP_VERIFICATION: &str =
    include_str!("../../abi/diap/DIAPVerification.json");
lazy_contract!(
    DIAP_VERIFICATION_CONTRACT,
    DIAP_VERIFICATION
);

/// DIAPGovernance 合约 ABI
pub const DIAP_GOVERNANCE: &str =
    include_str!("../../abi/diap/DIAPGovernance.json");
lazy_contract!(DIAP_GOVERNANCE_CONTRACT, DIAP_GOVERNANCE);

/// TimelockController 合约 ABI（来自 Hardhat 构建产物）
pub const TIMELOCK_CONTROLLER: &str =
    include_str!("../../abi/openzeppelin/TimelockController.json");
lazy_contract!(
    TIMELOCK_CONTROLLER_CONTRACT,
    TIMELOCK_CONTROLLER
);

/// ERC-4337 账户工厂 ABI
pub const DIAP_ACCOUNT_FACTORY: &str =
    include_str!("../../abi/aa/DIAPAccountFactory.json");
lazy_contract!(
    DIAP_ACCOUNT_FACTORY_CONTRACT,
    DIAP_ACCOUNT_FACTORY
);

/// DIAPAccount 实现 ABI
pub const DIAP_ACCOUNT: &str = include_str!("../../abi/aa/DIAPAccount.json");
lazy_contract!(DIAP_ACCOUNT_CONTRACT, DIAP_ACCOUNT);

/// DIAPPaymaster ABI
pub const DIAP_PAYMASTER: &str =
    include_str!("../../abi/aa/DIAPPaymaster.json");
lazy_contract!(DIAP_PAYMASTER_CONTRACT, DIAP_PAYMASTER);

/// EntryPoint 接口 ABI
pub const ENTRY_POINT: &str = include_str!("../../abi/aa/IEntryPoint.json");
lazy_contract!(ENTRY_POINT_CONTRACT, ENTRY_POINT);

/// 获取解析后的合约 ABI 引用
pub fn diap_token_contract() -> Arc<Contract> {
    DIAP_TOKEN_CONTRACT.clone()
}

pub fn diap_agent_network_contract() -> Arc<Contract> {
    DIAP_AGENT_NETWORK_CONTRACT.clone()
}

pub fn diap_payment_core_contract() -> Arc<Contract> {
    DIAP_PAYMENT_CORE_CONTRACT.clone()
}

pub fn diap_payment_channel_contract() -> Arc<Contract> {
    DIAP_PAYMENT_CHANNEL_CONTRACT.clone()
}

pub fn diap_payment_privacy_contract() -> Arc<Contract> {
    DIAP_PAYMENT_PRIVACY_CONTRACT.clone()
}

#[allow(dead_code)]
pub fn diap_verification_contract() -> Arc<Contract> {
    DIAP_VERIFICATION_CONTRACT.clone()
}

pub fn diap_governance_contract() -> Arc<Contract> {
    DIAP_GOVERNANCE_CONTRACT.clone()
}

pub fn timelock_controller_contract() -> Arc<Contract> {
    TIMELOCK_CONTROLLER_CONTRACT.clone()
}

pub fn diap_account_factory_contract() -> Arc<Contract> {
    DIAP_ACCOUNT_FACTORY_CONTRACT.clone()
}

pub fn diap_account_contract() -> Arc<Contract> {
    DIAP_ACCOUNT_CONTRACT.clone()
}

pub fn diap_paymaster_contract() -> Arc<Contract> {
    DIAP_PAYMASTER_CONTRACT.clone()
}

pub fn entry_point_contract() -> Arc<Contract> {
    ENTRY_POINT_CONTRACT.clone()
}

