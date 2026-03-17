//! API 配置管理
//!
//! 提供加密的本地配置存储，支持用户自定义 API 和 Workers 备用

use super::error::{AgentError, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// 用户自己的 API 配置（优先使用）
    pub user_apis: Vec<UserApiConfig>,

    /// Workers API 配置（备用）
    pub workers_api: WorkersApiConfig,

    /// 执行策略
    pub execution_strategy: ExecutionStrategy,

    /// 默认 Provider
    pub default_provider: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            user_apis: Vec::new(),
            workers_api: WorkersApiConfig {
                base_url: "https://alou-edge.yuanjieliu65.workers.dev".to_string(),
                api_key: None,
                enabled: true,
            },
            execution_strategy: ExecutionStrategy::LocalPriority,
            default_provider: "deepseek".to_string(),
        }
    }
}

/// 用户 API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserApiConfig {
    pub id: String,
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub is_active: bool,
}

/// Workers API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkersApiConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub enabled: bool,
}

/// 执行策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    /// 只用本地
    LocalOnly,
    /// 本地优先，失败用 Workers
    LocalPriority,
    /// Workers 作为备用
    WorkersBackup,
}

impl ApiConfig {
    /// 配置文件路径
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("alou");
        path.push("agent_config.json");
        path
    }

    /// 加载配置（自动解密）
    pub async fn load() -> Result<Self> {
        let config_path = Self::config_path();

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AgentError::ConfigError(format!("创建配置目录失败: {}", e)))?;
        }

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let encrypted = tokio::fs::read(&config_path)
            .await
            .map_err(|e| AgentError::ConfigError(format!("读取配置文件失败: {}", e)))?;

        let decrypted = Self::decrypt(&encrypted)?;
        let config: ApiConfig = serde_json::from_slice(&decrypted).map_err(|e| {
            AgentError::ConfigError(format!("解析配置文件失败: {}", e))
        })?;

        Ok(config)
    }

    /// 保存配置（自动加密）
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        let json = serde_json::to_vec(self)
            .map_err(|e| AgentError::ConfigError(format!("序列化配置失败: {}", e)))?;

        let encrypted = Self::encrypt(&json)?;

        tokio::fs::write(&config_path, encrypted)
            .await
            .map_err(|e| AgentError::ConfigError(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 获取当前可用的 API 配置
    pub fn get_active_api(&self) -> Option<&UserApiConfig> {
        self.user_apis.iter().find(|api| api.is_active)
    }

    /// 加密数据
    fn encrypt(data: &[u8]) -> Result<Vec<u8>> {
        // 从硬件/环境获取加密密钥
        let key_bytes = Self::get_encryption_key();

        let cipher = Aes256Gcm::new(&key_bytes);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| AgentError::ConfigError(format!("加密失败: {}", e)))?;

        // 将 nonce 和密文组合
        let mut result = nonce.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    /// 解密数据
    fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(AgentError::ConfigError("加密数据太短".to_string()));
        }

        let key_bytes = Self::get_encryption_key();
        let cipher = Aes256Gcm::new(&key_bytes);

        let (nonce, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AgentError::ConfigError(format!("解密失败: {}", e)))?;

        Ok(plaintext)
    }

    /// 获取加密密钥（基于机器信息生成固定密钥）
    fn get_encryption_key() -> aes_gcm::Key<Aes256Gcm> {
        use sha2::{Digest, Sha256};
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        // 使用机器信息生成密钥种子
        let mut hasher = DefaultHasher::new();

        // 使用机器名称
        if let Ok(hostname) = std::env::var("COMPUTERNAME") {
            hostname.hash(&mut hasher);
        } else if let Ok(hostname) = std::env::var("HOSTNAME") {
            hostname.hash(&mut hasher);
        }

        // 使用用户名
        if let Ok(username) = std::env::var("USERNAME") {
            username.hash(&mut hasher);
        } else if let Ok(username) = std::env::var("USER") {
            username.hash(&mut hasher);
        }

        // 使用固定盐值增强安全性
        "alou-agent-config-encryption-v1".hash(&mut hasher);

        let hash = hasher.finish();
        
        // 使用 SHA-256 将 8 字节扩展为 32 字节
        let mut sha_hasher = Sha256::new();
        sha_hasher.update(&hash.to_be_bytes());
        let sha_result = sha_hasher.finalize();
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&sha_result[..32]);

        *aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes)
    }
}
