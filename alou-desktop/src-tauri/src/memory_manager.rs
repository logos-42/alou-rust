/**
 * memory_manager.rs - Rust内存管理模块
 * 提供高性能的内存存储和管理功能
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// 内存存储项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub timestamp: u64,
    pub size: usize,
    pub expires_at: Option<u64>,
}

/// 内存存储统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_items: usize,
    pub total_size: usize,
    pub max_items: usize,
    pub usage_percentage: f64,
    pub expired_items: usize,
}

/// 内存管理器
#[derive(Debug)]
pub struct MemoryManager {
    items: Arc<Mutex<HashMap<String, MemoryItem>>>,
    max_items: usize,
    max_size_mb: usize,
}

impl MemoryManager {
    pub fn new(max_items: usize, max_size_mb: usize) -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
            max_items,
            max_size_mb,
        }
    }

    /// 存储数据
    pub fn set_item(&self, key: String, value: String) -> Result<(), String> {
        let mut items = self.items.lock().unwrap();
        
        // 检查容量限制
        if items.len() >= self.max_items {
            // LRU清理：移除最旧的项目
            let oldest_key = items
                .iter()
                .min_by_key(|(_, item)| item.timestamp)
                .map(|(k, _)| k.clone());
            if let Some(key) = oldest_key {
                items.remove(&key);
                println!("[MemoryManager] 容量已满，移除最旧项目: {}", key);
            }
        }

        let size = value.len();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;

        // 默认30天后自动过期
        let expires_at = now + (30 * 24 * 60 * 60); // 30天
        
        let item = MemoryItem {
            key: key.clone(),
            value,
            timestamp: now,
            size,
            expires_at: Some(expires_at), // 设置自动过期时间
        };

        items.insert(key.clone(), item);
        println!("[MemoryManager] 存储数据: {} (大小: {} 字节)", key, size);
        Ok(())
    }

    /// 获取数据
    pub fn get_item(&self, key: &str) -> Option<String> {
        let mut items = self.items.lock().unwrap();
        
        if let Some(item) = items.get(key) {
            // 检查是否过期
            if let Some(expires_at) = item.expires_at {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u64;
                if now > expires_at {
                    items.remove(key);
                    println!("[MemoryManager] 数据已过期，已删除: {}", key);
                    return None;
                }
            }
            
            println!("[MemoryManager] 读取数据: {} (存在: true)", key);
            Some(item.value.clone())
        } else {
            println!("[MemoryManager] 读取数据: {} (存在: false)", key);
            None
        }
    }

    /// 删除数据
    pub fn remove_item(&self, key: &str) -> bool {
        let mut items = self.items.lock().unwrap();
        let existed = items.remove(key).is_some();
        if existed {
            println!("[MemoryManager] 删除数据: {}", key);
        }
        existed
    }

    /// 清空所有数据
    pub fn clear(&self) {
        let mut items = self.items.lock().unwrap();
        let size = items.len();
        items.clear();
        println!("[MemoryManager] 清空所有数据，删除了 {} 个项目", size);
    }

    /// 获取所有键
    pub fn get_keys(&self) -> Vec<String> {
        let mut items = self.items.lock().unwrap();
        items.keys().cloned().collect()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> MemoryStats {
        let mut items = self.items.lock().unwrap();
        let total_items = items.len();
        let total_size: usize = items
            .values()
            .map(|item| item.size)
            .sum();
        
        let expired_items = items
            .values()
            .filter(|item| {
                if let Some(expires_at) = item.expires_at {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as u64;
                    now > expires_at
                } else {
                    false
                }
            })
            .count();

        let usage_percentage = if self.max_items > 0 {
            (total_items as f64 / self.max_items as f64) * 100.0
        } else {
            0.0
        };

        MemoryStats {
            total_items,
            total_size,
            max_items: self.max_items,
            usage_percentage,
            expired_items,
        }
    }

    /// 清理过期数据
    pub fn cleanup_expired(&self) -> usize {
        let mut items = self.items.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;

        let keys_to_remove: Vec<String> = items
            .iter()
            .filter(|(_, item)| {
                if let Some(expires_at) = item.expires_at {
                    now > expires_at
                } else {
                    false
                }
            })
            .map(|(k, _)| k.clone())
            .collect();

        let removed_count = keys_to_remove.len();
        for key in keys_to_remove {
            items.remove(&key);
        }

        if removed_count > 0 {
            println!("[MemoryManager] 清理过期数据，删除了 {} 个项目", removed_count);
        }

        removed_count
    }

    /// LRU清理：保留指定数量的项目
    pub fn cleanup_lru(&self, keep_count: usize) -> usize {
        let mut items = self.items.lock().unwrap();
        
        if items.len() <= keep_count {
            return 0;
        }

        // 按时间戳排序，保留最新的项目
        let mut sorted_items: Vec<_> = items
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        sorted_items.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));

        // 清空并重新插入保留的项目
        items.clear();
        let mut kept = 0;
        for (key, item) in sorted_items.iter().rev().take(keep_count) {
            items.insert(key.clone(), item.clone());
            kept += 1;
        }

        let removed = sorted_items.len() - kept;
        if removed > 0 {
            println!("[MemoryManager] LRU清理，保留 {} 个，删除 {} 个", keep_count, removed);
        }

        removed
    }

    /// 设置过期时间
    pub fn set_expiration(&self, key: &str, expires_in_seconds: u64) {
        let mut items = self.items.lock().unwrap();
        if let Some(item) = items.get_mut(key) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64;
            item.expires_at = Some(now + expires_in_seconds);
            println!("[MemoryManager] 设置过期时间: {} ({} 秒后)", key, expires_in_seconds);
        }
    }
}

/// Tauri命令：设置内存项
#[tauri::command]
pub fn set_memory_item(key: String, value: String) -> Result<(), String> {
    let manager = get_memory_manager();
    manager.set_item(key, value)
}

/// Tauri命令：获取内存项
#[tauri::command]
pub fn get_memory_item(key: String) -> Option<String> {
    let manager = get_memory_manager();
    manager.get_item(&key)
}

/// Tauri命令：删除内存项
#[tauri::command]
pub fn remove_memory_item(key: String) -> bool {
    let manager = get_memory_manager();
    manager.remove_item(&key)
}

/// Tauri命令：清空内存
#[tauri::command]
pub fn clear_memory() {
    let manager = get_memory_manager();
    manager.clear();
}

/// Tauri命令：获取所有键
#[tauri::command]
pub fn get_memory_keys() -> Vec<String> {
    let manager = get_memory_manager();
    manager.get_keys()
}

/// Tauri命令：获取统计信息
#[tauri::command]
pub fn get_memory_stats() -> MemoryStats {
    let manager = get_memory_manager();
    manager.get_stats()
}

/// Tauri命令：清理过期数据
#[tauri::command]
pub fn cleanup_expired_memory() -> usize {
    let manager = get_memory_manager();
    manager.cleanup_expired()
}

/// Tauri命令：LRU清理
#[tauri::command]
pub fn cleanup_lru_memory(keep_count: usize) -> usize {
    let manager = get_memory_manager();
    manager.cleanup_lru(keep_count)
}

/// Tauri命令：设置过期时间
#[tauri::command]
pub fn set_memory_expiration(key: String, expires_in_seconds: u64) {
    let manager = get_memory_manager();
    manager.set_expiration(&key, expires_in_seconds)
}

/// Tauri命令：定期清理（每周清理）
#[tauri::command]
pub fn schedule_weekly_cleanup() -> Result<String, String> {
    let manager = get_memory_manager();
    let cleaned = manager.cleanup_expired(); // 清理过期数据
    let lru_cleaned = manager.cleanup_lru(300); // 保留最近300个项目
    
    println!("[MemoryManager] 定期清理完成：过期数据 {} 个，LRU清理 {} 个", cleaned, lru_cleaned);
    
    Ok(format!("定期清理完成：清理了 {} 个项目", cleaned + lru_cleaned))
}

/// 获取全局内存管理器实例
fn get_memory_manager() -> &'static MemoryManager {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    static mut MANAGER: Option<MemoryManager> = None;
    
    INIT.call_once(|| {
        unsafe {
            MANAGER = Some(MemoryManager::new(
                1000,  // 最大1000个项目
                50    // 最大50MB
            ));
        }
    });
    
    unsafe { MANAGER.as_ref().unwrap() }
}
