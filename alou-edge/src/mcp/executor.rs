use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::context::AgentContext;
use crate::mcp::registry::McpRegistry;
use crate::utils::error::{AloudError, Result};

/// Tool call request from the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    
    /// Name of the tool to call
    pub name: String,
    
    /// Arguments to pass to the tool
    pub args: Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call identifier
    pub id: String,
    
    /// Tool name
    pub name: String,
    
    /// Result data
    pub result: Value,
    
    /// Optional error message
    pub error: Option<String>,
}

/// MCP Executor for handling tool calls
pub struct McpExecutor {
    registry: McpRegistry,
}

impl McpExecutor {
    /// Create a new executor with the given registry
    pub fn new(registry: McpRegistry) -> Self {
        Self { registry }
    }
    
    /// Execute a single tool call
    pub async fn execute(
        &self,
        tool_name: &str,
        args: Value,
        context: &AgentContext,
    ) -> Result<Value> {
        // Get the tool from registry
        let tool = self
            .registry
            .get_tool(tool_name)
            .ok_or_else(|| AloudError::ToolNotFound(tool_name.to_string()))?;
        
        // Validate arguments against schema (basic validation)
        self.validate_args(&args, &tool.input_schema())?;
        
        // Execute the tool
        tool.execute(args, context)
            .await
            .map_err(|e| AloudError::ToolExecutionError(e.to_string()))
    }
    
    /// Execute multiple tool calls in sequence
    pub async fn execute_batch(
        &self,
        calls: Vec<ToolCall>,
        context: &AgentContext,
    ) -> Vec<ToolResult> {
        let mut results = Vec::new();
        
        for call in calls {
            let result = match self.execute(&call.name, call.args.clone(), context).await {
                Ok(value) => ToolResult {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    result: value,
                    error: None,
                },
                Err(e) => ToolResult {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    result: Value::Null,
                    error: Some(e.to_string()),
                },
            };
            
            results.push(result);
        }
        
        results
    }
    
    /// Get list of available tools from registry
    pub fn list_tools(&self) -> Vec<crate::mcp::registry::ToolInfo> {
        self.registry.list_tools()
    }
    
    /// Check if a tool exists
    #[allow(dead_code)]
    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.has_tool(name)
    }
    
    /// Basic validation of arguments against schema
    fn validate_args(&self, args: &Value, schema: &Value) -> Result<()> {
        // Basic validation: check if args is an object when schema expects object
        if let Some(schema_type) = schema.get("type").and_then(|v| v.as_str()) {
            if schema_type == "object" {
                if !args.is_object() {
                    return Err(AloudError::InvalidToolArgs(
                        "Expected object arguments".to_string(),
                    ));
                }
                
                // Check required fields
                if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
                    let args_obj = args.as_object().unwrap();
                    for req_field in required {
                        if let Some(field_name) = req_field.as_str() {
                            if !args_obj.contains_key(field_name) {
                                return Err(AloudError::InvalidToolArgs(format!(
                                    "Missing required field: {}",
                                    field_name
                                )));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::registry::{McpRegistry, McpTool};
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Arc;
    
    struct TestTool;
    
    #[async_trait(?Send)]
    impl McpTool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }
        
        fn description(&self) -> &str {
            "A test tool"
        }
        
        fn input_schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            })
        }
        
        async fn execute(&self, args: Value, _context: &AgentContext) -> Result<Value> {
            let message = args
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Ok(json!({ "echo": message }))
        }
    }
    
    #[tokio::test]
    async fn test_executor_execute() {
        let mut registry = McpRegistry::new();
        registry.register(Arc::new(TestTool));
        
        let executor = McpExecutor::new(registry);
        let context = AgentContext::new("test_session".to_string());
        
        let result = executor
            .execute(
                "test_tool",
                json!({ "message": "hello" }),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result.get("echo").and_then(|v| v.as_str()), Some("hello"));
    }
    
    #[tokio::test]
    async fn test_executor_tool_not_found() {
        let registry = McpRegistry::new();
        let executor = McpExecutor::new(registry);
        let context = AgentContext::new("test_session".to_string());
        
        let result = executor
            .execute("nonexistent", json!({}), &context)
            .await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AloudError::ToolNotFound(_)));
    }
    
    #[tokio::test]
    async fn test_executor_validate_args() {
        let mut registry = McpRegistry::new();
        registry.register(Arc::new(TestTool));
        
        let executor = McpExecutor::new(registry);
        let context = AgentContext::new("test_session".to_string());
        
        // Missing required field
        let result = executor
            .execute("test_tool", json!({}), &context)
            .await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AloudError::InvalidToolArgs(_)));
    }
    
    #[tokio::test]
    async fn test_executor_batch() {
        let mut registry = McpRegistry::new();
        registry.register(Arc::new(TestTool));
        
        let executor = McpExecutor::new(registry);
        let context = AgentContext::new("test_session".to_string());
        
        let calls = vec![
            ToolCall {
                id: "call1".to_string(),
                name: "test_tool".to_string(),
                args: json!({ "message": "first" }),
            },
            ToolCall {
                id: "call2".to_string(),
                name: "test_tool".to_string(),
                args: json!({ "message": "second" }),
            },
        ];
        
        let results = executor.execute_batch(calls, &context).await;
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].error, None);
        assert_eq!(results[1].error, None);
    }
}
