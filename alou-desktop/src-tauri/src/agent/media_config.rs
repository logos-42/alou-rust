//! Media Config - 媒体 API 配置
//!
//! 配置所有媒体 Provider 的 API 密钥和参数
//! 使用 AES-256-GCM 加密存储，与 agent_config.json 相同的加密方案

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// 媒体 API 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaApiConfig {
    /// 媒体 Provider 配置
    pub providers: HashMap<String, ProviderConfig>,
}

/// Provider 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub capabilities: Vec<Capability>,
    pub config: Option<serde_json::Value>,
}

impl MediaApiConfig {
    /// 配置文件路径
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("alou");
        path.push("media_config.json");
        path
    }

    /// 加载配置（自动解密）
    pub fn load() -> Result<Self, String> {
        let config_path = Self::config_path();
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let encrypted = std::fs::read(&config_path)
            .map_err(|e| format!("读取配置文件失败：{}", e))?;

        let decrypted = Self::decrypt(&encrypted)?;
        let config: MediaApiConfig = serde_json::from_slice(&decrypted)
            .map_err(|e| format!("解析配置文件失败：{}", e))?;

        Ok(config)
    }

    /// 保存配置（自动加密）
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path();
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败：{}", e))?;
        }

        let json = serde_json::to_vec(self)
            .map_err(|e| format!("序列化配置失败：{}", e))?;

        let encrypted = Self::encrypt(&json)?;

        std::fs::write(&config_path, encrypted)
            .map_err(|e| format!("写入配置文件失败：{}", e))?;

        Ok(())
    }

    /// 获取启用的 Provider
    pub fn enabled_providers(&self) -> Vec<&String> {
        self.providers
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, _)| name)
            .collect()
    }

    /// 根据能力获取 Provider
    pub fn get_provider_by_capability(&self, capability: Capability) -> Option<&String> {
        for (name, config) in &self.providers {
            if config.enabled && config.capabilities.contains(&capability) {
                return Some(name);
            }
        }
        None
    }

    /// 加密数据
    fn encrypt(data: &[u8]) -> Result<Vec<u8>, String> {
        let key_bytes = Self::get_encryption_key();

        let cipher = Aes256Gcm::new(&key_bytes);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| format!("加密失败：{}", e))?;

        // 将 nonce 和密文组合
        let mut result = nonce.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    /// 解密数据
    fn decrypt(data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() < 12 {
            return Err("加密数据太短".to_string());
        }

        let key_bytes = Self::get_encryption_key();
        let cipher = Aes256Gcm::new(&key_bytes);

        let (nonce, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("解密失败：{}", e))?;

        Ok(plaintext)
    }

    /// 获取加密密钥（基于机器信息生成固定密钥）
    fn get_encryption_key() -> aes_gcm::Key<Aes256Gcm> {
        use sha2::{Digest, Sha256};
        
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

        // 使用固定盐值增强安全性（与 agent_config 不同的盐值）
        "alou-media-config-encryption-v1".hash(&mut hasher);

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

/// 媒体能力
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Image,
    Audio,
    Video,
    Music,
    Tts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_api_config() {
        let mut config = MediaApiConfig::default();

        config.providers.insert("seedance".to_string(), ProviderConfig {
            name: "seedance".to_string(),
            api_key: "test_key".to_string(),
            base_url: None,
            model: Some("seedance-1.0-pro".to_string()),
            enabled: true,
            capabilities: vec![Capability::Video],
            config: None,
        });

        config.providers.insert("seedream".to_string(), ProviderConfig {
            name: "seedream".to_string(),
            api_key: "test_key".to_string(),
            base_url: None,
            model: Some("doubao-seedream-5-0-260128".to_string()),
            enabled: true,
            capabilities: vec![Capability::Image],
            config: None,
        });

        assert_eq!(config.enabled_providers().len(), 2);
        assert_eq!(
            config.get_provider_by_capability(Capability::Video),
            Some(&"seedance".to_string())
        );
        assert_eq!(
            config.get_provider_by_capability(Capability::Image),
            Some(&"seedream".to_string())
        );
    }

    #[test]
    fn test_encrypt_decrypt() {
        let config = MediaApiConfig::default();
        let test_data = b"Hello, World!";
        
        let encrypted = MediaApiConfig::encrypt(test_data).unwrap();
        let decrypted = MediaApiConfig::decrypt(&encrypted).unwrap();
        
        assert_eq!(test_data.to_vec(), decrypted);
    }

    #[test]
    fn test_save_and_load() {
        let mut config = MediaApiConfig::default();
        config.providers.insert("test".to_string(), ProviderConfig {
            name: "test".to_string(),
            api_key: "sk-test123456".to_string(),
            base_url: Some("https://api.test.com".to_string()),
            model: None,
            enabled: true,
            capabilities: vec![Capability::Image],
            config: None,
        });

        // 保存配置
        let save_result = config.save();
        assert!(save_result.is_ok());

        // 加载配置
        let loaded = MediaApiConfig::load();
        assert!(loaded.is_ok());

        let loaded_config = loaded.unwrap();
        assert!(loaded_config.providers.contains_key("test"));
    }
}
