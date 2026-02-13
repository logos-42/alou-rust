//! Alou CLI - AI Agent 终端工具
//!
//! 用法: alou [命令] [参数]
//!
//! 示例:
//!   alou start        启动自主循环
//!   alou status       查看状态
//!   alou task add "测试"  添加任务
//!   alou agent chat "你好"  和 Agent 对话
//!   alou config show  显示配置

mod api;
mod agent;

use std::env;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::Utc;

// 导入新模块
use api::{Config, load_config, save_config};

// 颜色
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";
const BRIGHT: &str = "\x1b[1m";

fn log(color: &str, msg: &str) {
    println!("{}{}{}", color, msg, RESET);
}

fn log_success(msg: &str) { log(GREEN, &format!("✅ {}", msg)); }
fn log_error(msg: &str) { log(RED, &format!("❌ {}", msg)); }
fn log_info(msg: &str) { log(BLUE, &format!("ℹ️  {}", msg)); }

fn log_section(msg: &str) {
    println!("\n{}{}=== {} ==={}", CYAN, BRIGHT, msg, RESET);
}

// 状态结构
#[derive(Serialize, Deserialize, Debug, Default)]
struct LoopState {
    is_running: bool,
    is_paused: bool,
    current_task_id: Option<String>,
    tasks_completed: u64,
    tasks_failed: u64,
    total_iterations: u64,
    last_heartbeat: i64,
    config: LoopConfig,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct LoopConfig {
    heartbeat_interval_seconds: u64,
    task_check_interval_seconds: u64,
    memory_save_interval_seconds: u64,
    progress_report_interval_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: String,
    title: String,
    description: String,
    status: String,
    priority: String,
}

fn get_alou_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".alou")
        .join("autonomous")
}

fn load_state() -> LoopState {
    let path = get_alou_path().join("state.json");
    
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&content) {
                return state;
            }
        }
    }
    
    LoopState {
        is_running: false,
        is_paused: false,
        current_task_id: None,
        tasks_completed: 0,
        tasks_failed: 0,
        total_iterations: 0,
        last_heartbeat: Utc::now().timestamp(),
        config: LoopConfig {
            heartbeat_interval_seconds: 30,
            task_check_interval_seconds: 10,
            memory_save_interval_seconds: 60,
            progress_report_interval_seconds: 300,
        },
    }
}

fn save_state(state: &LoopState) {
    let path = get_alou_path();
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    
    let content = serde_json::to_string_pretty(state).unwrap_or_default();
    let _ = fs::write(path.join("state.json"), content);
}

fn load_tasks() -> Vec<Task> {
    let tasks_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".alou")
        .join("tasks")
        .join("queue.json");
    
    if tasks_path.exists() {
        if let Ok(content) = fs::read_to_string(&tasks_path) {
            if let Ok(tasks) = serde_json::from_str(&content) {
                return tasks;
            }
        }
    }
    Vec::new()
}

// 命令
fn cmd_start() {
    log_info("启动自主循环...");
    let mut state = load_state();
    state.is_running = true;
    state.is_paused = false;
    state.last_heartbeat = Utc::now().timestamp();
    save_state(&state);
    log_success("自主循环已启动");
    log_info("运行: alou status 查看状态");
}

fn cmd_stop() {
    log_info("停止自主循环...");
    let mut state = load_state();
    state.is_running = false;
    state.is_paused = false;
    save_state(&state);
    log_success("自主循环已停止");
}

fn cmd_pause() {
    log_info("暂停自主循环...");
    let mut state = load_state();
    if state.is_running {
        state.is_paused = true;
        save_state(&state);
        log_success("已暂停");
    } else {
        log_error("自主循环未运行");
    }
}

fn cmd_resume() {
    log_info("恢复自主循环...");
    let mut state = load_state();
    if state.is_running && state.is_paused {
        state.is_paused = false;
        save_state(&state);
        log_success("已恢复");
    } else {
        log_error("自主循环未运行或未暂停");
    }
}

fn cmd_status() {
    let state = load_state();
    
    log_section("自主循环状态");
    
    let status = if state.is_running {
        if state.is_paused { "已暂停" } else { "运行中" }
    } else {
        "已停止"
    };
    
    let status_color = if state.is_running {
        if state.is_paused { YELLOW } else { GREEN }
    } else {
        RED
    };
    
    print!("状态: {}{}{}", status_color, status, RESET);
    println!();
    println!("完成任务: {}", state.tasks_completed);
    println!("失败任务: {}", state.tasks_failed);
    println!("循环次数: {}", state.total_iterations);
    println!("当前任务: {}", state.current_task_id.as_ref().unwrap_or(&"无".to_string()));
    
    log_section("配置");
    println!("  心跳间隔: {}秒", state.config.heartbeat_interval_seconds);
    println!("  任务检查: {}秒", state.config.task_check_interval_seconds);
    println!("  记忆保存: {}秒", state.config.memory_save_interval_seconds);
    println!("  进度汇报: {}秒", state.config.progress_report_interval_seconds);
}

