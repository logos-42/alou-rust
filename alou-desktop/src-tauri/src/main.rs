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
mod ipns_verified;  // 新增验证过的IPNS解决方案
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
mod tool_api;  // 工具 API 模块

use std::path::PathBuf;
use tauri::Manager;

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
use crate::tools::initialize_tools;
use crate::tools::task_queue_tool::{initialize_task_queue_tool, add_task, get_next_task, update_task_status, set_task_result, list_tasks, get_task_stats, get_task_by_id};
use crate::prompts::PromptManager;
use crate::context::create_default_context_manager;
use crate::memory_manager::{
    set_memory_item, get_memory_item, remove_memory_item, clear_memory,
    get_memory_keys, get_memory_stats, archive_to_ipfs, pin_cid, unpin_cid,
    garbage_collect, pin_item, unpin_item, set_diap_identity, get_diap_identity,
    remove_diap_identity, get_all_diap_identities, archive_diap_identity_to_ipfs,
    pin_diap_identity, unpin_diap_identity,
};

// Agent commands
use crate::agent::commands::{
    execute_agent_task, execute_ai_conversation, get_agent_config, update_agent_config,
    test_api_connection, get_available_providers, health_check,
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
    bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<BridgeManager>>>,
) -> Result<serde_json::Value, String> {
    println!("[Tauri] Executing tool: {}", tool_id);

    let args_value: serde_json::Value = serde_json::from_str(&args)
        .map_err(|e| format!("Invalid JSON args: {}", e))?;

    // Get tool bridge and execute tool
    let manager = bridge_manager.lock().await;
    let tool_bridge = manager.tool_bridge();

    let request = crate::bridges::ToolCallRequest {
        session_id: "tauri_session".to_string(),
        user_id: None,
        tool_id: tool_id.clone(),
        args: args_value,
        working_directory: std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string())),
        environment: std::env::vars().collect(),
        timeout_seconds: timeout,
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    match tool_bridge.handle_request(request).await {
        Ok(response) => {
            if response.success {
                Ok(serde_json::json!({
                    "success": true,
                    "data": response.result.map(|r| r.data).unwrap_or_else(|| serde_json::json!({"status": "success"})),
                    "execution_time_ms": 100,
                    "output": format!("Tool '{}' executed successfully", tool_id)
                }))
            } else {
                Err(response.error.unwrap_or_else(|| format!("Tool '{}' execution failed", tool_id)))
            }
        }
        Err(e) => Err(format!("Tool bridge error: {}", e))
    }
}

#[tauri::command]
async fn get_tool_list(
    bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<BridgeManager>>>,
) -> Result<serde_json::Value, String> {
    // 从 ToolBridge 获取工具列表
    let manager = bridge_manager.lock().await;
    let tools = manager.tool_bridge().list_tools().await;
    Ok(serde_json::json!({ "tools": tools }))
}

#[tauri::command]
async fn cancel_tool_execution(
    execution_id: String,
    bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<BridgeManager>>>,
) -> Result<serde_json::Value, String> {
    println!("[Tauri] Cancelling tool execution: {}", execution_id);

    let manager = bridge_manager.lock().await;
    let result = manager.tool_bridge().cancel_execution(&execution_id).await;

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
    bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<BridgeManager>>>,
) -> Result<serde_json::Value, String> {
    println!("[Tauri] Getting execution history (limit: {:?})", limit);

    let manager = bridge_manager.lock().await;
    let history = manager.tool_bridge().get_execution_history(limit.unwrap_or(50)).await;

    Ok(serde_json::json!({
        "history": history,
        "limit": limit.unwrap_or(50),
        "count": history.len()
    }))
}

// Agent Skills command
#[tauri::command]
async fn agent_skills(
    action: String,
    skill_name: Option<String>,
    inputs: Option<serde_json::Value>,
    query: Option<String>,
    bridge_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<BridgeManager>>>,
) -> Result<serde_json::Value, String> {
    let args = serde_json::json!({
        "action": action,
        "skill_name": skill_name,
        "inputs": inputs,
        "query": query
    });
    
    let execution_id = format!("agent_skills_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
    
    let manager = bridge_manager.lock().await;
    
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
    
    let result = manager.tool_bridge().handle_request(request).await;
    
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
        .plugin(tauri_plugin_updater::Builder::new().build())
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
        .manage(std::sync::Arc::new(tokio::sync::Mutex::new(create_default_bridge_manager())))
        .manage(std::sync::Arc::new(tokio::sync::Mutex::new(initialize_task_queue_tool().unwrap())))
        .invoke_handler(tauri::generate_handler![
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
            // Agent commands
            execute_agent_task,
            execute_ai_conversation,
            get_agent_config,
            update_agent_config,
            test_api_connection,
            get_available_providers,
            health_check,
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
            // Set window title
            if let Some(window) = app.get_webview_window("main") {
                window.set_title("Alou").unwrap();
            }

            // Tools are now registered synchronously in ToolBridge::new_sync
            println!("ℹ️  Tool system initialized with all tools registered");

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
