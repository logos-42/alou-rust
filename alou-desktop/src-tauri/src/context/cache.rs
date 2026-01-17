//! 缓存系统
//!
//! 提供数据缓存功能，支持不同的缓存策略

use super::compressor::{ContextCompressor, CompressionStrategy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 缓存管理器
pub struct CacheManager {
    entries: std::sync::Arc<std::sync::Mutex<HashMap<String, CacheEntry>>>,
    max_size: usize,
    compressor: ContextCompressor,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new(max_size: usize, compression_strategy: CompressionStrategy) -> Self {
        Self {
            entries: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            max_size,
            compressor: ContextCompressor::new(compression_strategy),
        }
    }

    /// 存储数据到缓存
    pub async fn store(&self, key: String, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock().unwrap();

        // 检查是否需要清理空间
        if entries.len() >= self.max_size && !entries.contains_key(&key) {
            self.evict_entries(&mut entries, 1);
        }

        let entry = CacheEntry {
            key: key.clone(),
            data,
            created_at: chrono::Utc::now().timestamp(),
            accessed_at: chrono::Utc::now().timestamp(),
            access_count: 0,
        };

        entries.insert(key, entry);
        Ok(())
    }

    /// 从缓存检索数据
    pub async fn retrieve(&self, key: &str) -> Result<Option<CacheEntry>, Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock().unwrap();

        if let Some(entry) = entries.get_mut(key) {
            entry.accessed_at = chrono::Utc::now().timestamp();
            entry.access_count += 1;
            Ok(Some(entry.clone()))
        } else {
            Ok(None)
        }
    }

    /// 删除缓存条目
    pub async fn delete(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock().unwrap();
        Ok(entries.remove(key).is_some())
    }

    /// 清空缓存
    pub async fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
        Ok(())
    }

    /// 清理过期条目
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let max_age = 3600; // 1小时

        entries.retain(|_, entry| (now - entry.created_at) < max_age);
        Ok(())
    }

    /// 获取缓存大小
    pub async fn size(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.len())
    }

    /// 获取缓存条目数量
    pub fn get_entry_count(&self) -> usize {
        let entries = self.entries.lock().unwrap();
        entries.len()
    }

    /// 获取所有缓存键
    pub async fn keys(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.keys().cloned().collect())
    }

    /// 获取缓存统计信息
    pub async fn stats(&self) -> Result<CacheStats, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();

        let total_accesses: u64 = entries.values().map(|e| e.access_count).sum();
        let avg_accesses = if entries.is_empty() {
            0.0
        } else {
            total_accesses as f64 / entries.len() as f64
        };

        Ok(CacheStats {
            size: entries.len(),
            max_size: self.max_size,
            total_accesses,
            avg_accesses,
        })
    }

    /// 驱逐条目（LRU策略）
    fn evict_entries(&self, entries: &mut HashMap<String, CacheEntry>, count: usize) {
        let mut entry_list: Vec<_> = entries.iter().collect();
        entry_list.sort_by(|a, b| a.1.accessed_at.cmp(&b.1.accessed_at));

        let keys_to_remove: Vec<String> = entry_list.into_iter()
            .take(count)
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            entries.remove(&key);
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheEntry {
    /// 键
    pub key: String,
    /// 数据
    pub data: serde_json::Value,
    /// 创建时间
    pub created_at: i64,
    /// 最后访问时间
    pub accessed_at: i64,
    /// 访问次数
    pub access_count: u64,
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 当前大小
    pub size: usize,
    /// 最大大小
    pub max_size: usize,
    /// 总访问次数
    pub total_accesses: u64,
    /// 平均访问次数
    pub avg_accesses: f64,
}

/// 缓存策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    /// LRU (最近最少使用)
    LRU,
    /// LFU (最少使用频率)
    LFU,
    /// FIFO (先进先出)
    FIFO,
    /// TTL (基于时间)
    TTL,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_store_and_retrieve() {
        let manager = CacheManager::new(10, CompressionStrategy::Adaptive);

        let key = "test_key".to_string();
        let data = serde_json::json!({"message": "cached data"});

        // 存储
        manager.store(key.clone(), data).await.unwrap();

        // 检索
        let entry = manager.retrieve(&key).await.unwrap().unwrap();
        assert_eq!(entry.key, key);
        assert_eq!(entry.data["message"], "cached data");
        assert_eq!(entry.access_count, 1);
    }

    #[tokio::test]
    async fn test_cache_size_limit() {
        let manager = CacheManager::new(2, CompressionStrategy::Adaptive);

        // 存储超过限制的条目
        manager.store("key1".to_string(), serde_json::json!("data1")).await.unwrap();
        manager.store("key2".to_string(), serde_json::json!("data2")).await.unwrap();
        manager.store("key3".to_string(), serde_json::json!("data3")).await.unwrap();

        // 应该只有2个条目
        assert_eq!(manager.size().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let manager = CacheManager::new(10, CompressionStrategy::Adaptive);

        manager.store("key1".to_string(), serde_json::json!("data1")).await.unwrap();
        manager.store("key2".to_string(), serde_json::json!("data2")).await.unwrap();

        // 访问一次
        manager.retrieve("key1").await.unwrap();

        let stats = manager.stats().await.unwrap();
        assert_eq!(stats.size, 2);
        assert_eq!(stats.max_size, 10);
        assert_eq!(stats.total_accesses, 1);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let manager = CacheManager::new(10, CompressionStrategy::Adaptive);

        manager.store("key1".to_string(), serde_json::json!("data1")).await.unwrap();
        assert_eq!(manager.size().await.unwrap(), 1);

        manager.clear().await.unwrap();
        assert_eq!(manager.size().await.unwrap(), 0);
    }
}
