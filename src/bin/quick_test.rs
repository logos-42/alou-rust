use alou::connection_pool::{ConnectionPool, McpServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McpConfig {
    #[serde(rename = "mcpServers")]
    mcp_servers: HashMap<String, McpServerConfig>,
}

impl McpConfig {
    fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: McpConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = McpConfig::from_file("mcp.json")?;
    let pool = ConnectionPool::new();
    
    for (name, server_config) in config.mcp_servers {
        pool.register_server(name, server_config).await;
    }

    let client = pool.get_connection("filesystem").await?;
    let client_guard = client.lock().await;
    
    // 测试创建目录
    println!("创建目录 '444'...");
    let result = client_guard.call_tool("create_directory", serde_json::json!({
        "path": "444"
    })).await?;
    
    println!("结果: {:?}", result);
    
    pool.close_all_connections().await?;
    Ok(())
}
