// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridges;
mod browser;
mod context;
mod diap;
mod ipfs_api;
mod ipfs_commands;
mod ipfs_node;
mod ipfs;
mod ipfs_repair;  // 新增 IPFS 修复模块
mod ipns_verified;  // 新增验证过的IPNS解决方案
// Runtime - Session Actor 架构
mod runtime;
use crate::runtime::router::SessionRouter;


mod kubo;
mod kv_commands;  // 新增KV存储模块
mod lsp;
mod memory_manager;  // 新增内存管理模块
mod prompts;        // 新增Prompt管理模块
mod spec;
mod sync;
mod utils;
mod wallet;
mod workflow;
mod tools;
mod prompt_system;
mod ai_loop;
mod autonomous_agent;  // 自主智能体模块
mod autonomous_loop;    // 自主循环模块
mod autonomous_loop_commands; // 自主循环命令
mod agent;  // 新增 Agent 模块
mod diap_file_manager;  // 新增 DIAP 文件管理模块
mod tool_api;  // 工具 API 模块
mod media_api;  // 媒体 API 模块
mod bot_gateway;  // Bot Gateway 模块
mod heartbeat;  // 心跳模块
mod cron;  // Cron 定时任务模块
mod soul;  // 灵魂/人格管理模块
mod scheduler;  // Agent 调度器模块
mod tasks;  // 任务管理模块
mod logs;  // 日志管理模块
mod agent_runtime;  // Agent Runtime 模块（新增）

use std::path::PathBuf;
use std::sync::Arc;
use chrono;
use tauri::Manager;
use tauri::menu::{Menu, Submenu, PredefinedMenuItem, MenuItem};

use crate::ipfs_node::{bootstrap_ipfs, IpfsState};
use crate::kubo::download_kubo_binary;
use crate::workflow::WorkflowState;
use crate::workflow::AsyncWorkflowExecutor;
use crate::ipfs_api::{
    diagnose_ipfs_api, get_ipfs_api_address, test_ipfs_api, test_ipfs_api_with_config,
};
use crate::ipfs_commands::{
    ipfs_add_base64,
    ipfs_pubsub_publish,
    ipfs_pubsub_subscribe_once,
    ipfs_pubsub_peers,
    ipfs_pubsub_ls,
};
use crate::ipfs_node::{get_ipfs_daemon_status, get_ipfs_info, start_ipfs_node, stop_ipfs_node};
use crate::kv_commands::{kv_clear, kv_exists, kv_get, kv_get_batch, kv_keys, kv_remove, kv_set, kv_set_batch, kv_stats, KvState};
use crate::diap::{
    create_local_diap_identity,
    create_diap_identity_from_did_document,
    get_local_diap_identity,
    update_local_diap_identity,
    create_diap_identity_with_zkp,
    test_ipns_on_public_gateway,
    ipfs_publish_ipns,
};
use crate::wallet::{verify_wallet_signature, get_testnet_private_key};
use crate::browser::{open_browser, test_ipfs_node_connection};
use crate::sync::{
    read_wallet_sync_data, start_wallet_sync_server, write_wallet_sync_data,
};
use crate::tool_api::start_tool_api_server;
use crate::media_api::start_media_api_server;
use crate::lsp::{execute_lsp, get_supported_languages};
use crate::spec::{execute_spec, get_spec_templates, load_template_content};
use crate::workflow::{
    create_workflow, execute_workflow, get_workflow_status, list_workflows,
    delete_workflow, retry_workflow_step, pause_workflow, resume_workflow,
};
use crate::workflow::{
    get_execution_status, pause_execution, resume_execution, cancel_execution,
    get_execution_logs, get_performance_metrics, get_ralph_loop_history,
    rollback_ralph_loop_execution, cleanup_ralph_loop_histories,
    start_workflow_event_listener,
};
use crate::bridges::{BridgeManager, create_default_bridge_manager};
use crate::tools::task_queue_tool::{initialize_task_queue_tool, add_task, get_next_task, update_task_status, set_task_result, list_tasks, get_task_stats, get_task_by_id};
use crate::memory_manager::{
    set_memory_item, get_memory_item, remove_memory_item, clear_memory,
    get_memory_keys, get_memory_stats, archive_to_ipfs, pin_cid, unpin_cid,
    garbage_collect, pin_item, unpin_item, set_diap_identity, get_diap_identity,
    remove_diap_identity, get_all_diap_identities, archive_diap_identity_to_ipfs,
    pin_diap_identity, unpin_diap_identity,
};

