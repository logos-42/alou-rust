//! Bot Gateway 配置模块
//!
//! 提供各平台 Bot 的配置结构和管理功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bot Gateway 主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotGatewayConfig {
    /// 是否启用
    #[serde(default)]
    pub enabled: bool,
    
    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// 公网域名（用于 webhook 回调）
    pub public_domain: Option<String>,
    
    /// 平台配置
    #[serde(default)]
    pub platforms: PlatformConfigs,
    
    /// 全局权限设置
    #[serde(default)]
    pub auth: AuthConfig,
    
    /// 速率限制
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    
    /// 命令前缀
    #[serde(default = "default_command_prefix")]
    pub command_prefix: String,
}

fn default_port() -> u16 { 8080 }
fn default_command_prefix() -> String { "/".to_string() }

impl Default for BotGatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8080,
            public_domain: None,
            platforms: PlatformConfigs::default(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            command_prefix: "/".to_string(),
        }
    }
}

/// 平台配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformConfigs {
    pub telegram: Option<TelegramConfig>,
    pub feishu: Option<FeishuConfig>,
    pub discord: Option<DiscordConfig>,
    pub qq: Option<QQConfig>,
}

/// Telegram 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub allowed_user_ids: Vec<String>,
    pub allowed_chat_ids: Vec<String>,
    pub webhook_url: Option<String>,
    #[serde(default)]
    pub use_polling: bool,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            allowed_user_ids: vec![],
            allowed_chat_ids: vec![],
            webhook_url: None,
            use_polling: true,
        }
    }
}

/// 飞书配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuConfig {
    pub enabled: bool,
    pub app_id: String,
    pub app_secret: String,
    pub verify_token: String,
    pub encrypt_key: Option<String>,
    pub allowed_user_ids: Vec<String>,
    pub allowed_tenant_ids: Vec<String>,
}

impl Default for FeishuConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            app_id: String::new(),
            app_secret: String::new(),
            verify_token: String::new(),
            encrypt_key: None,
            allowed_user_ids: vec![],
            allowed_tenant_ids: vec![],
        }
    }
}

/// Discord 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub allowed_user_ids: Vec<String>,
    pub allowed_guild_ids: Vec<String>,
    pub allowed_channel_ids: Vec<String>,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            allowed_user_ids: vec![],
            allowed_guild_ids: vec![],
            allowed_channel_ids: vec![],
        }
    }
}

/// QQ 配置 (OneBot 协议)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QQConfig {
    pub enabled: bool,
    pub ws_url: String,
    pub access_token: Option<String>,
    pub allowed_user_ids: Vec<String>,
    pub allowed_group_ids: Vec<String>,
}

impl Default for QQConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ws_url: "ws://127.0.0.1:8080".to_string(),
            access_token: None,
            allowed_user_ids: vec![],
            allowed_group_ids: vec![],
        }
    }
}

/// 权限配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// 是否需要认证
    #[serde(default = "default_true")]
    pub require_auth: bool,
    
    /// 允许的用户列表
    #[serde(default)]
    pub allowed_users: Vec<String>,
    
    /// 管理员列表
    #[serde(default)]
    pub admins: Vec<String>,
    
    /// 默认权限
    #[serde(default)]
    pub default_permissions: Vec<String>,
}

fn default_true() -> bool { true }

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            require_auth: true,
            allowed_users: vec![],
            admins: vec![],
            default_permissions: vec!["execute_basic".to_string()],
        }
    }
}

/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// 每分钟最大请求数
    #[serde(default = "default_rate_limit")]
    pub max_requests_per_minute: u32,
    
    /// 每小时最大请求数
    #[serde(default = "default_hourly_limit")]
    pub max_requests_per_hour: u32,
}

fn default_rate_limit() -> u32 { 60 }
fn default_hourly_limit() -> u32 { 1000 }

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_minute: 60,
            max_requests_per_hour: 1000,
        }
    }
}

/// 工具权限映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermissionMap {
    pub tool_id: String,
    pub required_permission: String,
}

/// 命令配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub command: String,
    pub description: String,
    pub required_permission: String,
    pub allowed_platforms: Vec<String>,
}

/// 加载配置
pub fn load_config(config_path: &str) -> Result<BotGatewayConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(config_path)?;
    let config: BotGatewayConfig = toml::from_str(&content)?;
    Ok(config)
}

/// 保存配置
pub fn save_config(config_path: &str, config: &BotGatewayConfig) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(config_path, content)?;
    Ok(())
}

/// 获取默认配置
pub fn get_default_config() -> BotGatewayConfig {
    BotGatewayConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = get_default_config();
        assert!(!config.enabled);
        assert_eq!(config.port, 8080);
        assert_eq!(config.command_prefix, "/");
    }

    #[test]
    fn test_serialize_config() {
        let config = get_default_config();
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("port = 8080"));
    }
}
