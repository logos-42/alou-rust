//! 上下文压缩模块
//!
//! 提供无限循环版本的上下文压缩，支持智能上下文管理和记忆

pub mod compressor;
pub mod memory;
pub mod cache;

// 重新导出核心类型和接口
pub use compressor::{ContextCompressor, CompressionStrategy, CompressionResult};
pub use memory::{MemoryManager, MemoryEntry, MemoryType};
pub use cache::{CacheManager as ContextCache, CacheEntry, CacheStrategy};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 上下文配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// 最大上下文长度
    pub max_context_length: usize,
    /// 压缩阈值
    pub compression_threshold: usize,
    /// 保留重要内容的比例
    pub keep_important_ratio: f64,
    /// 启用智能压缩
    pub smart_compression: bool,
    /// 启用记忆系统
    pub memory_enabled: bool,
    /// 最大记忆条目数
    pub max_memory_entries: usize,
    /// 缓存配置
    pub cache_enabled: bool,
    /// 压缩策略
    pub compression_strategy: CompressionStrategy,
    /// 缓存策略
    pub cache_strategy: CacheStrategy,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_context_length: 8000,
            compression_threshold: 6000,
            keep_important_ratio: 0.7,
            smart_compression: true,
            memory_enabled: true,
            max_memory_entries: 1000,
            cache_enabled: true,
            compression_strategy: CompressionStrategy::Adaptive,
            cache_strategy: CacheStrategy::LRU,
        }
    }
}

/// 上下文条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    /// 条目ID
    pub id: String,
    /// 内容
    pub content: String,
    /// 重要性分数 (0.0-1.0)
    pub importance_score: f64,
    /// 时间戳
    pub timestamp: i64,
    /// 条目类型
    pub entry_type: ContextEntryType,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 上下文条目类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextEntryType {
    /// 用户消息
    UserMessage,
    /// 助手响应
    AssistantResponse,
    /// 系统消息
    SystemMessage,
    /// 工具调用
    ToolCall,
    /// 工具结果
    ToolResult,
    /// 代码片段
    CodeSnippet,
    /// 文件内容
    FileContent,
    /// 搜索结果
    SearchResult,
    /// 其他
    Other,
}

/// 上下文状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    /// 当前上下文条目
    pub entries: Vec<ContextEntry>,
    /// 总长度
    pub total_length: usize,
    /// 压缩次数
    pub compression_count: u64,
    /// 最后压缩时间
    pub last_compression_time: Option<i64>,
    /// 上下文质量分数
    pub quality_score: f64,
}

/// 上下文管理器
pub struct ContextManager {
    /// 配置
    config: ContextConfig,
    /// 状态
    state: ContextState,
    /// 压缩器
    compressor: ContextCompressor,
    /// 记忆管理器
    memory_manager: Option<MemoryManager>,
    /// 缓存
    cache: Option<ContextCache>,
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new(config: ContextConfig) -> Self {
        let compressor = ContextCompressor::new(config.compression_strategy.clone());
        let memory_manager = if config.memory_enabled {
            Some(MemoryManager::new(config.max_memory_entries as u64))
        } else {
            None
        };
        let cache = if config.cache_enabled {
            Some(ContextCache::new(1000, config.compression_strategy.clone()))
        } else {
            None
        };

        Self {
            config,
            state: ContextState {
                entries: Vec::new(),
                total_length: 0,
                compression_count: 0,
                last_compression_time: None,
                quality_score: 1.0,
            },
            compressor,
            memory_manager,
            cache,
        }
    }

    /// 添加上下文条目
    pub async fn add_entry(&mut self, entry: ContextEntry) -> Result<(), Box<dyn std::error::Error>> {
        // 检查是否需要压缩
        let new_total_length = self.state.total_length + entry.content.len();
        if new_total_length > self.config.max_context_length {
            self.compress_context().await?;
        }

            // 添加到记忆系统
            if let Some(memory) = &mut self.memory_manager {
                let memory_entry = MemoryEntry {
                    key: entry.id.clone(),
                    value: serde_json::json!({
                        "content": entry.content.clone(),
                        "importance_score": entry.importance_score,
                        "entry_type": entry.entry_type,
                        "metadata": entry.metadata.clone(),
                    }),
                    memory_type: MemoryType::Contextual, // 使用统一的上下文记忆类型
                    created_at: entry.timestamp,
                    expires_at: entry.timestamp + 3600 * 24 * 30, // 30天过期
                };
                memory.store(memory_entry.key.clone(), memory_entry.value.clone(), memory_entry.memory_type).await?;
            }

        // 添加条目
        self.state.entries.push(entry);
        self.state.total_length = new_total_length;

        // 更新质量分数
        self.update_quality_score();

        Ok(())
    }

    /// 获取当前上下文
    pub fn get_context(&self) -> &[ContextEntry] {
        &self.state.entries
    }

    /// 获取上下文状态
    pub fn get_state(&self) -> &ContextState {
        &self.state
    }

