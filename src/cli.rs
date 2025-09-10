use std::io::{self, Write};
use std::process;
use std::sync::Arc;
use anyhow::{Result, Context};
use clap::{Parser, Subcommand, Args};
use crate::config::Config;
use crate::agent::FileOperationAgent;
use crate::tool_registry::ToolRegistry;
use crate::prompt_registry::PromptRegistry;
use crate::env_config::*;

/// Alou3 - 智能文件操作助手
/// 一个基于Rust的交互式CLI代理，专注于软件工程任务
#[derive(Parser)]
#[command(name = "alou3")]
#[command(version = "0.1.0")]
#[command(about = "智能文件操作助手 - 基于Rust的交互式CLI代理")]
#[command(long_about = "Alou3是一个智能文件操作助手，专注于软件工程任务。它可以帮助你进行文件操作、代码分析、项目重构等各种开发任务。")]
pub struct Cli {
    /// 要执行的命令
    #[arg(short, long)]
    pub command: Option<String>,

    /// 交互式聊天模式
    #[arg(short, long)]
    pub interactive: bool,

    /// 列出所有可用的工具
    #[arg(long)]
    pub list_tools: bool,

    /// 列出所有可用的提示
    #[arg(long)]
    pub list_prompts: bool,

    /// 显示配置信息
    #[arg(long)]
    pub show_config: bool,

    /// 配置文件路径
    #[arg(long)]
    pub config: Option<String>,

    /// 工作区根目录
    #[arg(long)]
    pub workspace: Option<String>,

    /// 会话ID
    #[arg(long)]
    pub session: Option<String>,

    /// 模型名称
    #[arg(long)]
    pub model: Option<String>,

    /// 最大令牌数
    #[arg(long)]
    pub max_tokens: Option<usize>,

    /// 温度设置
    #[arg(long)]
    pub temperature: Option<f64>,

    /// 启用调试模式
    #[arg(long)]
    pub debug: bool,

    /// 启用沙盒模式
    #[arg(long)]
    pub sandbox: bool,

    /// 沙盒目录
    #[arg(long)]
    pub sandbox_dir: Option<String>,

    /// 子命令
    #[command(subcommand)]
    pub subcommand: Option<Commands>,
}

/// 子命令
#[derive(Subcommand)]
pub enum Commands {
    /// 聊天模式
    Chat(ChatArgs),
    /// 执行单个命令
    Run(RunArgs),
    /// 工具管理
    Tools(ToolsArgs),
    /// 配置管理
    Config(ConfigArgs),
    /// 帮助信息
    Info(HelpArgs),
}

/// 聊天模式参数
#[derive(Args)]
pub struct ChatArgs {
    /// 系统提示
    #[arg(long)]
    pub system_prompt: Option<String>,

    /// 用户记忆
    #[arg(long)]
    pub user_memory: Option<String>,

    /// 最大轮次
    #[arg(long, default_value = "100")]
    pub max_rounds: usize,

    /// 自动保存
    #[arg(long)]
    pub auto_save: bool,
}

/// 执行命令参数
#[derive(Args)]
pub struct RunArgs {
    /// 要执行的命令
    pub command: String,

    /// 命令参数
    pub args: Vec<String>,

    /// 工作目录
    #[arg(long)]
    pub work_dir: Option<String>,

    /// 超时时间（秒）
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// 重试次数
    #[arg(long, default_value = "3")]
    pub retries: usize,
}

/// 工具管理参数
#[derive(Args)]
pub struct ToolsArgs {
    /// 子命令
    #[command(subcommand)]
    pub subcommand: ToolsSubcommand,
}

/// 工具管理子命令
#[derive(Subcommand)]
pub enum ToolsSubcommand {
    /// 列出所有工具
    List,
    /// 显示工具详情
    Show { name: String },
    /// 测试工具
    Test { name: String },
    /// 启用工具
    Enable { name: String },
    /// 禁用工具
    Disable { name: String },
}

/// 配置管理参数
#[derive(Args)]
pub struct ConfigArgs {
    /// 子命令
    #[command(subcommand)]
    pub subcommand: ConfigSubcommand,
}

