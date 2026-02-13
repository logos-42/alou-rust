//! Alou CLI API Client
//! 处理与后端AI API的通信

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub timeout: u64,
}

/// AI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub temperature: f64,
    pub max_tokens: u64,
}

/// 完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub ai: AiConfig,
    #[serde(default)]
    pub autonomous: AutonomousConfig,
    #[serde(default)]
    pub group_chat: GroupChatConfig,
}

/// 自主循环配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousConfig {
    pub enabled: bool,
    #[serde(default = "default_heartbeat")]
    pub heartbeat_interval: u64,
    #[serde(default = "default_task_check")]
    pub task_check_interval: u64,
    #[serde(default = "default_memory_save")]
    pub memory_save_interval: u64,
    #[serde(default = "default_progress_report")]
    pub progress_report_interval: u64,
}

fn default_heartbeat() -> u64 { 30 }
fn default_task_check() -> u64 { 10 }
fn default_memory_save() -> u64 { 60 }
fn default_progress_report() -> u64 { 300 }

impl Default for AutonomousConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            heartbeat_interval: 30,
            task_check_interval: 10,
            memory_save_interval: 60,
            progress_report_interval: 300,
        }
    }
}

/// 群聊配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatConfig {
    pub enabled: bool,
    pub auto_join: bool,
    pub announce_presence: bool,
}

impl Default for GroupChatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_join: false,
            announce_presence: true,
        }
    }
}

/// AI聊天请求
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f64,
    pub max_tokens: u64,
}

/// 聊天消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// 为了兼容性，创建ChatMessage作为Message的别名
pub type ChatMessage = Message;

/// AI聊天响应
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

/// 选择
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
}

/// 加载配置文件
pub fn load_config() -> Config {
    let config_path = get_config_path();
    
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    
    // 返回默认配置
    Config {
        api: ApiConfig {
            base_url: "http://localhost:8787".to_string(),
            timeout: 30000,
        },
        ai: AiConfig {
            provider: "deepseek".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: "".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        },
        autonomous: AutonomousConfig::default(),
        group_chat: GroupChatConfig::default(),
    }
}

/// 获取配置路径
fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".alou")
        .join("config.json")
}

/// 保存配置文件
pub fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path();
    
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_path, content).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 发送聊天请求到AI API
pub async fn send_chat_request(config: &Config, messages: Vec<Message>) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let request = ChatRequest {
        model: config.ai.model.clone(),
        messages,
        temperature: config.ai.temperature,
        max_tokens: config.ai.max_tokens,
    };
    
    let url = format!("{}/v1/chat/completions", config.api.base_url);
    
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.ai.api_key))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API错误: {}", response.status()));
    }
    
    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;
    
    Ok(chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default())
}
