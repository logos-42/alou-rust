use crate::types::*;
use crate::mcp_tool::{DiscoveredMcpTool, McpToolFactory};
use crate::types::McpServerConfig;
use crate::tool_registry::ToolRegistry;
use crate::workspace_context::WorkspaceContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use serde_json;
use tracing::{info, warn, error, debug};
use async_trait::async_trait;

/// MCP默认超时时间（毫秒）
pub const MCP_DEFAULT_TIMEOUT_MSEC: u64 = 30 * 1000; // 30秒，用于更快的启动

/// 发现的MCP提示
#[derive(Debug, Clone)]
pub struct DiscoveredMcpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<serde_json::Value>>,
    pub server_name: String,
}

/// MCP客户端
#[derive(Clone)]
pub struct McpClient {
    name: String,
    version: String,
    capabilities: HashMap<String, serde_json::Value>,
}

impl McpClient {
    /// 创建新的MCP客户端
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            capabilities: HashMap::new(),
        }
    }

    /// 注册客户端能力
    pub fn register_capabilities(&mut self, capabilities: HashMap<String, serde_json::Value>) {
        self.capabilities = capabilities;
    }

    /// 连接到MCP服务器
    pub async fn connect(&mut self, transport: Box<dyn McpTransport + Send + Sync>) -> Result<()> {
        // 这里应该实现实际的连接逻辑
        // 由于MCP协议的具体实现比较复杂，这里先提供框架
        info!("连接到MCP服务器: {}", self.name);
        
        // TODO: 实现实际的MCP连接逻辑
        // 1. 建立传输连接
        // 2. 发送初始化消息
        // 3. 协商协议版本
        // 4. 注册客户端能力
        
        Ok(())
    }

    /// 列出可用工具
    pub async fn list_tools(&self) -> Result<Vec<serde_json::Value>> {
        // TODO: 实现实际的工具列表获取
        // 这里应该发送list_tools请求并解析响应
        Ok(vec![])
    }

    /// 调用工具
    pub async fn call_tool(&self, name: &str, arguments: HashMap<String, serde_json::Value>) -> Result<serde_json::Value> {
        // TODO: 实现实际的工具调用
        // 这里应该发送call_tool请求并返回结果
        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": format!("Mock response for tool: {}", name)
            }]
        }))
    }

    /// 关闭连接
    pub fn close(&self) {
        info!("关闭MCP客户端连接: {}", self.name);
        // TODO: 实现实际的连接关闭逻辑
    }

    /// 设置错误处理器
    pub fn set_error_handler<F>(&mut self, handler: F) 
    where 
        F: Fn(String) + Send + Sync + 'static 
    {
        // TODO: 实现错误处理器设置
    }
}

/// MCP传输trait
#[async_trait::async_trait]
pub trait McpTransport {
    async fn send(&self, message: serde_json::Value) -> Result<()>;
    async fn receive(&self) -> Result<serde_json::Value>;
    async fn close(&self) -> Result<()>;
}

/// 标准输入输出传输
pub struct StdioTransport {
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: Option<String>,
}

impl StdioTransport {
    pub fn new(command: String, args: Vec<String>, env: HashMap<String, String>, cwd: Option<String>) -> Self {
        Self {
            command,
            args,
            env,
            cwd,
        }
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send(&self, _message: serde_json::Value) -> Result<()> {
        // TODO: 实现stdio传输的发送逻辑
        Ok(())
    }

    async fn receive(&self) -> Result<serde_json::Value> {
        // TODO: 实现stdio传输的接收逻辑
        Ok(serde_json::json!({}))
    }

    async fn close(&self) -> Result<()> {
        // TODO: 实现stdio传输的关闭逻辑
        Ok(())
    }
}

/// HTTP传输
pub struct HttpTransport {
    url: String,
    headers: HashMap<String, String>,
}

impl HttpTransport {
    pub fn new(url: String, headers: HashMap<String, String>) -> Self {
        Self { url, headers }
    }
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn send(&self, _message: serde_json::Value) -> Result<()> {
        // TODO: 实现HTTP传输的发送逻辑
        Ok(())
    }

