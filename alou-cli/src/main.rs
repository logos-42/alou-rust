//! Alou CLI - AI Agent 终端工具
//!
//! 用法: alou [命令] [参数]
//!
//! 示例:
//!   alou start        启动自主循环
//!   alou status       查看状态
//!   alou task add "测试"  添加任务
//!   alou agent chat "你好"  和 Agent 对话

use std::env;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::Utc;

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

fn cmd_agent_chat(message: &str) {
    log_section("Agent 对话");
    log_info(&format!("发送: {}", message));
    
    println!();
    println!("🤖 Alou Agent 回复:");
    println!();
    println!("你好！我是 Alou AI。");
    println!("收到你的消息: \"{}\"", message);
    println!();
    println!("自主循环已就绪，可以开始工作了。");
    println!("输入 alou help 查看所有命令。");
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
    println!("{}帮助:{}", CYAN, RESET);
    println!("  help             帮助");
    println!();
    println!("{}示例:{}", GREEN, RESET);
    println!("  alou start");
    println!("  alou task add \"检查邮件\" \"检查未读邮件\" high");
    println!("  alou agent chat \"你好\"");
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
        
        "help" | "-h" | "--help" => cmd_help(),
        
        _ => {
            log_error(&format!("未知命令: {}", args[1]));
            println!("运行 {}alou help{} 查看命令", GREEN, RESET);
        }
    }
}
