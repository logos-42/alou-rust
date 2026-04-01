use crate::tools::{ToolExecutor, ToolResult, ToolError, ToolMetadata, ExecutionContext, ToolCategory, ToolPriority, ToolStatus};
use async_trait::async_trait;
use bip39::Mnemonic;
use ed25519_dalek::{SigningKey as DalekSigningKey, VerifyingKey};
use k256::ecdsa::SigningKey;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::SecretKey;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha512};
use sha3::Keccak256;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

const WALLET_STORAGE_DIR: &str = "agent_wallets";

#[derive(Clone)]
pub struct AgentWalletTool {
    metadata: ToolMetadata,
    wallets: Arc<RwLock<HashMap<String, WalletData>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub address: String,
    pub chain: String,
    pub created_at: i64,
    pub balance: String,
    pub transactions: Vec<Value>,
    pub mnemonic: Option<String>,
    pub private_key: Option<String>,
    pub name: Option<String>,
}

impl AgentWalletTool {
    pub fn new() -> Self {
        let metadata = ToolMetadata {
            id: "agent_wallet".to_string(),
            name: "agent_wallet".to_string(),
            description: "创建和管理真实的加密钱包。支持 Ethereum 和 Solana 链。Agent 可以根据用户指令创建钱包、查询余额、导入私钥等。创建的钱包包含真实的加密密钥对，可用于链上交互。".to_string(),
            category: ToolCategory::Web3,
            priority: ToolPriority::High,
            status: ToolStatus::Active,
            version: "2.0.0".to_string(),
            author: "Alou".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            dependencies: vec![],
            platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
            permissions: vec![],
            tags: vec!["wallet", "blockchain", "ethereum", "solana", "crypto"].iter().map(|s| s.to_string()).collect(),
        };

        Self {
            metadata,
            wallets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_storage_path(&self) -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("alou");
        path.push(WALLET_STORAGE_DIR);
        path
    }

    fn ensure_storage_dir(&self) -> Result<PathBuf, ToolError> {
        let path = self.get_storage_path();
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| ToolError::InternalError(format!("创建钱包存储目录失败: {}", e)))?;
        }
        Ok(path)
    }

    fn wallet_file_path(&self, session_id: &str, chain: &str) -> Result<PathBuf, ToolError> {
        let dir = self.ensure_storage_dir()?;
        let sanitized_session = session_id.replace(|c: char| !c.is_alphanumeric() && c != '-', "_");
        let sanitized_chain = chain.replace(|c: char| !c.is_alphanumeric(), "_");
        Ok(dir.join(format!("{}-{}.json", sanitized_session, sanitized_chain)))
    }

    fn generate_ethereum_wallet(mnemonic: Option<String>) -> Result<(String, String, String), ToolError> {
        if let Some(m) = mnemonic {
            let phrase = Mnemonic::from_str(&m)
                .map_err(|e| ToolError::InternalError(format!("无效助记词: {}", e)))?;
            let seed = phrase.to_seed("");
            let secret_key = SecretKey::from_slice(&seed[0..32])
                .map_err(|e| ToolError::InternalError(format!("私钥生成失败: {}", e)))?;
            let public_key = secret_key.public_key();
            let pub_bytes = public_key.to_encoded_point(false);
            let mut hasher = Keccak256::new();
            hasher.update(pub_bytes.as_bytes());
            let hash = hasher.finalize();
            let addr = format!("0x{}", hex::encode(&hash[12..]));
            Ok((m, format!("0x{}", hex::encode(secret_key.to_bytes())), addr))
        } else {
            let mut rng = rand::rng();
            let mut bytes = [0u8; 32];
            rng.fill_bytes(&mut bytes);
            let secret_key = SecretKey::from_slice(&bytes)
                .map_err(|e| ToolError::InternalError(format!("私钥生成失败: {}", e)))?;
            let public_key = secret_key.public_key();
            let pub_bytes = public_key.to_encoded_point(false);
            let mut hasher = Keccak256::new();
            hasher.update(pub_bytes.as_bytes());
            let hash = hasher.finalize();
            let addr = format!("0x{}", hex::encode(&hash[12..]));
            let mnemonic_phrase = Mnemonic::generate(12)
                .map_err(|e| ToolError::InternalError(format!("助记词生成失败: {}", e)))?;
            Ok((mnemonic_phrase.to_string(), format!("0x{}", hex::encode(secret_key.to_bytes())), addr))
        }
    }

    fn generate_solana_wallet(mnemonic: Option<String>) -> Result<(String, String, String), ToolError> {
        if let Some(m) = mnemonic {
            let phrase = Mnemonic::from_str(&m)
                .map_err(|e| ToolError::InternalError(format!("无效助记词: {}", e)))?;
            let seed = phrase.to_seed("");
            let mut hasher = Sha512::new();
            hasher.update(b"ed25519 seed");
            hasher.update(&seed);
            let hash = hasher.finalize();
            let secret_bytes: [u8; 32] = hash[..32].try_into()
                .map_err(|_| ToolError::InternalError("派生密钥失败".to_string()))?;
            let signing_key = DalekSigningKey::from_bytes(&secret_bytes);
            let verifying_key = VerifyingKey::from(&signing_key);
            let addr = bs58::encode(verifying_key.to_bytes()).into_string();
            let pk = hex::encode(signing_key.to_bytes());
            Ok((m, pk, addr))
        } else {
            let mut rng = rand::rng();
            let mut secret_bytes = [0u8; 32];
            rng.fill_bytes(&mut secret_bytes);
            let signing_key = DalekSigningKey::from_bytes(&secret_bytes);
            let verifying_key = VerifyingKey::from(&signing_key);
            let addr = bs58::encode(verifying_key.to_bytes()).into_string();
            let pk = hex::encode(signing_key.to_bytes());
            let mnemonic_phrase = Mnemonic::generate(12)
                .map_err(|e| ToolError::InternalError(format!("助记词生成失败: {}", e)))?;
            Ok((mnemonic_phrase.to_string(), pk, addr))
        }
    }

    async fn save_wallet_to_disk(&self, session_id: &str, chain: &str, wallet: &WalletData) -> Result<(), ToolError> {
        let path = self.wallet_file_path(session_id, chain)?;
        let data = serde_json::to_string_pretty(wallet)
            .map_err(|e| ToolError::InternalError(format!("序列化钱包数据失败: {}", e)))?;
        fs::write(&path, data)
            .map_err(|e| ToolError::InternalError(format!("保存钱包文件失败: {}", e)))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&path) {
                let _ = fs::set_permissions(&path, PermissionsExt::from_mode(0o600));
            }
        }
        Ok(())
    }

    async fn load_wallet_from_disk(&self, session_id: &str, chain: &str) -> Result<Option<WalletData>, ToolError> {
        let path = self.wallet_file_path(session_id, chain)?;
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&path)
            .map_err(|e| ToolError::InternalError(format!("读取钱包文件失败: {}", e)))?;
        let wallet: WalletData = serde_json::from_str(&data)
            .map_err(|e| ToolError::InternalError(format!("解析钱包数据失败: {}", e)))?;
        Ok(Some(wallet))
    }

    async fn create_wallet(&self, session_id: &str, chain: &str, mnemonic: Option<String>) -> Result<WalletData, ToolError> {
        let key = format!("agent_wallet:{}:{}", session_id, chain);
        {
            let wallets = self.wallets.read().await;
            if let Some(existing) = wallets.get(&key) {
                return Ok(existing.clone());
            }
        }

        let (mnemonic_phrase, private_key, address) = match chain.to_lowercase().as_str() {
            "ethereum" | "eth" => Self::generate_ethereum_wallet(mnemonic)?,
            "solana" | "sol" => Self::generate_solana_wallet(mnemonic)?,
            _ => Self::generate_ethereum_wallet(mnemonic)?,
        };

        let chain_name = match chain.to_lowercase().as_str() {
            "ethereum" | "eth" => "Ethereum",
            "solana" | "sol" => "Solana",
            _ => chain,
        };

        let wallet = WalletData {
            address: address.clone(),
            chain: chain.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            balance: "0".to_string(),
            transactions: vec![],
            mnemonic: Some(mnemonic_phrase.clone()),
            private_key: Some(private_key.clone()),
            name: Some(format!("Agent {} 钱包", chain_name)),
        };

        {
            let mut wallets = self.wallets.write().await;
            wallets.insert(key.clone(), wallet.clone());
        }

        let _ = self.save_wallet_to_disk(session_id, chain, &wallet).await;
        Ok(wallet)
    }

    async fn import_wallet(&self, session_id: &str, chain: &str, private_key: String, mnemonic: Option<String>) -> Result<WalletData, ToolError> {
        let key = format!("agent_wallet:{}:{}", session_id, chain);

        let (address, cleaned_key) = match chain.to_lowercase().as_str() {
            "ethereum" | "eth" => {
                let pk = private_key.strip_prefix("0x").unwrap_or(&private_key);
                let bytes = hex::decode(pk)
                    .map_err(|_| ToolError::InvalidArguments("无效的 Ethereum 私钥格式".to_string()))?;
                if bytes.len() != 32 {
                    return Err(ToolError::InvalidArguments("私钥长度不正确，应为 32 字节".to_string()));
                }
                let secret_key = SecretKey::from_slice(&bytes)
                    .map_err(|_| ToolError::InvalidArguments("无效的私钥".to_string()))?;
                let public_key = secret_key.public_key();
                let pub_bytes = public_key.to_encoded_point(false);
                let mut hasher = Keccak256::new();
                hasher.update(pub_bytes.as_bytes());
                let hash = hasher.finalize();
                let addr = format!("0x{}", hex::encode(&hash[12..]));
                (addr, format!("0x{}", pk))
            }
            "solana" | "sol" => {
                let pk = private_key.strip_prefix("0x").unwrap_or(&private_key);
                let bytes = hex::decode(pk)
                    .map_err(|_| ToolError::InvalidArguments("无效的 Solana 私钥格式".to_string()))?;
                if bytes.len() != 32 {
                    return Err(ToolError::InvalidArguments("私钥长度不正确，应为 32 字节".to_string()));
                }
                let secret_bytes: [u8; 32] = bytes.try_into().unwrap();
                let signing_key = DalekSigningKey::from_bytes(&secret_bytes);
                let verifying_key = VerifyingKey::from(&signing_key);
                let addr = bs58::encode(verifying_key.to_bytes()).into_string();
                (addr, format!("0x{}", pk))
            }
            _ => return Err(ToolError::InvalidArguments(format!("不支持的链: {}", chain))),
        };

        let chain_name = match chain.to_lowercase().as_str() {
            "ethereum" | "eth" => "Ethereum",
            "solana" | "sol" => "Solana",
            _ => chain,
        };

        let wallet = WalletData {
            address,
            chain: chain.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            balance: "0".to_string(),
            transactions: vec![],
            mnemonic,
            private_key: Some(cleaned_key),
            name: Some(format!("导入的 {} 钱包", chain_name)),
        };

        {
            let mut wallets = self.wallets.write().await;
            wallets.insert(key.clone(), wallet.clone());
        }

        let _ = self.save_wallet_to_disk(session_id, chain, &wallet).await;
        Ok(wallet)
    }

    async fn get_wallet(&self, session_id: &str, chain: &str) -> Result<Option<WalletData>, ToolError> {
        let key = format!("agent_wallet:{}:{}", session_id, chain);
        {
            let wallets = self.wallets.read().await;
            if let Some(wallet) = wallets.get(&key) {
                return Ok(Some(wallet.clone()));
            }
        }
        if let Some(wallet) = self.load_wallet_from_disk(session_id, chain).await? {
            let mut wallets = self.wallets.write().await;
            wallets.insert(key, wallet.clone());
            return Ok(Some(wallet));
        }
        Ok(None)
    }

    async fn list_wallets(&self, session_id: &str) -> Result<Vec<WalletData>, ToolError> {
        let chains = vec!["ethereum", "solana", "base", "polygon"];
        let mut result = vec![];
        for chain in chains {
            if let Ok(Some(wallet)) = self.get_wallet(session_id, chain).await {
                result.push(wallet);
            }
        }
        Ok(result)
    }

    async fn record_transaction(&self, session_id: &str, chain: &str, tx_data: Value) -> Result<(), ToolError> {
        let key = format!("agent_wallet:{}:{}", session_id, chain);
        let mut wallets = self.wallets.write().await;
        if let Some(wallet) = wallets.get_mut(&key) {
            wallet.transactions.push(tx_data);
            if wallet.transactions.len() > 100 {
                wallet.transactions.drain(0..wallet.transactions.len() - 100);
            }
        }
        Ok(())
    }
}

