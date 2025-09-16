use alou::agent::{McpAgent, AgentConfig, DeepSeekConfig, BehaviorConfig, WorkspaceConfig, ToolStrategy, Agent};
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

    info!("测试修复后的智能体工具调用");

    // 读取MCP配置
    let mcp_config = McpConfig::from_file("mcp.json")?;
    
    // 创建连接池
    let connection_pool = ConnectionPool::new();

    // 注册所有服务器配置
    for (name, server_config) in mcp_config.mcp_servers {
        connection_pool.register_server(name, server_config).await;
    }

    // 创建智能体配置
    let agent_config = AgentConfig {
        deepseek: DeepSeekConfig {
            base_url: "https://api.deepseek.com".to_string(),
            api_key: std::env::var("DEEPSEEK_API_KEY")
                .unwrap_or_else(|_| "your-api-key-here".to_string()),
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
            exclude_patterns: vec!["target".to_string(), ".git".to_string()],
        },
    };

    // 创建智能体
    let mut agent = McpAgent::new(agent_config).await?;

    // 初始化智能体
    agent.initialize().await?;

    info!("智能体初始化完成，等待工具加载...");
    
    // 等待工具加载完成
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // 测试工具调用
    info!("开始测试工具调用...");
    
    let test_input = "请创建一个名为test_folder的文件夹";
    
    match agent.process_input(test_input).await {
        Ok(response) => {
            info!("智能体响应: {}", response);
        }
        Err(e) => {
            info!("智能体处理失败: {}", e);
        }
    }

    Ok(())
}
