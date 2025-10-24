use async_trait::async_trait;
use serde_json::{json, Value};

use crate::agent::context::AgentContext;
use crate::mcp::registry::McpTool;
use crate::utils::error::Result;

/// Echo tool for testing - simply echoes back the input
pub struct EchoTool;

#[async_trait(?Send)]
impl McpTool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }
    
    fn description(&self) -> &str {
        "Echo tool that returns the input message. Useful for testing the MCP system."
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The message to echo back"
                }
            },
            "required": ["message"]
        })
    }
    
    async fn execute(&self, args: Value, context: &AgentContext) -> Result<Value> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        Ok(json!({
            "echo": message,
            "session_id": context.session_id,
            "wallet_address": context.wallet_address,
            "chain": context.chain
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_echo_tool() {
        let tool = EchoTool;
        let context = AgentContext::new("test_session".to_string());
        
        let result = tool
            .execute(json!({ "message": "hello world" }), &context)
            .await
            .unwrap();
        
        assert_eq!(
            result.get("echo").and_then(|v| v.as_str()),
            Some("hello world")
        );
        assert_eq!(
            result.get("session_id").and_then(|v| v.as_str()),
            Some("test_session")
        );
    }
}