/// 配置管理子命令
#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// 显示当前配置
    Show,
    /// 设置配置项
    Set { key: String, value: String },
    /// 重置配置
    Reset,
    /// 验证配置
    Validate,
    /// 导出配置
    Export { path: String },
    /// 导入配置
    Import { path: String },
}

/// 帮助信息参数
#[derive(Args)]
pub struct HelpArgs {
    /// 主题
    pub topic: Option<String>,
}

/// CLI应用程序
pub struct CliApp {
    config: Config,
    agent: FileOperationAgent,
    tool_registry: ToolRegistry,
    prompt_registry: PromptRegistry,
}

impl CliApp {
    /// 创建新的CLI应用程序
    pub fn new() -> Result<Self> {
        // 初始化环境配置
        init_env_config()?;

        // 创建配置
        let mut config = Config::new()?;

        // 创建工具注册表
        let tool_registry = ToolRegistry::new();

        // 创建提示注册表
        let prompt_registry = PromptRegistry::new();

        // 创建代理
        let agent = FileOperationAgent::new(config.clone(), Arc::new(tool_registry.clone()), Arc::new(prompt_registry.clone()))?;

        Ok(CliApp {
            config,
            agent,
            tool_registry,
            prompt_registry,
        })
    }

    /// 运行CLI应用程序
    pub async fn run(&mut self, cli: Cli) -> Result<()> {
        // 处理全局选项
        self.handle_global_options(&cli)?;

        // 处理子命令
        if let Some(subcommand) = cli.subcommand {
            self.handle_subcommand(subcommand).await?;
        } else if cli.interactive {
            self.run_interactive_mode().await?;
        } else if let Some(command) = cli.command {
            self.run_single_command(&command).await?;
        } else if cli.list_tools {
            self.list_tools().await?;
        } else if cli.list_prompts {
            self.list_prompts()?;
        } else if cli.show_config {
            self.show_config()?;
        } else {
            // 默认进入交互模式
            self.run_interactive_mode().await?;
        }

        Ok(())
    }

    /// 处理全局选项
    fn handle_global_options(&mut self, cli: &Cli) -> Result<()> {
        // 设置调试模式
        if cli.debug {
            std::env::set_var("DEBUG", "true");
            std::env::set_var("RUST_LOG", "debug");
        }

        // 设置工作区根目录
        if let Some(workspace) = &cli.workspace {
            self.config.workspace_root = std::path::PathBuf::from(workspace);
        }

        // 设置会话ID
        if let Some(session) = &cli.session {
            self.config.session_id = session.clone();
        }

        // 设置模型
        if let Some(model) = &cli.model {
            self.config.update_default_model(model.clone());
        }

        // 设置最大令牌数
        if let Some(max_tokens) = cli.max_tokens {
            self.config.update_max_tokens(max_tokens);
        }

        // 设置温度
        if let Some(temperature) = cli.temperature {
            self.config.update_temperature(temperature);
        }

        // 设置沙盒模式
        if cli.sandbox {
            let sandbox_dir = cli.sandbox_dir.as_ref()
                .map(|s| std::path::PathBuf::from(s));
            self.config.update_sandbox(true, sandbox_dir);
        }

        // 加载配置文件
        if let Some(config_path) = &cli.config {
            let file_config = Config::from_file(config_path)?;
            self.config = file_config;
        }

        Ok(())
    }

    /// 处理子命令
    async fn handle_subcommand(&mut self, subcommand: Commands) -> Result<()> {
        match subcommand {
            Commands::Chat(args) => self.run_chat_mode(args).await?,
            Commands::Run(args) => self.run_command(args).await?,
            Commands::Tools(args) => self.handle_tools_command(args).await?,
            Commands::Config(args) => self.handle_config_command(args)?,
            Commands::Info(args) => self.show_help(args)?,
        }
        Ok(())
    }