    async fn receive(&self) -> Result<serde_json::Value> {
        // TODO: 实现HTTP传输的接收逻辑
        Ok(serde_json::json!({}))
    }

    async fn close(&self) -> Result<()> {
        // TODO: 实现HTTP传输的关闭逻辑
        Ok(())
    }
}

/// MCP客户端管理器
pub struct McpClientManager {
    clients: Arc<RwLock<HashMap<String, Arc<McpClient>>>>,
    server_statuses: Arc<RwLock<HashMap<String, McpServerStatus>>>,
    discovery_state: Arc<RwLock<McpDiscoveryState>>,
}

impl McpClientManager {
    /// 创建新的MCP客户端管理器
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            server_statuses: Arc::new(RwLock::new(HashMap::new())),
            discovery_state: Arc::new(RwLock::new(McpDiscoveryState::NotStarted)),
        }
    }

    /// 发现所有MCP服务器的工具
    pub async fn discover_mcp_tools(
        &self,
        mcp_servers: HashMap<String, McpServerConfig>,
        tool_registry: Arc<ToolRegistry>,
        debug_mode: bool,
        workspace_context: Arc<dyn WorkspaceContext + Send + Sync>,
    ) -> Result<()> {
        let mut discovery_state = self.discovery_state.write().await;
        *discovery_state = McpDiscoveryState::InProgress;
        drop(discovery_state);

        let discovery_promises = mcp_servers.into_iter().map(|(server_name, server_config)| {
            self.connect_and_discover(
                server_name,
                server_config,
                tool_registry.clone(),
                debug_mode,
                workspace_context.clone(),
            )
        });

        // 等待所有发现任务完成
        let results = futures::future::join_all(discovery_promises).await;
        
        // 检查结果
        let mut success_count = 0;
        for result in results {
            if result.is_ok() {
                success_count += 1;
            }
        }

        let mut discovery_state = self.discovery_state.write().await;
        *discovery_state = McpDiscoveryState::Completed;
        
        info!("MCP工具发现完成，成功连接 {} 个服务器", success_count);
        Ok(())
    }

    /// 连接到MCP服务器并发现工具
    async fn connect_and_discover(
        &self,
        server_name: String,
        server_config: McpServerConfig,
        tool_registry: Arc<ToolRegistry>,
        debug_mode: bool,
        _workspace_context: Arc<dyn WorkspaceContext + Send + Sync>,
    ) -> Result<()> {
        info!("连接MCP服务器: {}", server_name);
        
        // 更新服务器状态
        {
            let mut statuses = self.server_statuses.write().await;
            statuses.insert(server_name.clone(), McpServerStatus::Connecting);
        }

        let mut mcp_client = match self.connect_to_mcp_server(&server_name, &server_config, debug_mode).await {
            Ok(client) => client,
            Err(e) => {
                error!("连接MCP服务器 '{}' 失败: {}", server_name, e);
                let mut statuses = self.server_statuses.write().await;
                statuses.insert(server_name, McpServerStatus::Disconnected);
                return Err(e);
            }
        };

        // 设置错误处理器
        let server_name_clone = server_name.clone();
        mcp_client.set_error_handler(move |error| {
            error!("MCP错误 ({}): {}", server_name_clone, error);
        });

        info!("已连接到MCP服务器: {}", server_name);

        // 发现提示和工具
        let _prompts = self.discover_prompts(&server_name, &mcp_client).await?;
        let tools = self.discover_tools(&server_name, &server_config, &mcp_client).await?;
        
        info!("发现 {} 个工具", tools.len());

        // 如果没有发现任何工具，认为发现失败
        if tools.is_empty() {
            return Err(anyhow::anyhow!("服务器上未发现任何工具"));
        }

        // 更新服务器状态为已连接
        {
            let mut statuses = self.server_statuses.write().await;
            statuses.insert(server_name.clone(), McpServerStatus::Connected);
        }

        // 注册发现的工具
        for tool in tools {
            info!("注册工具: {}", tool.name());
            tool_registry.register_tool(Box::new(tool)).await?;
        }

        info!("完成连接到MCP服务器: {}", server_name);
        Ok(())
    }

    /// 连接到MCP服务器
    async fn connect_to_mcp_server(
        &self,
        server_name: &str,
        server_config: &McpServerConfig,
        debug_mode: bool,
    ) -> Result<McpClient> {
        let mut mcp_client = McpClient::new(
            "alou3-rust-mcp-client".to_string(),
            "0.1.0".to_string(),
        );

        mcp_client.register_capabilities(HashMap::new());

        // 创建传输层
        let transport = self.create_transport(server_name, server_config, debug_mode).await?;
        
        // 连接到服务器
        mcp_client.connect(transport).await?;

        Ok(mcp_client)
    }

    /// 创建传输层
    async fn create_transport(
        &self,
        server_name: &str,
        server_config: &McpServerConfig,
        _debug_mode: bool,
    ) -> Result<Box<dyn McpTransport + Send + Sync>> {
        if let Some(command) = &server_config.command {
            // 创建stdio传输
            let args = server_config.args.clone().unwrap_or_default();
            let env = server_config.env.clone().unwrap_or_default();
            let cwd = server_config.cwd.clone();
            
            let transport = StdioTransport::new(command.clone(), args, env, cwd);
            Ok(Box::new(transport))
        } else if let Some(url) = &server_config.url {
            // 创建HTTP传输
            let headers = server_config.headers.clone().unwrap_or_default();
            let transport = HttpTransport::new(url.clone(), headers);
            Ok(Box::new(transport))
        } else if let Some(http_url) = &server_config.http_url {
            // 创建Streamable HTTP传输
            let headers = server_config.headers.clone().unwrap_or_default();
            let transport = HttpTransport::new(http_url.clone(), headers);
            Ok(Box::new(transport))
        } else {
            Err(anyhow::anyhow!(
                "无效配置：缺少 httpUrl（用于Streamable HTTP）、url（用于HTTP）和 command（用于stdio）"
            ))
        }
    }

    /// 发现工具
    async fn discover_tools(
        &self,
        server_name: &str,
        server_config: &McpServerConfig,
        mcp_client: &McpClient,
    ) -> Result<Vec<DiscoveredMcpTool>> {
        let tools = mcp_client.list_tools().await?;
        let mut discovered_tools = Vec::new();

        for tool_decl in tools {
            if let Some(tool_name) = tool_decl.get("name").and_then(|v| v.as_str()) {
                if !self.is_enabled(&tool_decl, server_name, server_config) {
                    continue;
                }

                if !self.has_valid_types(&tool_decl) {
                    warn!(
                        "跳过工具 '{}' 从MCP服务器 '{}'，因为其参数模式中缺少类型。请向MCP服务器的所有者提交问题。",
                        tool_name, server_name
                    );
                    continue;
                }

                // 使用inputSchema而不是parametersJsonSchema
                let schema = tool_decl.get("inputSchema")
                    .or_else(|| tool_decl.get("parametersJsonSchema"))
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({"type": "object", "properties": {}}));

                let description = tool_decl.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // 创建模拟的可调用工具
                let mcp_tool = Arc::new(MockCallableTool::new(mcp_client.clone()));
                
                let discovered_tool = McpToolFactory::create_discovered_tool(
                    mcp_tool,
                    server_name.to_string(),
                    tool_name.to_string(),
                    description,
                    schema,
                    server_config.timeout,
                    server_config.trust,
                );

                discovered_tools.push(discovered_tool);
            }
        }

        Ok(discovered_tools)
    }

    /// 发现提示
    async fn discover_prompts(
        &self,
        server_name: &str,
        _mcp_client: &McpClient,
    ) -> Result<Vec<DiscoveredMcpPrompt>> {
        // TODO: 实现提示发现逻辑
        // 这里应该发送prompts/list请求并解析响应
        Ok(vec![])
    }

    /// 检查工具是否启用
    fn is_enabled(&self, tool_decl: &serde_json::Value, server_name: &str, server_config: &McpServerConfig) -> bool {
        let tool_name = match tool_decl.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                warn!("从MCP服务器 '{}' 发现没有名称的函数声明。跳过。", server_name);
                return false;
            }
        };

        let include_tools = &server_config.include_tools;
        let exclude_tools = &server_config.exclude_tools;

        // excludeTools优先于includeTools
        if let Some(exclude_list) = exclude_tools {
            if exclude_list.contains(&tool_name.to_string()) {
                return false;
            }
        }

        match include_tools {
            Some(include_list) => {
                include_list.iter().any(|tool| {
                    tool == tool_name || tool.starts_with(&format!("{}(", tool_name))
                })
            }
            None => true, // 如果没有include列表，默认启用所有工具
        }
    }

    /// 检查模式是否有有效类型
    fn has_valid_types(&self, schema: &serde_json::Value) -> bool {
        // 简化的类型验证
        // 在实际实现中，这里应该递归验证整个模式
        if let Some(obj) = schema.as_object() {
            if obj.contains_key("type") {
                return true;
            }
            
            // 检查是否有子模式
            for keyword in &["anyOf", "allOf", "oneOf"] {
                if let Some(array) = obj.get(&keyword.to_string()).and_then(|v| v.as_array()) {
                    return array.iter().all(|sub_schema| self.has_valid_types(sub_schema));
                }
            }
        }
        
        false
    }

    /// 获取服务器状态
    pub async fn get_server_status(&self, server_name: &str) -> McpServerStatus {
        let statuses = self.server_statuses.read().await;
        statuses.get(server_name).cloned().unwrap_or(McpServerStatus::Disconnected)
    }

    /// 获取所有服务器状态
    pub async fn get_all_server_statuses(&self) -> HashMap<String, McpServerStatus> {
        let statuses = self.server_statuses.read().await;
        statuses.clone()
    }

    /// 获取发现状态
    pub async fn get_discovery_state(&self) -> McpDiscoveryState {
        let state = self.discovery_state.read().await;
        state.clone()
    }
}

