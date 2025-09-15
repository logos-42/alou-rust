use mcp_client_rs::connection_pool::{ConnectionPool, McpServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use std::time::Instant;

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

    info!("MCP连接池测试程序启动");

    // 读取配置文件
    let config = McpConfig::from_file("mcp.json")?;
    
    println!("=== 可用的MCP服务器 ===");
    for server_name in config.mcp_servers.keys() {
        println!("- {}", server_name);
    }

    // 创建连接池
    let pool = ConnectionPool::new();

    // 注册所有服务器配置
    for (name, server_config) in config.mcp_servers {
        pool.register_server(name, server_config).await;
    }

    // 测试连接池性能
    println!("\n=== 测试连接池性能 ===");
    
    // 第一次连接（创建新连接）
    let start = Instant::now();
    let client1 = pool.get_connection("filesystem").await?;
    let first_connection_time = start.elapsed();
    println!("第一次连接耗时: {:?}", first_connection_time);

    // 测试工具列表
    {
        let client = client1.lock().await;
        let tools_result = client.list_tools().await?;
        println!("📋 filesystem工具数量: {}", tools_result.tools.len());
    }

    // 第二次连接（复用现有连接）
    let start = Instant::now();
    let client2 = pool.get_connection("filesystem").await?;
    let second_connection_time = start.elapsed();
    println!("第二次连接耗时: {:?}", second_connection_time);

    // 验证是同一个连接
    println!("是否为同一连接: {}", Arc::ptr_eq(&client1, &client2));

    // 测试多个服务器
    println!("\n=== 测试多个服务器连接 ===");
    
    let servers = ["filesystem", "memory", "payment-npm"];
    for server_name in &servers {
        let start = Instant::now();
        let client = pool.get_connection(server_name).await?;
        let connection_time = start.elapsed();
        
        {
            let client_guard = client.lock().await;
            let tools_result = client_guard.list_tools().await?;
            println!("✅ {} 连接耗时: {:?}, 工具数量: {}", 
                server_name, connection_time, tools_result.tools.len());
        }
    }

    // 显示活跃连接
    println!("\n=== 活跃连接列表 ===");
    let active_connections = pool.list_active_connections().await;
    for conn in &active_connections {
        println!("- {}", conn);
    }

    // 测试连接健康检查
    println!("\n=== 连接健康检查 ===");
    for server_name in &servers {
        let is_healthy = pool.is_connection_healthy(server_name).await;
        println!("{} 连接健康状态: {}", server_name, if is_healthy { "✅ 健康" } else { "❌ 不健康" });
    }

    // 测试多次调用同一工具（验证连接复用）
    println!("\n=== 测试连接复用 ===");
    let client = pool.get_connection("filesystem").await?;
    
    for i in 1..=3 {
        let start = Instant::now();
        {
            let client_guard = client.lock().await;
            let _tools_result = client_guard.list_tools().await?;
        }
        let call_time = start.elapsed();
        println!("第{}次调用耗时: {:?}", i, call_time);
    }

    // 清理连接
    println!("\n=== 清理连接 ===");
    pool.close_all_connections().await?;
    println!("所有连接已关闭");

    Ok(())
}
