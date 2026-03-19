//! 前端桥接模块
//!
//! 提供前端与 Rust 后端的通信桥梁
//!
//! # 架构说明
//!
//! BridgeManager 是无状态的资源池管理器，使用 Semaphore 控制并发：
//! - LlmPool: LLM 连接池
//! - ToolPool: 工具调用池
//! - BrowserPool: 浏览器实例池

pub mod tool_bridge;
pub mod context_bridge;
pub mod pool;  // 资源池管理

// 重新导出核心类型和接口
pub use tool_bridge::{ToolBridge, ToolBridgeConfig, ToolCallRequest, ToolCallResponse};
pub use context_bridge::{ContextBridge, ContextBridgeConfig};
pub use pool::{LlmPool, LlmPoolConfig, ToolPool, ToolPoolConfig, BrowserPool, BrowserPoolConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// 桥接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// 工具桥接配置
    pub tool_bridge: ToolBridgeConfig,
    /// 上下文桥接配置
    pub context_bridge: ContextBridgeConfig,
    /// 启用桥接
    pub enabled: bool,
    /// 最大并发调用数
    pub max_concurrent_calls: usize,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
    /// 重试次数
    pub max_retries: u32,
    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,
    /// 启用调试模式
    pub debug_mode: bool,
}

/// 桥接管理器
/// 
/// 无状态资源池 + Semaphore 控制并发
pub struct BridgeManager {
    /// 工具桥接
    tool_bridge: Arc<ToolBridge>,
    /// 上下文桥接
    context_bridge: Arc<ContextBridge>,
    /// 配置
    config: Arc<BridgeConfig>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
}

impl BridgeManager {
    /// 创建新的桥接管理器
    pub fn new(config: BridgeConfig) -> Self {
        Self::new_with_toolbus(config, None)
    }

