//! Alou CLI - AI Agent 终端工具
//!
//! 功能:
//!   - 工具系统 (文件系统、搜索、Bash、网络、系统)
//!   - 群聊协作 (IPFS PubSub)
//!   - 智能体创建和管理
//!   - 自主循环控制
//!   - 任务管理

mod api;
mod commands;
pub mod swarm;
mod tui;
mod kaizen;

use std::env;

// 颜色常量
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";
const BRIGHT: &str = "\x1b[1m";

use commands::*;

// ============= 帮助 =============

fn cmd_help() {
    println!();
    println!("{}Alou CLI v0.2.0 - AI Agent 终端工具{}", CYAN, BRIGHT);
    println!();
    println!("{}使用: alou [命令] [参数]{}", YELLOW, RESET);
    println!();
    
    println!("{}自主循环:{}", CYAN, RESET);
    println!("  start                         启动");
    println!("  stop                          停止");
    println!("  pause                         暂停");
    println!("  resume                        恢复");
    println!("  status                        状态");
    println!("  run                           执行单次任务");
    println!();
    
    println!("{}任务管理:{}", CYAN, RESET);
    println!("  task add <标题> [描述] [优先级]  添加任务");
    println!("  task list                      任务列表");
    println!();
    
    println!("{}工具系统:{}", CYAN, RESET);
    println!("  tool list                     列出可用工具");
    println!("  tool exec <工具> [参数]       执行工具");
    println!();
    
    println!("{}自动迭代:{}", CYAN, RESET);
    println!("  auto <任务描述>               自动分析并执行任务");
    println!();

    println!("{}Kaizen 自进化循环:{}", CYAN, RESET);
    println!("  kaizen evolution [选项]        启动进化引擎");
    println!("  kaizen research [选项]         启动自动研究 (Karpathy 模式)");
    println!("  kaizen self-repair [选项]      自修复循环 (check->fix->build->restart)");
    println!("  kaizen status                  查看循环状态");
    println!("  kaizen stop                    停止循环");
    println!("  kaizen log [行数]              查看实验日志");
    println!();

    println!("{}Polymarket 预测市场:{}", CYAN, RESET);
    println!("  polymarket search <关键词>    搜索市场");
    println!("  polymarket markets [数量]     列出热门市场");
    println!("  polymarket price <token_id>   查询价格");
    println!("  polymarket orderbook <token_id> 订单簿");
    println!("  polymarket positions          查看持仓");
    println!("  polymarket orders             查看订单");
    println!("  polymarket buy <token_id> <金额> [价格]  买入");
    println!("  polymarket sell <token_id> <金额> [价格] 卖出");
    println!();

    println!("{}配置管理:{}", CYAN, RESET);
    println!("  config show                   显示配置");
    println!("  config set <key> <value>     设置配置");
    println!();
    
    println!("{}帮助:{}", CYAN, RESET);
    println!("  help                          帮助");
    println!();
    
    println!("{}示例:{}", GREEN, RESET);
    println!("  alou start");
    println!("  alou task add \"检查邮件\" \"检查未读邮件\" high");
    println!("  alou config set api_key your_key_here");
    println!("  alou tool list");
    println!("  alou tool exec bash {{\"command\": \"ls -la\"}}");
    println!("  alou auto \"列出当前目录的文件\"");
    println!("  alou status");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] != "help" {
        println!();
        println!("{}╔═══════════════════════════════════════╗", CYAN);
        println!("{}║     🤖 Alou CLI - Alou Agent          ║", CYAN);
        println!("{}╚═══════════════════════════════════════╝", CYAN);
    }

    if args.len() < 2 {
        cmd_help();
        return;
    }

    // 创建 Tokio 运行时
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // 在运行时中执行异步命令
    rt.block_on(async {
        run_main_command(&args).await;
    });
}

