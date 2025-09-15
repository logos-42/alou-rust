use alou::connection_pool::{ConnectionPool, McpServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

    info!("MCP工具调用测试程序启动");

    // 读取配置文件
    let config = McpConfig::from_file("mcp.json")?;
    
    // 创建连接池
    let pool = ConnectionPool::new();

    // 注册所有服务器配置
    for (name, server_config) in config.mcp_servers {
        pool.register_server(name, server_config).await;
    }

    println!("=== 测试MCP工具调用 ===\n");

    // 测试filesystem工具
    test_filesystem_tools(&pool).await?;
    
    // 测试memory工具
    test_memory_tools(&pool).await?;
    
    // 测试payment工具
    test_payment_tools(&pool).await?;

    // 清理连接
    println!("\n=== 清理连接 ===");
    pool.close_all_connections().await?;
    println!("所有连接已关闭");

    Ok(())
}

async fn test_filesystem_tools(pool: &ConnectionPool) -> Result<()> {
    println!("🔧 测试filesystem工具");
    println!("{}", "=".repeat(50));
    
    let client = pool.get_connection("filesystem").await?;
    let mut client_guard = client.lock().await;
    
    // 1. 列出允许的目录
    println!("1. 列出允许的目录:");
    let result = client_guard.call_tool("list_allowed_directories", serde_json::json!({})).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 2. 列出当前目录
    println!("\n2. 列出当前目录:");
    let result = client_guard.call_tool("list_directory", serde_json::json!({
        "path": "."
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 3. 创建测试文件
    println!("\n3. 创建测试文件:");
    let result = client_guard.call_tool("write_file", serde_json::json!({
        "path": "test_mcp_file.txt",
        "content": "这是一个通过MCP客户端创建的测试文件！\n创建时间: 2025-09-15\n"
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 4. 读取刚创建的文件
    println!("\n4. 读取刚创建的文件:");
    let result = client_guard.call_tool("read_file", serde_json::json!({
        "path": "test_mcp_file.txt"
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   文件内容: {}", text);
        }
    }
    
    // 5. 获取文件信息
    println!("\n5. 获取文件信息:");
    let result = client_guard.call_tool("get_file_info", serde_json::json!({
        "path": "test_mcp_file.txt"
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    println!("\n✅ filesystem工具测试完成\n");
    Ok(())
}

async fn test_memory_tools(pool: &ConnectionPool) -> Result<()> {
    println!("🧠 测试memory工具");
    println!("{}", "=".repeat(50));
    
    let client = pool.get_connection("memory").await?;
    let mut client_guard = client.lock().await;
    
    // 1. 创建实体
    println!("1. 创建知识图谱实体:");
    let result = client_guard.call_tool("create_entities", serde_json::json!({
        "entities": [
            {
                "name": "MCP协议",
                "entityType": "技术",
                "observations": [
                    "Model Context Protocol的缩写",
                    "用于AI模型与运行时环境通信的协议",
                    "支持工具调用和资源访问"
                ]
            },
            {
                "name": "Rust",
                "entityType": "编程语言",
                "observations": [
                    "系统级编程语言",
                    "内存安全且高性能",
                    "适合构建MCP客户端"
                ]
            }
        ]
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 2. 创建关系
    println!("\n2. 创建实体关系:");
    let result = client_guard.call_tool("create_relations", serde_json::json!({
        "relations": [
            {
                "from": "Rust",
                "to": "MCP协议",
                "relationType": "实现"
            }
        ]
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 3. 搜索节点
    println!("\n3. 搜索知识图谱节点:");
    let result = client_guard.call_tool("search_nodes", serde_json::json!({
        "query": "MCP"
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 4. 读取整个图谱
    println!("\n4. 读取整个知识图谱:");
    let result = client_guard.call_tool("read_graph", serde_json::json!({})).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    println!("\n✅ memory工具测试完成\n");
    Ok(())
}

async fn test_payment_tools(pool: &ConnectionPool) -> Result<()> {
    println!("💰 测试payment工具");
    println!("{}", "=".repeat(50));
    
    let client = pool.get_connection("payment-npm").await?;
    let mut client_guard = client.lock().await;
    
    // 1. 获取网络信息
    println!("1. 获取网络信息:");
    let result = client_guard.call_tool("get_network_info", serde_json::json!({})).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 2. 获取支持的代币列表
    println!("\n2. 获取支持的代币列表:");
    let result = client_guard.call_tool("get_supported_tokens", serde_json::json!({})).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 3. 创建新钱包
    println!("\n3. 创建新钱包:");
    let result = client_guard.call_tool("create_wallet", serde_json::json!({
        "label": "test_wallet"
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 4. 列出所有钱包
    println!("\n4. 列出所有钱包:");
    let result = client_guard.call_tool("list_wallets", serde_json::json!({})).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 5. 估算Gas费用
    println!("\n5. 估算Gas费用:");
    let result = client_guard.call_tool("estimate_gas_fees", serde_json::json!({})).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   {}", text);
        }
    }
    
    // 6. 验证地址格式
    println!("\n6. 验证地址格式:");
    let test_address = "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6";
    let result = client_guard.call_tool("validate_address", serde_json::json!({
        "address": test_address
    })).await?;
    for content in &result.content {
        if let alou::types::MessageContent::Text { text } = content {
            println!("   地址 {} 验证结果: {}", test_address, text);
        }
    }
    
    println!("\n✅ payment工具测试完成\n");
    Ok(())
}