// Agent DIAP identity file storage
use crate::diap_file_manager::{
    set_diap_identity_for_agent, get_diap_identity_for_agent,
    remove_diap_identity_for_agent, get_all_diap_identities_for_agent,
    init_app_handle,
};

// Agent commands
use crate::agent::commands::{
    execute_agent_task, execute_ai_conversation, get_agent_config, update_agent_config,
    test_api_connection, get_available_providers, health_check,
};

// Agent Hook commands
use crate::agent::hook_commands::{
    agent_hook_inject,
    agent_hook_inject_high_priority,
    agent_hook_pause,
    agent_hook_resume,
    agent_hook_cancel,
    agent_hook_get_status,
    agent_hook_get_history,
    agent_hook_get_pending,
    agent_hook_clear_queue,
    agent_hook_create,
};

// Autonomous Agent commands - 状态管理函数
use crate::autonomous_agent::{
    get_autonomous_agent_state, 
    save_autonomous_agent_state,
};

// Autonomous Loop commands - 自主循环命令
use crate::autonomous_loop_commands::{
    start_autonomous_loop,
    stop_autonomous_loop,
    pause_autonomous_loop,
    resume_autonomous_loop,
    get_autonomous_loop_state,
    add_autonomous_task,
};

// Heartbeat commands - 心跳命令
use crate::heartbeat::{
    start_heartbeat,
    stop_heartbeat,
    trigger_heartbeat_now,
    get_heartbeat_state,
    get_heartbeat_config,
    update_heartbeat_config,
    initialize_heartbeat,
    is_heartbeat_enabled,
    get_heartbeat_file_content,
    write_heartbeat_file,
    clear_heartbeat_file,
    initialize_heartbeat_manager,
};

// Skill Auto Selector commands
use crate::tools::skill_auto_selector_tool::{
    analyze_and_select_tools, generate_execution_plan,
    get_available_tools, update_auto_select_config, get_auto_select_config,
};

// Autonomous Executor commands (使用别名避免冲突)
use crate::tools::autonomous_executor_tool::{
    autonomous_execute_plan,
    autonomous_execute_tool,
    autonomous_get_history,
    clear_execution_history,
    update_executor_config,
    get_executor_config,
    handle_chat_message,
    update_respond_config,
    get_respond_config,
};

