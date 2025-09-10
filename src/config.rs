use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use crate::env_config::*;
use crate::mcp_config::*;
use crate::types::*;

/// 主配置类
/// 管理项目的各种配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 会话ID
    pub session_id: String,
    /// 模型配置
    pub models: ModelConfig,
    /// 沙盒设置
    pub sandbox: SandboxConfig,
    /// 工具注册表
    pub tool_registry: ToolRegistryConfig,
    /// DeepSeek API配置
    pub deepseek_api: DeepSeekApiConfig,
    /// 系统提示配置
    pub system_prompt_config: Option<SystemPromptConfig>,
    /// 工作区根目录
    pub workspace_root: PathBuf,
    /// MCP配置
    pub mcp_config: Option<McpServerConfig>,
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// 默认模型
    pub default_model: String,
    /// 最大令牌数
    pub max_tokens: usize,
    /// 温度设置
    pub temperature: f64,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    /// 最大重试次数
    pub max_retries: usize,
}

/// 沙盒配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// 是否启用沙盒模式
    pub enabled: bool,
    /// 沙盒目录
    pub directory: Option<PathBuf>,
    /// 允许的命令
    pub allowed_commands: Vec<String>,
    /// 禁止的命令
    pub forbidden_commands: Vec<String>,
}

/// 工具注册表配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistryConfig {
    /// 是否启用工具注册表
    pub enabled: bool,
    /// 工具超时时间（秒）
    pub tool_timeout: u64,
    /// 最大并发工具调用数
    pub max_concurrent_tools: usize,
    /// 工具重试次数
    pub tool_retries: usize,
}

/// DeepSeek API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekApiConfig {
    /// API密钥
    pub api_key: String,
    /// API端点
    pub api_endpoint: String,
    /// 是否启用流式响应
    pub streaming: bool,
    /// 请求头
    pub headers: HashMap<String, String>,
}

/// 系统提示配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptConfig {
    /// 系统提示映射
    pub system_prompt_mappings: Option<Vec<ModelTemplateMapping>>,
}

/// 模型模板映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTemplateMapping {
    /// 基础URL列表
    pub base_urls: Option<Vec<String>>,
    /// 模型名称列表
    pub model_names: Option<Vec<String>>,
    /// 模板内容
    pub template: Option<String>,
}

impl Config {
    /// 创建新的配置实例
    pub fn new() -> Result<Self> {
        // 初始化环境配置
        init_env_config()?;
        
        // 验证必需的环境变量
        validate_required_env()?;

        // 获取工作区根目录
        let workspace_root = get_workspace_root();

        // 创建模型配置
        let models = ModelConfig {
            default_model: get_openai_model(),
            max_tokens: get_max_tokens(),
            temperature: get_temperature(),
            request_timeout: get_request_timeout(),
            max_retries: get_max_retries(),
        };

        // 创建沙盒配置
        let sandbox = SandboxConfig {
            enabled: is_sandbox_mode(),
            directory: if get_sandbox().is_empty() {
                None
            } else {
                Some(PathBuf::from(get_sandbox()))
            },
            allowed_commands: vec![
                "ls".to_string(),
                "pwd".to_string(),
                "cat".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "git".to_string(),
            ],
            forbidden_commands: vec![
                "rm".to_string(),
                "rmdir".to_string(),
                "del".to_string(),
                "format".to_string(),
                "fdisk".to_string(),
            ],
        };

        // 创建工具注册表配置
        let tool_registry = ToolRegistryConfig {
            enabled: true,
            tool_timeout: 30,
            max_concurrent_tools: 5,
            tool_retries: 3,
        };

        // 创建DeepSeek API配置
        let deepseek_api = DeepSeekApiConfig {
            api_key: get_deepseek_api_key().unwrap_or_else(|_| {
                get_openai_api_key().unwrap_or_else(|_| String::new())
            }),
            api_endpoint: get_deepseek_api_endpoint(),
            streaming: true,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("User-Agent".to_string(), get_user_agent());
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers
            },
        };

        // 加载系统提示配置
        let system_prompt_config = load_system_prompt_config()?;

