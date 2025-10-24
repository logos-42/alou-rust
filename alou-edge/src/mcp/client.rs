use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::utils::error::{AloudError, Result};

trait StatusCodeExt {
    fn is_success(&self) -> bool;
}

impl StatusCodeExt for u16 {
    fn is_success(&self) -> bool {
        (200..300).contains(self)
    }
}

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP tool definition from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP client configuration
#[derive(Debug, Clone)]
pub struct McpClientConfig {
    /// MCP server endpoint URL
    pub server_url: String,
    
    /// Request timeout in milliseconds
    #[allow(dead_code)]
    pub timeout_ms: u64,
    
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// MCP Client for communicating with MCP servers
pub struct McpClient {
    config: McpClientConfig,
    request_id: Arc<AtomicU64>,
    cached_tools: Arc<crate::utils::async_lock::RwLock<Option<Vec<McpToolDefinition>>>>,
}

impl McpClient {
    /// Create a new MCP client with the given configuration
    pub fn new(config: McpClientConfig) -> Self {
        Self {
            config,
            request_id: Arc::new(AtomicU64::new(1)),
            cached_tools: Arc::new(crate::utils::async_lock::RwLock::new(None)),
        }
    }
    
    /// Create a new MCP client with server URL
    #[allow(dead_code)]
    pub fn with_url(server_url: String) -> Self {
        Self::new(McpClientConfig {
            server_url,
            ..Default::default()
        })
    }
    
    /// Get the next request ID
    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Send a JSON-RPC request to the MCP server
    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: method.to_string(),
            params,
        };
        
        // Retry logic
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts <= self.config.max_retries {
            match self.send_request_once(&request).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    
                    if attempts <= self.config.max_retries {
                        // Wait before retry
                        #[cfg(target_arch = "wasm32")]
                        {
                            use wasm_bindgen_futures::JsFuture;
                            use web_sys::js_sys;
                            let promise = js_sys::Promise::new(&mut |resolve, _| {
                                web_sys::window()
                                    .unwrap()
                                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                                        &resolve,
                                        self.config.retry_delay_ms as i32,
                                    )
                                    .unwrap();
                            });
                            let _ = JsFuture::from(promise).await;
                        }
                        
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                self.config.retry_delay_ms,
                            ))
                            .await;
                        }
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            AloudError::McpError("Max retries exceeded".to_string())
        }))
    }
    
    /// Send a single request without retry
    async fn send_request_once(&self, request: &JsonRpcRequest) -> Result<Value> {
        // In WASM environment, use worker's fetch
        #[cfg(target_arch = "wasm32")]
        {
            use worker::*;
            
            let headers = {
                let h = Headers::new();
                h.set("Content-Type", "application/json")?;
                h
            };
            
            let init = {
                let mut i = RequestInit::new();
                i.with_method(Method::Post)
                    .with_headers(headers)
                    .with_body(Some(serde_json::to_string(request)?.into()));
                i
            };
            
            let req = Request::new_with_init(&self.config.server_url, &init)?;
            let mut resp = Fetch::Request(req).send().await?;
            
            if !resp.status_code().is_success() {
                return Err(AloudError::McpError(format!(
                    "HTTP error: {}",
                    resp.status_code()
                )));
            }
            
            let response: JsonRpcResponse = resp.json().await?;
            
            if let Some(error) = response.error {
                return Err(AloudError::McpError(format!(
                    "JSON-RPC error {}: {}",
                    error.code, error.message
                )));
            }
            
            response.result.ok_or_else(|| {
                AloudError::McpError("No result in response".to_string())
            })
        }
        
        // In non-WASM environment, use reqwest
        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = reqwest::Client::new();
            let resp = client
                .post(&self.config.server_url)
                .json(request)
                .timeout(std::time::Duration::from_millis(self.config.timeout_ms))
                .send()
                .await
                .map_err(|e| AloudError::McpError(e.to_string()))?;
            
            if !resp.status().is_success() {
                return Err(AloudError::McpError(format!(
                    "HTTP error: {}",
                    resp.status()
                )));
            }
            
            let response: JsonRpcResponse = resp
                .json()
                .await
                .map_err(|e| AloudError::McpError(e.to_string()))?;
            
            if let Some(error) = response.error {
                return Err(AloudError::McpError(format!(
                    "JSON-RPC error {}: {}",
                    error.code, error.message
                )));
            }
            
            response.result.ok_or_else(|| {
                AloudError::McpError("No result in response".to_string())
            })
        }
    }
    
    /// Initialize connection to MCP server
    pub async fn initialize(&self) -> Result<Value> {
        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "alou-edge",
                "version": "0.1.0"
            }
        });
        
        self.send_request("initialize", Some(params)).await
    }
    
    /// List available tools from MCP server
    pub async fn list_tools(&self) -> Result<Vec<McpToolDefinition>> {
        // Check cache first
        {
            let cache = self.cached_tools.read().await;
            if let Some(tools) = cache.as_ref() {
                return Ok(tools.clone());
            }
        }
        
        // Fetch from server
        let result = self.send_request("tools/list", None).await?;
        
        let tools: Vec<McpToolDefinition> = serde_json::from_value(
            result.get("tools").cloned().unwrap_or(json!([]))
        ).map_err(|e| AloudError::McpError(format!("Failed to parse tools: {}", e)))?;
        
        // Update cache
        {
            let mut cache = self.cached_tools.write().await;
            *cache = Some(tools.clone());
        }
        
        Ok(tools)
    }
    
    /// Call a tool on the MCP server
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value> {
        let params = json!({
            "name": name,
            "arguments": arguments
        });
        
        let result = self.send_request("tools/call", Some(params)).await?;
        
        // Extract content from result
        if let Some(content) = result.get("content") {
            if let Some(array) = content.as_array() {
                if let Some(first) = array.first() {
                    if let Some(text) = first.get("text") {
                        return Ok(text.clone());
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Clear the tools cache
    #[allow(dead_code)]
    pub async fn clear_cache(&self) {
        let mut cache = self.cached_tools.write().await;
        *cache = None;
    }
    
    /// Check if the client is configured
    #[allow(dead_code)]
    pub fn is_configured(&self) -> bool {
        !self.config.server_url.is_empty()
    }
    
    /// Batch call multiple tools in a single request (optimization)
    #[allow(dead_code)]
    pub async fn call_tools_batch(&self, calls: Vec<(&str, Value)>) -> Result<Vec<Value>> {
        let mut results = Vec::with_capacity(calls.len());
        
        // For now, execute sequentially
        // Future optimization: implement true batch protocol if MCP server supports it
        for (name, arguments) in calls {
            let result = self.call_tool(name, arguments).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Get connection statistics for monitoring
    #[allow(dead_code)]
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("request_count".to_string(), self.request_id.load(Ordering::SeqCst));
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let config = McpClientConfig {
            server_url: "http://localhost:3000".to_string(),
            ..Default::default()
        };
        
        let client = McpClient::new(config);
        assert!(client.is_configured());
    }
    
    #[test]
    fn test_request_id_increment() {
        let client = McpClient::with_url("http://localhost:3000".to_string());
        
        let id1 = client.next_request_id();
        let id2 = client.next_request_id();
        let id3 = client.next_request_id();
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }
    
    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "tools/list".to_string(),
            params: None,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/list\""));
    }
    
    #[test]
    fn test_json_rpc_response_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"tools": []}
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }
    
    #[test]
    fn test_json_rpc_error_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, 1);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }
}
