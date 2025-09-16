use alou::connection_pool::{ConnectionPool, McpServerConfig};
use alou::task_flow::{TaskFlowManager, TaskFlowBuilder, DefaultTaskExecutor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::info;
use std::sync::Arc;

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

    info!("智能体工具调用流程测试程序启动");

    // 读取配置文件
    let config = McpConfig::from_file("mcp.json")?;
    
    // 创建连接池
    let pool = Arc::new(ConnectionPool::new());

    // 注册所有服务器配置
    for (name, server_config) in config.mcp_servers {
        pool.register_server(name, server_config).await;
    }

    println!("=== 测试智能体工具调用流程 ===\n");

    // 创建任务流程管理器
    let executor = Arc::new(DefaultTaskExecutor);
    let flow_manager = TaskFlowManager::new(executor, pool.clone());

    // 测试1: 简单的文件系统操作流程
    test_filesystem_flow(&flow_manager).await?;
    
    // 测试2: 内存工具操作流程
    test_memory_flow(&flow_manager).await?;
    
    // 测试3: 支付工具操作流程
    test_payment_flow(&flow_manager).await?;

    // 清理连接
    println!("\n=== 清理连接 ===");
    pool.close_all_connections().await?;
    println!("所有连接已关闭");

    Ok(())
}

async fn test_filesystem_flow(flow_manager: &TaskFlowManager) -> Result<()> {
    println!("🔧 测试文件系统工具调用流程");
    println!("{}", "=".repeat(50));
    
    // 创建文件系统操作流程
    let flow = TaskFlowBuilder::new(
        "文件系统测试流程".to_string(),
        "测试文件系统工具调用".to_string()
    )
    .add_tool_call(
        "列出当前目录".to_string(),
        "列出当前目录的内容".to_string(),
        "list_directory".to_string(),
        {
            let mut args = HashMap::new();
            args.insert("path".to_string(), serde_json::Value::String(".".to_string()));
            args
        },
        Vec::new(),
    )
    .add_tool_call(
        "创建测试文件".to_string(),
        "创建一个测试文件".to_string(),
        "write_file".to_string(),
        {
            let mut args = HashMap::new();
            args.insert("path".to_string(), serde_json::Value::String("agent_test_file.txt".to_string()));
            args.insert("content".to_string(), serde_json::Value::String("这是通过智能体任务流程创建的测试文件！\n时间: 2025-09-16\n".to_string()));
            args
        },
        Vec::new(),
    )
    .add_tool_call(
        "读取测试文件".to_string(),
        "读取刚创建的测试文件".to_string(),
        "read_file".to_string(),
        {
            let mut args = HashMap::new();
            args.insert("path".to_string(), serde_json::Value::String("agent_test_file.txt".to_string()));
            args
        },
        Vec::new(),
    )
    .build();
    
    let flow_id = flow_manager.create_flow(flow.name, flow.description, flow.tasks).await?;
    
    // 执行流程
    let result = flow_manager.execute_flow(&flow_id).await?;
    
    println!("流程执行结果:");
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    println!("\n✅ 文件系统工具调用流程测试完成\n");
    Ok(())
}

async fn test_memory_flow(flow_manager: &TaskFlowManager) -> Result<()> {
    println!("🧠 测试内存工具调用流程");
    println!("{}", "=".repeat(50));
    
    // 创建内存操作流程
    let flow = TaskFlowBuilder::new(
        "内存工具测试流程".to_string(),
        "测试内存工具调用".to_string()
    )
    .add_tool_call(
        "创建知识实体".to_string(),
        "创建知识图谱实体".to_string(),
        "create_entities".to_string(),
        {
            let mut args = HashMap::new();
            args.insert("entities".to_string(), serde_json::json!([
                {
                    "name": "智能体测试",
                    "entityType": "测试",
                    "observations": [
                        "通过任务流程创建的测试实体",
                        "验证智能体工具调用功能"
                    ]
                }
            ]));
            args
        },
        Vec::new(),
    )
    .add_tool_call(
        "搜索实体".to_string(),
        "搜索刚创建的实体".to_string(),
        "search_nodes".to_string(),
        {
            let mut args = HashMap::new();
            args.insert("query".to_string(), serde_json::Value::String("智能体".to_string()));
            args
        },
        Vec::new(),
    )
    .build();
    
    let flow_id = flow_manager.create_flow(flow.name, flow.description, flow.tasks).await?;
    
    // 执行流程
    let result = flow_manager.execute_flow(&flow_id).await?;
    
    println!("流程执行结果:");
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    println!("\n✅ 内存工具调用流程测试完成\n");
    Ok(())
}

async fn test_payment_flow(flow_manager: &TaskFlowManager) -> Result<()> {
    println!("💰 测试支付工具调用流程");
    println!("{}", "=".repeat(50));
    
    // 创建支付操作流程
    let flow = TaskFlowBuilder::new(
        "支付工具测试流程".to_string(),
        "测试支付工具调用".to_string()
    )
    .add_tool_call(
        "获取网络信息".to_string(),
        "获取当前网络信息".to_string(),
        "get_network_info".to_string(),
        HashMap::new(),
        Vec::new(),
    )
    .add_tool_call(
        "获取支持的代币".to_string(),
        "获取支持的代币列表".to_string(),
        "get_supported_tokens".to_string(),
        HashMap::new(),
        Vec::new(),
    )
    .add_tool_call(
        "创建测试钱包".to_string(),
        "创建一个测试钱包".to_string(),
        "create_wallet".to_string(),
        {
            let mut args = HashMap::new();
            args.insert("label".to_string(), serde_json::Value::String("agent_test_wallet".to_string()));
            args
        },
        Vec::new(),
    )
    .build();
    
    let flow_id = flow_manager.create_flow(flow.name, flow.description, flow.tasks).await?;
    
    // 执行流程
    let result = flow_manager.execute_flow(&flow_id).await?;
    
    println!("流程执行结果:");
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    println!("\n✅ 支付工具调用流程测试完成\n");
    Ok(())
}
