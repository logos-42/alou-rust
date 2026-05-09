// ============================================
// Tools Integration Bridge
// ============================================
//
// This module provides integration with alou-code's tools crate,
// which contains 40+ built-in tools including:
// - Bash, ReadFile, WriteFile, EditFile
// - GrepSearch, GlobSearch
// - WebSearch, WebFetch
// - Agent, Skill, TodoWrite
// - Polymarket tools
// - LSP tools
// - Task/Worker tools
//
// The bridge allows:
// - Access to alou-code tool definitions
// - Tool execution via runtime
// - Permission enforcement
// - Plugin tool support

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use tools::{
    GlobalToolRegistry, ToolRegistry, ToolSpec, ToolSource,
    mvp_tool_specs, execute_tool_with_enforcer,
};
use runtime::{
    PermissionEnforcer, PermissionMode,
    ToolError,
};
use api::ToolDefinition;

use crate::error::Error;

/// Bridge to alou-code tools system
pub struct AlouToolsBridge {
    registry: GlobalToolRegistry,
    enforcer: Option<PermissionEnforcer>,
}

impl AlouToolsBridge {
    /// Create a new tools bridge with default registry
    pub fn new() -> Self {
        Self {
            registry: GlobalToolRegistry::builtin(),
            enforcer: None,
        }
    }

    /// Create with custom permission enforcer
    pub fn with_enforcer(enforcer: PermissionEnforcer) -> Self {
        let mut registry = GlobalToolRegistry::builtin();
        registry.set_enforcer(enforcer.clone());
        Self {
            registry,
            enforcer: Some(enforcer),
        }
    }

    /// Get all tool definitions for API exposure
    pub fn get_tool_definitions(&self, allowed_tools: Option<BTreeSet<String>>) -> Vec<ToolDefinition> {
        self.registry.definitions(allowed_tools.as_ref())
    }

    /// Get MVP tool specs (built-in tools)
    pub fn mvp_tool_specs() -> Vec<ToolSpec> {
        mvp_tool_specs()
    }

    /// Execute a tool by name with input
    pub fn execute_tool(&self, name: &str, input: &Value) -> Result<String, Error> {
        self.registry
            .execute(name, input)
            .map_err(|e| Error::Other(format!("Tool execution failed: {}", e)))
    }

    /// Execute tool with permission enforcement
    pub fn execute_with_permission(
        &self,
        name: &str,
        input: &Value,
    ) -> Result<String, Error> {
        if let Some(ref enforcer) = self.enforcer {
            execute_tool_with_enforcer(Some(enforcer), name, input)
                .map_err(|e| Error::Other(format!("Permission denied: {}", e)))
        } else {
            self.execute_tool(name, input)
        }
    }

    /// Check if a tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.has_runtime_tool(name)
    }

    /// Get tool specification by name
    pub fn get_tool_spec(&self, name: &str) -> Option<&'static ToolSpec> {
        mvp_tool_specs().iter().find(|spec| spec.name == name)
    }

    /// Normalize allowed tools from comma-separated string
    pub fn normalize_allowed_tools(
        &self,
        values: &[String],
    ) -> Result<Option<BTreeSet<String>>, String> {
        self.registry.normalize_allowed_tools(values)
    }

    /// Search tools by query
    pub fn search_tools(
        &self,
        query: &str,
        max_results: usize,
    ) -> tools::ToolSearchOutput {
        self.registry.search(query, max_results, None, None)
    }

    /// Get permission specs for tools
    pub fn permission_specs(
        &self,
        allowed_tools: Option<&BTreeSet<String>>,
    ) -> Result<Vec<(String, PermissionMode)>, String> {
        self.registry.permission_specs(allowed_tools)
    }

    /// Convert tool specs to JSON schema
    pub fn tool_specs_to_json() -> Vec<Value> {
        mvp_tool_specs()
            .iter()
            .map(|spec| {
                serde_json::json!({
                    "name": spec.name,
                    "description": spec.description,
                    "input_schema": spec.input_schema,
                    "required_permission": format!("{:?}", spec.required_permission),
                })
            })
            .collect()
    }
}

impl Default for AlouToolsBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool execution context
#[derive(Debug, Clone)]
pub struct ToolExecutionContext {
    /// Current working directory
    pub cwd: String,
    /// Workspace directories
    pub workspace_dirs: Vec<String>,
    /// Environment variables
    pub env: BTreeMap<String, String>,
    /// Allow dangerous operations
    pub danger_full_access: bool,
}

impl Default for ToolExecutionContext {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            workspace_dirs: vec![],
            env: std::env::vars()
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
            danger_full_access: false,
        }
    }
}

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// Tool name
    pub tool: String,
    /// Success status
    pub success: bool,
    /// Output or error message
    pub result: Value,
    /// Execution duration in ms
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions() {
        let bridge = AlouToolsBridge::new();
        let defs = bridge.get_tool_definitions(None);
        assert!(!defs.is_empty());
    }

    #[test]
    fn test_has_tool() {
        let bridge = AlouToolsBridge::new();
        assert!(bridge.has_tool("bash"));
        assert!(bridge.has_tool("read_file"));
        assert!(!bridge.has_tool("nonexistent_tool"));
    }

    #[test]
    fn test_mvp_tool_specs() {
        let specs = AlouToolsBridge::mvp_tool_specs();
        let names: Vec<&str> = specs.iter().map(|s| s.name).collect();
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"write_file"));
    }

    #[test]
    fn test_tool_specs_to_json() {
        let json = AlouToolsBridge::tool_specs_to_json();
        assert!(!json.is_empty());
        assert!(json[0].get("name").is_some());
    }
}
