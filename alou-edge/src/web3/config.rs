use crate::web3::contracts::{contracts_for, Network, NetworkContracts};
use serde::Serialize;
use worker::Env;

/// 单个网络的 RPC 配置
#[derive(Debug, Clone, Serialize)]
pub struct RpcConfig {
    pub network: Network,
    pub url: String,
    pub source: RpcSource,
}

/// RPC URL 的配置来源
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RpcSource {
    /// 来自 Cloudflare Secret（优先级最高）
    Secret,
    /// 来自 Cloudflare 环境变量
    EnvVar,
    /// 使用默认内置的公共 RPC
    Default,
}

/// 合约运行环境配置，包含地址与 RPC 信息
#[derive(Debug, Clone, Serialize)]
pub struct ContractEnvironment {
    pub contracts: &'static NetworkContracts,
    pub rpc: RpcConfig,
}

/// 从 Cloudflare `Env` 中解析指定网络的 RPC URL，带有 fallback 逻辑。
pub fn resolve_rpc_config(env: &Env, network: Network) -> RpcConfig {
    let env_key = network.rpc_env_key();

    if let Ok(value) = env.secret(env_key) {
        return RpcConfig {
            network,
            url: value.to_string(),
            source: RpcSource::Secret,
        };
    }

    if let Ok(value) = env.var(env_key) {
        return RpcConfig {
            network,
            url: value.to_string(),
            source: RpcSource::EnvVar,
        };
    }

    RpcConfig {
        network,
        url: network.default_rpc_url().to_string(),
        source: RpcSource::Default,
    }
}

/// 构建完整的合约环境配置（地址 + RPC）
pub fn load_contract_environment(env: &Env, network: Network) -> ContractEnvironment {
    let contracts = contracts_for(network);
    let rpc = resolve_rpc_config(env, network);

    ContractEnvironment { contracts, rpc }
}

