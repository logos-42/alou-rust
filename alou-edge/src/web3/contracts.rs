use serde::{Deserialize, Serialize};

/// 支持的测试网枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    /// Ethereum Sepolia 测试网
    Sepolia,
    /// Base Sepolia 测试网
    BaseSepolia,
}

impl Network {
    /// 获取链 ID
    pub const fn chain_id(self) -> u64 {
        match self {
            Network::Sepolia => 11155111,
            Network::BaseSepolia => 84532,
        }
    }

    /// 返回网络名称标识
    pub const fn as_str(self) -> &'static str {
        match self {
            Network::Sepolia => "sepolia",
            Network::BaseSepolia => "base_sepolia",
        }
    }

    /// 默认 RPC 环境变量名称（可通过 Cloudflare 环境变量覆盖）
    pub const fn rpc_env_key(self) -> &'static str {
        match self {
            Network::Sepolia => "ETH_TESTNET_RPC_URL",
            Network::BaseSepolia => "BASE_TESTNET_RPC_URL",
        }
    }

    /// 推荐使用的公共 RPC 地址（若未配置环境变量，可作为 fallback）
    pub const fn default_rpc_url(self) -> &'static str {
        match self {
            Network::Sepolia => "https://ethereum-sepolia-rpc.publicnode.com",
            Network::BaseSepolia => "https://sepolia.base.org",
        }
    }
}

/// 代理合约地址（proxy + implementation）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyContractAddresses {
    pub proxy: &'static str,
    pub implementation: &'static str,
}

/// 单地址合约（无代理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddress {
    pub address: &'static str,
}

/// ERC-4337 账户工厂与实现地址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountFactoryAddresses {
    pub factory: &'static str,
    pub account_implementation: &'static str,
}

/// Paymaster 地址信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymasterAddresses {
    pub address: &'static str,
}

/// 核心合约地址集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContracts {
    pub diap_token: ProxyContractAddresses,
    pub diap_agent_network: ProxyContractAddresses,
    pub diap_verification: ProxyContractAddresses,
    pub diap_payment_core: ProxyContractAddresses,
    pub diap_payment_channel: ProxyContractAddresses,
    pub diap_payment_privacy: ProxyContractAddresses,
    pub diap_governance: ContractAddress,
    pub timelock_controller: ContractAddress,
    pub account_factory: Option<AccountFactoryAddresses>,
    pub paymaster: Option<PaymasterAddresses>,
    pub entry_point: &'static str,
}

/// Sepolia 网络合约地址
pub const SEPOLIA_CONTRACTS: NetworkContracts = NetworkContracts {
    diap_token: ProxyContractAddresses {
        proxy: "0x2a5b6A672e9028962Ab4DaF20d256C0978604Cb3",
        implementation: "0xbD9d07d42978798dAC0452A5eB24A8446A3Aa4b6",
    },
    diap_agent_network: ProxyContractAddresses {
        proxy: "0x9eF71FD5be68ebab2ABE20c5Fab826b14BfBc089",
        implementation: "0xc83663765d62db77802b0Fd0EAC80144f23E3E9B",
    },
    diap_verification: ProxyContractAddresses {
        proxy: "0x8F513135a6865173b6fC08e7A1138211ba174109",
        implementation: "0xC36b92Cf8722eDbD5E5CD365Ccd08725ebcc5493",
    },
    diap_payment_core: ProxyContractAddresses {
        proxy: "0x498CbdD8d509058FfDe7335391B8a053Bb4Ab0e7",
        implementation: "0xED2203795bF2Be6872530A1A62b1741fD00503c9",
    },
    diap_payment_channel: ProxyContractAddresses {
        proxy: "0x471cB216e5bF64d9E33b92E12d6AE3327c7a7a80",
        implementation: "0x37cA7821a995F67eaE49CB38473Bd5f99f08b121",
    },
    diap_payment_privacy: ProxyContractAddresses {
        proxy: "0x69bd0c763F86B80C043eA7CF1af58186E23E21cc",
        implementation: "0xba9aaC051FbC0ad8CA78E7aE283F29E0D6e7F8Ef",
    },
    diap_governance: ContractAddress {
        address: "0xFBD843F3ECDd5398639d849763088BF9Cd36f2Be",
    },
    timelock_controller: ContractAddress {
        address: "0x4CFDC3D8aAabDB6E9f78a0CEe5d32Fb062eCD17A",
    },
    account_factory: Some(AccountFactoryAddresses {
        factory: "0xeaf2cb64685695497bf20f70c6F74bA86851edfD",
        account_implementation: "0x9Fb4fCDF304f27bBf9E7869279a2347bCFc6d20C",
    }),
    paymaster: Some(PaymasterAddresses {
        address: "0xA960cf9053FA76278e16f9D4BA35225f7634DC54",
    }),
    entry_point: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
};

