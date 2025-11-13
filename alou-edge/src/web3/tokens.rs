use serde::Serialize;

/// 标准化的链标识
///
/// - `ethereum` 表示以太坊主网
/// - `eth_sepolia` 表示以太坊 Sepolia 测试网
/// - `base` 表示 Base 主网
/// - `base_sepolia` 表示 Base Sepolia 测试网
#[derive(Debug, Clone, Serialize)]
pub struct TokenMetadata {
    pub symbol: &'static str,
    pub name: &'static str,
    pub decimals: u8,
    pub chain: &'static str,
    pub address: &'static str,
    #[serde(default)]
    pub is_testnet: bool,
}

const TOKENS: &[TokenMetadata] = &[
    // Ethereum Sepolia
    TokenMetadata {
        symbol: "USDC",
        name: "USD Coin (Sepolia)",
        decimals: 6,
        chain: "eth_sepolia",
        // Circle 官方 Sepolia 合约
        address: "0xd35CCeEAD182dcee0F148EbaC9447DA2c4D449c4",
        is_testnet: true,
    },
    TokenMetadata {
        symbol: "USDT",
        name: "Tether USD (Sepolia)",
        decimals: 6,
        chain: "eth_sepolia",
        // 参考社区部署，实际使用时可按需替换
        address: "0x509ee0d083ddf8ac028f2a56731412edd63223b9",
        is_testnet: true,
    },
    TokenMetadata {
        symbol: "DAI",
        name: "DAI Stablecoin (Sepolia)",
        decimals: 18,
        chain: "eth_sepolia",
        address: "0x4741e8437b1f5dfd31c109b9fccc546e7d0aa731",
        is_testnet: true,
    },
    // Base Sepolia
    TokenMetadata {
        symbol: "USDC",
        name: "USD Coin (Base Sepolia)",
        decimals: 6,
        chain: "base_sepolia",
        address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        is_testnet: true,
    },
    TokenMetadata {
        symbol: "USDT",
        name: "Tether USD (Base Sepolia)",
        decimals: 6,
        chain: "base_sepolia",
        address: "0x5fd84259d66Cd46123540766Be93DFE6D43130D7",
        is_testnet: true,
    },
];

/// 获取全部支持的稳定币元数据
pub fn all_tokens() -> &'static [TokenMetadata] {
    TOKENS
}

/// 按链过滤支持的稳定币
pub fn tokens_for_chain(chain: &str) -> Vec<&'static TokenMetadata> {
    let normalized = normalize_chain_identifier(chain);
    TOKENS
        .iter()
        .filter(|token| token.chain == normalized)
        .collect()
}

/// 根据链 & 合约地址查找元数据
pub fn find_token(chain: Option<&str>, address: &str) -> Option<&'static TokenMetadata> {
    let normalized_chain = chain.map(normalize_chain_identifier);
    let normalized_address = address.to_ascii_lowercase();

    TOKENS.iter().find(|token| {
        token.address.eq_ignore_ascii_case(&normalized_address)
            && normalized_chain
                .as_deref()
                .map_or(true, |chain| chain == token.chain)
    })
}

/// 根据代币符号与链查找
pub fn find_token_by_symbol(chain: Option<&str>, symbol: &str) -> Option<&'static TokenMetadata> {
    let normalized_chain = chain.map(normalize_chain_identifier);
    let normalized_symbol = symbol.trim().to_ascii_uppercase();

    TOKENS.iter().find(|token| {
        token.symbol == normalized_symbol
            && normalized_chain
                .as_deref()
                .map_or(true, |chain| chain == token.chain)
    })
}

/// 返回用于前端展示的符号列表
pub fn supported_symbols(chain: Option<&str>) -> Vec<&'static str> {
    let normalized_chain = chain.map(normalize_chain_identifier);

    TOKENS
        .iter()
        .filter(|token| {
            normalized_chain
                .as_deref()
                .map_or(true, |chain| chain == token.chain)
        })
        .map(|token| token.symbol)
        .collect()
}

/// 统一链标识
pub fn normalize_chain_identifier(chain: &str) -> String {
    match chain.trim().to_ascii_lowercase().as_str() {
        "eth" | "ethereum" | "0x1" => "ethereum".to_string(),
        "base" | "0x2105" => "base".to_string(),
        "eth_sepolia" | "ethereum-sepolia" | "sepolia" | "0xaa36a7" => "eth_sepolia".to_string(),
        "base_sepolia" | "base-sepolia" | "basesepolia" | "0x14a34" => "base_sepolia".to_string(),
        other => other.to_string(),
    }
}

