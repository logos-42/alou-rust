// ============================================
// 测试Claude Agent适配器
// ============================================

use std::env;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::{info, error, warn};

use alou::agent::{
    Agent, Adapter, AgentConfig, DeepSeekConfig, BehaviorConfig, 
    WorkspaceConfig, ToolStrategy
};
use alou::connection_pool::ConnectionPool;

/// 测试Claude Agent适配器
#[derive(Parser)]
#[command(name = "test-adapter")]
#[command(about = "测试Claude Agent适配器")]
struct Cli {
    /// 静默模式，减少日志输出
    #[arg(short, long)]
    quiet: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 测试适配器功能
    Test {
        /// 测试消息
        #[arg(short, long, default_value = "你好，请介绍一下你的功能")]
        message: String,
        /// 配置文件路径
        #[arg(short, long, default_value = "agent_config.json")]
        config: String,
    },
    /// 初始化配置文件
    Init {
        /// 输出配置文件路径
        #[arg(short, long, default_value = "agent_config.json")]
        output: String,
    },
}

/// 默认智能体配置
fn get_default_config() -> AgentConfig {
    AgentConfig {
        deepseek: DeepSeekConfig {
            base_url: env::var("DEEPSEEK_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            api_key: env::var("DEEPSEEK_API_KEY")
                .unwrap_or_else(|_| {
                    warn!("未设置DEEPSEEK_API_KEY环境变量，请设置正确的API密钥");
                    "your-api-key-here".to_string()
                }),
            model: env::var("DEEPSEEK_MODEL")
                .unwrap_or_else(|_| "deepseek-chat".to_string()),
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
    }
}

/// 加载配置文件
fn load_config(config_path: &str) -> Result<AgentConfig> {
    let mut config = if std::path::Path::new(config_path).exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: AgentConfig = serde_json::from_str(&content)?;
        config
    } else {
        warn!("配置文件 {} 不存在，使用默认配置", config_path);
        get_default_config()
    };
    
    // 优先使用环境变量中的API密钥
    if let Ok(api_key) = env::var("DEEPSEEK_API_KEY") {
        if !api_key.is_empty() && api_key != "your-api-key-here" {
            config.deepseek.api_key = api_key;
            info!("使用环境变量中的DeepSeek API密钥");
        }
    }
    
    // 检查API密钥是否有效
    if config.deepseek.api_key == "your-api-key-here" || config.deepseek.api_key.is_empty() {
        error!("DeepSeek API密钥未设置或无效！");
        error!("请设置环境变量 DEEPSEEK_API_KEY 或编辑配置文件 {}", config_path);
        return Err(anyhow::anyhow!("API密钥未设置"));
    }
    
    Ok(config)
}

/// 保存配置文件
fn save_config(config: &AgentConfig, config_path: &str) -> Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(config_path, content)?;
    info!("配置文件已保存到: {}", config_path);
    Ok(())
}

/// 初始化MCP连接池
async fn init_connection_pool() -> Result<ConnectionPool> {
    let pool = ConnectionPool::new();
    Ok(pool)
}

/// 测试适配器功能
async fn test_adapter(config_path: &str, message: &str) -> Result<()> {
    let config = load_config(config_path)?;
    
    // 初始化连接池
    let connection_pool = Arc::new(init_connection_pool().await?);
    
    // 创建适配器
    let mut adapter = Adapter::with_connection_pool(config, connection_pool.clone()).await?;
    
    // 初始化适配器
    adapter.initialize().await?;
    
    info!("测试消息: {}", message);
    
    // 处理API调用
    let result = adapter.process_input(message).await;
    
    match result {
        Ok(response) => {
            println!("适配器响应: {}", response);
        }
        Err(e) => {
            error!("测试失败: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// 初始化配置文件
fn init_config(output_path: &str) -> Result<()> {
    let config = get_default_config();
    save_config(&config, output_path)?;
    
    println!("配置文件已创建: {}", output_path);
    println!("请编辑配置文件，设置正确的DeepSeek API密钥和其他参数");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 加载 .env 文件
    if let Err(e) = dotenv::dotenv() {
        warn!("无法加载 .env 文件: {}", e);
    }
    
    let cli = Cli::parse();
    
    // 根据模式设置日志级别
    if cli.quiet {
        // 静默模式：只显示错误
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_target(false)
            .with_ansi(false)
            .init();
    } else {
        // 正常模式：显示重要信息
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .with_ansi(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .init();
    }
    
    match cli.command {
        Commands::Test { message, config } => {
            test_adapter(&config, &message).await?;
        }
        Commands::Init { output } => {
            init_config(&output)?;
        }
    }
    
    Ok(())
}
