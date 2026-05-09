//! 上下文桥接
//!
//! 提供前端与上下文管理器的桥接功能

use super::super::context::{ContextManager, CompressionResult, MemoryEntry, CacheEntry, ContextStatistics};
use serde::{Deserialize, Serialize};

/// 上下文桥接
pub struct ContextBridge {
    context_manager: ContextManager,
    request_count: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl ContextBridge {
    /// 创建新的上下文桥接
    pub fn new(config: ContextBridgeConfig) -> Self {
        Self {
            context_manager: ContextManager::new(config.context_config),
            request_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// 处理上下文请求
    pub async fn handle_request(&mut self, request: ContextRequest) -> Result<ContextResponse, Box<dyn std::error::Error>> {
        let mut count = self.request_count.lock().unwrap();
        *count += 1;

        match request.operation {
            ContextOperation::Compress { text } => {
                let result = self.context_manager.compress_text(&text).await?;
                Ok(ContextResponse::CompressionResult(result))
            }
            ContextOperation::Decompress { compressed } => {
                let result = self.context_manager.decompress_context(&compressed).await?;
                Ok(ContextResponse::DecompressedText(result))
            }
            ContextOperation::StoreMemory { key, value, memory_type } => {
                self.context_manager.store_memory(key, value, memory_type).await?;
                Ok(ContextResponse::Success)
            }
            ContextOperation::RetrieveMemory { key } => {
                let result = self.context_manager.retrieve_memory(&key).await?;
                Ok(ContextResponse::MemoryEntry(result.and_then(|v| serde_json::from_value(v).ok())))
            }
            ContextOperation::SearchMemory { query, memory_type } => {
                let results = self.context_manager.search_memory(&query, memory_type).await?;
                Ok(ContextResponse::MemoryEntries(results.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect()))
            }
            ContextOperation::CacheData { key, data } => {
                self.context_manager.cache_data(key, data).await?;
                Ok(ContextResponse::Success)
            }
            ContextOperation::GetCachedData { key } => {
                let result = self.context_manager.get_cached_data(&key).await?;
                Ok(ContextResponse::CacheEntry(result.and_then(|v| serde_json::from_value(v).ok())))
            }
            ContextOperation::GetStats => {
                let stats = self.context_manager.get_statistics().await;
                Ok(ContextResponse::Stats(stats))
            }
            ContextOperation::Cleanup => {
                self.context_manager.cleanup().await?;
                Ok(ContextResponse::Success)
            }
        }
    }

    /// 获取请求计数
    pub async fn get_request_count(&self) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(*self.request_count.lock().unwrap())
    }

    /// 更新配置
    pub fn update_config(&mut self, _config: ContextBridgeConfig) {
        // TODO: 实现配置更新逻辑
    }

    /// 健康检查
    pub async fn health_check(&self) -> super::ComponentHealthStatus {
        super::ComponentHealthStatus {
            is_healthy: true,
            message: "Context bridge is healthy".to_string(),
            last_check: chrono::Utc::now().timestamp(),
            error_details: None,
        }
    }
}

/// 上下文桥接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBridgeConfig {
    /// 上下文配置
    pub context_config: super::super::context::ContextConfig,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用缓存
    pub enable_cache: bool,
}

impl Default for ContextBridgeConfig {
    fn default() -> Self {
        Self {
            context_config: super::super::context::ContextConfig::default(),
            enable_compression: true,
            enable_cache: true,
        }
    }
}

/// 上下文请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRequest {
    /// 操作类型
    pub operation: ContextOperation,
}

/// 上下文操作
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContextOperation {
    /// 压缩文本
    Compress { text: String },
    /// 解压文本
    Decompress { compressed: CompressionResult },
    /// 存储记忆
    StoreMemory { key: String, value: serde_json::Value, memory_type: super::super::context::MemoryType },
    /// 检索记忆
    RetrieveMemory { key: String },
    /// 搜索记忆
    SearchMemory { query: String, memory_type: Option<super::super::context::MemoryType> },
    /// 缓存数据
    CacheData { key: String, data: serde_json::Value },
    /// 获取缓存数据
    GetCachedData { key: String },
    /// 获取统计信息
    GetStats,
    /// 清理过期数据
    Cleanup,
}

/// 上下文响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContextResponse {
    /// 压缩结果
    CompressionResult(CompressionResult),
    /// 解压后的文本
    DecompressedText(String),
    /// 记忆条目
    MemoryEntry(Option<MemoryEntry>),
    /// 记忆条目列表
    MemoryEntries(Vec<MemoryEntry>),
    /// 缓存条目
    CacheEntry(Option<CacheEntry>),
    /// 统计信息
    Stats(ContextStatistics),
    /// 成功响应
    Success,
}