impl Default for AgentWalletTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for AgentWalletTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: Value, context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let start = std::time::Instant::now();
        let action = args.get("action").and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("缺少 'action' 字段。可用操作: create_wallet, import_wallet, get_wallet, list_wallets, record_transaction".to_string()))?;
        let session_id = &context.session_id;

        let result = match action {
            "create_wallet" => {
                let chain = args.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("缺少 'chain' 字段。支持的链: ethereum, solana, base, polygon".to_string())
                })?;
                let mnemonic = args.get("mnemonic").and_then(|v| v.as_str()).map(|s| s.to_string());
                let wallet = self.create_wallet(session_id, chain, mnemonic).await?;

                let safe_wallet = WalletData {
                    private_key: wallet.private_key.as_ref().map(|pk| {
                        if pk.len() > 10 { format!("{}...{}", &pk[..6], &pk[pk.len()-4..]) } else { "***".to_string() }
                    }),
                    ..wallet.clone()
                };

                Ok(json!({
                    "success": true,
                    "wallet": safe_wallet,
                    "full_wallet": wallet,
                    "message": format!("成功创建 {} 钱包，地址: {}", chain, wallet.address),
                    "warning": "请务必保存助记词和私钥！丢失后将无法恢复。",
                    "is_new": true
                }))
            }
            "import_wallet" => {
                let chain = args.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("缺少 'chain' 字段".to_string())
                })?;
                let private_key = args.get("private_key").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("缺少 'private_key' 字段".to_string())
                })?;
                let mnemonic = args.get("mnemonic").and_then(|v| v.as_str()).map(|s| s.to_string());
                let wallet = self.import_wallet(session_id, chain, private_key.to_string(), mnemonic).await?;

                let safe_wallet = WalletData {
                    private_key: wallet.private_key.as_ref().map(|pk| {
                        if pk.len() > 10 { format!("{}...{}", &pk[..6], &pk[pk.len()-4..]) } else { "***".to_string() }
                    }),
                    ..wallet.clone()
                };

                Ok(json!({
                    "success": true,
                    "wallet": safe_wallet,
                    "full_wallet": wallet,
                    "message": format!("成功导入 {} 钱包，地址: {}", chain, wallet.address),
                }))
            }
            "get_wallet" => {
                let chain = args.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("缺少 'chain' 字段".to_string())
                })?;
                match self.get_wallet(session_id, chain).await? {
                    Some(wallet) => {
                        let safe_wallet = WalletData {
                            private_key: wallet.private_key.as_ref().map(|pk| {
                                if pk.len() > 10 { format!("{}...{}", &pk[..6], &pk[pk.len()-4..]) } else { "***".to_string() }
                            }),
                            ..wallet.clone()
                        };
                        Ok(json!({
                            "success": true,
                            "wallet": safe_wallet,
                            "full_wallet": wallet,
                            "message": format!("已获取 {} 钱包信息", chain)
                        }))
                    }
                    None => Ok(json!({
                        "success": false,
                        "message": format!("未找到 {} 链的钱包", chain)
                    })),
                }
            }
            "list_wallets" => {
                let wallets = self.list_wallets(session_id).await?;
                let safe_wallets: Vec<WalletData> = wallets.iter().map(|w| WalletData {
                    private_key: w.private_key.as_ref().map(|pk| {
                        if pk.len() > 10 { format!("{}...{}", &pk[..6], &pk[pk.len()-4..]) } else { "***".to_string() }
                    }),
                    ..w.clone()
                }).collect();
                Ok(json!({
                    "success": true,
                    "wallets": safe_wallets,
                    "full_wallets": wallets,
                    "count": wallets.len(),
                    "message": format!("共找到 {} 个钱包", wallets.len())
                }))
            }
            "record_transaction" => {
                let chain = args.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
                    ToolError::InvalidArguments("缺少 'chain' 字段".to_string())
                })?;
                let transaction = args.get("transaction").ok_or_else(|| {
                    ToolError::InvalidArguments("缺少 'transaction' 字段".to_string())
                })?.clone();
                self.record_transaction(session_id, chain, transaction).await?;
                Ok(json!({ "success": true, "message": "交易记录已保存" }))
            }
            _ => Err(ToolError::InvalidArguments(format!(
                "未知操作: {}。可用操作: create_wallet, import_wallet, get_wallet, list_wallets, record_transaction", action
            ))),
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;
        match result {
            Ok(data) => Ok(ToolResult { success: true, data, error: None, execution_time_ms, output: None, warnings: vec![], context: None }),
            Err(e) => Ok(ToolResult { success: false, data: json!({}), error: Some(e.to_string()), execution_time_ms, output: None, warnings: vec![], context: None }),
        }
    }

    async fn validate_args(&self, args: &Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("参数必须是一个对象".to_string()));
        }
        if let Some(action) = args.get("action").and_then(|v| v.as_str()) {
            match action {
                "create_wallet" | "get_wallet" => {
                    if !args.get("chain").is_some() {
                        return Err(ToolError::InvalidArguments("缺少 'chain' 字段".to_string()));
                    }
                }
                "import_wallet" => {
                    if !args.get("chain").is_some() || !args.get("private_key").is_some() {
                        return Err(ToolError::InvalidArguments("缺少 'chain' 或 'private_key' 字段".to_string()));
                    }
                }
                "record_transaction" => {
                    if !args.get("chain").is_some() || !args.get("transaction").is_some() {
                        return Err(ToolError::InvalidArguments("缺少 'chain' 或 'transaction' 字段".to_string()));
                    }
                }
                "list_wallets" => {}
                _ => return Err(ToolError::InvalidArguments(format!("未知操作: {}", action))),
            }
        } else {
            return Err(ToolError::InvalidArguments("缺少 'action' 字段".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        r#"Agent 钱包工具 - 创建和管理真实的加密钱包

支持链: Ethereum, Solana, Base, Polygon

操作:
- create_wallet: 创建新钱包（生成真实密钥对）
  参数: chain (必需), mnemonic (可选，使用已有助记词派生)
  示例: {"action": "create_wallet", "chain": "ethereum"}

- import_wallet: 导入已有钱包
  参数: chain (必需), private_key (必需), mnemonic (可选)
  示例: {"action": "import_wallet", "chain": "solana", "private_key": "0x..."}

- get_wallet: 查询钱包信息
  参数: chain (必需)
  示例: {"action": "get_wallet", "chain": "ethereum"}

- list_wallets: 列出所有钱包
  示例: {"action": "list_wallets"}

- record_transaction: 记录交易
  参数: chain (必需), transaction (必需)
  示例: {"action": "record_transaction", "chain": "ethereum", "transaction": {...}}"#.to_string()
    }
}