    /// 获取上下文统计信息
    pub async fn get_statistics(&self) -> ContextStatistics {
        let mut type_counts = HashMap::new();
        let mut avg_importance = 0.0;

        for entry in &self.state.entries {
            *type_counts.entry(entry.entry_type).or_insert(0) += 1;
            avg_importance += entry.importance_score;
        }

        if !self.state.entries.is_empty() {
            avg_importance /= self.state.entries.len() as f64;
        }

        let memory_entries = if let Some(memory) = &self.memory_manager {
            memory.get_entry_count().await
        } else {
            0
        };

        let cache_entries = if let Some(cache) = &self.cache {
            cache.get_entry_count()
        } else {
            0
        };

        ContextStatistics {
            total_entries: self.state.entries.len(),
            total_length: self.state.total_length,
            type_counts,
            avg_importance_score: avg_importance,
            compression_count: self.state.compression_count,
            quality_score: self.state.quality_score,
            memory_entries,
            cache_entries,
        }
    }

    /// 压缩上下文
    pub async fn compress_context(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.state.entries.len() <= 1 {
            return Ok(());
        }

        // 使用压缩器压缩上下文
        let compression_result = self.compressor.compress(
            &self.state.entries,
            self.config.max_context_length,
            self.config.keep_important_ratio,
        ).await?;

        // 更新状态
        self.state.entries = compression_result.entries;
        self.state.total_length = self.state.entries.iter()
            .map(|e| e.content.len())
            .sum();
        self.state.compression_count += 1;
        self.state.last_compression_time = Some(chrono::Utc::now().timestamp());

        // 更新质量分数
        self.update_quality_score();

        Ok(())
    }

    /// 更新质量分数
    fn update_quality_score(&mut self) {
        if self.state.entries.is_empty() {
            self.state.quality_score = 1.0;
            return;
        }

        // 基于重要性分数和条目分布计算质量
        let avg_importance: f64 = self.state.entries.iter()
            .map(|e| e.importance_score)
            .sum::<f64>() / self.state.entries.len() as f64;

        // 基于条目类型多样性计算分数
        let mut type_counts = HashMap::new();
        for entry in &self.state.entries {
            *type_counts.entry(entry.entry_type).or_insert(0) += 1;
        }

        let type_diversity = type_counts.len() as f64 / 8.0; // 8种条目类型

        // 综合计算质量分数
        self.state.quality_score = (avg_importance * 0.7) + (type_diversity * 0.3);
    }

