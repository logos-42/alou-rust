//! Desktop Tools Adapter for alou_code Kernel
//!
//! This crate provides adapters that wrap desktop-specific tools
//! and register them with alou_code's GlobalToolRegistry.

use alou_code_api::ToolDefinition;
use alou_code_runtime::PermissionMode;
use alou_code_tools::{GlobalToolRegistry, InProcessPluginTool};
use serde_json::Value;

// Critical desktop tools
pub mod filesystem;
pub mod search;
pub mod bash;
pub mod network;
pub mod system;

pub fn get_desktop_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        filesystem::tool_definition(),
        search::tool_definition(),
        bash::tool_definition(),
        network::tool_definition(),
        system::tool_definition(),
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
        search::tool_spec(),
        bash::tool_spec(),
        network::tool_spec(),
        system::tool_spec(),
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