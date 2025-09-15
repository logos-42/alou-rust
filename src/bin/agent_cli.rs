use std::env;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::{info, error, warn, debug};
use tokio;
use std::time::Duration;

use mcp_client_rs::agent::{
    Agent, McpAgent, AgentConfig, DeepSeekConfig, BehaviorConfig, 
    WorkspaceConfig, ToolStrategy
};
use mcp_client_rs::connection_pool::{ConnectionPool, McpServerConfig};

/// 智能体CLI工具
#[derive(Parser)]
#[command(name = "agent-cli")]
#[command(about = "智能体CLI工具，使用MCP工具和DeepSeek API")]
struct Cli {
    /// 静默模式，减少日志输出
    #[arg(short, long)]
    quiet: bool,
    
    /// 清洁模式，隐藏所有技术细节，只显示用户界面
    #[arg(long)]
    clean: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动交互式智能体
    Chat {
        /// 配置文件路径
        #[arg(short, long, default_value = "agent_config.json")]
        config: String,
    },
    /// 测试智能体功能
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

/// 简单的加载动画
async fn show_loading_animation(message: &str) {
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut index = 0;
    
    print!("\r{} {}", spinner_chars[index], message);
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        index = (index + 1) % spinner_chars.len();
        print!("\r{} {}", spinner_chars[index], message);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
}

/// 带超时的加载动画
async fn show_loading_with_timeout(message: &str, timeout: Duration) {
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut index = 0;
    let start_time = std::time::Instant::now();
    
    while start_time.elapsed() < timeout {
        print!("\r{} {}", spinner_chars[index], message);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        index = (index + 1) % spinner_chars.len();
    }
    
    // 清除加载动画
    print!("\r{}", " ".repeat(message.len() + 3));
    print!("\r");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}

/// 初始化MCP连接池
async fn init_connection_pool() -> Result<ConnectionPool> {
    // 创建空的连接池，服务器配置将在agent初始化时注册
    let pool = ConnectionPool::new();
    Ok(pool)
}

/// 启动交互式聊天
async fn start_chat(config_path: &str) -> Result<()> {
    let config = load_config(config_path)?;
    
    // 初始化连接池
    let connection_pool = Arc::new(init_connection_pool().await?);
    
    // 创建智能体（使用指定的连接池）
    let mut agent = McpAgent::with_connection_pool(config, connection_pool.clone()).await?;
    
    // 显示启动信息
    println!("🚀 正在启动Alou智能助手...");
    
    // 初始化智能体
    agent.initialize().await?;
    
    // 显示欢迎界面
    println!("\n✨ Alou智能助手已就绪！");
    println!("💡 输入 'exit' 或 'quit' 退出程序");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // 简单的交互循环
    loop {
        print!("👤 我: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "exit" || input == "quit" {
            println!("\n👋 感谢使用Alou智能助手，再见！");
            break;
        }
        
        // 显示加载动画
        let loading_handle = tokio::spawn(async {
            show_loading_animation("🤔 正在思考，请稍候...").await;
        });
        
        // 处理API调用
        let result = agent.process_input(input).await;
        
        // 停止加载动画
        loading_handle.abort();
        
        // 清除加载动画行
        print!("\r{}", " ".repeat(50));
        print!("\r");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        match result {
            Ok(response) => {
                println!("🧠 Alou: {}", response);
            }
            Err(e) => {
                error!("处理输入时出错: {}", e);
            }
        }
    }
    
    // 优雅关闭连接
    if let Err(e) = connection_pool.close_all_connections().await {
        debug!("关闭连接时出现错误: {}", e);
    }
    
    Ok(())
}

/// 测试智能体功能
async fn test_agent(config_path: &str, message: &str) -> Result<()> {
    let config = load_config(config_path)?;
    
    // 初始化连接池
    let connection_pool = Arc::new(init_connection_pool().await?);
    
    // 创建智能体（使用指定的连接池）
    let mut agent = McpAgent::with_connection_pool(config, connection_pool.clone()).await?;
    
    // 初始化智能体
    agent.initialize().await?;
    
    info!("测试消息: {}", message);
    
    // 显示加载动画
    let loading_handle = tokio::spawn(async {
        show_loading_animation("🤔 正在思考，请稍候...").await;
    });
    
    // 处理API调用
    let result = agent.process_input(message).await;
    
    // 停止加载动画
    loading_handle.abort();
    
    // 清除加载动画行
    print!("\r{}", " ".repeat(50));
    print!("\r");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    match result {
        Ok(response) => {
            println!("智能体响应: {}", response);
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
    if cli.clean {
        // 清洁模式：完全隐藏所有日志，只显示用户界面
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_target(false)
            .with_ansi(false)
            .init();
    } else if cli.quiet {
        // 静默模式：只显示错误
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_target(false)
            .with_ansi(false)
            .init();
    } else {
        // 正常模式：只显示重要信息，隐藏所有技术细节和调试信息
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .with_target(false)
            .with_ansi(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .init();
    }
    
    match cli.command {
        Commands::Chat { config } => {
            start_chat(&config).await?;
        }
        Commands::Test { message, config } => {
            test_agent(&config, &message).await?;
        }
        Commands::Init { output } => {
            init_config(&output)?;
        }
    }
    
    Ok(())
}
