//! Commands module - CLI 命令实现

use crate::api::{self, check_tool_api_available, execute_tool, get_tool_list, ToolExecuteResponse};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::path::PathBuf;
use std::fs;

// 状态结构
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LoopState {
    pub is_running: bool,
    pub is_paused: bool,
    pub current_task_id: Option<String>,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub total_iterations: u64,
    pub last_heartbeat: i64,
    pub config: LoopConfig,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LoopConfig {
    pub heartbeat_interval_seconds: u64,
    pub task_check_interval_seconds: u64,
    pub memory_save_interval_seconds: u64,
    pub progress_report_interval_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
}

// 颜色常量
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

pub fn log_success(msg: &str) { log(GREEN, &format!("✅ {}", msg)); }
pub fn log_error(msg: &str) { log(RED, &format!("❌ {}", msg)); }
pub fn log_info(msg: &str) { log(BLUE, &format!("ℹ️  {}", msg)); }
pub fn log_warn(msg: &str) { log(YELLOW, &format!("⚠ {}", msg)); }

pub fn log_section(msg: &str) {
    println!("\n{}{}=== {} ==={}", CYAN, BRIGHT, msg, RESET);
}

fn get_alou_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".alou")
        .join("autonomous")
}

pub fn load_state() -> LoopState {
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

pub fn save_state(state: &LoopState) {
    let path = get_alou_path();
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    
    let content = serde_json::to_string(state).unwrap_or_default();
    let _ = fs::write(path.join("state.json"), content);
}

pub fn load_tasks() -> Vec<Task> {
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

pub fn cmd_start() {
    log_info("启动自主循环...");
    let mut state = load_state();
    state.is_running = true;
    state.is_paused = false;
    state.last_heartbeat = Utc::now().timestamp();
    save_state(&state);
    log_success("自主循环已启动");
    log_info("运行: alou status 查看状态");
}

pub fn cmd_stop() {
    log_info("停止自主循环...");
    let mut state = load_state();
    state.is_running = false;
    state.is_paused = false;
    save_state(&state);
    log_success("自主循环已停止");
}

pub fn cmd_pause() {
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

pub fn cmd_resume() {
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
pub fn cmd_run() {
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
    
    let tasks = load_tasks();
    
    if tasks.is_empty() {
        log_info("没有待执行的任务");
        println!("使用 'alou task add <标题>' 添加任务");
        return;
    }
    
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
        
        let mut updated_tasks = tasks;
        if let Some(t) = updated_tasks.iter_mut().find(|t| t.id == task_id) {
            t.status = "completed".to_string();
        }
        
        let tasks_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".alou")
            .join("tasks");
        if !tasks_path.exists() {
            let _ = fs::create_dir_all(&tasks_path);
        }
        let content = serde_json::to_string(&updated_tasks).unwrap_or_default();
        let _ = fs::write(tasks_path.join("queue.json"), content);
        
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
pub fn cmd_loop() {
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
        
        let mut tasks = load_tasks();
        
        if tasks.is_empty() {
            println!("没有待执行的任务");
            println!("使用 'alou task add <标题>' 添加任务");
        } else {
            let mut completed_count = 0u64;
            
            let pending_ids: Vec<String> = tasks.iter()
                .filter(|t| t.status == "pending")
                .map(|t| t.id.clone())
                .collect();
            
            for task_id in pending_ids {
                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                    println!("执行任务: {} - {}", task.title, task.description);
                    task.status = "running".to_string();
                }
            }
            
            let tasks_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".alou")
                .join("tasks");
            let content = serde_json::to_string(&tasks).unwrap_or_default();
            let _ = fs::write(tasks_path.join("queue.json"), content);
            
            thread::sleep(Duration::from_millis(500));
            
            for task in tasks.iter_mut() {
                if task.status == "running" {
                    task.status = "completed".to_string();
                    completed_count += 1;
                    println!("✅ 任务完成: {}", task.title);
                }
            }
            
            let content = serde_json::to_string(&tasks).unwrap_or_default();
            let _ = fs::write(tasks_path.join("queue.json"), content);
            
            let mut state = load_state();
            state.total_iterations = iteration;
            state.tasks_completed += completed_count;
            state.last_heartbeat = Utc::now().timestamp();
            save_state(&state);
        }
        
        let state = load_state();
        let interval = state.config.task_check_interval_seconds;
        println!();
        println!("等待 {} 秒...", interval);
        thread::sleep(Duration::from_secs(interval));
    }
}

pub fn cmd_status() {
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

pub fn cmd_task_add(title: &str, description: &str, priority: &str) {
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

pub fn cmd_task_list() {
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

// ============= 工具命令 =============

pub fn cmd_tool_list() {
    log_section("可用工具列表");
    
    if !check_tool_api_available() {
        log_warn("工具 API 不可用!");
        println!("请确保 Alou Desktop 正在运行");
        return;
    }
    
    log_info("正在获取工具列表...");
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(get_tool_list()) {
        Ok(tools) => {
            let count = tools.len();
            if tools.is_empty() {
                println!("没有找到可用工具");
            } else {
                for tool in &tools {
                    let desc = tool.description.as_ref().map(|s| s.as_str()).unwrap_or("");
                    println!("  - {}: {}", tool.name, desc);
                }
            }
            log_success(&format!("共 {} 个工具", count));
        }
        Err(e) => {
            log_error(&format!("获取工具列表失败: {}", e));
        }
    }
}

pub fn cmd_tool_exec(tool_id: &str, args_json: &str) {
    log_section(&format!("执行工具: {}", tool_id));
    
    if !check_tool_api_available() {
        log_warn("工具 API 不可用!");
        println!("请确保 Alou Desktop 正在运行");
        return;
    }
    
    let args: serde_json::Value = if args_json.is_empty() {
        serde_json::json!({})
    } else {
        match serde_json::from_str(args_json) {
            Ok(v) => v,
            Err(e) => {
                log_error(&format!("参数解析失败: {}", e));
                return;
            }
        }
    };
    
    log_info(&format!("工具参数: {}", args));
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(execute_tool(tool_id, args, None, None)) {
        Ok(response) => {
            if response.success {
                log_success("工具执行成功!");
                if let Some(data) = response.data {
                    println!("\n结果:");
                    println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
                }
                if let Some(ms) = response.execution_time_ms {
                    println!("\n执行时间: {}ms", ms);
                }
            } else {
                log_error(&format!("工具执行失败: {}", response.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            log_error(&format!("请求失败: {}", e));
        }
    }
}

// ============= 自动迭代命令 =============

async fn execute_tool_async(tool_id: &str, args: serde_json::Value) -> Result<ToolExecuteResponse, String> {
    execute_tool(tool_id, args, None, None).await
}

pub fn cmd_auto(task_description: &str) {
    log_section("自动迭代执行");
    
    if !check_tool_api_available() {
        log_warn("工具 API 不可用!");
        println!("请确保 Alou Desktop 正在运行");
        return;
    }
    
    let config = api::load_config();
    
    if config.ai.api_key.is_empty() {
        log_error("请先设置 API Key: alou config set api_key <your_key>");
        return;
    }
    
    log_info(&format!("任务: {}", task_description));
    println!();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let analysis_prompt = format!(
        r#"用户任务: {}

请分析这个任务，确定需要使用哪些工具来完成任务。

可用工具:
- filesystem: 文件系统操作 (read, write, edit, delete, list)
- bash: 执行终端命令
- search: 搜索文件和文本
- plan: 创建和更新计划

请以 JSON 格式返回你的计划:
{{"step": 1, "tool": "工具名", "action": "具体操作", "reason": "为什么这样做"}}
"#,
        task_description
    );
    
    log_info("分析任务...");
    
    let messages = vec![
        api::Message { role: "user".to_string(), content: analysis_prompt },
    ];
    
    match rt.block_on(api::send_chat_request(&config, messages)) {
        Ok(response) => {
            log_success("任务分析完成");
            println!("\nAI 建议: {}", response);
            println!();
            
            if let Ok(plan) = serde_json::from_str::<serde_json::Value>(&response) {
                if let Some(tool) = plan.get("tool").and_then(|v| v.as_str()) {
                    let args = plan.get("args").cloned().unwrap_or(serde_json::json!({}));
                    
                    log_info(&format!("执行工具: {}...", tool));
                    
                    match rt.block_on(execute_tool_async(tool, args)) {
                        Ok(result) => {
                            if result.success {
                                log_success("工具执行成功!");
                                if let Some(data) = result.data {
                                    println!("\n结果:");
                                    println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
                                }
                            } else {
                                log_error(&format!("工具执行失败: {}", result.error.unwrap_or_default()));
                            }
                        }
                        Err(e) => {
                            log_error(&format!("执行出错: {}", e));
                        }
                    }
                }
            }
            
            log_info("自动迭代完成");
            println!("\n提示: 使用 'alou tool exec <tool> <args>' 来执行特定工具");
        }
        Err(e) => {
            log_error(&format!("AI 请求失败: {}", e));
        }
    }
}
