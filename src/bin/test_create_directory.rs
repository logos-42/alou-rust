use alou::connection_pool::{ConnectionPool, McpServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::info;

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
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("测试create_directory工具");

    // 读取配置文件
    let config = McpConfig::from_file("mcp.json")?;
    
    // 创建连接池
    let pool = ConnectionPool::new();

    // 注册所有服务器配置
    for (name, server_config) in config.mcp_servers {
        pool.register_server(name, server_config).await;
    }

    println!("=== 测试create_directory工具 ===\n");

    // 测试创建目录
    let client = pool.get_connection("filesystem").await?;
    let client_guard = client.lock().await;
    
    println!("1. 创建目录 '120':");
    let result = client_guard.call_tool("create_directory", serde_json::json!({
        "path": "120"
    })).await?;
    
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   结果: {}", text);
        }
    }
    
    println!("\n2. 列出当前目录验证目录是否创建成功:");
    let result = client_guard.call_tool("list_directory", serde_json::json!({
        "path": "."
    })).await?;
    
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   目录内容:\n{}", text);
        }
    }
    
    println!("\n✅ create_directory工具测试完成\n");

    // 清理连接
    println!("=== 清理连接 ===");
    pool.close_all_connections().await?;
    println!("所有连接已关闭");

    Ok(())
}