async fn run_main_command(args: &[String]) {
    match args[1].as_str() {
        // 自主循环
        "start" => cmd_start(),
        "stop" => cmd_stop(),
        "pause" => cmd_pause(),
        "resume" => cmd_resume(),
        "status" => cmd_status(),
        "run" => cmd_run(),
        "loop" => cmd_loop(),
        
        // 任务管理
        "task" => {
            if args.len() < 3 {
                log_error("缺少任务操作");
                return;
            }
            match args[2].as_str() {
                "add" => {
                    let title = args.get(3).map(|s| s.as_str()).unwrap_or("新任务");
                    let description = args.get(4).map(|s| s.as_str()).unwrap_or("");
                    let priority = args.get(5).map(|s| s.as_str()).unwrap_or("medium");
                    cmd_task_add(title, description, priority);
                }
                "list" => cmd_task_list(),
                _ => log_error(&format!("未知操作: {}", args[2])),
            }
        }
        
        // 工具系统
        "tool" => {
            if args.len() < 3 {
                log_error("缺少工具操作");
                return;
            }
            match args[2].as_str() {
                "list" => cmd_tool_list(),
                "exec" => {
                    let tool_id = args.get(3).map(|s| s.as_str()).unwrap_or("");
                    let args_json = args.get(4).map(|s| s.as_str()).unwrap_or("");
                    if tool_id.is_empty() {
                        log_error("请指定工具名称");
                        println!("用法: alou tool exec <tool_id> [args_json]");
                    } else {
                        cmd_tool_exec(tool_id, args_json);
                    }
                }
                _ => log_error(&format!("未知操作: {}", args[2])),
            }
        }
        
        // 自动迭代
        "auto" => {
            let task_description = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if task_description.is_empty() {
                log_error("请指定任务描述");
                println!("用法: alou auto <任务描述>");
            } else {
                cmd_auto(task_description);
            }
        }
        
        // 配置管理
        "config" => {
            if args.len() < 3 {
                cmd_config_show();
                return;
            }
            match args[2].as_str() {
                "show" => cmd_config_show(),
                "set" => {
                    if args.len() < 5 {
                        log_error("缺少配置参数");
                        println!("用法: alou config set <key> <value>");
                        return;
                    }
                    cmd_config_set(&args[3], &args[4]);
                }
                _ => log_error(&format!("未知操作: {}", args[2])),
            }
        }

        // TUI界面
        "tui" | "ui" | "gui" => {
            if let Err(e) = run_tui() {
                log_error(&format!("TUI启动失败: {}", e));
            }
        }

        // Kaizen 自进化循环
        "kaizen" => {
            if let Err(e) = run_kaizen_command(&args[2..]).await {
                log_error(&format!("Kaizen 命令执行失败: {}", e));
            }
        }

        // Polymarket 预测市场
        "polymarket" | "pm" => {
            if args.len() < 3 {
                log_error("缺少 Polymarket 操作");
                println!("用法: alou polymarket <search|markets|price|orderbook|positions|orders|buy|sell|auth|health> [参数]");
                return;
            }
            run_polymarket_command(&args[2..]).await;
        }

        // 帮助
        "help" | "-h" | "--help" => cmd_help(),

        _ => {
            log_error(&format!("未知命令: {}", args[1]));
            println!("运行 {}alou help{} 查看命令", GREEN, RESET);
        }
    }
}

/// 运行TUI界面
fn run_tui() -> std::io::Result<()> {
    use crate::tui::{Tui, TuiApp};
    
    println!("🚀 启动Alou CLI TUI界面...");
    println!("🤖 Alou CLI v0.2.0 - AI智能体终端界面");
    println!("💬 支持PubSub群聊、任务管理、技能调用");
    
    // 创建TUI
    let mut tui = Tui::new()?;
    
    // 进入TUI模式
    tui.enter()?;
    
    // 创建应用程序
    let mut app = TuiApp::new();
    
    // 主事件循环
    while !app.should_quit {
        // 绘制界面
        tui.draw(&mut app)?;
        
        // 处理事件
        match tui.events.next() {
            Ok(event) => {
                app.handle_event(event);
            }
            Err(_) => {
                break;
            }
        }
    }
    
    // 退出TUI模式
    tui.exit()?;
    
    println!("👋 TUI界面已退出");
    Ok(())
}

// ============= 配置命令 =============

fn cmd_config_show() {
    log_section("当前配置");
    let config = api::load_config();

    println!("API配置:");
    println!("  Base URL: {}", config.api.base_url);
    println!("  Timeout: {}ms", config.api.timeout);
    println!();
    println!("AI配置:");
    println!("  Provider: {}", config.ai.provider);
    println!("  Model: {}", config.ai.model);
    println!("  API Key: {}", if config.ai.api_key.is_empty() { "(未设置)" } else { "******" });
    println!();
    
    // 显示工具 API 状态
    log_section("工具 API 状态");
    if let Some(url) = api::get_tool_api_url() {
        println!("  状态: ✅ 已连接");
        println!("  URL: {}", url);
    } else {
        println!("  状态: ❌ 不可用");
        println!("  提示: 请确保 Alou Desktop 正在运行");
    }
}

