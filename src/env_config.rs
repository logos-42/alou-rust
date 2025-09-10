use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use dotenv::dotenv;

/// 从用户工作目录或包目录加载.env文件
/// 优先从用户工作目录加载，如果不存在则从包目录加载
pub fn load_env_file() -> Result<()> {
    // 首先尝试从用户工作目录加载.env文件
    if let Ok(current_dir) = std::env::current_dir() {
        let user_env_path = current_dir.join(".env");
        if user_env_path.exists() {
            dotenv::from_path(&user_env_path)?;
            return Ok(());
        }
    }

    // 如果用户工作目录没有.env文件，尝试从包目录加载
    if let Ok(package_dir) = get_package_directory() {
        let package_env_path = package_dir.join(".env");
        if package_env_path.exists() {
            dotenv::from_path(&package_env_path)?;
        }
    }

    Ok(())
}

/// 获取包目录路径
fn get_package_directory() -> Result<PathBuf> {
    // 尝试从CARGO_MANIFEST_DIR环境变量获取包目录
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return Ok(PathBuf::from(manifest_dir));
    }

    // 如果CARGO_MANIFEST_DIR不可用，尝试从当前可执行文件路径推断
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return Ok(parent.to_path_buf());
        }
    }

    // 最后回退到当前工作目录
    std::env::current_dir().context("无法获取包目录")
}

/// 获取DeepSeek API密钥
/// 从环境变量DEEPSEEK_API_KEY获取
pub fn get_deepseek_api_key() -> Result<String> {
    std::env::var("DEEPSEEK_API_KEY")
        .context("DEEPSEEK_API_KEY环境变量未设置")
}

/// 获取DeepSeek API端点
/// 从环境变量DEEPSEEK_API_ENDPOINT获取，默认为https://api.deepseek.com
pub fn get_deepseek_api_endpoint() -> String {
    std::env::var("DEEPSEEK_API_ENDPOINT")
        .unwrap_or_else(|_| "https://api.deepseek.com".to_string())
}

/// 获取OpenAI API密钥（用于兼容性）
/// 从环境变量OPENAI_API_KEY获取
pub fn get_openai_api_key() -> Result<String> {
    std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY环境变量未设置")
}

/// 获取OpenAI API端点（用于兼容性）
/// 从环境变量OPENAI_BASE_URL获取，默认为https://api.openai.com/v1
pub fn get_openai_api_endpoint() -> String {
    std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
}

/// 获取OpenAI模型名称
/// 从环境变量OPENAI_MODEL获取，默认为gpt-4o
pub fn get_openai_model() -> String {
    std::env::var("OPENAI_MODEL")
        .unwrap_or_else(|_| "gpt-4o".to_string())
}

/// 获取沙盒环境设置
/// 从环境变量SANDBOX获取，默认为空字符串
pub fn get_sandbox() -> String {
    std::env::var("SANDBOX")
        .unwrap_or_else(|_| String::new())
}

/// 获取会话ID
/// 从环境变量SESSION_ID获取，如果未设置则生成一个
pub fn get_session_id() -> String {
    std::env::var("SESSION_ID")
        .unwrap_or_else(|_| {
            use uuid::Uuid;
            Uuid::new_v4().to_string()
        })
}

/// 获取Gemini系统提示文件路径
/// 从环境变量GEMINI_SYSTEM_MD获取
pub fn get_gemini_system_md() -> Option<String> {
    std::env::var("GEMINI_SYSTEM_MD").ok()
}

/// 获取Gemini写入系统提示文件设置
/// 从环境变量GEMINI_WRITE_SYSTEM_MD获取
pub fn get_gemini_write_system_md() -> Option<String> {
    std::env::var("GEMINI_WRITE_SYSTEM_MD").ok()
}

/// 检查是否启用了调试模式
/// 从环境变量DEBUG获取，默认为false
pub fn is_debug_enabled() -> bool {
    std::env::var("DEBUG")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}

/// 获取日志级别
/// 从环境变量LOG_LEVEL获取，默认为info
pub fn get_log_level() -> String {
    std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
}

