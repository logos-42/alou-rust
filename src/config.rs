use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// 要执行的命令
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 工作目录（可选）
    pub directory: Option<String>,
    /// 环境变量（可选）
    pub env: Option<HashMap<String, String>>,
}

/// MCP配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// MCP服务器配置映射
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

impl McpConfig {
    /// 从文件读取MCP配置
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: McpConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 获取指定名称的服务器配置
    pub fn get_server(&self, name: &str) -> Option<&McpServerConfig> {
        self.mcp_servers.get(name)
    }

    /// 列出所有可用的服务器名称
    pub fn list_servers(&self) -> Vec<&String> {
        self.mcp_servers.keys().collect()
    }
}
