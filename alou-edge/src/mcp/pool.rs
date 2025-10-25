use std::collections::HashMap;
use std::sync::Arc;
use crate::utils::async_lock::RwLock;

use crate::mcp::client::{McpClient, McpClientConfig};
use crate::utils::error::{AloudError, Result};

/// Connection pool for managing MCP client connections
pub struct McpConnectionPool {
    clients: Arc<RwLock<HashMap<String, Arc<McpClient>>>>,
    default_config: McpClientConfig,
}

impl McpConnectionPool {
    /// Create a new connection pool
    pub fn new(default_config: McpClientConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            default_config,
        }
    }
    
    /// Create a connection pool with default configuration
    pub fn with_defaults() -> Self {
        Self::new(McpClientConfig::default())
    }
    
    /// Get or create a client for the given server URL
    pub async fn get_client(&self, server_url: &str) -> Result<Arc<McpClient>> {
        // Check if client already exists
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(server_url) {
                return Ok(client.clone());
            }
        }
        
        // Create new client
        let mut config = self.default_config.clone();
        config.server_url = server_url.to_string();
        
        let client = Arc::new(McpClient::new(config));
        
        // Initialize the client
        client.initialize().await.map_err(|e| {
            AloudError::McpError(format!("Failed to initialize MCP client: {}", e))
        })?;
        
        // Store in pool
        {
            let mut clients = self.clients.write().await;
            clients.insert(server_url.to_string(), client.clone());
        }
        
        Ok(client)
    }
    
    /// Remove a client from the pool
    #[allow(dead_code)]
    pub async fn remove_client(&self, server_url: &str) -> Option<Arc<McpClient>> {
        let mut clients = self.clients.write().await;
        clients.remove(server_url)
    }
    
    /// Clear all clients from the pool
    #[allow(dead_code)]
    pub async fn clear(&self) {
        let mut clients = self.clients.write().await;
        clients.clear();
    }
    
    /// Get the number of active clients
    #[allow(dead_code)]
    pub async fn client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }
    
    /// Check if a client exists for the given server URL
    #[allow(dead_code)]
    pub async fn has_client(&self, server_url: &str) -> bool {
        let clients = self.clients.read().await;
        clients.contains_key(server_url)
    }
    
    /// Get all server URLs in the pool
    pub async fn list_servers(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }
}

impl Default for McpConnectionPool {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pool_creation() {
        let pool = McpConnectionPool::with_defaults();
        assert_eq!(pool.client_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_pool_has_client() {
        let pool = McpConnectionPool::with_defaults();
        assert!(!pool.has_client("http://localhost:3000").await);
    }
    
    #[tokio::test]
    async fn test_pool_list_servers() {
        let pool = McpConnectionPool::with_defaults();
        let servers = pool.list_servers().await;
        assert_eq!(servers.len(), 0);
    }
    
    #[tokio::test]
    async fn test_pool_clear() {
        let pool = McpConnectionPool::with_defaults();
        pool.clear().await;
        assert_eq!(pool.client_count().await, 0);
    }
}
