//! Desktop Tools Adapter for alou_code Kernel
//!
//! This crate provides adapters that wrap desktop tools (from `crate::tools`)
//! and register them with alou_code's GlobalToolRegistry, making them available
//! through the embedded alou_code kernel.
//!
//! Desktop tools are adapted to implement `alou_code_tools::ToolSpec` interface
//! so they can be registered as runtime tools in the alou_code tool registry.

use alou_code_api::types::ToolDefinition;
use alou_code_runtime::permissions::PermissionMode;
use alou_code_tools::{GlobalToolRegistry, RuntimeToolDefinition, ToolSpec};
use serde_json::{json, Value};
use std::collections::BTreeMap;

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

pub use filesystem::FilesystemToolAdapter;
pub use bash::BashToolAdapter;
pub use search::SearchToolAdapter;
pub use network::NetworkToolAdapter;
pub use system::SystemToolAdapter;
pub use todolist::TodoListToolAdapter;
pub use plan::PlanToolAdapter;
pub use git_helper::GitHelperToolAdapter;
pub use browser_tool::BrowserToolAdapter;
pub use wallet_manager::WalletManagerToolAdapter;
pub use agent_wallet::AgentWalletToolAdapter;
pub use query_blockchain::QueryBlockchainToolAdapter;
pub use build_transaction::BuildTransactionToolAdapter;
pub use broadcast_transaction::BroadcastTransactionToolAdapter;
pub use polymarket::PolymarketToolAdapter;
pub use media_tools::MediaToolsAdapter;
pub use self_repair_tool::SelfRepairToolAdapter;
pub use agent_creator::AgentCreatorToolAdapter;
pub use group_coordinator::GroupCoordinatorToolAdapter;
pub use tool_creation::ToolCreationToolAdapter;
pub use agent_skills::AgentSkillsToolAdapter;
pub use spec_tool::SpecToolAdapter;
pub use skill_auto_selector::SkillAutoSelectorToolAdapter;
pub use autonomous_executor::AutonomousExecutorToolAdapter;

pub trait DesktopToolSpec: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn required_permission(&self) -> PermissionMode;
    fn execute(&self, input: &Value) -> Result<String, String>;
}

pub struct DesktopToolAdapter<T: DesktopToolSpec> {
    tool: T,
}

impl<T: DesktopToolSpec> DesktopToolAdapter<T> {
    pub fn new(tool: T) -> Self {
        Self { tool }
    }

    pub fn to_runtime_tool(&self) -> RuntimeToolDefinition {
        RuntimeToolDefinition {
            name: self.tool.name().to_string(),
            description: Some(self.tool.description().to_string()),
            input_schema: self.tool.input_schema(),
            required_permission: self.tool.required_permission(),
        }
    }
}

impl<T: DesktopToolSpec + 'static> alou_code_tools::ToolSpec for DesktopToolAdapter<T> {
    fn name(&self) -> &str {
        self.tool.name()
    }

    fn description(&self) -> &str {
        self.tool.description()
    }

    fn input_schema(&self) -> Value {
        self.tool.input_schema()
    }

    fn required_permission(&self) -> PermissionMode {
        self.tool.required_permission()
    }
}

pub fn register_all_desktop_tools(
    registry: GlobalToolRegistry,
) -> Result<GlobalToolRegistry, String> {
    let mut registry = registry;

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

    for (name, description, schema, permission, executor) in desktop_tools {
        let runtime_tool = RuntimeToolDefinition {
            name: name.clone(),
            description: Some(description),
            input_schema: schema,
            required_permission: permission,
        };

        registry = registry.with_runtime_tools(vec![runtime_tool]).map_err(|e| {
            format!("Failed to register desktop tool {}: {}", name, e)
        })?;
    }

    Ok(registry)
}

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
