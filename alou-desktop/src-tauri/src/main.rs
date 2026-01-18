// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridges;
mod browser;
mod context;
mod diap;
mod ipfs_api;
mod ipfs_commands;
mod ipfs_node;
mod kubo;
mod lsp;
mod memory_manager;  // 新增内存管理模块
mod spec;
mod sync;
mod utils;
mod wallet;
mod workflow;
mod workflow_types;
mod workflow_commands;
mod workflow_ralph_loop;
mod workflow_executor;
mod workflow_events;
mod workflow_history;
mod workflow_monitor;
mod workflow_storage;
mod tools;

use std::path::PathBuf;
use tauri::Manager;

use crate::ipfs_node::{bootstrap_ipfs, IpfsState};
use crate::kubo::download_kubo_binary;
use crate::workflow::WorkflowState;
use crate::workflow_executor::AsyncWorkflowExecutor;
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
use crate::diap::{
    create_local_diap_identity,
    get_local_diap_identity,
    update_local_diap_identity,
    test_ipns_on_public_gateway,
};
use crate::wallet::{verify_wallet_signature, get_testnet_private_key};
use crate::browser::{open_browser, test_ipfs_node_connection};
use crate::sync::{
    read_wallet_sync_data, start_wallet_sync_server, write_wallet_sync_data,
};
use crate::lsp::{execute_lsp, get_supported_languages};
use crate::spec::{execute_spec, get_spec_templates, load_template_content};
use crate::workflow::{
    create_workflow, execute_workflow, get_workflow_status, list_workflows,
    delete_workflow, retry_workflow_step, pause_workflow, resume_workflow,
};
use crate::workflow_executor::{
    get_execution_status, pause_execution, resume_execution, cancel_execution,
    get_execution_logs, get_performance_metrics,
};
use crate::workflow_events::start_workflow_event_listener;
use crate::bridges::{BridgeManager, create_default_bridge_manager};
use crate::tools::initialize_tools;
use crate::context::create_default_context_manager;
use crate::memory_manager::{
    set_memory_item, get_memory_item, remove_memory_item, clear_memory,
    get_memory_keys, get_memory_stats, cleanup_expired_memory,
    cleanup_lru_memory, set_memory_expiration, schedule_weekly_cleanup,
};

// Tool commands
#[tauri::command]
async fn execute_tool(
    tool_id: String,
    args: String,
    timeout: Option<u64>,
    bridge_manager: tauri::State<'_, BridgeManager>,
) -> Result<serde_json::Value, String> {
    println!("[Tauri] Executing tool: {}", tool_id);

    let args_value: serde_json::Value = serde_json::from_str(&args)
        .map_err(|e| format!("Invalid JSON args: {}", e))?;

    // Get tool bridge and execute tool
    let tool_bridge = bridge_manager.tool_bridge();

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
    bridge_manager: tauri::State<'_, BridgeManager>,
) -> Result<serde_json::Value, String> {
    // TODO: Get tool list from registry
    Ok(serde_json::json!({
        "tools": [
            {
                "id": "filesystem",
                "name": "File System",
                "category": "FileSystem",
                "description": "File system operations"
            },
            {
                "id": "search",
                "name": "Search",
                "category": "Search",
                "description": "Text and file search"
            },
            {
                "id": "bash",
                "name": "Bash",
                "category": "Terminal",
                "description": "Terminal command execution"
            },
            {
                "id": "plan",
                "name": "Task Planning",
                "category": "Planning",
                "description": "Task planning and management"
            },
            {
                "id": "skills",
                "name": "Skills",
                "category": "Skills",
                "description": "Extensible skills system"
            }
        ]
    }))
}

#[tauri::command]
async fn cancel_tool_execution(
    execution_id: String,
    bridge_manager: tauri::State<'_, BridgeManager>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement cancellation
    Ok(serde_json::json!({
        "cancelled": true,
        "execution_id": execution_id
    }))
}

#[tauri::command]
async fn get_execution_history(
    limit: Option<usize>,
    bridge_manager: tauri::State<'_, BridgeManager>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement execution history
    Ok(serde_json::json!({
        "history": [],
        "limit": limit.unwrap_or(50)
    }))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(tauri::async_runtime::Mutex::new(IpfsState {
            process: None,
            data_dir: PathBuf::new(),
        }))
        .manage(WorkflowState::default())
        .manage(AsyncWorkflowExecutor::new().expect("Failed to create workflow executor"))
        .manage(create_default_bridge_manager())
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
            // PubSub commands
            ipfs_pubsub_publish,
            ipfs_pubsub_subscribe_once,
            ipfs_pubsub_peers,
            ipfs_pubsub_ls,
            // DIAP commands
            create_local_diap_identity,
            get_local_diap_identity,
            update_local_diap_identity,
            test_ipns_on_public_gateway,
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
            // Tool commands
            execute_tool,
            get_tool_list,
            cancel_tool_execution,
            get_execution_history,
            // Memory management commands
            set_memory_item,
            get_memory_item,
            remove_memory_item,
            clear_memory,
            get_memory_keys,
            get_memory_stats,
            cleanup_expired_memory,
            cleanup_lru_memory,
            set_memory_expiration,
            schedule_weekly_cleanup
        ])
        .setup(|app| {
            // Set window title
            if let Some(window) = app.get_webview_window("main") {
                window.set_title("Alou").unwrap();
            }

            // Initialize tools (synchronous logging for now)
            // Note: In Tauri, we can't modify state directly in setup
            // Tools are registered in the ToolBridge constructor
            println!("ℹ️  Tool system initialized (tools registered in ToolBridge constructor)");

            // Start wallet sync server on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_wallet_sync_server(app_handle).await {
                    eprintln!("Failed to start wallet sync server: {}", e);
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