        // 加载MCP配置
        let mcp_config = crate::mcp_config::McpConfigLoader::load_mcp_config()?;

        Ok(Config {
            session_id: get_session_id(),
            models,
            sandbox,
            tool_registry,
            deepseek_api,
            system_prompt_config,
            workspace_root,
            mcp_config,
        })
    }

    /// 从文件加载配置
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("无法读取配置文件: {}", path.as_ref().display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| "无法解析配置文件")?;
        
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .with_context(|| "无法序列化配置")?;
        
        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("无法写入配置文件: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    /// 获取API密钥
    pub fn get_api_key(&self) -> &str {
        &self.deepseek_api.api_key
    }

    /// 获取API端点
    pub fn get_api_endpoint(&self) -> &str {
        &self.deepseek_api.api_endpoint
    }

    /// 获取默认模型
    pub fn get_default_model(&self) -> &str {
        &self.models.default_model
    }

    /// 获取最大令牌数
    pub fn get_max_tokens(&self) -> usize {
        self.models.max_tokens
    }

    /// 获取温度设置
    pub fn get_temperature(&self) -> f64 {
        self.models.temperature
    }

    /// 获取请求超时时间
    pub fn get_request_timeout(&self) -> u64 {
        self.models.request_timeout
    }

    /// 获取最大重试次数
    pub fn get_max_retries(&self) -> usize {
        self.models.max_retries
    }

    /// 检查是否启用沙盒模式
    pub fn is_sandbox_enabled(&self) -> bool {
        self.sandbox.enabled
    }

    /// 获取沙盒目录
    pub fn get_sandbox_directory(&self) -> Option<&PathBuf> {
        self.sandbox.directory.as_ref()
    }

    /// 检查命令是否被允许
    pub fn is_command_allowed(&self, command: &str) -> bool {
        if self.sandbox.forbidden_commands.contains(&command.to_string()) {
            return false;
        }
        
        if self.sandbox.allowed_commands.is_empty() {
            return true;
        }
        
        self.sandbox.allowed_commands.contains(&command.to_string())
    }

    /// 获取工具超时时间
    pub fn get_tool_timeout(&self) -> u64 {
        self.tool_registry.tool_timeout
    }

    /// 获取最大并发工具调用数
    pub fn get_max_concurrent_tools(&self) -> usize {
        self.tool_registry.max_concurrent_tools
    }

    /// 获取工具重试次数
    pub fn get_tool_retries(&self) -> usize {
        self.tool_registry.tool_retries
    }

    /// 获取工作区根目录
    pub fn get_workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }

    /// 获取MCP配置
    pub fn get_mcp_config(&self) -> Option<&McpServerConfig> {
        self.mcp_config.as_ref()
    }

    /// 获取系统提示配置
    pub fn get_system_prompt_config(&self) -> Option<&SystemPromptConfig> {
        self.system_prompt_config.as_ref()
    }

    /// 更新API密钥
    pub fn update_api_key(&mut self, api_key: String) {
        self.deepseek_api.api_key = api_key;
    }

    /// 更新API端点
    pub fn update_api_endpoint(&mut self, api_endpoint: String) {
        self.deepseek_api.api_endpoint = api_endpoint;
    }

    /// 更新默认模型
    pub fn update_default_model(&mut self, model: String) {
        self.models.default_model = model;
    }

    /// 更新最大令牌数
    pub fn update_max_tokens(&mut self, max_tokens: usize) {
        self.models.max_tokens = max_tokens;
    }

    /// 更新温度设置
    pub fn update_temperature(&mut self, temperature: f64) {
        self.models.temperature = temperature;
    }

    /// 更新沙盒设置
    pub fn update_sandbox(&mut self, enabled: bool, directory: Option<PathBuf>) {
        self.sandbox.enabled = enabled;
        self.sandbox.directory = directory;
    }

    /// 添加允许的命令
    pub fn add_allowed_command(&mut self, command: String) {
        if !self.sandbox.allowed_commands.contains(&command) {
            self.sandbox.allowed_commands.push(command);
        }
    }

    /// 添加禁止的命令
    pub fn add_forbidden_command(&mut self, command: String) {
        if !self.sandbox.forbidden_commands.contains(&command) {
            self.sandbox.forbidden_commands.push(command);
        }
    }

    /// 移除允许的命令
    pub fn remove_allowed_command(&mut self, command: &str) {
        self.sandbox.allowed_commands.retain(|c| c != command);
    }

    /// 移除禁止的命令
    pub fn remove_forbidden_command(&mut self, command: &str) {
        self.sandbox.forbidden_commands.retain(|c| c != command);
    }

    /// 更新工具注册表配置
    pub fn update_tool_registry(&mut self, enabled: bool, timeout: u64, max_concurrent: usize, retries: usize) {
        self.tool_registry.enabled = enabled;
        self.tool_registry.tool_timeout = timeout;
        self.tool_registry.max_concurrent_tools = max_concurrent;
        self.tool_registry.tool_retries = retries;
    }

    /// 获取配置摘要
    pub fn get_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert("session_id".to_string(), self.session_id.clone());
        summary.insert("default_model".to_string(), self.models.default_model.clone());
        summary.insert("max_tokens".to_string(), self.models.max_tokens.to_string());
        summary.insert("temperature".to_string(), self.models.temperature.to_string());
        summary.insert("request_timeout".to_string(), self.models.request_timeout.to_string());
        summary.insert("max_retries".to_string(), self.models.max_retries.to_string());
        summary.insert("sandbox_enabled".to_string(), self.sandbox.enabled.to_string());
        summary.insert("tool_registry_enabled".to_string(), self.tool_registry.enabled.to_string());
        summary.insert("tool_timeout".to_string(), self.tool_registry.tool_timeout.to_string());
        summary.insert("max_concurrent_tools".to_string(), self.tool_registry.max_concurrent_tools.to_string());
        summary.insert("tool_retries".to_string(), self.tool_registry.tool_retries.to_string());
        summary.insert("api_endpoint".to_string(), self.deepseek_api.api_endpoint.clone());
        summary.insert("streaming".to_string(), self.deepseek_api.streaming.to_string());
        summary.insert("workspace_root".to_string(), self.workspace_root.to_string_lossy().to_string());
        
        if let Some(sandbox_dir) = &self.sandbox.directory {
            summary.insert("sandbox_directory".to_string(), sandbox_dir.to_string_lossy().to_string());
        }
        
        summary
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        // 验证API密钥
        if self.deepseek_api.api_key.is_empty() {
            return Err(anyhow::anyhow!("API密钥不能为空"));
        }

        // 验证API端点
        if self.deepseek_api.api_endpoint.is_empty() {
            return Err(anyhow::anyhow!("API端点不能为空"));
        }

        // 验证模型名称
        if self.models.default_model.is_empty() {
            return Err(anyhow::anyhow!("默认模型不能为空"));
        }

        // 验证最大令牌数
        if self.models.max_tokens == 0 {
            return Err(anyhow::anyhow!("最大令牌数必须大于0"));
        }

        // 验证温度设置
        if self.models.temperature < 0.0 || self.models.temperature > 2.0 {
            return Err(anyhow::anyhow!("温度设置必须在0.0到2.0之间"));
        }

        // 验证请求超时时间
        if self.models.request_timeout == 0 {
            return Err(anyhow::anyhow!("请求超时时间必须大于0"));
        }

        // 验证最大重试次数
        if self.models.max_retries == 0 {
            return Err(anyhow::anyhow!("最大重试次数必须大于0"));
        }

        // 验证工具超时时间
        if self.tool_registry.tool_timeout == 0 {
            return Err(anyhow::anyhow!("工具超时时间必须大于0"));
        }

        // 验证最大并发工具调用数
        if self.tool_registry.max_concurrent_tools == 0 {
            return Err(anyhow::anyhow!("最大并发工具调用数必须大于0"));
        }

        // 验证工具重试次数
        if self.tool_registry.tool_retries == 0 {
            return Err(anyhow::anyhow!("工具重试次数必须大于0"));
        }

        // 验证工作区根目录
        if !self.workspace_root.exists() {
            return Err(anyhow::anyhow!(
                "工作区根目录不存在: {}",
                self.workspace_root.display()
            ));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // 如果无法创建配置，返回一个基本的默认配置
            Config {
                session_id: "default".to_string(),
                models: ModelConfig {
                    default_model: "gpt-4o".to_string(),
                    max_tokens: 4096,
                    temperature: 0.7,
                    request_timeout: 30,
                    max_retries: 3,
                },
                sandbox: SandboxConfig {
                    enabled: false,
                    directory: None,
                    allowed_commands: vec![],
                    forbidden_commands: vec![],
                },
                tool_registry: ToolRegistryConfig {
                    enabled: true,
                    tool_timeout: 30,
                    max_concurrent_tools: 5,
                    tool_retries: 3,
                },
                deepseek_api: DeepSeekApiConfig {
                    api_key: String::new(),
                    api_endpoint: "https://api.deepseek.com".to_string(),
                    streaming: true,
                    headers: HashMap::new(),
                },
                system_prompt_config: None,
                workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                mcp_config: None,
            }
        })
    }
}

