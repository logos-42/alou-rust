//! Alou CLI API Client
//! 处理与后端AI API的通信

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

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
    #[serde(default)]
    pub tool_api: ToolApiConfig,
}

/// 工具 API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolApiConfig {
    pub enabled: bool,
    pub base_url: String,
}

impl Default for ToolApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: String::new(),
        }
    }
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
    /// Ralph Loop 协作配置
    pub ralph_loop: RalphLoopConfig,
}

/// Ralph Loop 智能体协作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphLoopConfig {
    /// 是否启用协作
    pub enabled: bool,
    /// 最大并发回复数
    pub max_concurrent_replies: usize,
    /// 回复间隔（毫秒）
    pub reply_interval_ms: u64,
    /// 最大单轮回复次数
    pub max_replies_per_round: usize,
    /// 协作超时（毫秒）
    pub collaboration_timeout_ms: u64,
    /// 是否允许智能体互相调用
    pub allow_agent_calls: bool,
}

impl Default for RalphLoopConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_replies: 3,
            reply_interval_ms: 2000,
            max_replies_per_round: 5,
            collaboration_timeout_ms: 30000,
            allow_agent_calls: true,
        }
    }
}

impl Default for GroupChatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_join: false,
            announce_presence: true,
            ralph_loop: RalphLoopConfig::default(),
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

// 聊天消息 - 公开导出

/// 聊天消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

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
        tool_api: ToolApiConfig::default(),
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

/// 工具执行请求
#[derive(Debug, Serialize)]
pub struct ToolExecuteRequest {
    pub tool_id: String,
    pub args: serde_json::Value,
    pub working_directory: Option<String>,
    pub timeout_seconds: Option<u64>,
}

/// 工具执行响应
#[derive(Debug, Deserialize)]
pub struct ToolExecuteResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    #[serde(rename = "execution_time_ms")]
    pub execution_time_ms: Option<u64>,
}

/// 工具信息
#[derive(Debug, Deserialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// 获取工具 API 端口
pub fn get_tool_api_port() -> Option<u16> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("alou");
    let port_file = config_dir.join("tool_api_port");
    
    if port_file.exists() {
        if let Ok(content) = fs::read_to_string(&port_file) {
            if let Ok(port) = content.trim().parse::<u16>() {
                return Some(port);
            }
        }
    }
    None
}

/// 获取工具 API 基础 URL
pub fn get_tool_api_url() -> Option<String> {
    let port = get_tool_api_port()?;
    Some(format!("http://127.0.0.1:{}", port))
}

/// 检查工具 API 是否可用
pub fn check_tool_api_available() -> bool {
    get_tool_api_url().is_some()
}

/// 执行工具
pub async fn execute_tool(
    tool_id: &str,
    args: serde_json::Value,
    working_directory: Option<String>,
    timeout_seconds: Option<u64>,
) -> Result<ToolExecuteResponse, String> {
    let base_url = get_tool_api_url().ok_or("Tool API not available. Make sure Alou Desktop is running.")?;
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds.unwrap_or(60)))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let request = ToolExecuteRequest {
        tool_id: tool_id.to_string(),
        args,
        working_directory,
        timeout_seconds,
    };
    
    let url = format!("{}/api/tools/execute", base_url);
    
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }
    
    response
        .json::<ToolExecuteResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// 获取可用工具列表
pub async fn get_tool_list() -> Result<Vec<ToolInfo>, String> {
    let base_url = get_tool_api_url().ok_or("Tool API not available. Make sure Alou Desktop is running.")?;
    
    let client = reqwest::Client::new();
    
    let url = format!("{}/api/tools/list", base_url);
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }
    
    response
        .json::<Vec<ToolInfo>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
