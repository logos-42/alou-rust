use alou::agent::{McpAgent, AgentConfig, DeepSeekConfig, BehaviorConfig, WorkspaceConfig, ToolStrategy, Agent};
use alou::connection_pool::{ConnectionPool, McpServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::info;
use serde_json;

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

    info!("调试工具schema格式");

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

    // 获取工具schema
    let tools_schema = {
        // 这里我们需要访问智能体的内部状态来获取工具schema
        // 由于McpAgent没有公开方法获取tools_schema，我们需要通过反射或者其他方式
        // 让我们先打印一下智能体的状态
        info!("智能体已初始化，工具应该已加载");
    };

    // 手动构建工具schema来测试格式
    info!("让我们手动测试工具schema格式...");
    
    // 模拟create_directory工具的schema
    let test_schema = serde_json::json!({
        "type": "function",
        "function": {
            "name": "create_directory",
            "description": "Create a new directory or ensure a directory exists. Can create multiple nested directories in one operation. If the directory already exists, this operation will succeed silently. Perfect for setting up directory structures for projects or ensuring required paths exist. Only works within allowed directories.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path of the directory to create"
                    }
                },
                "required": ["path"]
            }
        }
    });

    info!("测试工具schema格式:");
    println!("{}", serde_json::to_string_pretty(&test_schema)?);

    Ok(())
}
