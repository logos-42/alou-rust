//! 前端桥接模块
//!
//! 提供前端与 Rust 后端的通信桥梁

pub mod tool_bridge;
pub mod context_bridge;

// 重新导出核心类型和接口
pub use tool_bridge::{ToolBridge, ToolBridgeConfig};
pub use context_bridge::{ContextBridge, ContextBridgeConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// 桥接统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatistics {
    /// 总调用次数
    pub total_calls: u64,
    /// 成功调用次数
    pub successful_calls: u64,
    /// 失败调用次数
    pub failed_calls: u64,
    /// 平均响应时间（毫秒）
    pub average_response_time_ms: f64,
    /// 最后调用时间
    pub last_call_time: Option<i64>,
    /// 活跃调用数
    pub active_calls: usize,
}

/// 桥接管理器
pub struct BridgeManager {
    /// 工具桥接
    tool_bridge: ToolBridge,
    /// 上下文桥接
    context_bridge: ContextBridge,
    /// 配置
    config: BridgeConfig,
    /// 统计信息
    statistics: BridgeStatistics,
}

impl BridgeManager {
    /// 创建新的桥接管理器
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            tool_bridge: ToolBridge::new(config.tool_bridge.clone()),
            context_bridge: ContextBridge::new(config.context_bridge.clone()),
            config,
            statistics: BridgeStatistics {
                total_calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                average_response_time_ms: 0.0,
                last_call_time: None,
                active_calls: 0,
            },
        }
    }

    /// 获取工具桥接
    pub fn tool_bridge(&self) -> &ToolBridge {
        &self.tool_bridge
    }

    /// 获取工具桥接（可变引用）
    pub fn tool_bridge_mut(&mut self) -> &mut ToolBridge {
        &mut self.tool_bridge
    }

    /// 获取上下文桥接
    pub fn context_bridge(&self) -> &ContextBridge {
        &self.context_bridge
    }

    /// 获取上下文桥接（可变引用）
    pub fn context_bridge_mut(&mut self) -> &mut ContextBridge {
        &mut self.context_bridge
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> &BridgeStatistics {
        &self.statistics
    }

    /// 更新统计信息
    pub fn update_statistics(&mut self, call_duration_ms: u64, success: bool) {
        self.statistics.total_calls += 1;
        self.statistics.last_call_time = Some(chrono::Utc::now().timestamp());

        if success {
            self.statistics.successful_calls += 1;
        } else {
            self.statistics.failed_calls += 1;
        }

        // 更新平均响应时间
        let total_calls = self.statistics.total_calls as f64;
        let current_avg = self.statistics.average_response_time_ms;
        self.statistics.average_response_time_ms =
            (current_avg * (total_calls - 1.0) + call_duration_ms as f64) / total_calls;
    }

    /// 获取配置
    pub fn get_config(&self) -> &BridgeConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: BridgeConfig) {
        self.config = config.clone();

        // 更新子桥接的配置
        self.tool_bridge.update_config(config.tool_bridge.clone());

        self.context_bridge.update_config(config.context_bridge.clone());
    }

    /// 重置统计信息
    pub fn reset_statistics(&mut self) {
        self.statistics = BridgeStatistics {
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            average_response_time_ms: 0.0,
            last_call_time: None,
            active_calls: 0,
        };
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
    BridgeManager::new(BridgeConfig {
        tool_bridge: ToolBridgeConfig::default(),
        context_bridge: ContextBridgeConfig::default(),
        enabled: true,
        max_concurrent_calls: 10,
        timeout_seconds: 30,
        max_retries: 3,
        retry_delay_ms: 1000,
        debug_mode: false,
    })
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
    /// 事件ID
    pub event_id: String,
    /// 时间戳
    pub timestamp: i64,
    /// 事件数据
    pub data: HashMap<String, serde_json::Value>,
    /// 相关会话ID
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
        assert_eq!(config.timeout_seconds, 30);
    }

    #[tokio::test]
    async fn test_bridge_statistics_update() {
        let mut manager = create_default_bridge_manager();

        // 初始状态
        let stats = manager.get_statistics();
        assert_eq!(stats.total_calls, 0);
        assert_eq!(stats.successful_calls, 0);

        // 更新统计信息
        manager.update_statistics(100, true);
        let stats = manager.get_statistics();
        assert_eq!(stats.total_calls, 1);
        assert_eq!(stats.successful_calls, 1);
        assert_eq!(stats.average_response_time_ms, 100.0);

        // 再次更新
        manager.update_statistics(200, false);
        let stats = manager.get_statistics();
        assert_eq!(stats.total_calls, 2);
        assert_eq!(stats.successful_calls, 1);
        assert_eq!(stats.failed_calls, 1);
        assert_eq!(stats.average_response_time_ms, 150.0);
    }
}