// Tool commands
#[tauri::command]
async fn execute_tool(
    tool_id: String,
    args: String,
    timeout: Option<u64>,
    bridge_manager: tauri::State<'_, std::sync::Arc<BridgeManager>>,
) -> Result<serde_json::Value, String> {
    // 记录工具调用开始
    println!("\n========================================");
    println!("[Tauri] 开始执行工具：{}", tool_id);
    println!("[Tauri] 原始参数 JSON: {}", args);
    println!("========================================");

    // 解析参数
    let args_value: serde_json::Value = serde_json::from_str(&args)
        .map_err(|e| {
            let error_msg = format!("Invalid JSON args: {}", e);
            println!("[Tauri] ❌ 参数解析失败：{}", error_msg);
            error_msg
        })?;

    // 记录解析后的参数
    println!("[Tauri] 解析后的参数：{:#}", args_value);
    
    // 检查关键工具的 operation 字段
    if tool_id == "filesystem" || tool_id == "bash" || tool_id == "search" {
        if let Some(operation) = args_value.get("operation") {
            println!("[Tauri] ✓ 操作类型：{}", operation);
        } else {
            println!("[Tauri] ⚠️ 警告：{} 工具缺少 operation 字段", tool_id);
        }
        
        // 检查 shell 字段（bash 工具需要）
        if tool_id == "bash" {
            if let Some(shell) = args_value.get("shell") {
                println!("[Tauri] ✓ Shell 类型：{}", shell);
            } else {
                println!("[Tauri] ⚠️ Bash 工具缺少 shell 字段");
            }
            if let Some(command) = args_value.get("command") {
                println!("[Tauri] ✓ Command：{}", command);
            } else {
                println!("[Tauri] ⚠️ Bash 工具缺少 command 字段");
            }
        }
    }

    // Get tool bridge and execute tool
    let tool_bridge = bridge_manager.tool_bridge();

    let request = crate::bridges::ToolCallRequest {
        session_id: "tauri_session".to_string(),
        user_id: None,
        tool_id: tool_id.clone(),
        args: args_value.clone(),
        working_directory: std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        environment: std::env::vars().collect(),
        timeout_seconds: timeout,
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    println!("[Tauri] 调用 tool_bridge.handle_request...");
    
    match tool_bridge.handle_request(request).await {
        Ok(response) => {
            if response.success {
                println!("[Tauri] ✓ 工具执行成功：{}", tool_id);
                let execution_time = response.result.as_ref()
                    .map(|r| r.execution_time_ms)
                    .unwrap_or(100);
                let output = response.result.as_ref()
                    .and_then(|r| r.output.clone())
                    .unwrap_or_else(|| format!("Tool '{}' executed successfully", tool_id));
                let data = response.result.as_ref()
                    .map(|r| r.data.clone())
                    .unwrap_or_else(|| serde_json::json!({"status": "success"}));
                Ok(serde_json::json!({
                    "success": true,
                    "data": data,
                    "execution_time_ms": execution_time,
                    "output": output
                }))
            } else {
                let error_msg = response.error.unwrap_or_else(|| format!("Tool '{}' execution failed", tool_id));
                println!("[Tauri] ❌ 工具执行失败：{} - {}", tool_id, error_msg);
                println!("[Tauri] 请求参数：{:#}", args_value);
                Err(error_msg)
            }
        }
        Err(e) => {
            println!("[Tauri] ❌ Tool bridge error: {}", e);
            println!("[Tauri] 请求参数：{:#}", args_value);
            Err(format!("Tool bridge error: {}", e))
        }
    }
}

#[tauri::command]
async fn get_tool_list(
    bridge_manager: tauri::State<'_, std::sync::Arc<BridgeManager>>,
) -> Result<serde_json::Value, String> {
    // 从 ToolBridge 获取工具列表
    let tools = bridge_manager.tool_bridge().list_tools().await;
    Ok(serde_json::json!({ "tools": tools }))
}

#[tauri::command]
async fn cancel_tool_execution(
    execution_id: String,
    bridge_manager: tauri::State<'_, std::sync::Arc<BridgeManager>>,
) -> Result<serde_json::Value, String> {
    println!("[Tauri] Cancelling tool execution: {}", execution_id);

    let result = bridge_manager.tool_bridge().cancel_execution(&execution_id).await;

    match result {
        Ok(_) => Ok(serde_json::json!({
            "cancelled": true,
            "execution_id": execution_id,
            "message": "Tool execution cancelled successfully"
        })),
        Err(e) => Err(format!("Failed to cancel execution: {}", e))
    }
}

#[tauri::command]
async fn get_execution_history(
    limit: Option<usize>,
    bridge_manager: tauri::State<'_, std::sync::Arc<BridgeManager>>,
) -> Result<serde_json::Value, String> {
    println!("[Tauri] Getting execution history (limit: {:?})", limit);

    let history = bridge_manager.tool_bridge().get_execution_history(limit.unwrap_or(50)).await;

    Ok(serde_json::json!({
        "history": history,
        "limit": limit.unwrap_or(50),
        "count": history.len()
    }))
}

// Agent Documents commands - 获取文档路径和确保文档存在
#[tauri::command]
fn get_agent_documents_path(app: tauri::AppHandle, agent_id: String) -> Result<String, String> {
    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| format!("获取路径失败: {}", e))?;
    
    let doc_path = app_data_dir
        .join("agent-documents")
        .join(&agent_id);
    
    Ok(doc_path.to_string_lossy().to_string())
}