    /// 运行交互模式
    async fn run_interactive_mode(&mut self) -> Result<()> {
        println!("欢迎使用 Alou3 - 智能文件操作助手！");
        println!("输入 '/help' 查看帮助信息，输入 '/exit' 退出程序。");
        println!();

        let mut round_count = 0;
        const MAX_ROUNDS: usize = 100;

        loop {
            if round_count >= MAX_ROUNDS {
                println!("已达到最大轮次限制，程序退出。");
                break;
            }

            print!("alou3> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            // 处理特殊命令
            if input.starts_with('/') {
                if self.handle_special_command(input).await? {
                    break;
                }
                continue;
            }

            // 处理用户请求
            match self.agent.process_request(input).await {
                Ok(response) => {
                    if let Some(content) = response.content {
                        println!("{}", content);
                    }
                    if let Some(memory_updates) = response.memory_updates {
                        // TODO: 需要根据实际的memory_updates结构来处理
                        println!("💾 已保存到记忆: {:?}", memory_updates);
                    }
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                }
            }

            round_count += 1;
        }

        Ok(())
    }

    /// 运行聊天模式
    async fn run_chat_mode(&mut self, args: ChatArgs) -> Result<()> {
        println!("进入聊天模式...");
        
        // 设置系统提示
        if let Some(system_prompt) = args.system_prompt {
            // 这里可以设置自定义系统提示
            println!("使用自定义系统提示");
        }

        // 设置用户记忆
        if let Some(user_memory) = args.user_memory {
            // 这里可以设置用户记忆
            println!("加载用户记忆");
        }

        let mut round_count = 0;

        loop {
            if round_count >= args.max_rounds {
                println!("已达到最大轮次限制，程序退出。");
                break;
            }

            print!("chat> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if input == "/exit" {
                break;
            }

            // 处理用户请求
            match self.agent.process_request(input).await {
                Ok(response) => {
                    if let Some(content) = response.content {
                        println!("{}", content);
                    }
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                }
            }

            round_count += 1;
        }

        Ok(())
    }

    /// 运行单个命令
    async fn run_single_command(&mut self, command: &str) -> Result<()> {
        println!("执行命令: {}", command);
        
        match self.agent.process_request(command).await {
            Ok(response) => {
                if let Some(content) = response.content {
                    println!("{}", content);
                }
            }
            Err(e) => {
                eprintln!("错误: {}", e);
                process::exit(1);
            }
        }

        Ok(())
    }

    /// 运行命令
    async fn run_command(&mut self, args: RunArgs) -> Result<()> {
        let full_command = format!("{} {}", args.command, args.args.join(" "));
        println!("执行命令: {}", full_command);
        
        match self.agent.process_request(&full_command).await {
            Ok(response) => {
                if let Some(content) = response.content {
                    println!("{}", content);
                }
            }
            Err(e) => {
                eprintln!("错误: {}", e);
                process::exit(1);
            }
        }

        Ok(())
    }

    /// 处理工具命令
    async fn handle_tools_command(&mut self, args: ToolsArgs) -> Result<()> {
        match args.subcommand {
            ToolsSubcommand::List => self.list_tools().await?,
            ToolsSubcommand::Show { name } => self.show_tool(&name).await?,
            ToolsSubcommand::Test { name } => self.test_tool(&name)?,
            ToolsSubcommand::Enable { name } => self.enable_tool(&name)?,
            ToolsSubcommand::Disable { name } => self.disable_tool(&name)?,
        }
        Ok(())
    }

    /// 处理配置命令
    fn handle_config_command(&mut self, args: ConfigArgs) -> Result<()> {
        match args.subcommand {
            ConfigSubcommand::Show => self.show_config()?,
            ConfigSubcommand::Set { key, value } => self.set_config(&key, &value)?,
            ConfigSubcommand::Reset => self.reset_config()?,
            ConfigSubcommand::Validate => self.validate_config()?,
            ConfigSubcommand::Export { path } => self.export_config(&path)?,
            ConfigSubcommand::Import { path } => self.import_config(&path)?,
        }
        Ok(())
    }

    /// 显示帮助信息
    fn show_help(&self, args: HelpArgs) -> Result<()> {
        if let Some(topic) = args.topic {
            self.show_topic_help(&topic)?;
        } else {
            self.show_general_help()?;
        }
        Ok(())
    }

    /// 处理特殊命令
    async fn handle_special_command(&mut self, command: &str) -> Result<bool> {
        match command {
            "/help" => {
                self.show_general_help()?;
                Ok(false)
            }
            "/exit" | "/quit" => {
                println!("再见！");
                Ok(true)
            }
            "/tools" => {
                // 异步调用需要特殊处理
                println!("工具列表功能暂时不可用");
                Ok(false)
            }
            "/prompts" => {
                self.list_prompts()?;
                Ok(false)
            }
            "/config" => {
                self.show_config()?;
                Ok(false)
            }
            "/clear" => {
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush()?;
                Ok(false)
            }
            "/status" => {
                self.show_status().await?;
                Ok(false)
            }
            _ => {
                println!("未知命令: {}", command);
                println!("输入 '/help' 查看可用命令");
                Ok(false)
            }
        }
    }

    /// 列出所有工具
    async fn list_tools(&self) -> Result<()> {
        println!("可用的工具:");
        let tools = self.tool_registry.get_all_tools().await;
        for tool in tools {
            println!("  - {}: {}", tool.name(), tool.description());
        }
        Ok(())
    }

    /// 列出所有提示
    fn list_prompts(&self) -> Result<()> {
        println!("可用的提示:");
        let prompts = self.prompt_registry.get_all_prompts();
        for prompt in prompts {
            println!("  - {}: {:?}", prompt.name, prompt.description);
        }
        Ok(())
    }

    /// 显示配置信息
    fn show_config(&self) -> Result<()> {
        println!("当前配置:");
        let summary = self.config.get_summary();
        for (key, value) in summary {
            println!("  {}: {}", key, value);
        }
        Ok(())
    }

    /// 显示工具详情
    async fn show_tool(&self, name: &str) -> Result<()> {
        if let Some(tool) = self.tool_registry.get_tool(name).await {
            println!("工具: {}", tool.name());
            println!("描述: {}", tool.description());
            println!("参数: {:?}", tool.parameter_schema());
        } else {
            println!("未找到工具: {}", name);
        }
        Ok(())
    }

    /// 测试工具
    fn test_tool(&self, name: &str) -> Result<()> {
        println!("测试工具: {}", name);
        // 这里可以实现工具测试逻辑
        Ok(())
    }

    /// 启用工具
    fn enable_tool(&mut self, name: &str) -> Result<()> {
        println!("启用工具: {}", name);
        // 这里可以实现工具启用逻辑
        Ok(())
    }

    /// 禁用工具
    fn disable_tool(&mut self, name: &str) -> Result<()> {
        println!("禁用工具: {}", name);
        // 这里可以实现工具禁用逻辑
        Ok(())
    }

    /// 设置配置项
    fn set_config(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "model" => self.config.update_default_model(value.to_string()),
            "max_tokens" => {
                if let Ok(tokens) = value.parse::<usize>() {
                    self.config.update_max_tokens(tokens);
                } else {
                    return Err(anyhow::anyhow!("无效的最大令牌数: {}", value));
                }
            }
            "temperature" => {
                if let Ok(temp) = value.parse::<f64>() {
                    self.config.update_temperature(temp);
                } else {
                    return Err(anyhow::anyhow!("无效的温度值: {}", value));
                }
            }
            _ => return Err(anyhow::anyhow!("未知的配置项: {}", key)),
        }
        println!("已设置 {} = {}", key, value);
        Ok(())
    }

