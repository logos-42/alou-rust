use std::sync::Arc;

use crate::mcp::pool::McpConnectionPool;
use crate::mcp::registry::McpRegistry;
use crate::mcp::tools::ProxyTool;
use crate::utils::error::Result;

/// Bridge between local MCP registry and external MCP servers
pub struct McpBridge {
    pool: Arc<McpConnectionPool>,
    registry: Arc<crate::utils::async_lock::RwLock<McpRegistry>>,
}

impl McpBridge {
    /// Create a new MCP bridge
    pub fn new(pool: Arc<McpConnectionPool>) -> Self {
        Self {
            pool,
            registry: Arc::new(crate::utils::async_lock::RwLock::new(McpRegistry::new())),
        }
    }
    
    /// Create a new MCP bridge with default pool
    #[allow(dead_code)]
    pub fn with_defaults() -> Self {
        Self::new(Arc::new(McpConnectionPool::with_defaults()))
    }
    
    /// Connect to an external MCP server and register its tools
    pub async fn connect_server(&self, server_url: &str) -> Result<usize> {
        // Get or create client
        let client = self.pool.get_client(server_url).await?;
        
        // List tools from the server
        let tools = client.list_tools().await?;
        
        // Register each tool as a proxy
        let mut registry = self.registry.write().await;
        let tool_count = tools.len();
        
        for tool_def in tools {
            let proxy = ProxyTool::new(
                tool_def.name,
                tool_def.description,
                tool_def.input_schema,
                client.clone(),
            );
            
            registry.register(Arc::new(proxy));
        }
        
        Ok(tool_count)
    }
    
    /// Disconnect from an MCP server and remove its tools
    #[allow(dead_code)]
    pub async fn disconnect_server(&self, server_url: &str) -> Result<()> {
        self.pool.remove_client(server_url).await;
        // Note: We don't remove tools from registry as they might be in use
        // In a production system, you'd want to track which tools came from which server
        Ok(())
    }
    
    /// Get the registry for tool execution
    #[allow(dead_code)]
    pub async fn get_registry(&self) -> McpRegistry {
        let _registry = self.registry.read().await;
        // Clone the registry (this is cheap as tools are Arc'd)
        McpRegistry::new() // TODO: Implement Clone for McpRegistry
    }
    
    /// Get a reference to the registry
    #[allow(dead_code)]
    pub fn registry(&self) -> Arc<crate::utils::async_lock::RwLock<McpRegistry>> {
        self.registry.clone()
    }
    
    /// Get the connection pool
    #[allow(dead_code)]
    pub fn pool(&self) -> Arc<McpConnectionPool> {
        self.pool.clone()
    }
    
    /// List all connected servers
    #[allow(dead_code)]
    pub async fn list_servers(&self) -> Vec<String> {
        self.pool.list_servers().await
    }
    
    /// Get the total number of registered tools
    #[allow(dead_code)]
    pub async fn tool_count(&self) -> usize {
        let registry = self.registry.read().await;
        registry.tool_count()
    }
    
    /// Register a local tool (not from an MCP server)
    #[allow(dead_code)]
    pub async fn register_local_tool(&self, tool: Arc<dyn crate::mcp::registry::McpTool>) {
        let mut registry = self.registry.write().await;
        registry.register(tool);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bridge_creation() {
        let bridge = McpBridge::with_defaults();
        assert_eq!(bridge.tool_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_bridge_list_servers() {
        let bridge = McpBridge::with_defaults();
        let servers = bridge.list_servers().await;
        assert_eq!(servers.len(), 0);
    }
}
