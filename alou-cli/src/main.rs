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

        // 帮助
        "help" | "-h" | "--help" => cmd_help(),

        _ => {
            log_error(&format!("未知命令: {}", args[1]));
            println!("运行 {}alou help{} 查看命令", GREEN, RESET);
        }
    }
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