    /// 重置配置
    fn reset_config(&mut self) -> Result<()> {
        self.config = Config::new()?;
        println!("配置已重置");
        Ok(())
    }

    /// 验证配置
    fn validate_config(&self) -> Result<()> {
        match self.config.validate() {
            Ok(_) => {
                println!("配置验证通过");
                Ok(())
            }
            Err(e) => {
                eprintln!("配置验证失败: {}", e);
                Err(e)
            }
        }
    }

    /// 导出配置
    fn export_config(&self, path: &str) -> Result<()> {
        self.config.save_to_file(path)?;
        println!("配置已导出到: {}", path);
        Ok(())
    }

    /// 导入配置
    fn import_config(&mut self, path: &str) -> Result<()> {
        self.config = Config::from_file(path)?;
        println!("配置已从 {} 导入", path);
        Ok(())
    }

    /// 显示状态
    async fn show_status(&self) -> Result<()> {
        println!("Alou3 状态:");
        println!("  会话ID: {}", self.config.session_id);
        println!("  工作区: {}", self.config.workspace_root.display());
        println!("  模型: {}", self.config.get_default_model());
        println!("  工具数量: {}", self.tool_registry.get_all_tools().await.len());
        println!("  提示数量: {}", self.prompt_registry.get_all_prompts().len());
        Ok(())
    }

