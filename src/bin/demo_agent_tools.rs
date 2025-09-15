use std::env;
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error, warn};
use tokio;
use serde_json::json;

use alou::agent::{
    Agent, McpAgent, AgentConfig, DeepSeekConfig, BehaviorConfig, 
    WorkspaceConfig, ToolStrategy, ToolCall, ToolCallStatus
};
use alou::connection_pool::{ConnectionPool, McpServerConfig};

/// 演示智能体工具调用能力
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 智能体工具调用演示程序启动");
    
    // 创建测试配置
    let config = AgentConfig {
        deepseek: DeepSeekConfig {
            base_url: "https://api.deepseek.com".to_string(),
            api_key: "demo-key".to_string(),
            model: "deepseek-chat".to_string(),
            max_tokens: 2000,
            temperature: 0.7,
        },
        behavior: BehaviorConfig {
            max_retries: 3,
            timeout_seconds: 30,
            verbose_logging: true,
            tool_strategy: ToolStrategy::Auto,
        },
        workspace: WorkspaceConfig {
            directories: vec![".".to_string()],
            smart_detection: true,
            exclude_patterns: vec!["target".to_string(), "node_modules".to_string()],
        },
    };
    
    // 初始化连接池
    let pool = Arc::new(init_connection_pool().await?);
    
    // 创建智能体（使用指定的连接池）
    let mut agent = McpAgent::with_connection_pool(config, pool).await?;
    
    // 初始化智能体
    info!("🔍 正在初始化智能体...");
    agent.initialize().await?;
    info!("✅ 智能体初始化完成！");
    
    // 演示工具调用场景
    demo_file_operations(&mut agent).await?;
    demo_memory_operations(&mut agent).await?;
    demo_payment_operations(&mut agent).await?;
    
    info!("🎉 智能体工具调用演示完成！");
    
    Ok(())
}

/// 初始化MCP连接池
async fn init_connection_pool() -> Result<ConnectionPool> {
    let pool = ConnectionPool::new();
    
    // 从mcp.json加载服务器配置
    if std::path::Path::new("mcp.json").exists() {
        let content = std::fs::read_to_string("mcp.json")?;
        let mcp_config: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(servers) = mcp_config.get("mcpServers").and_then(|s| s.as_object()) {
            for (name, config) in servers {
                let server_config = McpServerConfig {
                    command: config.get("command")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string(),
                    args: config.get("args")
                        .and_then(|a| a.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                        .unwrap_or_default(),
                    env: Some(config.get("env")
                        .and_then(|e| e.as_object())
                        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect::<std::collections::HashMap<String, String>>())
                        .unwrap_or_default()),
                    directory: config.get("directory")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string()),
                };
                
                pool.register_server(name.clone(), server_config).await;
                info!("✅ 已注册MCP服务器: {}", name);
            }
        }
    }
    
    Ok(pool)
}

/// 演示文件操作工具调用
async fn demo_file_operations(agent: &mut McpAgent) -> Result<()> {
    info!("📁 演示文件操作工具调用...");
    
    // 模拟AI决定调用文件操作工具
    let tool_call = ToolCall {
        name: "read_file".to_string(),
        arguments: serde_json::from_value(json!({
            "file_path": "README.md"
        }))?,
        call_id: "file_read_001".to_string(),
        status: ToolCallStatus::Pending,
    };
    
    info!("🤖 AI决定调用工具: {}", tool_call.name);
    info!("📋 工具参数: {}", serde_json::to_string_pretty(&tool_call.arguments)?);
    
    // 执行工具调用
    match agent.execute_tool(&tool_call).await {
        Ok(result) => {
            info!("✅ 工具调用成功！");
            info!("📄 文件内容预览: {}", 
                serde_json::to_string(&result)?.chars().take(200).collect::<String>());
        }
        Err(e) => {
            info!("❌ 工具调用失败: {}", e);
        }
    }
    
    info!("✅ 文件操作演示完成");
    Ok(())
}

/// 演示记忆管理工具调用
async fn demo_memory_operations(agent: &mut McpAgent) -> Result<()> {
    info!("🧠 演示记忆管理工具调用...");
    
    // 模拟AI决定创建记忆实体
    let tool_call = ToolCall {
        name: "create_entities".to_string(),
        arguments: serde_json::from_value(json!({
            "entities": [
                {
                    "name": "智能体演示",
                    "entityType": "demo",
                    "observations": [
                        "这是一个智能体工具调用演示",
                        "演示了MCP协议的实际应用",
                        "时间: 2025-09-15"
                    ]
                }
            ]
        }))?,
        call_id: "memory_create_001".to_string(),
        status: ToolCallStatus::Pending,
    };
    
    info!("🤖 AI决定调用工具: {}", tool_call.name);
    info!("📋 工具参数: {}", serde_json::to_string_pretty(&tool_call.arguments)?);
    
    // 执行工具调用
    match agent.execute_tool(&tool_call).await {
        Ok(result) => {
            info!("✅ 工具调用成功！");
            info!("💾 记忆创建结果: {}", serde_json::to_string(&result)?);
        }
        Err(e) => {
            info!("❌ 工具调用失败: {}", e);
        }
    }
    
    info!("✅ 记忆管理演示完成");
    Ok(())
}

/// 演示支付工具调用
async fn demo_payment_operations(agent: &mut McpAgent) -> Result<()> {
    info!("💰 演示支付工具调用...");
    
    // 模拟AI决定查询支持的代币
    let tool_call = ToolCall {
        name: "get_supported_tokens".to_string(),
        arguments: serde_json::from_value(json!({
            "random_string": "demo"
        }))?,
        call_id: "payment_tokens_001".to_string(),
        status: ToolCallStatus::Pending,
    };
    
    info!("🤖 AI决定调用工具: {}", tool_call.name);
    info!("📋 工具参数: {}", serde_json::to_string_pretty(&tool_call.arguments)?);
    
    // 执行工具调用
    match agent.execute_tool(&tool_call).await {
        Ok(result) => {
            info!("✅ 工具调用成功！");
            info!("🪙 支持的代币: {}", serde_json::to_string(&result)?);
        }
        Err(e) => {
            info!("❌ 工具调用失败: {}", e);
        }
    }
    
    info!("✅ 支付工具演示完成");
    Ok(())
}

