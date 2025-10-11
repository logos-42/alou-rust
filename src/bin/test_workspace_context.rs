use alou::agent::{AgentConfig, DeepSeekConfig, BehaviorConfig, WorkspaceConfig, ToolStrategy};
use alou::connection_pool::ConnectionPool;
use std::sync::Arc;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔍 测试Workspace上下文机制");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建测试配置
    let config = AgentConfig {
        deepseek: DeepSeekConfig {
            base_url: "https://api.deepseek.com".to_string(),
            api_key: "test-key".to_string(),
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
            directories: vec![
                ".".to_string(),
                "src".to_string(),
                "tests".to_string(),
            ],
            smart_detection: true,
            exclude_patterns: vec!["target".to_string(), "node_modules".to_string()],
        },
    };

    println!("\n📋 配置信息:");
    println!("   📁 工作空间目录: {:?}", config.workspace.directories);
    println!("   🧠 智能检测: {}", config.workspace.smart_detection);
    println!("   🚫 排除模式: {:?}", config.workspace.exclude_patterns);

    // 创建连接池
    let connection_pool = Arc::new(ConnectionPool::new());

    // 创建智能体
    println!("\n🤖 创建智能体...");
    let agent = alou::agent::McpAgent::with_connection_pool(config, connection_pool).await?;

    // 获取workspace上下文信息
    println!("\n📊 Workspace上下文信息:");
    let workspace_dirs = agent.get_workspace_info().await;
    println!("   📁 检测到的工作空间目录:");
    for (i, dir) in workspace_dirs.iter().enumerate() {
        println!("      {}. {}", i + 1, dir.display());
    }

    // 测试智能检测
    println!("\n🔍 智能检测测试:");
    if workspace_dirs.len() > 1 {
        println!("   ✅ 智能检测成功！检测到多个项目目录");
    } else {
        println!("   ⚠️  智能检测只找到一个目录");
    }

    // 检查当前工作目录
    println!("\n📍 当前工作目录:");
    let current_dir = std::env::current_dir()?;
    println!("   {}", current_dir.display());

    // 检查是否为项目根目录
    println!("\n🔍 项目根目录检测:");
    let project_indicators = [
        "Cargo.toml",
        "package.json", 
        ".git",
        "Makefile",
    ];

    for indicator in &project_indicators {
        let path = current_dir.join(indicator);
        if path.exists() {
            println!("   ✅ 发现项目标识: {}", indicator);
        }
    }

    println!("\n✅ Workspace上下文测试完成！");

    Ok(())
}