impl Default for McpClientManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 模拟可调用工具（用于测试和演示）
struct MockCallableTool {
    client: McpClient,
}

impl MockCallableTool {
    fn new(client: McpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl CallableTool for MockCallableTool {
    async fn call_tool(&self, function_calls: Vec<FunctionCall>) -> Result<Vec<Part>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        
        for call in function_calls {
            let args = call.args;
            let result = self.client.call_tool(&call.name, args).await?;
            
            let part = Part {
                text: None,
                inline_data: None,
                function_response: Some(FunctionResponse {
                    name: call.name,
                    response: result,
                }),
            };
            
            results.push(part);
        }
        
        Ok(results)
    }
}

/// 错误消息提取
fn get_error_message(error: &dyn std::error::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_client_creation() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        assert_eq!(client.name, "test-client");
        assert_eq!(client.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_mcp_client_manager_creation() {
        let manager = McpClientManager::new();
        let state = manager.get_discovery_state().await;
        assert_eq!(state, McpDiscoveryState::NotStarted);
    }

    #[test]
    fn test_has_valid_types() {
        let manager = McpClientManager::new();
        
        let valid_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });
        
        assert!(manager.has_valid_types(&valid_schema));
        
        let invalid_schema = serde_json::json!({
            "properties": {
                "name": {"type": "string"}
            }
        });
        
        assert!(!manager.has_valid_types(&invalid_schema));
    }
}
