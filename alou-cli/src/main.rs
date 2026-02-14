//! Alou CLI - AI Agent 终端工具
//!
//! 功能:
//!   - 工具系统 (文件系统、搜索、Bash、网络、系统)
//!   - 群聊协作 (IPFS PubSub)
//!   - 智能体创建和管理
//!   - 自主循环控制
//!   - 任务管理

mod api;
mod agent;

use std::env;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::Utc;

// 导入配置模块
use api::{load_config, save_config};

// 颜色常量
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";
const BRIGHT: &str = "\x1b[1m";

fn log(color: &str, msg: &str) {
    println!("{}{}{}", color, msg, RESET);
}

fn log_success(msg: &str) { log(GREEN, &format!("✅ {}", msg)); }
fn log_error(msg: &str) { log(RED, &format!("❌ {}", msg)); }
fn log_info(msg: &str) { log(BLUE, &format!("ℹ️  {}", msg)); }
fn log_warn(msg: &str) { log(YELLOW, &format!("⚠ {}", msg)); }

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
            heartbeat_interval_seconds: 60,
            task_check_interval_seconds: 30,
            memory_save_interval_seconds: 300,
            progress_report_interval_seconds: 120,
        },
    }
}

fn save_state(state: &LoopState) {
    let path = get_alou_path();
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    
    let content = serde_json::to_string(state).unwrap_or_default();
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

// ============= 自主循环命令 =============

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

// Run command - execute a single task iteration
fn cmd_run() {
    log_info("执行单次任务迭代...");
    
    let state = load_state();
    
    if !state.is_running {
        log_error("自主循环未运行。请先使用 'alou start' 启动。");
        return;
    }
    
    if state.is_paused {
        log_error("自主循环已暂停。请先使用 'alou resume' 恢复。");
        return;
    }
    
    // Load tasks
    let tasks = load_tasks();
    
    if tasks.is_empty() {
        log_info("没有待执行的任务");
        println!("使用 'alou task add <标题>' 添加任务");
        return;
    }
    
    // Find the first pending task
    if let Some(task) = tasks.iter().find(|t| t.status == "pending") {
        let task_id = task.id.clone();
        let task_title = task.title.clone();
        
        log_section("执行任务");
        println!("任务ID: {}", task.id);
        println!("标题: {}", task.title);
        println!("描述: {}", task.description);
        println!("优先级: {}", task.priority);
        println!();
        
        log_info("任务执行模拟中 (连接 AI API 以执行真实任务)");
        log_success(&format!("任务 '{}' 已完成", task_title));
        
        // Update task status
        let mut updated_tasks = tasks;
        if let Some(t) = updated_tasks.iter_mut().find(|t| t.id == task_id) {
            t.status = "completed".to_string();
        }
        
        // Save updated tasks
        let tasks_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".alou")
            .join("tasks");
        if !tasks_path.exists() {
            let _ = fs::create_dir_all(&tasks_path);
        }
        let content = serde_json::to_string(&updated_tasks).unwrap_or_default();
        let _ = fs::write(tasks_path.join("queue.json"), content);
        
        // Update state
        let mut state = load_state();
        state.tasks_completed += 1;
        state.total_iterations += 1;
        state.last_heartbeat = Utc::now().timestamp();
        save_state(&state);
    } else {
        log_info("没有找到待执行的任务");
    }
}

// Loop command - continuously run tasks
fn cmd_loop() {
    use std::thread;
    use std::time::Duration;
    
    log_section("启动自主循环");
    
    let mut state = load_state();
    
    if state.is_running && !state.is_paused {
        log_warn("循环已在运行中");
        return;
    }
    
    state.is_running = true;
    state.is_paused = false;
    state.last_heartbeat = Utc::now().timestamp();
    save_state(&state);
    
    log_success("自主循环已启动 (按 Ctrl+C 停止)");
    println!();
    
    let mut iteration = 0u64;
    
    loop {
        // Check if still running
        let state = load_state();
        if !state.is_running {
            log_info("循环已停止");
            break;
        }
        if state.is_paused {
            println!("⏸  循环已暂停");
            thread::sleep(Duration::from_secs(2));
            continue;
        }
        
        iteration += 1;
        
        println!();
        log_section(&format!("迭代 #{}", iteration));
        
        // Load and process tasks
        let mut tasks = load_tasks();
        
        if tasks.is_empty() {
            println!("没有待执行的任务");
            println!("使用 'alou task add <标题>' 添加任务");
        } else {
            // Find pending tasks
            let mut completed_count = 0u64;
            
            // Get pending task IDs first
            let pending_ids: Vec<String> = tasks.iter()
                .filter(|t| t.status == "pending")
                .map(|t| t.id.clone())
                .collect();
            
            // Process each pending task
            for task_id in pending_ids {
                // Find and update task
                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                    println!("执行任务: {} - {}", task.title, task.description);
                    
                    // Mark as running
                    task.status = "running".to_string();
                }
            }
            
            // Save running state
            let tasks_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".alou")
                .join("tasks");
            let content = serde_json::to_string(&tasks).unwrap_or_default();
            let _ = fs::write(tasks_path.join("queue.json"), content);
            
            // Simulate task execution
            thread::sleep(Duration::from_millis(500));
            
            // Mark as completed
            for task in tasks.iter_mut() {
                if task.status == "running" {
                    task.status = "completed".to_string();
                    completed_count += 1;
                    println!("✅ 任务完成: {}", task.title);
                }
            }
            
            // Save completed state
            let content = serde_json::to_string(&tasks).unwrap_or_default();
            let _ = fs::write(tasks_path.join("queue.json"), content);
            
            // Update state
            let mut state = load_state();
            state.total_iterations = iteration;
            state.tasks_completed += completed_count;
            state.last_heartbeat = Utc::now().timestamp();
            save_state(&state);
        }
        
        // Wait before next iteration
        let state = load_state();
        let interval = state.config.task_check_interval_seconds;
        println!();
        println!("等待 {} 秒...", interval);
        thread::sleep(Duration::from_secs(interval));
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

// ============= 任务命令 =============

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
        let status_color = match task.status.as_str() {
            "pending" => YELLOW,
            "running" => BLUE,
            "completed" => GREEN,
            _ => RED,
        };
        println!("{}. {}[{}]{} {} - {}", 
                 i + 1, 
                 status_color, task.status.to_uppercase(), RESET,
                 task.title, task.description);
    }
}

// ============= 配置命令 =============

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
        _ => {
            log_error(&format!("未知配置项: {}", key));
            println!("支持的配置项: api_url, api_key, model");
            return;
        }
    }

    if let Err(e) = save_config(&config) {
        log_error(&format!("保存配置失败: {}", e));
    }
}

// ============= 帮助 =============

fn cmd_help() {
    println!();
    println!("{}{}Alou CLI v0.2.0 - AI Agent 终端工具{}", CYAN, BRIGHT, RESET);
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
