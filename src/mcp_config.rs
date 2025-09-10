use crate::types::{McpServerConfig, McpConfig, OAuthConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

/// 原始MCP服务器配置结构（从mcp.json文件读取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMcpServerConfig {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
    pub url: Option<String>,
    pub http_url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub tcp: Option<String>,
    pub timeout: Option<u64>,
    pub trust: Option<bool>,
    pub description: Option<String>,
    pub include_tools: Option<Vec<String>>,
    pub exclude_tools: Option<Vec<String>>,
    pub extension_name: Option<String>,
    pub oauth: Option<serde_json::Value>,
    pub auth_provider_type: Option<String>,
}

/// MCP配置加载器
pub struct McpConfigLoader;

impl McpConfigLoader {
    /// 从mcp.json文件加载MCP服务器配置
    /// 
    /// # Arguments
    /// * `project_root` - 项目根目录
    /// 
    /// # Returns
    /// MCP服务器配置的映射，如果文件不存在则返回None
    pub fn load_mcp_config() -> Result<Option<McpServerConfig>> {
        // 使用当前工作目录作为项目根目录
        let project_root = std::env::current_dir()
            .context("Failed to get current directory")?
            .to_string_lossy()
            .to_string();
        
        match Self::load_mcp_config_with_root(&project_root)? {
            Some(servers) => {
                // 返回第一个服务器配置，或者创建一个默认配置
                if let Some((_, config)) = servers.into_iter().next() {
                    Ok(Some(config))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
    
    pub fn load_mcp_config_with_root(project_root: &str) -> Result<Option<HashMap<String, McpServerConfig>>> {
        let mcp_config_path = Path::new(project_root).join("mcp.json");
        
        // 检查mcp.json文件是否存在
        if !mcp_config_path.exists() {
            return Ok(None);
        }
        
        // 读取并解析mcp.json文件
        let mcp_config_content = fs::read_to_string(&mcp_config_path)
            .with_context(|| format!("无法读取MCP配置文件: {}", mcp_config_path.display()))?;
        
        let mcp_config: serde_json::Value = serde_json::from_str(&mcp_config_content)
            .with_context(|| "无法解析MCP配置文件JSON")?;
        
        // 转换原始配置对象为McpServerConfig实例
        let mut mcp_servers = HashMap::new();
        
        if let Some(servers) = mcp_config.get("mcpServers") {
            if let Some(servers_obj) = servers.as_object() {
                for (server_name, server_config) in servers_obj {
                    let raw_config: RawMcpServerConfig = serde_json::from_value(server_config.clone())
                        .with_context(|| format!("无法解析服务器配置: {}", server_name))?;
                    
                    let config = Self::convert_raw_config(raw_config)?;
                    mcp_servers.insert(server_name.clone(), config);
                }
            }
        }
        
        Ok(Some(mcp_servers))
    }

    /// 将原始配置转换为McpServerConfig
    fn convert_raw_config(raw: RawMcpServerConfig) -> Result<McpServerConfig> {
        let oauth = if let Some(oauth_value) = raw.oauth {
            Some(serde_json::from_value(oauth_value)?)
        } else {
            None
        };

        Ok(McpServerConfig {
            command: raw.command,
            args: raw.args,
            env: raw.env,
            cwd: raw.cwd,
            url: raw.url,
            http_url: raw.http_url,
            headers: raw.headers,
            timeout: raw.timeout,
            trust: raw.trust,
            include_tools: raw.include_tools,
            exclude_tools: raw.exclude_tools,
            oauth,
        })
    }

    /// 从多个可能的路径加载MCP配置
    /// 
    /// # Arguments
    /// * `possible_paths` - 可能的配置文件路径列表
    /// 
    /// # Returns
    /// 找到的第一个有效配置
    pub fn load_from_possible_paths(possible_paths: Vec<PathBuf>) -> Result<Option<HashMap<String, McpServerConfig>>> {
        for path in possible_paths {
            if path.exists() {
                if let Some(parent) = path.parent() {
                    return Self::load_mcp_config_with_root(parent.to_str().unwrap_or("."));
                }
            }
        }
        Ok(None)
    }

    /// 保存MCP配置到文件
    /// 
    /// # Arguments
    /// * `config` - MCP配置
    /// * `file_path` - 保存路径
    pub fn save_mcp_config(config: &McpConfig, file_path: &Path) -> Result<()> {
        let json_content = serde_json::to_string_pretty(config)
            .with_context(|| "无法序列化MCP配置")?;
        
        fs::write(file_path, json_content)
            .with_context(|| format!("无法写入MCP配置文件: {}", file_path.display()))?;
        
        Ok(())
    }

    /// 验证MCP配置
    /// 
    /// # Arguments
    /// * `config` - 要验证的配置
    /// 
    /// # Returns
    /// 验证结果
    pub fn validate_config(config: &McpConfig) -> Result<()> {
        for (server_name, server_config) in &config.mcp_servers {
            Self::validate_server_config(server_name, server_config)?;
        }
        Ok(())
    }

    /// 验证单个服务器配置
    fn validate_server_config(server_name: &str, config: &McpServerConfig) -> Result<()> {
        // 检查是否有有效的连接方式
        let has_command = config.command.is_some();
        let has_url = config.url.is_some();
        let has_http_url = config.http_url.is_some();

        if !has_command && !has_url && !has_http_url {
            return Err(anyhow::anyhow!(
                "服务器 '{}' 缺少连接配置：需要 command、url 或 http_url 中的至少一个",
                server_name
            ));
        }

        // 验证URL格式
        if let Some(url) = &config.url {
            url::Url::parse(url)
                .with_context(|| format!("服务器 '{}' 的URL格式无效: {}", server_name, url))?;
        }

        if let Some(http_url) = &config.http_url {
            url::Url::parse(http_url)
                .with_context(|| format!("服务器 '{}' 的HTTP URL格式无效: {}", server_name, http_url))?;
        }

        // 验证超时设置
        if let Some(timeout) = config.timeout {
            if timeout == 0 {
                return Err(anyhow::anyhow!(
                    "服务器 '{}' 的超时时间不能为0",
                    server_name
                ));
            }
        }

        Ok(())
    }
}

/// MCP配置构建器
pub struct McpConfigBuilder {
    servers: HashMap<String, McpServerConfig>,
}

impl McpConfigBuilder {
    /// 创建新的配置构建器
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    /// 添加服务器配置
    /// 
    /// # Arguments
    /// * `name` - 服务器名称
    /// * `config` - 服务器配置
    pub fn add_server(mut self, name: String, config: McpServerConfig) -> Self {
        self.servers.insert(name, config);
        self
    }

    /// 添加stdio服务器
    /// 
    /// # Arguments
    /// * `name` - 服务器名称
    /// * `command` - 命令
    /// * `args` - 参数
    pub fn add_stdio_server(mut self, name: String, command: String, args: Vec<String>) -> Self {
        let config = McpServerConfig {
            command: Some(command),
            args: Some(args),
            env: None,
            cwd: None,
            url: None,
            http_url: None,
            headers: None,
            timeout: None,
            trust: None,
            include_tools: None,
            exclude_tools: None,
            oauth: None,
        };
        self.servers.insert(name, config);
        self
    }

    /// 添加HTTP服务器
    /// 
    /// # Arguments
    /// * `name` - 服务器名称
    /// * `url` - 服务器URL
    /// * `headers` - 可选的请求头
    pub fn add_http_server(mut self, name: String, url: String, headers: Option<HashMap<String, String>>) -> Self {
        let config = McpServerConfig {
            command: None,
            args: None,
            env: None,
            cwd: None,
            url: Some(url),
            http_url: None,
            headers,
            timeout: None,
            trust: None,
            include_tools: None,
            exclude_tools: None,
            oauth: None,
        };
        self.servers.insert(name, config);
        self
    }

    /// 添加Streamable HTTP服务器
    /// 
    /// # Arguments
    /// * `name` - 服务器名称
    /// * `http_url` - HTTP URL
    /// * `headers` - 可选的请求头
    pub fn add_streamable_http_server(mut self, name: String, http_url: String, headers: Option<HashMap<String, String>>) -> Self {
        let config = McpServerConfig {
            command: None,
            args: None,
            env: None,
            cwd: None,
            url: None,
            http_url: Some(http_url),
            headers,
            timeout: None,
            trust: None,
            include_tools: None,
            exclude_tools: None,
            oauth: None,
        };
        self.servers.insert(name, config);
        self
    }

    /// 构建MCP配置
    pub fn build(self) -> McpConfig {
        McpConfig {
            mcp_servers: self.servers,
        }
    }
}

impl Default for McpConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP配置工厂
pub struct McpConfigFactory;

impl McpConfigFactory {
    /// 创建默认配置
    pub fn create_default() -> McpConfig {
        McpConfigBuilder::new()
            .add_stdio_server(
                "filesystem".to_string(),
                "npx".to_string(),
                vec!["@modelcontextprotocol/server-filesystem".to_string(), "/Users".to_string()],
            )
            .build()
    }

    /// 创建开发环境配置
    pub fn create_development() -> McpConfig {
        McpConfigBuilder::new()
            .add_stdio_server(
                "filesystem".to_string(),
                "npx".to_string(),
                vec!["@modelcontextprotocol/server-filesystem".to_string(), "/Users".to_string()],
            )
            .add_stdio_server(
                "memory".to_string(),
                "npx".to_string(),
                vec!["@modelcontextprotocol/server-memory".to_string()],
            )
            .build()
    }

    /// 创建生产环境配置
    pub fn create_production() -> McpConfig {
        McpConfigBuilder::new()
            .add_stdio_server(
                "filesystem".to_string(),
                "npx".to_string(),
                vec!["@modelcontextprotocol/server-filesystem".to_string(), "/Users".to_string()],
            )
            .build()
    }

    /// 从环境变量创建配置
    pub fn from_env() -> Result<McpConfig> {
        let mut builder = McpConfigBuilder::new();

        // 从环境变量读取服务器配置
        if let Ok(servers_json) = std::env::var("ALOU_MCP_SERVERS") {
            let servers: HashMap<String, RawMcpServerConfig> = serde_json::from_str(&servers_json)?;
            for (name, raw_config) in servers {
                let config = McpConfigLoader::convert_raw_config(raw_config)?;
                builder = builder.add_server(name, config);
            }
        }

        Ok(builder.build())
    }
}

/// 配置工具函数
pub struct ConfigUtils;

impl ConfigUtils {
    /// 合并两个配置
    /// 
    /// # Arguments
    /// * `base` - 基础配置
    /// * `override_config` - 覆盖配置
    /// 
    /// # Returns
    /// 合并后的配置
    pub fn merge_configs(base: McpConfig, override_config: McpConfig) -> McpConfig {
        let mut merged_servers = base.mcp_servers;
        
        for (name, config) in override_config.mcp_servers {
            merged_servers.insert(name, config);
        }
        
        McpConfig {
            mcp_servers: merged_servers,
        }
    }

    /// 过滤配置中的服务器
    /// 
    /// # Arguments
    /// * `config` - 原始配置
    /// * `server_names` - 要保留的服务器名称列表
    /// 
    /// # Returns
    /// 过滤后的配置
    pub fn filter_servers(config: McpConfig, server_names: Vec<String>) -> McpConfig {
        let filtered_servers: HashMap<String, McpServerConfig> = config
            .mcp_servers
            .into_iter()
            .filter(|(name, _)| server_names.contains(name))
            .collect();
        
        McpConfig {
            mcp_servers: filtered_servers,
        }
    }

    /// 获取配置摘要
    /// 
    /// # Arguments
    /// * `config` - MCP配置
    /// 
    /// # Returns
    /// 配置摘要字符串
    pub fn get_config_summary(config: &McpConfig) -> String {
        let mut summary = format!("MCP配置摘要 ({} 个服务器):\n", config.mcp_servers.len());
        
        for (name, server_config) in &config.mcp_servers {
            let connection_type = if server_config.command.is_some() {
                "stdio"
            } else if server_config.http_url.is_some() {
                "streamable_http"
            } else if server_config.url.is_some() {
                "http"
            } else {
                "unknown"
            };
            
            summary.push_str(&format!("  - {}: {}\n", name, connection_type));
        }
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_load_mcp_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");
        
        let config_content = r#"{
            "mcpServers": {
                "test_server": {
                    "command": "npx",
                    "args": ["@modelcontextprotocol/server-filesystem", "/Users"],
                    "timeout": 30000
                }
            }
        }"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let result = McpConfigLoader::load_mcp_config(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert!(config.is_some());
        
        let servers = config.unwrap();
        assert!(servers.contains_key("test_server"));
    }

    #[test]
    fn test_config_builder() {
        let config = McpConfigBuilder::new()
            .add_stdio_server(
                "test".to_string(),
                "npx".to_string(),
                vec!["test-server".to_string()],
            )
            .build();
        
        assert_eq!(config.mcp_servers.len(), 1);
        assert!(config.mcp_servers.contains_key("test"));
    }

    #[test]
    fn test_config_validation() {
        let valid_config = McpConfigBuilder::new()
            .add_stdio_server(
                "test".to_string(),
                "npx".to_string(),
                vec!["test-server".to_string()],
            )
            .build();
        
        assert!(McpConfigLoader::validate_config(&valid_config).is_ok());
    }

    #[test]
    fn test_config_utils() {
        let config1 = McpConfigBuilder::new()
            .add_stdio_server("server1".to_string(), "cmd1".to_string(), vec![])
            .build();
        
        let config2 = McpConfigBuilder::new()
            .add_stdio_server("server2".to_string(), "cmd2".to_string(), vec![])
            .build();
        
        let merged = ConfigUtils::merge_configs(config1, config2);
        assert_eq!(merged.mcp_servers.len(), 2);
        
        let filtered = ConfigUtils::filter_servers(merged, vec!["server1".to_string()]);
        assert_eq!(filtered.mcp_servers.len(), 1);
    }
}