#[tauri::command]
fn ensure_agent_document(app: tauri::AppHandle, agent_id: String, document_type: String) -> Result<String, String> {
    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| format!("获取路径失败: {}", e))?;
    
    let doc_dir = app_data_dir.join("agent-documents").join(&agent_id);
    std::fs::create_dir_all(&doc_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;
    
    let doc_path = doc_dir.join(format!("{}.md", document_type.to_uppercase()));
    
    // 如果文件不存在，创建初始内容
    if !doc_path.exists() {
        let initial_content = generate_initial_document_content(&document_type, &agent_id);
        std::fs::write(&doc_path, initial_content)
            .map_err(|e| format!("创建文件失败: {}", e))?;
        log::info!("[main] 自动创建文档: {:?}", doc_path);
    }
    
    Ok(doc_path.to_string_lossy().to_string())
}

/// 生成初始文档内容
fn generate_initial_document_content(document_type: &str, agent_id: &str) -> String {
    let now = chrono::Utc::now().to_rfc3339();
    match document_type.to_lowercase().as_str() {
        "memory" => format!(r#"# 长期记忆

**智能体 ID**: {}

## 用户偏好
（暂无记录）

## 项目信息
（暂无记录）

## 学到的知识
（暂无记录）

## 重要对话
（暂无记录）

---
最后更新: {}
"#, agent_id, now),
        "soul" => format!(r#"# 核心身份

**智能体 ID**: {}

## 角色定位
专业的AI助手

## 核心价值观
- 准确性：提供准确可靠的信息
- 效率：快速完成任务
- 安全性：注重操作安全
- 学习性：从每次交互中学习和改进

## 个性特点
- 友好且专业
- 注重细节
- 善于沟通

---
最后更新: {}
"#, agent_id, now),
        "identity" => format!(r#"# 身份定义

**智能体 ID**: {}

## 名称
智能体

## 角色
专业的AI助手

## 专长领域
（根据实际使用情况更新）

## 工作方式
- 理解用户需求
- 选择合适工具
- 执行任务
- 反馈结果

---
最后更新: {}
"#, agent_id, now),
        "capabilities" => format!(r#"# 能力清单

## 核心能力
- 文件操作：读取、写入、编辑、搜索文件
- 终端命令：执行系统命令
- 网络操作：搜索信息、获取网页内容
- 任务规划：制定和管理任务计划
- 代码理解：分析和修改代码

## 工具使用
- 熟练使用所有可用工具
- 能够组合多个工具完成复杂任务
- 理解工具的限制和最佳实践

## 学习能力
- 从用户反馈中学习
- 记录成功的解决方案
- 避免重复错误

---
最后更新: {}
"#, now),
        "constraints" => format!(r#"# 约束和限制

## 操作限制
- 不执行危险命令
- 不访问敏感文件
- 不进行未经授权的网络操作

## 行为准则
- 始终征求用户确认重要操作
- 清晰解释操作步骤
- 提供操作结果反馈

## 安全原则
- 保护用户数据安全
- 遵守系统安全策略
- 及时报告异常情况

---
最后更新: {}
"#, now),
        "tools" => format!(r#"# 工具使用记录

## 常用工具
（根据实际使用情况更新）

## 工具组合
（记录有效的工具组合方案）

## 最佳实践
（记录工具使用的最佳实践）

---
最后更新: {}
"#, now),
        "agents" => format!(r#"# 协作智能体

## 已知智能体
（暂无记录）

## 协作经验
（暂无记录）

## 协作模式
（暂无记录）

---
最后更新: {}
"#, now),
        _ => format!(r#"# {}

---
最后更新: {}
"#, document_type, now),
    }
}

// Agent Skills command
#[tauri::command]
async fn agent_skills(
    action: String,
    skill_name: Option<String>,
    inputs: Option<serde_json::Value>,
    query: Option<String>,
    bridge_manager: tauri::State<'_, std::sync::Arc<BridgeManager>>,
) -> Result<serde_json::Value, String> {
    let args = serde_json::json!({
        "action": action,
        "skill_name": skill_name,
        "inputs": inputs,
        "query": query
    });
    
    let execution_id = format!("agent_skills_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

    let request = crate::bridges::ToolCallRequest {
        session_id: "agent_skills_session".to_string(),
        user_id: None,
        tool_id: "agent_skills".to_string(),
        args: args,
        working_directory: std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        environment: std::env::vars().collect(),
        timeout_seconds: Some(30),
        permissions: vec!["read".to_string(), "write".to_string(), "execute".to_string()],
    };

    let result = bridge_manager.tool_bridge().handle_request(request).await;
    
    match result {
        Ok(response) => {
            let mut json_result = serde_json::json!({
                "success": response.success,
                "execution_id": execution_id
            });
            
            if response.success {
                if let Some(tool_result) = response.result {
                    json_result["result"] = tool_result.data;
                    json_result["execution_time_ms"] = tool_result.execution_time_ms.into();
                }
            } else {
                json_result["error"] = serde_json::Value::String(response.error.unwrap_or_else(|| "Unknown error".to_string()));
            }
            
            Ok(json_result)
        }
        Err(e) => {
            Ok(serde_json::json!({
                "success": false,
                "execution_id": execution_id,
                "error": e.to_string()
            }))
        }
    }
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(tauri::async_runtime::Mutex::new(IpfsState {
            process: None,
            data_dir: PathBuf::new(),
        }))
        .manage(KvState::new())
        .manage(WorkflowState::default())
        .manage(AsyncWorkflowExecutor::new().expect("Failed to create workflow executor"))
        .manage(std::sync::Arc::new(create_default_bridge_manager()))
        .manage(std::sync::Arc::new(SessionRouter::new(
            std::sync::Arc::new(create_default_bridge_manager()),
            std::sync::Arc::new(crate::tools::ToolRegistry::new()),
        )))
        .manage(std::sync::Arc::new(tokio::sync::Mutex::new(initialize_task_queue_tool().unwrap())))
        .manage(initialize_heartbeat_manager())
        .manage(cron::initialize_cron().expect("Failed to initialize Cron scheduler"))
        // AgentRuntimeState 将在 init_agent_runtime 命令中创建
        .invoke_handler(tauri::generate_handler![
            // Agent Runtime commands
            agent_runtime::commands::init_agent_runtime,
            agent_runtime::commands::register_backend_agent,
            agent_runtime::commands::send_group_message,
            agent_runtime::commands::get_active_agents,
            agent_runtime::commands::stop_agent,
            agent_runtime::commands::get_group_history,
            agent_runtime::commands::submit_task,
            agent_runtime::commands::list_tools,
            agent_runtime::commands::execute_tool,
            agent_runtime::commands::get_runtime_status,
            // Group chat subscription commands
            agent_runtime::commands::agent_subscribe_group,
            agent_runtime::commands::agent_unsubscribe_group,
            agent_runtime::commands::agent_list_subscriptions,
            
            download_kubo_binary,
            start_ipfs_node,
            stop_ipfs_node,
            get_ipfs_info,
            get_ipfs_daemon_status,
            get_ipfs_api_address,
            diagnose_ipfs_api,
            test_ipfs_api,
            test_ipfs_api_with_config,
            ipfs_add_base64,
            ipfs_pubsub_publish,
            ipfs_pubsub_subscribe_once,
            ipfs_pubsub_peers,
            ipfs_pubsub_ls,
            // KV commands
            kv_set,
            kv_get,
            kv_remove,
            kv_exists,
            kv_keys,
            kv_clear,
            kv_stats,
            kv_get_batch,
            kv_set_batch,
            // DIAP commands
            create_local_diap_identity,
            create_diap_identity_from_did_document,
            get_local_diap_identity,
            update_local_diap_identity,
            create_diap_identity_with_zkp,
            test_ipns_on_public_gateway,
            ipfs_publish_ipns,
            // LSP SDK commands
            execute_lsp,
            get_supported_languages,
            // Spec SDK commands
            execute_spec,
            get_spec_templates,
            load_template_content,
            verify_wallet_signature,
            get_testnet_private_key,
            open_browser,
            test_ipfs_node_connection,
            write_wallet_sync_data,
            read_wallet_sync_data,
            start_wallet_sync_server,
            // Workflow commands
            create_workflow,
            execute_workflow,
            get_workflow_status,
            list_workflows,
            delete_workflow,
            retry_workflow_step,
            pause_workflow,
            resume_workflow,
            // Async execution commands
            get_execution_status,
            pause_execution,
            resume_execution,
            cancel_execution,
            // Monitoring commands
            get_execution_logs,
            get_performance_metrics,
            // Ralph Loop commands
            get_ralph_loop_history,
            rollback_ralph_loop_execution,
            cleanup_ralph_loop_histories,
            // Tool commands
            execute_tool,
            get_tool_list,
            cancel_tool_execution,
            get_execution_history,
            // Agent Skills commands
            agent_skills,
            // Agent Documents commands
            get_agent_documents_path,
            ensure_agent_document,
            // Memory management commands
            set_memory_item,
            get_memory_item,
            remove_memory_item,
            clear_memory,
            get_memory_keys,
            get_memory_stats,
            archive_to_ipfs,
            pin_cid,
            unpin_cid,
            garbage_collect,
            pin_item,
            unpin_item,
            // DIAP identity management commands
            set_diap_identity,
            get_diap_identity,
            remove_diap_identity,
            get_all_diap_identities,
            archive_diap_identity_to_ipfs,
            pin_diap_identity,
            unpin_diap_identity,
            // DIAP identity for agent commands (file-based)
            set_diap_identity_for_agent,
            get_diap_identity_for_agent,
            remove_diap_identity_for_agent,
            get_all_diap_identities_for_agent,
            // Agent commands
            execute_agent_task,
            execute_ai_conversation,
            get_agent_config,
            update_agent_config,
            test_api_connection,
            get_available_providers,
            health_check,
            // Agent Hook commands
            agent_hook_inject,
            agent_hook_inject_high_priority,
            agent_hook_pause,
            agent_hook_resume,
            agent_hook_cancel,
            agent_hook_get_status,
            agent_hook_get_history,
            agent_hook_get_pending,
            agent_hook_clear_queue,
            agent_hook_create,
            // Task Queue commands
            add_task,
            get_next_task,
            update_task_status,
            set_task_result,
            list_tasks,
            get_task_stats,
            get_task_by_id,
            // Autonomous Agent commands
            get_autonomous_agent_state,
            save_autonomous_agent_state,
            // Autonomous Loop commands
            start_autonomous_loop,
            stop_autonomous_loop,
            pause_autonomous_loop,
            resume_autonomous_loop,
            get_autonomous_loop_state,
            add_autonomous_task,
            // Skill Auto Selector commands
            analyze_and_select_tools,
            generate_execution_plan,
            get_available_tools,
            update_auto_select_config,
            get_auto_select_config,
            // Autonomous Executor commands
            autonomous_execute_plan,
            autonomous_execute_tool,
            autonomous_get_history,
            clear_execution_history,
            update_executor_config,
            get_executor_config,
            handle_chat_message,
            update_respond_config,
            get_respond_config,
            // Bot Gateway commands
            bot_gateway::commands::get_bot_gateway_config,
            bot_gateway::commands::update_bot_gateway_config,
            bot_gateway::commands::test_platform_connection,
            bot_gateway::commands::get_bot_gateway_status,
            bot_gateway::commands::toggle_bot_gateway,
            bot_gateway::commands::get_bot_gateway_logs,
            // Heartbeat commands
            start_heartbeat,
            stop_heartbeat,
            trigger_heartbeat_now,
            get_heartbeat_state,
            get_heartbeat_config,
            update_heartbeat_config,
            initialize_heartbeat,
            is_heartbeat_enabled,
            get_heartbeat_file_content,
            write_heartbeat_file,
            clear_heartbeat_file,
            // Cron commands
            cron::commands::start_cron_scheduler,
            cron::commands::stop_cron_scheduler,
            cron::commands::pause_cron_scheduler,
            cron::commands::resume_cron_scheduler,
            cron::commands::get_cron_scheduler_state,
            cron::commands::add_cron_job,
            cron::commands::remove_cron_job,
            cron::commands::list_cron_jobs,
            cron::commands::run_cron_job_now,
            cron::commands::get_cron_job_history,
            cron::commands::get_cron_config,
            cron::commands::update_cron_config,
            cron::commands::clear_cron_job_history,
            cron::commands::get_cron_config_path,
            cron::commands::create_default_cron_config,
            cron::commands::toggle_cron_job,
            // Agent Document commands
            get_agent_documents_path,
            ensure_agent_document,
        ])
        
        // Autonomous Loop state
        .manage(std::sync::Arc::new(tokio::sync::Mutex::new(
            crate::autonomous_loop::AutonomousLoop::new(
                std::sync::Arc::new(tokio::sync::Mutex::new(
                    match crate::tools::task_queue::TaskQueueManager::new(None) {
                        Ok(manager) => manager,
                        Err(e) => {
                            eprintln!("Failed to create task queue manager: {}", e);
                            crate::tools::task_queue::TaskQueueManager::new(Some(
                                std::path::PathBuf::from("/tmp/alou/tasks")
                            )).unwrap_or_else(|_| {
                                panic!("Cannot create TaskQueueManager")
                            })
                        }
                    }
                )),
                std::sync::Arc::new(tokio::sync::Mutex::new(crate::tools::ToolRegistry::new()))
            )
        )))
        .setup(|app| {
            // 初始化 DIAP 文件管理器（用于 DIAP 身份文件存储）
            crate::diap_file_manager::init_app_handle(app.handle().clone());
            log::info!("[main] DIAP file manager initialized");
            
            // 自动加载所有已有身份到内存
            crate::diap_file_manager::load_all_identities_to_memory(app.handle());
            log::info!("[main] DIAP identities loaded from files");
            
            // Set window title
            if let Some(window) = app.get_webview_window("main") {
                window.set_title("Alou").unwrap();
                
                // Create menu with developer tools
                let menu = Menu::new(app)?;
                
                // Add standard edit menu (macOS)
                #[cfg(target_os = "macos")]
                {
                    let edit_menu = Submenu::new(app, "Edit", true)?;
                    edit_menu.append(&PredefinedMenuItem::undo(app, None)?)?;
                    edit_menu.append(&PredefinedMenuItem::redo(app, None)?)?;
                    edit_menu.append(&PredefinedMenuItem::separator(app)?)?;
                    edit_menu.append(&PredefinedMenuItem::cut(app, None)?)?;
                    edit_menu.append(&PredefinedMenuItem::copy(app, None)?)?;
                    edit_menu.append(&PredefinedMenuItem::paste(app, None)?)?;
                    edit_menu.append(&PredefinedMenuItem::select_all(app, None)?)?;
                    menu.append(&edit_menu)?;
                }
                
                // Add View menu with developer tools
                let view_menu = Submenu::new(app, "View", true)?;
                let dev_tools_item = MenuItem::with_id(
                    app,
                    "devtools",
                    "Toggle Developer Tools",
                    true,
                    Some("F12"),
                )?;
                view_menu.append(&dev_tools_item)?;
                view_menu.append(&PredefinedMenuItem::separator(app)?)?;
                menu.append(&view_menu)?;
                
                // Add Window menu
                let window_menu = Submenu::new(app, "Window", true)?;
                window_menu.append(&PredefinedMenuItem::minimize(app, None)?)?;
                window_menu.append(&PredefinedMenuItem::separator(app)?)?;
                window_menu.append(&PredefinedMenuItem::close_window(app, None)?)?;
                menu.append(&window_menu)?;
                
                // Set the menu
                app.set_menu(menu)?;

                // Handle menu events - toggle developer tools
                // NOTE: In Tauri v2, devtools are controlled via CLI flags, not programmatically.
                // The following methods do NOT exist in Tauri v2:
                // - window.open_devtools()
                // - window.close_devtools()
                // - window.is_devtools_open()
                // 
                // To enable devtools in development, run the app with:
                //   cargo tauri dev --debug
                // Or set the following in tauri.conf.json:
                //   "tauri": { "security": { "devtools": true } }
                // 
                // This menu item is kept for documentation purposes. Users should use
                // browser-style devtools (F12) when running in development mode.
                let _window_handle = window.clone();
                app.on_menu_event(move |_app_handle, event| {
                    if event.id() == "devtools" {
                        log::info!("DevTools menu clicked. In Tauri v2, devtools are controlled via CLI flags or tauri.conf.json settings, not programmatically.");
                        log::info!("To enable devtools, run: cargo tauri dev --debug");
                    }
                });
            }

            // Tools are now registered synchronously in ToolBridge::new_sync
            println!("ℹ️  Tool system initialized with all tools registered");

            // Initialize Bot Gateway Manager
            let bot_gateway_manager = bot_gateway::BotGatewayManager::new(
                bot_gateway::config::BotGatewayConfig::default(),
                Arc::new(tokio::sync::Mutex::new(crate::tools::executor::ToolExecutionManager::new(crate::tools::ToolConfig::default()))),
            );
            app.manage(bot_gateway_manager);

            // Start wallet sync server on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_wallet_sync_server(app_handle).await {
                    eprintln!("Failed to start wallet sync server: {}", e);
                }
            });

            // Start tool API server on startup (for CLI)
            let app_handle_tools = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match start_tool_api_server(app_handle_tools).await {
                    Ok(port) => {
                        println!("Tool API server for CLI started on port {}", port);
                        // Write port to config file for CLI to read
                        let config_dir = dirs::config_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                            .join("alou");
                        let _ = std::fs::create_dir_all(&config_dir);
                        let port_file = config_dir.join("tool_api_port");
                        let _ = std::fs::write(port_file, port.to_string());
                    }
                    Err(e) => eprintln!("Failed to start tool API server: {}", e),
                }
            });

            // Start media API server on startup (for frontend media generation)
            let app_handle_media = app.handle().clone();
            let runtime_state_for_media = app.state::<crate::agent_runtime::AgentRuntimeState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                match start_media_api_server(runtime_state_for_media).await {
                    Ok(port) => {
                        println!("Media API server started on port {}", port);
                        // Write port to config file for frontend to read
                        let config_dir = dirs::config_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                            .join("alou");
                        let _ = std::fs::create_dir_all(&config_dir);
                        let port_file = config_dir.join("media_api_port");
                        let _ = std::fs::write(port_file, port.to_string());
                    }
                    Err(e) => eprintln!("Failed to start media API server: {}", e),
                }
            });

            // Ensure Kubo binary exists and auto-start IPFS
            let ipfs_app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                bootstrap_ipfs(ipfs_app_handle).await;
            });

            // Start workflow event listener
            let workflow_app_handle = app.handle().clone();
            let workflow_executor = app.state::<AsyncWorkflowExecutor>().inner().clone();
            start_workflow_event_listener(workflow_app_handle, workflow_executor);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
