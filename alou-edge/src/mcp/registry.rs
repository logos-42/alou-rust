use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::agent::context::AgentContext;
use crate::utils::error::Result;

/// MCP Tool trait that all tools must implement
#[async_trait(?Send)]
pub trait McpTool {
    /// Get the tool name
    fn name(&self) -> &str;
    
    /// Get the tool description
    fn description(&self) -> &str;
    
    /// Get the JSON schema for tool input parameters
    fn input_schema(&self) -> Value;
    
    /// Execute the tool with given arguments and context
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value>;
}

/// Tool information for listing available tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    
    /// Tool description
    pub description: String,
    
    /// JSON schema for input parameters
    pub input_schema: Value,
}

/// MCP Tool Registry for managing available tools
pub struct McpRegistry {
    tools: HashMap<String, Arc<dyn McpTool>>,
}

impl McpRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    /// Register a new tool
    pub fn register(&mut self, tool: Arc<dyn McpTool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }
    
    /// Get a list of all registered tools
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|tool| ToolInfo {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
            })
            .collect()
    }
    
    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn McpTool>> {
        self.tools.get(name).cloned()
    }
    
    /// Check if a tool exists
    #[allow(dead_code)]
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
    
    /// Get the number of registered tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    struct MockTool {
        name: String,
    }
    
    #[async_trait(?Send)]
    impl McpTool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            "A mock tool for testing"
        }
        
        fn input_schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": {
                    "test": { "type": "string" }
                }
            })
        }
        
        async fn execute(&self, _args: Value, _context: &AgentContext) -> Result<Value> {
            Ok(json!({"result": "success"}))
        }
    }
    
    #[test]
    fn test_registry_register_and_get() {
        let mut registry = McpRegistry::new();
        let tool = Arc::new(MockTool {
            name: "test_tool".to_string(),
        });
        
        registry.register(tool);
        
        assert_eq!(registry.tool_count(), 1);
        assert!(registry.has_tool("test_tool"));
        assert!(registry.get_tool("test_tool").is_some());
    }
    
    #[test]
    fn test_registry_list_tools() {
        let mut registry = McpRegistry::new();
        
        registry.register(Arc::new(MockTool {
            name: "tool1".to_string(),
        }));
        registry.register(Arc::new(MockTool {
            name: "tool2".to_string(),
        }));
        
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 2);
    }
}
