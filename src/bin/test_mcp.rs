use alou::client::ClientBuilder;
use alou::types::ClientCapabilities;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McpServerConfig {
    command: String,
    args: Vec<String>,
    directory: Option<String>,
    env: Option<HashMap<String, String>>,
}

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

    info!("MCP客户端测试程序启动");

    // 读取配置文件
    let config = McpConfig::from_file("mcp.json")?;
    
    println!("=== 可用的MCP服务器 ===");
    for server_name in config.mcp_servers.keys() {
        println!("- {}", server_name);
    }

    // 测试所有服务器
    for (server_name, server_config) in &config.mcp_servers {
        println!("\n=== 测试{}服务器 ===", server_name);
        test_server(server_name, server_config).await?;
    }

    Ok(())
}

async fn test_server(name: &str, config: &McpServerConfig) -> Result<()> {
    info!("正在测试服务器: {}", name);
    
    let mut builder = ClientBuilder::new(&config.command);
    
    // 添加命令参数
    for arg in &config.args {
        builder = builder.arg(arg);
    }
    
    // 设置工作目录
    if let Some(dir) = &config.directory {
        builder = builder.directory(dir);
    }
    
    // 设置环境变量
    if let Some(env_vars) = &config.env {
        for (key, value) in env_vars {
            builder = builder.env(key, value);
        }
    }
    
    // 设置客户端实现信息
    builder = builder.implementation("mcp-test-client", "0.1.0");
    
    // 设置客户端能力
    let capabilities = ClientCapabilities {
        custom: None,
    };
    builder = builder.capabilities(capabilities);
    
    debug!("正在启动MCP服务器进程...");
    
    match builder.spawn_and_initialize().await {
        Ok(mut client) => {
            println!("✅ 服务器 {} 连接成功！", name);
            
            // 测试列出工具
            match client.list_tools().await {
                Ok(tools_result) => {
                    println!("📋 可用工具数量: {}", tools_result.tools.len());
                    for tool in &tools_result.tools {
                        println!("  - {}: {}", tool.name, tool.description);
                    }
                }
                Err(e) => {
                    println!("❌ 列出工具失败: {}", e);
                }
            }
            
            // 测试列出资源
            match client.list_resources().await {
                Ok(resources_result) => {
                    println!("📁 可用资源数量: {}", resources_result.resources.len());
                    for resource in &resources_result.resources {
                        println!("  - {}: {}", resource.uri, resource.title);
                    }
                }
                Err(e) => {
                    println!("❌ 列出资源失败: {}", e);
                }
            }
            
            // 关闭客户端
            client.shutdown().await?;
            println!("🔌 服务器 {} 已断开连接", name);
        }
        Err(e) => {
            println!("❌ 服务器 {} 连接失败: {}", name, e);
        }
    }
    
    Ok(())
}