    /// 创建新的桥接管理器（带 ToolBus）
    pub fn new_with_toolbus(config: BridgeConfig, tool_bus: Option<Arc<crate::agent_runtime::tool_bus::ToolBus>>) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_calls));

        Self {
            tool_bridge: Arc::new(ToolBridge::new_sync_with_toolbus(config.tool_bridge.clone(), tool_bus)),
            context_bridge: Arc::new(ContextBridge::new(config.context_bridge.clone())),
            config: Arc::new(config),
            semaphore,
        }
    }

    /// 获取并发许可
    pub async fn acquire_permit(&self) -> Result<tokio::sync::SemaphorePermit<'_>, Box<dyn std::error::Error + Send + Sync>> {
        let permit = self.semaphore.acquire().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        Ok(permit)
    }

    /// 获取工具桥接
    pub fn tool_bridge(&self) -> Arc<ToolBridge> {
        self.tool_bridge.clone()
    }

    /// 获取上下文桥接
    pub fn context_bridge(&self) -> Arc<ContextBridge> {
        self.context_bridge.clone()
    }

    /// 更新 ToolBus（用于配置更新后重新加载）
    pub async fn update_tool_bus(&self, tool_bus: Arc<crate::agent_runtime::tool_bus::ToolBus>) {
        self.tool_bridge.update_tool_bus(tool_bus).await;
    }

    /// 获取配置
    pub fn get_config(&self) -> &BridgeConfig {
        &self.config
    }

    /// 检查桥接健康状态
    pub async fn health_check(&self) -> BridgeHealthStatus {
        let tool_bridge_healthy = self.tool_bridge.health_check().await;
        let context_bridge_healthy = self.context_bridge.health_check().await;

        let overall_healthy = tool_bridge_healthy.is_healthy && context_bridge_healthy.is_healthy;

        BridgeHealthStatus {
            overall_healthy,
            tool_bridge: tool_bridge_healthy,
            context_bridge: context_bridge_healthy,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 获取当前活跃的连接数
    pub fn active_connections(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// 桥接健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeHealthStatus {
    /// 整体健康状态
    pub overall_healthy: bool,
    /// 工具桥接状态
    pub tool_bridge: ComponentHealthStatus,
    /// 上下文桥接状态
    pub context_bridge: ComponentHealthStatus,
    /// 检查时间戳
    pub timestamp: i64,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthStatus {
    /// 是否健康
    pub is_healthy: bool,
    /// 状态消息
    pub message: String,
    /// 最后检查时间
    pub last_check: i64,
    /// 错误详情（如果有）
    pub error_details: Option<String>,
}

/// 创建默认桥接管理器
pub fn create_default_bridge_manager() -> BridgeManager {
    create_default_bridge_manager_with_toolbus(None)
}

/// 创建默认桥接管理器（带 ToolBus）
pub fn create_default_bridge_manager_with_toolbus(tool_bus: Option<Arc<crate::agent_runtime::tool_bus::ToolBus>>) -> BridgeManager {
    BridgeManager::new_with_toolbus(BridgeConfig {
        tool_bridge: ToolBridgeConfig::default(),
        context_bridge: ContextBridgeConfig::default(),
        enabled: true,
        max_concurrent_calls: 10,
        timeout_seconds: 30,
        max_retries: 3,
        retry_delay_ms: 1000,
        debug_mode: false,
    }, tool_bus)
}

/// 桥接事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEventType {
    /// 工具调用开始
    ToolCallStarted,
    /// 工具调用完成
    ToolCallCompleted,
    /// 工具调用失败
    ToolCallFailed,
    /// 上下文操作开始
    ContextOperationStarted,
    /// 上下文操作完成
    ContextOperationCompleted,
    /// 上下文操作失败
    ContextOperationFailed,
    /// 配置更新
    ConfigUpdated,
    /// 统计重置
    StatisticsReset,
}

/// 桥接事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeEvent {
    /// 事件类型
    pub event_type: BridgeEventType,
    /// 事件 ID
    pub event_id: String,
    /// 时间戳
    pub timestamp: i64,
    /// 事件数据
    pub data: HashMap<String, serde_json::Value>,
    /// 相关会话 ID
    pub session_id: Option<String>,
}

/// 桥接事件处理器 trait
#[async_trait::async_trait]
pub trait BridgeEventHandler: Send + Sync {
    /// 处理桥接事件
    async fn handle_event(&self, event: BridgeEvent) -> Result<(), Box<dyn std::error::Error>>;
}

/// 桥接事件总线
pub struct BridgeEventBus {
    handlers: Vec<Box<dyn BridgeEventHandler>>,
}

impl BridgeEventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// 注册事件处理器
    pub fn register_handler(&mut self, handler: Box<dyn BridgeEventHandler>) {
        self.handlers.push(handler);
    }

    /// 发送事件
    pub async fn emit_event(&self, event: BridgeEvent) {
        for handler in &self.handlers {
            if let Err(e) = handler.handle_event(event.clone()).await {
                eprintln!("Event handler error: {}", e);
            }
        }
    }
}

impl Default for BridgeEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_config_defaults() {
        let config = BridgeConfig {
            tool_bridge: ToolBridgeConfig::default(),
            context_bridge: ContextBridgeConfig::default(),
            enabled: true,
            max_concurrent_calls: 10,
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            debug_mode: false,
        };

        assert!(config.enabled);
        assert_eq!(config.max_concurrent_calls, 10);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_bridge_manager_creation() {
        let manager = create_default_bridge_manager();
        let config = manager.get_config();

        assert!(config.enabled);
        assert_eq!(config.max_concurrent_calls, 10);
        assert_eq!(manager.active_connections(), 10);
    }

    #[tokio::test]
    async fn test_bridge_manager_semaphore() {
        let manager = create_default_bridge_manager();
        
        // 获取一个许可
        let _permit1 = manager.acquire_permit().await.unwrap();
        assert_eq!(manager.active_connections(), 9);
        
        // 获取第二个许可
        let _permit2 = manager.acquire_permit().await.unwrap();
        assert_eq!(manager.active_connections(), 8);
        
        // 释放许可（permit 离开作用域自动释放）
        drop(_permit1);
        assert_eq!(manager.active_connections(), 9);
    }
}