    /// 从记忆中恢复相关内容
    pub async fn recall_from_memory(&mut self, query: &str, limit: usize) -> Result<Vec<ContextEntry>, Box<dyn std::error::Error>> {
        if let Some(memory) = &self.memory_manager {
            let memory_entries = memory.search_similar(query, limit).await?;

            // 转换为上下文条目
            let context_entries = memory_entries.into_iter()
                .map(|mem_entry| {
                    // 从JSON值中提取内容
                    let content = mem_entry.value.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let importance_score = mem_entry.value.get("importance_score")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.5);

                    let entry_type = mem_entry.value.get("entry_type")
                        .and_then(|v| v.as_str())
                        .and_then(|s| match s {
                            "UserMessage" => Some(ContextEntryType::UserMessage),
                            "AssistantResponse" => Some(ContextEntryType::AssistantResponse),
                            "ToolCall" => Some(ContextEntryType::ToolCall),
                            "ToolResult" => Some(ContextEntryType::ToolResult),
                            "CodeSnippet" => Some(ContextEntryType::CodeSnippet),
                            "FileContent" => Some(ContextEntryType::FileContent),
                            "SearchResult" => Some(ContextEntryType::SearchResult),
                            _ => Some(ContextEntryType::Other),
                        })
                        .unwrap_or(ContextEntryType::Other);

                    let metadata = mem_entry.value.get("metadata")
                        .and_then(|v| v.as_object())
                        .map(|obj| obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect())
                        .unwrap_or_default();

                    ContextEntry {
                        id: mem_entry.key.clone(),
                        content: content.to_string(),
                        importance_score,
                        timestamp: mem_entry.created_at,
                        entry_type,
                        metadata,
                    }
                })
                .collect();

            Ok(context_entries)
        } else {
            Ok(Vec::new())
        }
    }

    /// 清除上下文
    pub async fn clear_context(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.entries.clear();
        self.state.total_length = 0;
        self.state.compression_count = 0;
        self.state.last_compression_time = None;
        self.state.quality_score = 1.0;

        if let Some(memory) = &mut self.memory_manager {
            memory.clear().await?;
        }

        if let Some(cache) = &mut self.cache {
            cache.clear().await?;
        }

        Ok(())
    }

    /// 导出上下文
    pub async fn export_context(&self) -> Result<String, Box<dyn std::error::Error>> {
        let export_data = serde_json::json!({
            "config": self.config,
            "state": self.state,
            "statistics": self.get_statistics().await,
            "exported_at": chrono::Utc::now().timestamp(),
        });

        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    /// 导入上下文
    pub fn import_context(&mut self, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        let import_data: serde_json::Value = serde_json::from_str(data)?;

        if let Some(state) = import_data.get("state") {
            self.state = serde_json::from_value(state.clone())?;
        }

        Ok(())
    }

    /// 压缩文本（桥接方法）
    pub async fn compress_text(&mut self, text: &str) -> Result<CompressionResult, Box<dyn std::error::Error>> {
        use compressor::ContextCompressor;
        let compressor = ContextCompressor::new(self.config.compression_strategy.clone());
        let dummy_entries = vec![ContextEntry {
            id: "compress".to_string(),
            content: text.to_string(),
            importance_score: 1.0,
            timestamp: chrono::Utc::now().timestamp(),
            entry_type: ContextEntryType::Other,
            metadata: HashMap::new(),
        }];
        compressor.compress(&dummy_entries, text.len(), 0.5).await
    }

    /// 解压上下文（桥接方法）
    pub async fn decompress_context(&mut self, compressed: &CompressionResult) -> Result<String, Box<dyn std::error::Error>> {
        // 简单实现，返回压缩结果的描述
        Ok(format!("Compressed content: {} entries, ratio: {:.2}",
            compressed.retained_entries, compressed.compression_ratio))
    }

    /// 存储记忆
    pub async fn store_memory(&mut self, key: String, value: serde_json::Value, memory_type: MemoryType) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(memory) = &mut self.memory_manager {
            let entry = MemoryEntry {
                key,
                value,
                memory_type,
                created_at: chrono::Utc::now().timestamp(),
                expires_at: chrono::Utc::now().timestamp() + 3600 * 24 * 30, // 30天
            };
            memory.store(entry.key, entry.value, entry.memory_type).await?;
        }
        Ok(())
    }

    /// 检索记忆
    pub async fn retrieve_memory(&mut self, key: &str) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        if let Some(memory) = &mut self.memory_manager {
            let result = memory.retrieve(key).await?;
            Ok(result.map(|entry| entry.value))
        } else {
            Ok(None)
        }
    }

    /// 搜索记忆
    pub async fn search_memory(&mut self, query: &str, memory_type: Option<MemoryType>) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        if let Some(memory) = &mut self.memory_manager {
            let limit = 10; // 默认限制
            let entries = memory.search_similar(query, limit).await?;
            Ok(entries.into_iter().map(|e| e.value).collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// 缓存数据
    pub async fn cache_data(&mut self, key: String, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(cache) = &mut self.cache {
            cache.store(key, data).await?;
        }
        Ok(())
    }

    /// 获取缓存数据
    pub async fn get_cached_data(&mut self, key: &str) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        if let Some(cache) = &mut self.cache {
            let result = cache.retrieve(key).await?;
            Ok(result.map(|entry| entry.data))
        } else {
            Ok(None)
        }
    }

    /// 清理过期数据
    pub async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(memory) = &mut self.memory_manager {
            memory.clear().await?;
        }
        if let Some(cache) = &mut self.cache {
            cache.clear().await?;
        }
        Ok(())
    }
}

/// 上下文统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStatistics {
    /// 总条目数
    pub total_entries: usize,
    /// 总长度
    pub total_length: usize,
    /// 按类型统计
    pub type_counts: HashMap<ContextEntryType, usize>,
    /// 平均重要性分数
    pub avg_importance_score: f64,
    /// 压缩次数
    pub compression_count: u64,
    /// 质量分数
    pub quality_score: f64,
    /// 记忆条目数
    pub memory_entries: usize,
    /// 缓存条目数
    pub cache_entries: usize,
}

/// 创建默认上下文管理器
pub fn create_default_context_manager() -> ContextManager {
    ContextManager::new(ContextConfig::default())
}

/// 创建上下文条目
pub fn create_context_entry(
    id: String,
    content: String,
    importance_score: f64,
    entry_type: ContextEntryType,
    metadata: HashMap<String, serde_json::Value>,
) -> ContextEntry {
    ContextEntry {
        id,
        content,
        importance_score,
        timestamp: chrono::Utc::now().timestamp(),
        entry_type,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_manager() {
        let mut manager = create_default_context_manager();

        let entry = create_context_entry(
            "test_1".to_string(),
            "This is a test message".to_string(),
            0.8,
            ContextEntryType::UserMessage,
            HashMap::new(),
        );

        manager.add_entry(entry).await.unwrap();

        assert_eq!(manager.get_context().len(), 1);
        assert!(manager.get_state().total_length > 0);

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(*stats.type_counts.get(&ContextEntryType::UserMessage).unwrap_or(&0), 1);
    }

    #[tokio::test]
    async fn test_context_export_import() {
        let mut manager = create_default_context_manager();

        let entry = create_context_entry(
            "test_1".to_string(),
            "Export/import test".to_string(),
            0.9,
            ContextEntryType::AssistantResponse,
            HashMap::new(),
        );

        manager.add_entry(entry).await.unwrap();

        // 导出
        let export_data = manager.export_context().await.unwrap();

        // 创建新管理器并导入
        let mut new_manager = create_default_context_manager();
        new_manager.import_context(&export_data).unwrap();

        assert_eq!(new_manager.get_context().len(), 1);
    }
}