fn cmd_config_set(key: &str, value: &str) {
    let mut config = api::load_config();

    match key {
        "api_url" => {
            config.api.base_url = value.to_string();
            log_success(&format!("API URL 设置为: {}", value));
        }
        "api_key" => {
            config.ai.api_key = value.to_string();
            log_success("API Key 已设置");
        }
        "model" => {
            config.ai.model = value.to_string();
            log_success(&format!("模型设置为: {}", value));
        }
        _ => {
            log_error(&format!("未知配置项: {}", key));
            println!("支持的配置项: api_url, api_key, model");
            return;
        }
    }

    if let Err(e) = api::save_config(&config) {
        log_error(&format!("保存配置失败: {}", e));
    }
}

// ============= Kaizen 命令 =============

use kaizen::config::{KaizenConfig, KaizenMode};

async fn run_kaizen_command(args: &[String]) -> anyhow::Result<()> {
    if args.is_empty() {
        println!();
        println!("{}Kaizen 自进化循环命令{}", CYAN, BRIGHT);
        println!();
        println!("{}使用: alou kaizen [命令] [选项]{}", YELLOW, RESET);
        println!();
        println!("{}命令:{}", CYAN, RESET);
        println!("  evolution [选项]    启动进化引擎");
        println!("  research [选项]     启动自动研究 (Karpathy 模式)");
        println!("  self-repair [选项]  自修复循环 (check->fix->build->restart)");
        println!("  status              查看循环状态");
        println!("  stop                停止循环");
        println!("  log [行数]          查看实验日志");
        println!();
        println!("{}通用选项:{}", CYAN, RESET);
        println!("  --provider <名称>   LLM 提供商 (见下方列表)");
        println!("  --model <名称>      LLM 模型名称");
        println!("  --base-url <URL>    API 端点 (部分提供商自动推断)");
        println!();
        println!("{}支持的 Provider:{}", CYAN, RESET);
        println!("  ollama          Ollama (本地/云端, 无需 API Key)");
        println!("  openai          OpenAI (gpt-4o, gpt-4-turbo...)");
        println!("  deepseek        DeepSeek (deepseek-chat, deepseek-coder...)");
        println!("  glm / zhipuai   智谱 AI (glm-4-plus, glm-5...)");
        println!("  qwen            通义千问 (qwen-max, qwen-plus...)");
        println!("  minimax         MiniMax (minimax-text...)");
        println!("  openrouter      OpenRouter (聚合多种模型)");
        println!("  openai-compat   任意 OpenAI 兼容 API");
        println!();
        println!("{}进化引擎选项:{}", CYAN, RESET);
        println!("  --iterations <N>    迭代次数 (默认: 5)");
        println!("  --task <描述>       任务描述");
        println!();
        println!("{}自动研究/自修复选项:{}", CYAN, RESET);
        println!("  --iterations <N>    迭代次数 (默认: 5)");
        println!("  --auto-push         自动推送到 GitHub");
        println!("  --strict            严格模式");
        println!("  --dry-run           安全模式 (不实际修改)");
        println!("  --no-dry-run        关闭安全模式");
        println!("  --target <文件>     目标文件 (可多次指定)");
        println!();
        println!("{}Ollama 示例 (本地/云端, 无需 API Key):{}", GREEN, RESET);
        println!("  alou kaizen self-repair --provider ollama --model qwen2.5-coder");
        println!("  alou kaizen research --provider ollama --model deepseek-coder-v2 --no-dry-run");
        println!();
        println!("{}云端模型示例 (需设置 LLM_API_KEY):{}", GREEN, RESET);
        println!("  alou kaizen self-repair --provider deepseek --model deepseek-chat");
        println!("  alou kaizen research --provider glm --model glm-4-plus");
        println!("  alou kaizen evolution --provider qwen --model qwen-max -n 3");
        println!();
        println!("{}自定义端点示例:{}", GREEN, RESET);
        println!("  alou kaizen self-repair --provider openai-compat --model my-model --base-url https://my-api.com/v1");
        println!("  alou kaizen status");
        println!("  alou kaizen log --last 20");
        return Ok(());
    }

    match args[0].as_str() {
        "evolution" => {
            // 解析参数
            let mut config = KaizenConfig::load_from_env();
            config.mode = KaizenMode::Evolution;
            
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--iterations" | "-n" => {
                        if i + 1 < args.len() {
                            config.max_iterations = args[i + 1].parse()
                                .unwrap_or(config.max_iterations);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--task" | "-t" => {
                        if i + 1 < args.len() {
                            config.task_description = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--provider" | "-p" => {
                        if i + 1 < args.len() {
                            config.llm_provider = args[i + 1].clone();
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--model" | "-m" => {
                        if i + 1 < args.len() {
                            config.llm_model = args[i + 1].clone();
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--base-url" => {
                        if i + 1 < args.len() {
                            config.llm_base_url = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }

            // 加载持久化配置
            if let Ok(path) = KaizenConfig::config_path() {
                if path.exists() {
                    if let Ok(file_config) = KaizenConfig::load(&path) {
                        // 合并配置（命令行参数优先）
                        if config.task_description.is_none() {
                            config.task_description = file_config.task_description;
                        }
                    }
                }
            }

            // 检查 API Key（Ollama 本地/云端不需要）
            if config.llm_provider != "ollama" && config.llm_api_key.is_empty() {
                log_error("请先设置 LLM API Key");
                println!("方法 1: 设置环境变量 LLM_API_KEY=your_key");
                println!("方法 2: 创建 .env 文件包含 LLM_API_KEY=your_key");
                println!("方法 3: 使用 Ollama: alou kaizen evolution --provider ollama --model qwen2.5-coder");
                return Ok(());
            }

            // 自动默认模型
            if config.llm_model == "gpt-4o" {
                config.llm_model = match config.llm_provider.as_str() {
                    "ollama" => "qwen2.5-coder".to_string(),
                    "deepseek" => "deepseek-chat".to_string(),
                    "glm" | "zhipuai" => "glm-4-plus".to_string(),
                    "qwen" => "qwen-max".to_string(),
                    "minimax" => "minimax-text".to_string(),
                    _ => "gpt-4o".to_string(),
                };
            }

            println!();
            log_section("启动 Kaizen 进化引擎");
            println!("模式: 进化引擎");
            println!("迭代次数: {}", config.max_iterations);
            println!("任务: {}", config.task_description.as_deref().unwrap_or("默认任务"));
            println!("安全模式: {}", if config.dry_run { "是" } else { "否" });
            println!();

            // 初始化 tracing
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .init();

            // 加载环境变量
            dotenvy::dotenv().ok();

            // 运行进化引擎
            kaizen::evolution_mode::run_evolution(&config).await?;
        }

        "research" => {
            // 解析参数
            let mut config = KaizenConfig::load_from_env();
            config.mode = KaizenMode::Research;
            
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--iterations" | "-n" => {
                        if i + 1 < args.len() {
                            config.max_iterations = args[i + 1].parse()
                                .unwrap_or(config.max_iterations);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--auto-push" => {
                        config.auto_push = true;
                        i += 1;
                    }
                    "--strict" => {
                        config.strict = true;
                        i += 1;
                    }
                    "--dry-run" => {
                        config.dry_run = true;
                        i += 1;
                    }
                    "--no-dry-run" => {
                        config.dry_run = false;
                        i += 1;
                    }
                    "--target" => {
                        if i + 1 < args.len() {
                            config.target_files.push(args[i + 1].clone());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--provider" | "-p" => {
                        if i + 1 < args.len() {
                            config.llm_provider = args[i + 1].clone();
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--model" | "-m" => {
                        if i + 1 < args.len() {
                            config.llm_model = args[i + 1].clone();
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--base-url" => {
                        if i + 1 < args.len() {
                            config.llm_base_url = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }

            // 检查 API Key（Ollama 本地/云端不需要）
            if config.llm_provider != "ollama" && config.llm_api_key.is_empty() {
                log_error("请先设置 LLM API Key");
                println!("方法 1: 设置环境变量 LLM_API_KEY=your_key");
                println!("方法 2: 创建 .env 文件包含 LLM_API_KEY=your_key");
                println!("方法 3: 使用 Ollama: alou kaizen research --provider ollama --model qwen2.5-coder");
                return Ok(());
            }

            // 自动默认模型
            if config.llm_model == "gpt-4o" {
                config.llm_model = match config.llm_provider.as_str() {
                    "ollama" => "qwen2.5-coder".to_string(),
                    "deepseek" => "deepseek-chat".to_string(),
                    "glm" | "zhipuai" => "glm-4-plus".to_string(),
                    "qwen" => "qwen-max".to_string(),
                    "minimax" => "minimax-text".to_string(),
                    _ => "gpt-4o".to_string(),
                };
            }

            println!();
            log_section("启动 Kaizen 自动研究");
            println!("模式: 自动研究 (Karpathy 风格)");
            println!("迭代次数: {}", config.max_iterations);
            println!("自动推送: {}", if config.auto_push { "是" } else { "否" });
            println!("严格模式: {}", if config.strict { "是" } else { "否" });
            println!("安全模式: {}", if config.dry_run { "是" } else { "否" });
            if !config.target_files.is_empty() {
                println!("目标文件: {:?}", config.target_files);
            }
            println!();

            // 初始化 tracing
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .init();

            // 加载环境变量
            dotenvy::dotenv().ok();

            // 运行自动研究
            kaizen::research_mode::run_research(&config).await?;
        }

        "self-repair" | "repair" => {
            // 解析参数
            let mut config = KaizenConfig::load_from_env();
            config.mode = KaizenMode::Research; // 复用 research 模式

            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--iterations" | "-n" => {
                        if i + 1 < args.len() {
                            config.max_iterations = args[i + 1].parse()
                                .unwrap_or(config.max_iterations);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--provider" | "-p" => {
                        if i + 1 < args.len() {
                            config.llm_provider = args[i + 1].clone();
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--model" | "-m" => {
                        if i + 1 < args.len() {
                            config.llm_model = args[i + 1].clone();
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--base-url" => {
                        if i + 1 < args.len() {
                            config.llm_base_url = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "--target" => {
                        if i + 1 < args.len() {
                            config.target_files.push(args[i + 1].clone());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }

            // 检查 API Key（Ollama 本地/云端不需要）
            if config.llm_provider != "ollama" && config.llm_api_key.is_empty() {
                log_error("请先设置 LLM API Key");
                println!("方法 1: 设置环境变量 LLM_API_KEY=your_key");
                println!("方法 2: 创建 .env 文件包含 LLM_API_KEY=your_key");
                println!("方法 3: 使用 Ollama: alou kaizen self-repair --provider ollama --model qwen2.5-coder");
                return Ok(());
            }

            // 自动默认模型
            if config.llm_model == "gpt-4o" {
                config.llm_model = match config.llm_provider.as_str() {
                    "ollama" => "qwen2.5-coder".to_string(),
                    "deepseek" => "deepseek-chat".to_string(),
                    "glm" | "zhipuai" => "glm-4-plus".to_string(),
                    "qwen" => "qwen-max".to_string(),
                    "minimax" => "minimax-text".to_string(),
                    _ => "gpt-4o".to_string(),
                };
            }

            // 自修复模式：关闭 dry_run，目标文件指向 alou 项目自身
            config.dry_run = false;
            config.auto_push = false;
            config.strict = false;

            // 默认目标：alou-desktop 和 alou-cli 核心文件
            if config.target_files.is_empty() {
                config.target_files = vec![
                    "src-tauri/src/main.rs".to_string(),
                    "src-tauri/src/tools/self_repair_tool.rs".to_string(),
                    "src-tauri/src/agent/executor/reasoning.rs".to_string(),
                    "src-tauri/src/bridges/tool_bridge.rs".to_string(),
                    "src-tauri/src/kappa_loop/mod.rs".to_string(),
                ];
            }

            println!();
            log_section("启动 Kaizen 自修复循环");
            println!("模式: 自修复 (Self-Repair)");
            println!("LLM: {} / {}", config.llm_provider, config.llm_model);
            println!("迭代次数: {}", config.max_iterations);
            println!("目标文件: {:?}", config.target_files);
            println!();

            // 初始化 tracing
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .init();

            // 加载环境变量
            dotenvy::dotenv().ok();

            // 运行自修复（复用 research 模式，但目标是自身代码）
            kaizen::research_mode::run_research(&config).await?;
        }

        "status" => {
            println!();
            log_section("Kaizen 循环状态");
            
            // 尝试显示两种模式的状态
            if let Err(e) = kaizen::evolution_mode::show_evolution_status() {
                tracing::warn!("无法加载进化引擎状态: {}", e);
            }
            
            println!();
            
            if let Err(e) = kaizen::research_mode::show_research_status() {
                tracing::warn!("无法加载自动研究状态: {}", e);
            }
        }

        "stop" => {
            log_info("停止 Kaizen 循环...");
            
            // 尝试停止两种模式
            let _ = kaizen::evolution_mode::stop_evolution();
            let _ = kaizen::research_mode::stop_research();
            
            log_success("循环已停止");
        }

        "log" => {
            let last_n = if args.len() > 2 {
                args[2].parse().unwrap_or(20)
            } else {
                20
            };
            
            println!();
            if let Err(e) = kaizen::research_mode::show_research_log(last_n) {
                log_error(&format!("查看日志失败: {}", e));
            }
        }

        _ => {
            log_error(&format!("未知 Kaizen 命令: {}", args[0]));
            println!("运行 {}alou kaizen{} 查看帮助", GREEN, RESET);
        }
    }

    Ok(())
}

// ============= Polymarket 命令 =============

async fn run_polymarket_command(args: &[String]) {
    if args.is_empty() {
        log_error("缺少 Polymarket 操作");
        println!("用法: alou polymarket <操作> [参数]");
        return;
    }

    // 尝试通过 Desktop API 执行
    if !api::check_tool_api_available() {
        run_polymarket_standalone(&args).await;
        return;
    }

    let action = args[0].as_str();
    let tool_args = match action {
        "help" | "-h" => { pm_help(); return; }
        _ => pm_build_tool_args(action, &args[1..]),
    };

    if tool_args.is_null() { return; }

    log_info(&format!("执行 Polymarket: {}...", action));
    match api::execute_tool("polymarket", tool_args, None, None).await {
        Ok(response) => {
            if response.success {
                log_success("执行成功");
                if let Some(data) = response.data {
                    println!("\n{}", serde_json::to_string_pretty(&data).unwrap_or_default());
                }
            } else {
                log_error(&format!("执行失败: {}", response.error.unwrap_or_default()));
            }
        }
        Err(e) => log_error(&format!("请求失败: {}", e)),
    }
}

/// 构建 Desktop 工具调用参数，返回 Null 表示用法错误已打印
fn pm_build_tool_args(action: &str, params: &[String]) -> serde_json::Value {
    match action {
        "search" | "s" => {
            let query = params.get(0).map(|s| s.as_str()).unwrap_or("");
            if query.is_empty() { log_error("用法: alou polymarket search <关键词>"); return serde_json::Value::Null; }
            serde_json::json!({ "action": "search", "query": query, "limit": params.get(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(10) })
        }
        "markets" | "list" | "ls" => {
            serde_json::json!({ "action": "markets", "limit": params.get(0).and_then(|v| v.parse::<u32>().ok()).unwrap_or(20), "active_only": true })
        }
        "price" | "p" => {
            let id = params.get(0).map(|s| s.as_str()).unwrap_or("");
            if id.is_empty() { log_error("用法: alou polymarket price <token_id>"); return serde_json::Value::Null; }
            serde_json::json!({ "action": "price", "token_id": id })
        }
        "last_price" | "lp" => {
            let id = params.get(0).map(|s| s.as_str()).unwrap_or("");
            if id.is_empty() { log_error("用法: alou polymarket last_price <token_id>"); return serde_json::Value::Null; }
            serde_json::json!({ "action": "last_price", "token_id": id })
        }
        "orderbook" | "book" | "ob" => {
            let id = params.get(0).map(|s| s.as_str()).unwrap_or("");
            if id.is_empty() { log_error("用法: alou polymarket orderbook <token_id>"); return serde_json::Value::Null; }
            serde_json::json!({ "action": "orderbook", "token_id": id })
        }
        "market" | "details" => {
            let id = params.get(0).map(|s| s.as_str()).unwrap_or("");
            if id.is_empty() { log_error("用法: alou polymarket market <condition_id>"); return serde_json::Value::Null; }
            serde_json::json!({ "action": "market_details", "condition_id": id })
        }
        "positions" | "pos" => serde_json::json!({ "action": "positions" }),
        "orders" | "ods"    => serde_json::json!({ "action": "orders" }),
        "trades"            => serde_json::json!({ "action": "trades" }),
        "balance" | "bal"   => serde_json::json!({ "action": "balance" }),
        "buy"  => pm_build_order_args("buy", params),
        "sell" => pm_build_order_args("sell", params),
        "cancel" => {
            let id = params.get(0).map(|s| s.as_str()).unwrap_or("");
            if id.is_empty() { log_error("用法: alou polymarket cancel <order_id>"); return serde_json::Value::Null; }
            serde_json::json!({ "action": "cancel_order", "order_id": id })
        }
        "cancel_all" => serde_json::json!({ "action": "cancel_all" }),
        "auth" => {
            if params.len() < 3 { log_error("用法: alou polymarket auth <api_key> <api_secret> <api_passphrase>"); return serde_json::Value::Null; }
            serde_json::json!({ "action": "set_credentials", "api_key": params[0], "api_secret": params[1], "api_passphrase": params[2] })
        }
        "auth_status" => serde_json::json!({ "action": "auth_status" }),
        "health"      => serde_json::json!({ "action": "health" }),
        "geoblock"    => serde_json::json!({ "action": "geoblock" }),
        _ => {
            log_error(&format!("未知 Polymarket 操作: {}", action));
            println!("运行 {}alou polymarket help{} 查看帮助", GREEN, RESET);
            serde_json::Value::Null
        }
    }
}

/// 构建买卖订单参数
fn pm_build_order_args(side: &str, params: &[String]) -> serde_json::Value {
    let token_id = params.get(0).map(|s| s.as_str()).unwrap_or("");
    let amount_str = params.get(1).map(|s| s.as_str()).unwrap_or("");
    let price_str = params.get(2).map(|s| s.as_str());

    if token_id.is_empty() || amount_str.is_empty() {
        log_error(&format!("用法: alou polymarket {} <token_id> <amount> [price]", side));
        return serde_json::Value::Null;
    }

    if let Some(price) = price_str {
        let size: f64 = amount_str.parse().unwrap_or(0.0);
        let price: f64 = price.parse().unwrap_or(0.0);
        if size <= 0.0 || price <= 0.0 { log_error("金额和价格必须大于 0"); return serde_json::Value::Null; }
        serde_json::json!({ "action": format!("{}_limit", side), "token_id": token_id, "size": size, "price": price })
    } else {
        let amount: f64 = amount_str.parse().unwrap_or(0.0);
        if amount <= 0.0 { log_error("金额必须大于 0"); return serde_json::Value::Null; }
        serde_json::json!({ "action": format!("{}_market", side), "token_id": token_id, "amount": amount })
    }
}

/// Polymarket 帮助
fn pm_help() {
    println!();
    println!("{}Polymarket 预测市场命令{}", CYAN, BRIGHT);
    println!();
    println!("{}只读操作 (无需认证):{}", CYAN, RESET);
    println!("  search <关键词> [数量]     搜索市场");
    println!("  markets [数量]             列出热门市场");
    println!("  price <token_id>           查询中间价");
    println!("  last_price <token_id>      查询最新成交价");
    println!("  orderbook <token_id>       查看订单簿");
    println!("  market <condition_id>      查看市场详情");
    println!("  health                     API 健康检查");
    println!("  geoblock                   检查地区限制");
    println!();
    println!("{}认证操作:{}", CYAN, RESET);
    println!("  auth <api_key> <secret> <passphrase>  设置凭证");
    println!("  auth_status                查看认证状态");
    println!();
    println!("{}交易操作 (需认证):{}", CYAN, RESET);
    println!("  positions                  查看持仓");
    println!("  orders                     查看订单");
    println!("  trades                     交易历史");
    println!("  balance                    账户余额");
    println!("  buy <token_id> <金额>      市价买入");
    println!("  buy <token_id> <数量> <价格>  限价买入");
    println!("  sell <token_id> <金额>     市价卖出");
    println!("  sell <token_id> <数量> <价格> 限价卖出");
    println!("  cancel <order_id>          取消订单");
    println!("  cancel_all                 取消所有订单");
    println!();
    println!("{}快捷别名:{}", GREEN, RESET);
    println!("  s=search | p=price | lp=last_price | ob=orderbook");
    println!("  pos=positions | ods=orders | bal=balance");
    println!();
    println!("{}示例:{}", GREEN, RESET);
    println!("  alou polymarket search bitcoin");
    println!("  alou polymarket markets 5");
    println!("  alou polymarket price <token_id>");
    println!("  alou polymarket buy <token_id> 25");
}

// --- Polymarket 独立模式 (Desktop 不可用时) ---

async fn run_polymarket_standalone(args: &[String]) {
    const CLOB_API: &str = "https://clob.polymarket.com";
    const GAMMA_API: &str = "https://gamma-api.polymarket.com";

    log_info("Alou Desktop 未运行，使用独立模式直接连接 Polymarket API...");
    println!();

    let client = reqwest::Client::new();

    match args[0].as_str() {
        "search" | "s" => pm_standalone_search(&client, GAMMA_API, &args[1..]).await,
        "markets" | "list" | "ls" => pm_standalone_markets(&client, GAMMA_API, &args[1..]).await,
        "price" | "p" => pm_standalone_price(&client, CLOB_API, &args[1..]).await,
        "health" => pm_standalone_health(&client, CLOB_API).await,
        "help" | "-h" => {
            println!("独立模式仅支持只读操作: search, markets, price, health");
            println!("交易操作需要 Alou Desktop 运行中");
        }
        _ => {
            log_warn("独立模式仅支持只读操作 (search, markets, price, health)");
            log_info("交易操作需要先启动 Alou Desktop");
        }
    }
}

async fn pm_standalone_search(client: &reqwest::Client, gamma_api: &str, params: &[String]) {
    let query = params.get(0).map(|s| s.as_str()).unwrap_or("");
    if query.is_empty() { log_error("请指定搜索关键词"); return; }
    let limit = params.get(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(10);
    let url = format!("{}/events", gamma_api);
    let limit_str = limit.to_string();
    let active_str = "true".to_string();
    match client.get(&url).query(&[("q", query), ("limit", &limit_str), ("active", &active_str)]).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let output = if let Some(events) = data.as_array() {
                    let simplified: Vec<serde_json::Value> = events.iter().take(limit as usize).map(|e| {
                        serde_json::json!({
                            "title": e.get("title"),
                            "active": e.get("active"),
                            "markets": e.get("markets").and_then(|m| m.as_array()).map(|arr| {
                                arr.iter().map(|m| serde_json::json!({
                                    "question": m.get("question"),
                                    "outcome_prices": m.get("outcome_prices"),
                                    "volume": m.get("volume"),
                                })).collect::<Vec<_>>()
                            }),
                        })
                    }).collect();
                    serde_json::json!({ "query": query, "count": simplified.len(), "events": simplified })
                } else { data };
                log_success("查询成功");
                println!("\n{}", serde_json::to_string_pretty(&output).unwrap_or_default());
            }
            Err(e) => log_error(&format!("解析失败: {}", e)),
        },
        Err(e) => log_error(&format!("请求失败: {}", e)),
    }
}

async fn pm_standalone_markets(client: &reqwest::Client, gamma_api: &str, params: &[String]) {
    let limit = params.get(0).and_then(|v| v.parse::<u32>().ok()).unwrap_or(20);
    let url = format!("{}/markets", gamma_api);
    let limit_str = limit.to_string();
    let active_str = "true".to_string();
    let order_str = "volume24hr".to_string();
    let asc_str = "false".to_string();
    match client.get(&url).query(&[("limit", &limit_str), ("active", &active_str), ("order", &order_str), ("ascending", &asc_str)]).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(data) => { log_success("查询成功"); println!("\n{}", serde_json::to_string_pretty(&data).unwrap_or_default()); }
            Err(e) => log_error(&format!("解析失败: {}", e)),
        },
        Err(e) => log_error(&format!("请求失败: {}", e)),
    }
}

async fn pm_standalone_price(client: &reqwest::Client, clob_api: &str, params: &[String]) {
    let token_id = params.get(0).map(|s| s.as_str()).unwrap_or("");
    if token_id.is_empty() { log_error("请指定 token_id"); return; }
    let url = format!("{}/midpoint", clob_api);
    match client.get(&url).query(&[("token_id", token_id)]).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(data) => { log_success("查询成功"); println!("\n{}", serde_json::to_string_pretty(&data).unwrap_or_default()); }
            Err(e) => log_error(&format!("解析失败: {}", e)),
        },
        Err(e) => log_error(&format!("请求失败: {}", e)),
    }
}

async fn pm_standalone_health(client: &reqwest::Client, clob_api: &str) {
    let url = format!("{}/ok", clob_api);
    match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() { log_success("Polymarket API 正常"); }
            else { log_error(&format!("API 状态异常: {}", resp.status())); }
        }
        Err(e) => log_error(&format!("连接失败: {}", e)),
    }
}