fn cmd_task_add(title: &str, description: &str, priority: &str) {
    log_info(&format!("添加任务: {}", title));
    
    let tasks_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".alou")
        .join("tasks");
    
    if !tasks_path.exists() {
        let _ = fs::create_dir_all(&tasks_path);
    }
    
    let mut tasks = load_tasks();
    
    let task = Task {
        id: format!("task_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string()),
        title: title.to_string(),
        description: description.to_string(),
        status: "pending".to_string(),
        priority: priority.to_string(),
    };
    
    tasks.push(task);
    
    let content = serde_json::to_string(&tasks).unwrap_or_default();
    let _ = fs::write(tasks_path.join("queue.json"), content);
    
    log_success("任务已添加");
}

fn cmd_task_list() {
    log_section("任务队列");
    let tasks = load_tasks();
    
    if tasks.is_empty() {
        println!("没有待执行任务");
        return;
    }
    
    for (i, task) in tasks.iter().enumerate() {
        println!("{}. [{}] {} - {}", 
                 i + 1, 
                 task.status.to_uppercase(),
                 task.title,
                 task.description);
    }
}

// Agent聊天命令
fn cmd_agent_chat(message: &str) {
    log_section("Agent 对话");
    log_info(&format!("发送: {}", message));
    
    // 加载配置
    let config = load_config();
    
    // 检查API配置
    if config.ai.api_key.is_empty() {
        println!();
        println!("⚠️  请先配置API密钥:");
        println!("   alou config set api_key <你的API密钥>");
        println!();
        return;
    }
    
    println!();
    println!("🤖 Alou Agent 回复:");
    println!();
    println!("你好！我是 Alou AI。");
    println!("收到你的消息: \"{}\"", message);
    println!();
    println!("(要启用真正的AI对话，请配置API密钥)");
    println!("输入 alou help 查看所有命令。");
}

// 配置命令
fn cmd_config_show() {
    log_section("当前配置");
    let config = load_config();

    println!("API配置:");
    println!("  Base URL: {}", config.api.base_url);
    println!("  Timeout: {}ms", config.api.timeout);
    println!();
    println!("AI配置:");
    println!("  Provider: {}", config.ai.provider);
    println!("  Model: {}", config.ai.model);
    println!("  API Key: {}", if config.ai.api_key.is_empty() { "(未设置)" } else { "******" });
    println!("  Temperature: {}", config.ai.temperature);
    println!("  Max Tokens: {}", config.ai.max_tokens);
    println!();
    println!("自主循环配置:");
    println!("  Enabled: {}", config.autonomous.enabled);
    println!("  Heartbeat: {}s", config.autonomous.heartbeat_interval);
    println!("  Task Check: {}s", config.autonomous.task_check_interval);
}

fn cmd_config_set(key: &str, value: &str) {
    let mut config = load_config();

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
        "provider" => {
            config.ai.provider = value.to_string();
            log_success(&format!("提供商设置为: {}", value));
        }
        _ => {
            log_error(&format!("未知配置项: {}", key));
            println!("支持的配置项: api_url, api_key, model, provider");
            return;
        }
    }

    if let Err(e) = save_config(&config) {
        log_error(&format!("保存配置失败: {}", e));
    }
}

// 存档功能相关函数
fn get_archive_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".alou")
        .join("archive")
}

fn cmd_archive_list() {
    let archive_path = get_archive_path();
    if !archive_path.exists() {
        println!("没有存档文件");
        return;
    }

    let entries = fs::read_dir(&archive_path);
    match entries {
        Ok(dir) => {
            let mut archives: Vec<String> = Vec::new();
            for entry in dir {
                if let Ok(entry) = entry {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".json") {
                            archives.push(file_name.to_string());
                        }
                    }
                }
            }
            
            if archives.is_empty() {
                println!("没有存档文件");
            } else {
                log_section("存档列表");
                for (i, archive) in archives.iter().enumerate() {
                    println!("{}. {}", i + 1, archive);
                }
            }
        }
        Err(_) => {
            println!("无法读取存档目录");
        }
    }
}

fn cmd_archive_create(name: &str) {
    log_info(&format!("创建存档: {}", name));
    
    let archive_path = get_archive_path();
    if !archive_path.exists() {
        let _ = fs::create_dir_all(&archive_path);
    }
    
    // 创建存档内容 - 包含当前状态和任务
    let state = load_state();
    let tasks = load_tasks();
    
    let archive_content = serde_json::json!({
        "timestamp": chrono::Utc::now().timestamp(),
        "state": state,
        "tasks": tasks,
        "version": "0.1.10"
    });
    
    let archive_file = archive_path.join(format!("{}.json", name));
    let content = serde_json::to_string_pretty(&archive_content).unwrap_or_default();
    match fs::write(&archive_file, content) {
        Ok(_) => {
            log_success(&format!("存档 '{}' 已创建", name));
        }
        Err(e) => {
            log_error(&format!("创建存档失败: {}", e));
        }
    }
}