    /// 显示一般帮助
    fn show_general_help(&self) -> Result<()> {
        println!("Alou3 - 智能文件操作助手");
        println!();
        println!("可用命令:");
        println!("  /help          - 显示此帮助信息");
        println!("  /exit, /quit   - 退出程序");
        println!("  /tools         - 列出所有可用工具");
        println!("  /prompts       - 列出所有可用提示");
        println!("  /config        - 显示当前配置");
        println!("  /clear         - 清屏");
        println!("  /status        - 显示状态信息");
        println!();
        println!("使用示例:");
        println!("  alou3 --interactive                    # 进入交互模式");
        println!("  alou3 --command \"分析这个项目\"          # 执行单个命令");
        println!("  alou3 --list-tools                     # 列出所有工具");
        println!("  alou3 --show-config                    # 显示配置");
        println!("  alou3 chat --max-rounds 50             # 聊天模式，最多50轮");
        println!("  alou3 run \"ls -la\"                     # 执行命令");
        println!("  alou3 tools list                       # 列出工具");
        println!("  alou3 config show                      # 显示配置");
        Ok(())
    }

    /// 显示主题帮助
    fn show_topic_help(&self, topic: &str) -> Result<()> {
        match topic {
            "tools" => {
                println!("工具帮助:");
                println!("  tools list     - 列出所有工具");
                println!("  tools show <name> - 显示工具详情");
                println!("  tools test <name> - 测试工具");
                println!("  tools enable <name> - 启用工具");
                println!("  tools disable <name> - 禁用工具");
            }
            "config" => {
                println!("配置帮助:");
                println!("  config show    - 显示当前配置");
                println!("  config set <key> <value> - 设置配置项");
                println!("  config reset   - 重置配置");
                println!("  config validate - 验证配置");
                println!("  config export <path> - 导出配置");
                println!("  config import <path> - 导入配置");
            }
            "chat" => {
                println!("聊天模式帮助:");
                println!("  chat --system-prompt <prompt> - 设置系统提示");
                println!("  chat --user-memory <memory> - 设置用户记忆");
                println!("  chat --max-rounds <number> - 设置最大轮次");
                println!("  chat --auto-save - 启用自动保存");
            }
            _ => {
                println!("未知主题: {}", topic);
                println!("可用主题: tools, config, chat");
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(&["alou3", "--interactive"]);
        assert!(cli.is_ok());
        
        let cli = cli.unwrap();
        assert!(cli.interactive);
    }

    #[test]
    fn test_cli_commands() {
        let cli = Cli::try_parse_from(&["alou3", "chat", "--max-rounds", "10"]);
        assert!(cli.is_ok());
        
        let cli = cli.unwrap();
        assert!(matches!(cli.subcommand, Some(Commands::Chat(_))));
    }

    #[test]
    fn test_cli_tools() {
        let cli = Cli::try_parse_from(&["alou3", "tools", "list"]);
        assert!(cli.is_ok());
        
        let cli = cli.unwrap();
        assert!(matches!(cli.subcommand, Some(Commands::Tools(_))));
    }

    #[test]
    fn test_cli_config() {
        let cli = Cli::try_parse_from(&["alou3", "config", "show"]);
        assert!(cli.is_ok());
        
        let cli = cli.unwrap();
        assert!(matches!(cli.subcommand, Some(Commands::Config(_))));
    }
}
