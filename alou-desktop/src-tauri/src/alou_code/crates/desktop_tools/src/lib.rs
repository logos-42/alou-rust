//! Desktop Tools Adapter for alou_code Kernel
//!
//! This crate provides adapters that wrap desktop tools (from `crate::tools`)
//! and register them with alou_code's GlobalToolRegistry, making them available
//! through the embedded alou_code kernel.

use alou_code_api::ToolDefinition;
use alou_code_runtime::PermissionMode;
use alou_code_tools::{GlobalToolRegistry, InProcessPluginTool, RuntimeToolDefinition};
use serde_json::Value;

pub mod filesystem;
pub mod bash;
pub mod search;
pub mod network;
pub mod system;
pub mod todolist;
pub mod plan;
pub mod git_helper;
pub mod browser_tool;
pub mod wallet_manager;
pub mod agent_wallet;
pub mod query_blockchain;
pub mod build_transaction;
pub mod broadcast_transaction;
pub mod polymarket;
pub mod media_tools;
pub mod self_repair_tool;
pub mod agent_creator;
pub mod group_coordinator;
pub mod tool_creation;
pub mod agent_skills;
pub mod spec_tool;
pub mod skill_auto_selector;
pub mod autonomous_executor;

pub fn get_desktop_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        filesystem::tool_definition(),
        bash::tool_definition(),
        search::tool_definition(),
        network::tool_definition(),
        system::tool_definition(),
        todolist::tool_definition(),
        plan::tool_definition(),
        git_helper::tool_definition(),
        browser_tool::tool_definition(),
        wallet_manager::tool_definition(),
        agent_wallet::tool_definition(),
        query_blockchain::tool_definition(),
        build_transaction::tool_definition(),
        broadcast_transaction::tool_definition(),
        polymarket::tool_definition(),
        media_tools::tool_definition(),
        self_repair_tool::tool_definition(),
        agent_creator::tool_definition(),
        group_coordinator::tool_definition(),
        tool_creation::tool_definition(),
        agent_skills::tool_definition(),
        spec_tool::tool_definition(),
        skill_auto_selector::tool_definition(),
        autonomous_executor::tool_definition(),
    ]
}

pub fn get_in_process_tools() -> Vec<InProcessPluginTool> {
    let desktop_tools: Vec<(
        String,
        String,
        Value,
        PermissionMode,
        Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
    )> = vec![
        filesystem::tool_spec(),
        bash::tool_spec(),
        search::tool_spec(),
        network::tool_spec(),
        system::tool_spec(),
        todolist::tool_spec(),
        plan::tool_spec(),
        git_helper::tool_spec(),
        browser_tool::tool_spec(),
        wallet_manager::tool_spec(),
        agent_wallet::tool_spec(),
        query_blockchain::tool_spec(),
        build_transaction::tool_spec(),
        broadcast_transaction::tool_spec(),
        polymarket::tool_spec(),
        media_tools::tool_spec(),
        self_repair_tool::tool_spec(),
        agent_creator::tool_spec(),
        group_coordinator::tool_spec(),
        tool_creation::tool_spec(),
        agent_skills::tool_spec(),
        spec_tool::tool_spec(),
        skill_auto_selector::tool_spec(),
        autonomous_executor::tool_spec(),
    ];

    desktop_tools
        .into_iter()
        .map(|(name, description, input_schema, required_permission, executor)| {
            InProcessPluginTool::new(
                name,
                Some(description),
                input_schema,
                required_permission.as_str().to_string(),
                executor,
            )
        })
        .collect()
}