fn cmd_archive_load(name: &str) {
    log_info(&format!("加载存档: {}", name));
    
    let archive_file = get_archive_path().join(format!("{}.json", name));
    if !archive_file.exists() {
        log_error(&format!("存档 '{}' 不存在", name));
        return;
    }
    
    match fs::read_to_string(&archive_file) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(archive_data) => {
                    // 恢复状态
                    if let Some(state_val) = archive_data.get("state") {
                        if let Ok(state) = serde_json::from_value::<LoopState>(state_val.clone()) {
                            save_state(&state);
                            log_success("状态已恢复");
                        }
                    }
                    
                    // 恢复任务
                    if let Some(tasks_val) = archive_data.get("tasks") {
                        if let Ok(tasks) = serde_json::from_value::<Vec<Task>>(tasks_val.clone()) {
                            let tasks_path = dirs::home_dir()
                                .unwrap_or_else(|| PathBuf::from("."))
                                .join(".alou")
                                .join("tasks");
                            
                            if !tasks_path.exists() {
                                let _ = fs::create_dir_all(&tasks_path);
                            }
                            
                            let content = serde_json::to_string(&tasks).unwrap_or_default();
                            let _ = fs::write(tasks_path.join("queue.json"), content);
                            log_success("任务已恢复");
                        }
                    }
                    
                    log_success(&format!("存档 '{}' 已加载", name));
                }
                Err(e) => {
                    log_error(&format!("解析存档失败: {}", e));
                }
            }
        }
        Err(e) => {
            log_error(&format!("读取存档失败: {}", e));
        }
    }
}

fn cmd_help() {
    println!();
    println!("{}{}Alou CLI - AI Agent 终端工具{}", CYAN, BRIGHT, RESET);
    println!();
    println!("{}使用: alou [命令] [参数]{}", YELLOW, RESET);
    println!();
    println!("{}自主循环控制:{}", CYAN, RESET);
    println!("  start           启动");
    println!("  stop            停止");
    println!("  pause           暂停");
    println!("  resume          恢复");
    println!("  status          状态");
    println!();
    println!("{}任务管理:{}", CYAN, RESET);
    println!("  task add <标题> [描述] [优先级]  添加");
    println!("  task list                        列表");
    println!();
    println!("{}Agent 对话:{}", CYAN, RESET);
    println!("  agent chat <消息>               对话");
    println!();
    println!("{}配置管理:{}", CYAN, RESET);
    println!("  config show                      显示配置");
    println!("  config set <key> <value>        设置配置");
    println!();
    println!("{}存档管理:{}", CYAN, RESET);
    println!("  archive create <名称>            创建存档");
    println!("  archive load <名称>              加载存档");
    println!("  archive list                     列出存档");
    println!();
    println!("{}帮助:{}", CYAN, RESET);
    println!("  help             帮助");
    println!();
    println!("{}示例:{}", GREEN, RESET);
    println!("  alou start");
    println!("  alou task add \"检查邮件\" \"检查未读邮件\" high");
    println!("  alou agent chat \"你好\"");
    println!("  alou config set api_key your_key_here");
    println!("  alou archive create backup");
    println!("  alou status");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args[1] != "help" {
        println!();
        println!("{}╔═══════════════════════════════════════╗", CYAN);
        println!("{}║     🤖 Alou CLI - AI Agent 终端      ║", CYAN);
        println!("{}╚═══════════════════════════════════════╝", CYAN);
    }
    
    if args.len() < 2 {
        cmd_help();
        return;
    }
    
    match args[1].as_str() {
        "start" => cmd_start(),
        "stop" => cmd_stop(),
        "pause" => cmd_pause(),
        "resume" => cmd_resume(),
        "status" => cmd_status(),
        
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
        
        "agent" => {
            if args.len() < 3 {
                log_error("请提供对话内容");
                return;
            }
            if args[2] == "chat" {
                let message = args[3..].join(" ");
                cmd_agent_chat(&message);
            } else {
                log_error(&format!("未知操作: {}", args[2]));
            }
        }
        
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

        "archive" => {
            if args.len() < 3 {
                log_error("缺少存档操作");
                return;
            }
            match args[2].as_str() {
                "create" => {
                    let name = args.get(3).map(|s| s.as_str()).unwrap_or("");
                    if name.is_empty() {
                        log_error("请提供存档名称");
                    } else {
                        cmd_archive_create(name);
                    }
                }
                "load" => {
                    let name = args.get(3).map(|s| s.as_str()).unwrap_or("");
                    if name.is_empty() {
                        log_error("请提供存档名称");
                    } else {
                        cmd_archive_load(name);
                    }
                }
                "list" => cmd_archive_list(),
                _ => log_error(&format!("未知操作: {}", args[2])),
            }
        }

        "help" | "-h" | "--help" => cmd_help(),

        _ => {
            log_error(&format!("未知命令: {}", args[1]));
            println!("运行 {}alou help{} 查看命令", GREEN, RESET);
        }
    }
}