/// 加载系统提示配置
fn load_system_prompt_config() -> Result<Option<SystemPromptConfig>> {
    // 尝试从环境变量加载系统提示配置
    if let Some(config_str) = std::env::var("SYSTEM_PROMPT_CONFIG").ok() {
        let config: SystemPromptConfig = serde_json::from_str(&config_str)
            .with_context(|| "无法解析系统提示配置")?;
        return Ok(Some(config));
    }

    // 尝试从文件加载系统提示配置
    let config_path = get_workspace_root().join("system-prompt-config.json");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("无法读取系统提示配置文件: {}", config_path.display()))?;
        
        let config: SystemPromptConfig = serde_json::from_str(&content)
            .with_context(|| "无法解析系统提示配置文件")?;
        
        return Ok(Some(config));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_new() {
        // 设置必要的环境变量
        env::set_var("DEEPSEEK_API_KEY", "test-key");
        env::set_var("DEEPSEEK_API_ENDPOINT", "https://api.deepseek.com");
        env::set_var("OPENAI_MODEL", "gpt-4");
        
        let config = Config::new();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.get_default_model(), "gpt-4");
        assert_eq!(config.get_api_endpoint(), "https://api.deepseek.com");
        assert!(config.get_api_key().contains("test-key"));
        
        // 清理环境变量
        env::remove_var("DEEPSEEK_API_KEY");
        env::remove_var("DEEPSEEK_API_ENDPOINT");
        env::remove_var("OPENAI_MODEL");
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // 测试有效配置
        assert!(config.validate().is_ok());
        
        // 测试无效配置
        config.deepseek_api.api_key = String::new();
        assert!(config.validate().is_err());
        
        config.deepseek_api.api_key = "test-key".to_string();
        config.models.max_tokens = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sandbox_commands() {
        let mut config = Config::default();
        
        // 测试默认命令检查
        assert!(config.is_command_allowed("ls"));
        assert!(config.is_command_allowed("pwd"));
        
        // 添加禁止的命令
        config.add_forbidden_command("rm".to_string());
        assert!(!config.is_command_allowed("rm"));
        
        // 移除禁止的命令
        config.remove_forbidden_command("rm");
        assert!(config.is_command_allowed("rm"));
    }

    #[test]
    fn test_config_updates() {
        let mut config = Config::default();
        
        // 测试更新API密钥
        config.update_api_key("new-key".to_string());
        assert_eq!(config.get_api_key(), "new-key");
        
        // 测试更新模型
        config.update_default_model("gpt-3.5-turbo".to_string());
        assert_eq!(config.get_default_model(), "gpt-3.5-turbo");
        
        // 测试更新温度
        config.update_temperature(0.5);
        assert_eq!(config.get_temperature(), 0.5);
    }

    #[test]
    fn test_config_summary() {
        let config = Config::default();
        let summary = config.get_summary();
        
        assert!(summary.contains_key("session_id"));
        assert!(summary.contains_key("default_model"));
        assert!(summary.contains_key("max_tokens"));
        assert!(summary.contains_key("temperature"));
        assert!(summary.contains_key("workspace_root"));
    }
}
