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

    info!("列出所有MCP服务器工具");

    // 读取配置文件
    let config = McpConfig::from_file("mcp.json")?;
    
    // 创建连接池
    let pool = ConnectionPool::new();

    // 注册所有服务器配置
    for (name, server_config) in config.mcp_servers {
        pool.register_server(name, server_config).await;
    }

    println!("=== 列出所有MCP服务器工具 ===\n");

    // 获取所有已注册的服务器
    let servers = pool.list_registered_servers().await;
    
    for server_name in &servers {
        println!("🔧 服务器: {}", server_name);
        println!("{}", "=".repeat(50));
        
        if let Ok(connection) = pool.get_connection(server_name).await {
            let client = connection.lock().await;
            
            // 获取工具列表
            match client.request("tools/list", None).await {
                Ok(result) => {
                    if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                        println!("可用工具数量: {}", tools.len());
                        for (i, tool) in tools.iter().enumerate() {
                            if let (Some(name), Some(description)) = (
                                tool.get("name").and_then(|n| n.as_str()),
                                tool.get("description").and_then(|d| d.as_str())
                            ) {
                                println!("{}. {} - {}", i + 1, name, description);
                            }
                        }
                    } else {
                        println!("未找到工具列表");
                    }
                }
                Err(e) => {
                    println!("获取工具列表失败: {}", e);
                }
            }
        } else {
            println!("连接服务器失败");
        }
        
        println!("\n");
    }

    // 清理连接
    println!("=== 清理连接 ===");
    pool.close_all_connections().await?;
    println!("所有连接已关闭");

    Ok(())
}
