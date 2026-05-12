//! Tool Adapter
//!
//! Adapts desktop tools to work with alou_code's tool registry and execution model.

use serde_json::Value;
use alou_code_tools::{GlobalToolRegistry, RuntimeToolDefinition};
use alou_code_api::types::ToolDefinition;
use alou_code_runtime::permissions::PermissionMode;

pub struct ToolAdapter {
    registry: GlobalToolRegistry,
}

impl ToolAdapter {
    pub fn new() -> Self {
        let registry = GlobalToolRegistry::builtin();
        Self { registry }
    }

    pub fn with_desktop_tools() -> Self {
        let registry = GlobalToolRegistry::builtin();
        let desktop_defs = alou_code_desktop_tools::get_desktop_tool_definitions();

        let runtime_tools: Vec<RuntimeToolDefinition> = desktop_defs
            .into_iter()
            .map(|def| RuntimeToolDefinition {
                name: def.name,
                description: def.description,
                input_schema: def.input_schema,
                required_permission: PermissionMode::WorkspaceWrite,
            })
            .collect();

        let registry = registry
            .with_runtime_tools(runtime_tools)
            .expect("Failed to register desktop tools");

        Self { registry }
    }

    pub fn register_desktop_tool(
        &mut self,
        name: String,
        description: String,
        input_schema: Value,
        required_permission: PermissionMode,
    ) -> Result<(), String> {
        let runtime_tool = RuntimeToolDefinition {
            name: name.clone(),
            description: Some(description),
            input_schema: input_schema.clone(),
            required_permission,
        };

        self.registry = self.registry
            .with_runtime_tools(vec![runtime_tool])
            .map_err(|e| format!("Failed to register tool {}: {}", name, e))?;

        Ok(())
    }

    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.registry.definitions(None)
    }

    pub fn get_tool_definition(&self, name: &str) -> Option<ToolDefinition> {
        self.list_tools()
            .into_iter()
            .find(|t| t.name == name)
    }

    pub fn execute_tool(&self, name: &str, input: &Value) -> Result<String, String> {
        self.registry.execute(name, input)
    }

    pub fn registry(&self) -> &GlobalToolRegistry {
        &self.registry
    }

    pub fn list_desktop_tools() -> Vec<ToolDefinition> {
        alou_code_desktop_tools::get_desktop_tool_definitions()
    }
}

impl Default for ToolAdapter {
    fn default() -> Self {
        Self::new()
    }
}