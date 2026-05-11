//! Tool Adapter
//!
//! Adapts desktop tools to work with alou_code's tool registry and execution model.

use serde_json::{Value, json};
use alou_code_tools::{GlobalToolRegistry, RuntimeToolDefinition, ToolDefinition, PermissionMode};

pub struct ToolAdapter {
    registry: GlobalToolRegistry,
}

impl ToolAdapter {
    pub fn new() -> Self {
        let registry = GlobalToolRegistry::builtin();
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
}

impl Default for ToolAdapter {
    fn default() -> Self {
        Self::new()
    }
}