/// 获取最大重试次数
/// 从环境变量MAX_RETRIES获取，默认为3
pub fn get_max_retries() -> usize {
    std::env::var("MAX_RETRIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

/// 获取请求超时时间（秒）
/// 从环境变量REQUEST_TIMEOUT获取，默认为30
pub fn get_request_timeout() -> u64 {
    std::env::var("REQUEST_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
}

/// 获取最大令牌数
/// 从环境变量MAX_TOKENS获取，默认为4096
pub fn get_max_tokens() -> usize {
    std::env::var("MAX_TOKENS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4096)
}

/// 获取温度设置
/// 从环境变量TEMPERATURE获取，默认为0.7
pub fn get_temperature() -> f64 {
    std::env::var("TEMPERATURE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.7)
}

/// 获取MCP配置目录
/// 从环境变量MCP_CONFIG_DIR获取，默认为当前目录
pub fn get_mcp_config_dir() -> PathBuf {
    std::env::var("MCP_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// 获取MCP配置文件路径
/// 默认为mcp.json
pub fn get_mcp_config_path() -> PathBuf {
    get_mcp_config_dir().join("mcp.json")
}

/// 获取工作区根目录
/// 从环境变量WORKSPACE_ROOT获取，默认为当前目录
pub fn get_workspace_root() -> PathBuf {
    std::env::var("WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// 获取允许的目录列表
/// 从环境变量ALLOWED_DIRECTORIES获取，以逗号分隔
pub fn get_allowed_directories() -> Vec<PathBuf> {
    std::env::var("ALLOWED_DIRECTORIES")
        .map(|v| {
            v.split(',')
                .map(|s| PathBuf::from(s.trim()))
                .collect()
        })
        .unwrap_or_else(|_| vec![get_workspace_root()])
}

/// 检查是否启用了沙盒模式
/// 从环境变量SANDBOX_MODE获取，默认为false
pub fn is_sandbox_mode() -> bool {
    std::env::var("SANDBOX_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}

/// 获取用户代理字符串
/// 从环境变量USER_AGENT获取，默认为Alou3-Rust/0.1.0
pub fn get_user_agent() -> String {
    std::env::var("USER_AGENT")
        .unwrap_or_else(|_| "Alou3-Rust/0.1.0".to_string())
}

/// 获取代理设置
/// 从环境变量HTTP_PROXY和HTTPS_PROXY获取
pub fn get_proxy_settings() -> (Option<String>, Option<String>) {
    let http_proxy = std::env::var("HTTP_PROXY").ok();
    let https_proxy = std::env::var("HTTPS_PROXY").ok();
    (http_proxy, https_proxy)
}

/// 获取缓存目录
/// 从环境变量CACHE_DIR获取，默认为系统临时目录下的alou3-cache
pub fn get_cache_dir() -> PathBuf {
    std::env::var("CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::temp_dir().join("alou3-cache")
        })
}

/// 获取日志文件路径
/// 从环境变量LOG_FILE获取，默认为cache目录下的alou3.log
pub fn get_log_file() -> PathBuf {
    std::env::var("LOG_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            get_cache_dir().join("alou3.log")
        })
}

/// 初始化环境配置
/// 加载.env文件并设置默认环境变量
pub fn init_env_config() -> Result<()> {
    // 加载.env文件
    load_env_file()?;

    // 设置默认环境变量（如果未设置）
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", &get_log_level());
    }

    if std::env::var("RUST_BACKTRACE").is_err() && is_debug_enabled() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    Ok(())
}

/// 验证必需的环境变量
/// 检查所有必需的环境变量是否已设置
pub fn validate_required_env() -> Result<()> {
    // 检查API密钥
    if get_deepseek_api_key().is_err() && get_openai_api_key().is_err() {
        return Err(anyhow::anyhow!(
            "必须设置DEEPSEEK_API_KEY或OPENAI_API_KEY环境变量"
        ));
    }

    // 检查工作区根目录是否存在
    let workspace_root = get_workspace_root();
    if !workspace_root.exists() {
        return Err(anyhow::anyhow!(
            "工作区根目录不存在: {}",
            workspace_root.display()
        ));
    }

    // 检查MCP配置文件是否存在
    let mcp_config_path = get_mcp_config_path();
    if !mcp_config_path.exists() {
        // 创建默认的MCP配置文件
        create_default_mcp_config()?;
    }

    Ok(())
}

/// 创建默认的MCP配置文件
fn create_default_mcp_config() -> Result<()> {
    let mcp_config_path = get_mcp_config_path();
    
    // 确保目录存在
    if let Some(parent) = mcp_config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 创建默认配置
    let default_config = r#"{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "env": {}
    },
    "fetch": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-fetch"],
      "env": {}
    },
    "memory": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"],
      "env": {}
    }
  }
}"#;

    fs::write(&mcp_config_path, default_config)?;
    Ok(())
}

/// 获取所有环境变量的摘要
/// 用于调试和配置验证
pub fn get_env_summary() -> std::collections::HashMap<String, String> {
    let mut summary = std::collections::HashMap::new();
    
    summary.insert("DEEPSEEK_API_KEY".to_string(), 
        get_deepseek_api_key().map(|_| "***".to_string()).unwrap_or_else(|_| "未设置".to_string()));
    summary.insert("DEEPSEEK_API_ENDPOINT".to_string(), get_deepseek_api_endpoint());
    summary.insert("OPENAI_API_KEY".to_string(), 
        get_openai_api_key().map(|_| "***".to_string()).unwrap_or_else(|_| "未设置".to_string()));
    summary.insert("OPENAI_BASE_URL".to_string(), get_openai_api_endpoint());
    summary.insert("OPENAI_MODEL".to_string(), get_openai_model());
    summary.insert("SANDBOX".to_string(), get_sandbox());
    summary.insert("SESSION_ID".to_string(), get_session_id());
    summary.insert("DEBUG".to_string(), is_debug_enabled().to_string());
    summary.insert("LOG_LEVEL".to_string(), get_log_level());
    summary.insert("MAX_RETRIES".to_string(), get_max_retries().to_string());
    summary.insert("REQUEST_TIMEOUT".to_string(), get_request_timeout().to_string());
    summary.insert("MAX_TOKENS".to_string(), get_max_tokens().to_string());
    summary.insert("TEMPERATURE".to_string(), get_temperature().to_string());
    summary.insert("WORKSPACE_ROOT".to_string(), get_workspace_root().to_string_lossy().to_string());
    summary.insert("MCP_CONFIG_PATH".to_string(), get_mcp_config_path().to_string_lossy().to_string());
    summary.insert("CACHE_DIR".to_string(), get_cache_dir().to_string_lossy().to_string());
    summary.insert("LOG_FILE".to_string(), get_log_file().to_string_lossy().to_string());
    
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_deepseek_api_endpoint() {
        env::set_var("DEEPSEEK_API_ENDPOINT", "https://custom.deepseek.com");
        assert_eq!(get_deepseek_api_endpoint(), "https://custom.deepseek.com");
        
        env::remove_var("DEEPSEEK_API_ENDPOINT");
        assert_eq!(get_deepseek_api_endpoint(), "https://api.deepseek.com");
    }

    #[test]
    fn test_get_openai_model() {
        env::set_var("OPENAI_MODEL", "gpt-4");
        assert_eq!(get_openai_model(), "gpt-4");
        
        env::remove_var("OPENAI_MODEL");
        assert_eq!(get_openai_model(), "gpt-4o");
    }

    #[test]
    fn test_is_debug_enabled() {
        env::set_var("DEBUG", "true");
        assert!(is_debug_enabled());
        
        env::set_var("DEBUG", "1");
        assert!(is_debug_enabled());
        
        env::set_var("DEBUG", "false");
        assert!(!is_debug_enabled());
        
        env::remove_var("DEBUG");
        assert!(!is_debug_enabled());
    }

    #[test]
    fn test_get_max_retries() {
        env::set_var("MAX_RETRIES", "5");
        assert_eq!(get_max_retries(), 5);
        
        env::remove_var("MAX_RETRIES");
        assert_eq!(get_max_retries(), 3);
    }

    #[test]
    fn test_get_temperature() {
        env::set_var("TEMPERATURE", "0.5");
        assert_eq!(get_temperature(), 0.5);
        
        env::remove_var("TEMPERATURE");
        assert_eq!(get_temperature(), 0.7);
    }

    #[test]
    fn test_get_allowed_directories() {
        env::set_var("ALLOWED_DIRECTORIES", "/tmp,/var/tmp");
        let dirs = get_allowed_directories();
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&PathBuf::from("/tmp")));
        assert!(dirs.contains(&PathBuf::from("/var/tmp")));
        
        env::remove_var("ALLOWED_DIRECTORIES");
        let dirs = get_allowed_directories();
        assert_eq!(dirs.len(), 1);
    }

    #[test]
    fn test_get_proxy_settings() {
        env::set_var("HTTP_PROXY", "http://proxy.example.com:8080");
        env::set_var("HTTPS_PROXY", "https://proxy.example.com:8080");
        
        let (http_proxy, https_proxy) = get_proxy_settings();
        assert_eq!(http_proxy, Some("http://proxy.example.com:8080".to_string()));
        assert_eq!(https_proxy, Some("https://proxy.example.com:8080".to_string()));
        
        env::remove_var("HTTP_PROXY");
        env::remove_var("HTTPS_PROXY");
    }
}
