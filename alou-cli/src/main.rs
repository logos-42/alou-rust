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
    println!("  kaizen status                  查看循环状态");
    println!("  kaizen stop                    停止循环");
    println!("  kaizen log [行数]              查看实验日志");
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
        println!("  status              查看循环状态");
        println!("  stop                停止循环");
        println!("  log [行数]          查看实验日志");
        println!();
        println!("{}进化引擎选项:{}", CYAN, RESET);
        println!("  --iterations <N>    迭代次数 (默认: 5)");
        println!("  --task <描述>       任务描述");
        println!();
        println!("{}自动研究选项:{}", CYAN, RESET);
        println!("  --iterations <N>    迭代次数 (默认: 5)");
        println!("  --auto-push         自动推送到 GitHub");
        println!("  --strict            严格模式");
        println!("  --dry-run           安全模式 (不实际修改)");
        println!("  --target <文件>     目标文件 (可多次指定)");
        println!();
        println!("{}示例:{}", GREEN, RESET);
        println!("  alou kaizen evolution --iterations 10 --task \"优化性能\"");
        println!("  alou kaizen research --auto-push --iterations 5");
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

            // 检查 API Key
            if config.llm_api_key.is_empty() {
                log_error("请先设置 LLM API Key");
                println!("方法 1: 设置环境变量 LLM_API_KEY=your_key");
                println!("方法 2: 创建 .env 文件包含 LLM_API_KEY=your_key");
                return Ok(());
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
                    _ => {
                        i += 1;
                    }
                }
            }

            // 检查 API Key
            if config.llm_api_key.is_empty() {
                log_error("请先设置 LLM API Key");
                println!("方法 1: 设置环境变量 LLM_API_KEY=your_key");
                println!("方法 2: 创建 .env 文件包含 LLM_API_KEY=your_key");
                return Ok(());
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
