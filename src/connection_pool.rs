use crate::client::{Client, ClientBuilder};
use crate::types::ClientCapabilities;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use anyhow::Result;
use tracing::{info, debug, warn};

/// 连接池管理器
pub struct ConnectionPool {
    /// 活跃的连接映射
    connections: Arc<RwLock<HashMap<String, Arc<Mutex<Client>>>>>,
    /// 连接配置
    configs: Arc<RwLock<HashMap<String, McpServerConfig>>>,
}

/// MCP服务器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub directory: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

impl ConnectionPool {
    /// 创建新的连接池
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册服务器配置
    pub async fn register_server(&self, name: String, config: McpServerConfig) {
        let name_clone = name.clone();
        let mut configs = self.configs.write().await;
        configs.insert(name, config);
        info!("已注册服务器配置: {}", name_clone);
    }

    /// 获取或创建连接
    pub async fn get_connection(&self, server_name: &str) -> Result<Arc<Mutex<Client>>> {
        // 首先检查是否已有活跃连接
        {
            let connections = self.connections.read().await;
            if let Some(client) = connections.get(server_name) {
                // 检查连接是否健康
                if self.is_connection_healthy(server_name).await {
                    debug!("✅ 复用现有健康连接: {}", server_name);
                    return Ok(client.clone());
                } else {
                    info!("⚠️  现有连接不健康，将重新创建: {}", server_name);
                }
            }
        }

        // 如果没有连接或连接不健康，创建新连接
        debug!("🔗 创建新MCP连接: {}", server_name);
        let configs = self.configs.read().await;
        let config = configs.get(server_name)
            .ok_or_else(|| anyhow::anyhow!("未找到服务器配置: {}", server_name))?;

        let client = self.create_client(server_name, config).await?;
        
        // 将新连接添加到池中
        let client_arc = Arc::new(Mutex::new(client));
        {
            let mut connections = self.connections.write().await;
            connections.insert(server_name.to_string(), client_arc.clone());
        }

        debug!("✅ 成功建立MCP连接: {}", server_name);
        Ok(client_arc)
    }

    /// 创建新的客户端连接
    async fn create_client(&self, name: &str, config: &McpServerConfig) -> Result<Client> {
        debug!("正在创建客户端连接: {}", name);
        
        let mut builder = ClientBuilder::new(&config.command);
        
        // 添加命令参数
        for arg in &config.args {
            builder = builder.arg(arg);
        }
        
        // 设置工作目录
        if let Some(dir) = &config.directory {
            builder = builder.directory(dir);
        }
        
        // 设置环境变量
        if let Some(env_vars) = &config.env {
            for (key, value) in env_vars {
                builder = builder.env(key, value);
            }
        }
        
        // 设置客户端实现信息
        builder = builder.implementation("mcp-connection-pool", "0.1.0");
        
        // 设置客户端能力
        let capabilities = ClientCapabilities {
            custom: None,
        };
        builder = builder.capabilities(capabilities);
        
        let client = builder.spawn_and_initialize().await?;
        debug!("成功创建客户端连接: {}", name);
        
        Ok(client)
    }

    /// 关闭指定连接
    pub async fn close_connection(&self, server_name: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(client_arc) = connections.remove(server_name) {
            let mut client = client_arc.lock().await;
            client.shutdown().await?;
            info!("已关闭连接: {}", server_name);
        }
        Ok(())
    }

    /// 关闭所有连接
    pub async fn close_all_connections(&self) -> Result<()> {
        let mut connections = self.connections.write().await;
        for (name, client_arc) in connections.drain() {
            let mut client = client_arc.lock().await;
            if let Err(e) = client.shutdown().await {
                warn!("关闭连接 {} 时出错: {}", name, e);
            } else {
                info!("已关闭连接: {}", name);
            }
        }
        Ok(())
    }

    /// 获取活跃连接列表
    pub async fn list_active_connections(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// 获取所有已注册的服务器列表
    pub async fn list_registered_servers(&self) -> Vec<String> {
        let configs = self.configs.read().await;
        configs.keys().cloned().collect()
    }

    /// 显示连接池状态
    pub async fn show_pool_status(&self) {
        let connections = self.connections.read().await;
        let configs = self.configs.read().await;
        
        info!("=== MCP连接池状态 ===");
        info!("已配置服务器: {}", configs.len());
        info!("活跃连接数: {}", connections.len());
        
        for (name, _) in &*connections {
            let is_healthy = self.is_connection_healthy(name).await;
            let status = if is_healthy { "✅ 健康" } else { "❌ 不健康" };
            info!("  - {}: {}", name, status);
        }
        
        for (name, _) in &*configs {
            if !connections.contains_key(name) {
                info!("  - {}: ⏸️  未连接", name);
            }
        }
        info!("====================");
    }

    /// 检查连接是否健康
    pub async fn is_connection_healthy(&self, server_name: &str) -> bool {
        let connections = self.connections.read().await;
        if let Some(client_arc) = connections.get(server_name) {
            let client = client_arc.lock().await;
            // 尝试列出工具来检查连接是否还活跃
            match client.list_tools().await {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// 清理不健康的连接
    pub async fn cleanup_unhealthy_connections(&self) -> Result<()> {
        let connections = self.connections.read().await;
        let connection_names: Vec<String> = connections.keys().cloned().collect();
        drop(connections);

        let mut unhealthy_connections = Vec::new();
        for name in connection_names {
            if !self.is_connection_healthy(&name).await {
                unhealthy_connections.push(name);
            }
        }

        for name in unhealthy_connections {
            warn!("发现不健康连接，正在清理: {}", name);
            self.close_connection(&name).await?;
        }

        Ok(())
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ConnectionPool {
    fn drop(&mut self) {
        // 注意：在Drop中不能使用async，所以这里只是记录日志
        // 实际应用中应该在程序退出前显式调用close_all_connections
        info!("ConnectionPool正在被销毁，请确保已关闭所有连接");
    }
}
