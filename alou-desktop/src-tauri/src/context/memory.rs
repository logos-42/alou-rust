//! 记忆管理
//!
//! 提供记忆存储、检索和搜索功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆管理器
pub struct MemoryManager {
    entries: std::sync::Arc<std::sync::Mutex<HashMap<String, MemoryEntry>>>,
    ttl_seconds: u64,
}

impl MemoryManager {
    /// 创建新的记忆管理器
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            entries: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            ttl_seconds,
        }
    }

    /// 存储DIAP身份（永久存储，不过期）
    pub async fn store_diap_identity(&self, session_id: String, identity: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("diap_identity_{}", session_id);
        let entry = MemoryEntry {
            key: key.clone(),
            value: identity,
            memory_type: MemoryType::DiapIdentity,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: i64::MAX, // 永不过期
        };

        let mut entries = self.entries.lock().unwrap();
        entries.insert(key, entry);
        Ok(())
    }

    /// 获取DIAP身份
    pub async fn get_diap_identity(&self, session_id: &str) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        let key = format!("diap_identity_{}", session_id);
        let entries = self.entries.lock().unwrap();
        
        if let Some(entry) = entries.get(&key) {
            if entry.memory_type == MemoryType::DiapIdentity {
                return Ok(Some(entry.value.clone()));
            }
        }
        Ok(None)
    }

    /// 删除DIAP身份
    pub async fn remove_diap_identity(&self, session_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let key = format!("diap_identity_{}", session_id);
        let mut entries = self.entries.lock().unwrap();
        Ok(entries.remove(&key).is_some())
    }

    /// 获取所有DIAP身份
    pub async fn get_all_diap_identities(&self) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();
        let mut identities = HashMap::new();
        
        for (key, entry) in entries.iter() {
            if entry.memory_type == MemoryType::DiapIdentity && key.starts_with("diap_identity_") {
                let session_id = key.strip_prefix("diap_identity_").unwrap_or(key);
                identities.insert(session_id.to_string(), entry.value.clone());
            }
        }
        
        Ok(identities)
    }

    /// 检查DIAP身份是否存在
    pub async fn has_diap_identity(&self, session_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let key = format!("diap_identity_{}", session_id);
        let entries = self.entries.lock().unwrap();
        
        if let Some(entry) = entries.get(&key) {
            Ok(entry.memory_type == MemoryType::DiapIdentity)
        } else {
            Ok(false)
        }
    }

    /// 存储记忆
    pub async fn store(&self, key: String, value: serde_json::Value, memory_type: MemoryType) -> Result<(), Box<dyn std::error::Error>> {
        let entry = MemoryEntry {
            key: key.clone(),
            value,
            memory_type,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() + self.ttl_seconds as i64,
        };

        let mut entries = self.entries.lock().unwrap();
        entries.insert(key, entry);
        Ok(())
    }

    /// 检索记忆
    pub async fn retrieve(&self, key: &str) -> Result<Option<MemoryEntry>, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.get(key).cloned())
    }

    /// 搜索记忆
    pub async fn search(&self, query: &str, memory_type: Option<MemoryType>) -> Result<Vec<MemoryEntry>, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();
        let mut results = Vec::new();

        for entry in entries.values() {
            // 检查类型过滤
            if let Some(ref filter_type) = memory_type {
                if entry.memory_type != *filter_type {
                    continue;
                }
            }

            // 检查是否过期
            if entry.expires_at < chrono::Utc::now().timestamp() {
                continue;
            }

            // 简单文本搜索
            let searchable_text = format!("{} {}", entry.key, entry.value.to_string());
            if searchable_text.to_lowercase().contains(&query.to_lowercase()) {
                results.push(entry.clone());
            }
        }

        Ok(results)
    }

    /// 删除记忆
    pub async fn delete(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock().unwrap();
        Ok(entries.remove(key).is_some())
    }

    /// 清理过期记忆
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        entries.retain(|_, entry| entry.expires_at >= now);
        Ok(())
    }

    /// 获取记忆数量
    pub async fn count(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.len())
    }

    /// 获取记忆条目数量
    pub async fn get_entry_count(&self) -> usize {
        let entries = self.entries.lock().unwrap();
        entries.len()
    }

    /// 搜索相似记忆
    pub async fn search_similar(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();
        let mut results = Vec::new();

        for entry in entries.values() {
            // 检查是否过期
            if entry.expires_at < chrono::Utc::now().timestamp() {
                continue;
            }

            // 简单文本搜索
            let searchable_text = format!("{} {}", entry.key, entry.value.to_string());
            if searchable_text.to_lowercase().contains(&query.to_lowercase()) {
                results.push(entry.clone());
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// 清除所有记忆
    pub async fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
        Ok(())
    }

    /// 获取所有记忆
    pub async fn all(&self) -> Result<Vec<MemoryEntry>, Box<dyn std::error::Error>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.values().cloned().collect())
    }
}

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryEntry {
    /// 键
    pub key: String,
    /// 值
    pub value: serde_json::Value,
    /// 记忆类型
    pub memory_type: MemoryType,
    /// 创建时间
    pub created_at: i64,
    /// 过期时间
    pub expires_at: i64,
}

/// 记忆类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MemoryType {
    /// 短期记忆
    #[default]
    ShortTerm,
    /// 长期记忆
    LongTerm,
    /// 工作记忆
    Working,
    /// 上下文记忆
    Contextual,
    /// DIAP身份记忆
    DiapIdentity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_store_and_retrieve() {
        let manager = MemoryManager::new(3600);

        let key = "test_key".to_string();
        let value = serde_json::json!({"message": "test data"});

        // 存储
        manager.store(key.clone(), value, MemoryType::ShortTerm).await.unwrap();

        // 检索
        let entry = manager.retrieve(&key).await.unwrap().unwrap();
        assert_eq!(entry.key, key);
        assert_eq!(entry.value["message"], "test data");
        assert_eq!(entry.memory_type, MemoryType::ShortTerm);
    }

    #[tokio::test]
    async fn test_memory_search() {
        let manager = MemoryManager::new(3600);

        // 存储多个条目
        manager.store("key1".to_string(), serde_json::json!({"text": "hello world"}), MemoryType::ShortTerm).await.unwrap();
        manager.store("key2".to_string(), serde_json::json!({"text": "foo bar"}), MemoryType::LongTerm).await.unwrap();

        // 搜索
        let results = manager.search("hello", None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "key1");

        // 按类型搜索
        let results = manager.search("bar", Some(MemoryType::LongTerm)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "key2");
    }

    #[tokio::test]
    async fn test_memory_cleanup() {
        let manager = MemoryManager::new(0); // 立即过期

        manager.store("key1".to_string(), serde_json::json!({"text": "test"}), MemoryType::ShortTerm).await.unwrap();

        // 清理前应该有1个条目
        assert_eq!(manager.count().await.unwrap(), 1);

        // 清理过期条目
        manager.cleanup().await.unwrap();

        // 清理后应该有0个条目
        assert_eq!(manager.count().await.unwrap(), 0);
    }
}