/// Base Sepolia 网络合约地址
pub const BASE_SEPOLIA_CONTRACTS: NetworkContracts = NetworkContracts {
    diap_token: ProxyContractAddresses {
        proxy: "0xFBD843F3ECDd5398639d849763088BF9Cd36f2Be",
        implementation: "0xCd9B0781A933B3650d1c72b92eaD0E4bf6eB698c",
    },
    diap_agent_network: ProxyContractAddresses {
        proxy: "0xA960cf9053FA76278e16f9D4BA35225f7634DC54",
        implementation: "0x8F513135a6865173b6fC08e7A1138211ba174109",
    },
    diap_verification: ProxyContractAddresses {
        proxy: "0x3d21CbF5C3bc48b19A4627181ABA42baC76AE93b",
        implementation: "0x498CbdD8d509058FfDe7335391B8a053Bb4Ab0e7",
    },
    diap_payment_core: ProxyContractAddresses {
        proxy: "0xfb433bac16C391956367409257262C402b49ba5e",
        implementation: "0x471cB216e5bF64d9E33b92E12d6AE3327c7a7a80",
    },
    diap_payment_channel: ProxyContractAddresses {
        proxy: "0x793FCa0108F87D106f8E5Aa60443f52f6D1EE345",
        implementation: "0x69bd0c763F86B80C043eA7CF1af58186E23E21cc",
    },
    diap_payment_privacy: ProxyContractAddresses {
        proxy: "0xef2252273BbBfa20d6a510C304C0224A5f9b88ac",
        implementation: "0xF684594C83D5d940e3bec7D25FaAF27A562133dB",
    },
    diap_governance: ContractAddress {
        address: "0x4e058BbE725C26375f8782B922fA26DBcc180B96",
    },
    timelock_controller: ContractAddress {
        address: "0x82d4a4171a255D0f5b759Dfa5EFf77c00C94b3ca",
    },
    account_factory: None, // 当前未提供 Base Sepolia 版本部署地址，后续可补充
    paymaster: None,
    entry_point: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
};

/// 根据网络枚举获取对应的合约地址集合
pub const fn contracts_for(network: Network) -> &'static NetworkContracts {
    match network {
        Network::Sepolia => &SEPOLIA_CONTRACTS,
        Network::BaseSepolia => &BASE_SEPOLIA_CONTRACTS,
    }
}

/// 根据链 ID 推断网络（若不匹配则返回 None）
pub fn network_from_chain_id(chain_id: u64) -> Option<Network> {
    match chain_id {
        11155111 => Some(Network::Sepolia),
        84532 => Some(Network::BaseSepolia),
        _ => None,
    }
}

/// 根据字符串解析网络标识（大小写不敏感）
pub fn network_from_str(value: &str) -> Option<Network> {
    match value.trim().to_lowercase().as_str() {
        "sepolia" | "eth-sepolia" | "ethereum-sepolia" => Some(Network::Sepolia),
        "base" | "base-sepolia" | "basesepolia" => Some(Network::BaseSepolia),
        _ => None,
    }
}

