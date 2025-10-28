use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::agent::context::AgentContext;
use crate::mcp::client::McpClient;
use crate::mcp::registry::McpTool;
use crate::utils::error::Result;

/// Proxy tool that forwards calls to an external MCP server
pub struct ProxyTool {
    name: String,
    description: String,
    input_schema: Value,
    client: Arc<McpClient>,
}

impl ProxyTool {
    /// Create a new proxy tool
    pub fn new(
        name: String,
        description: String,
        input_schema: Value,
        client: Arc<McpClient>,
    ) -> Self {
        Self {
            name,
            description,
            input_schema,
            client,
        }
    }
}

#[async_trait(?Send)]
impl McpTool for ProxyTool {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }
    
    async fn execute(&self, args: Value, _context: &AgentContext) -> Result<Value> {
        // Forward the call to the external MCP server
        self.client.call_tool(&self.name, args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_proxy_tool_creation() {
        let client = Arc::new(McpClient::with_url("http://localhost:3000".to_string()));
        
        let tool = ProxyTool::new(
            "test_tool".to_string(),
            "A test tool".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            }),
            client,
        );
        
        assert_eq!(tool.name(), "test_tool");
        assert_eq!(tool.description(), "A test tool");
    }
}
