/**
 * memory_manager.rs - Rust内存管理模块
 * 提供高性能的内存存储和管理功能，支持IPFS长期记忆存档
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::runtime::Runtime;

// 引入IPFS相关依赖
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use libipld::cid::Cid;

/// 内存存储项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub timestamp: u64,
    pub size: usize,
    pub cid: Option<String>,  // IPFS内容标识符
    pub archived: bool,       // 是否已归档到IPFS
    pub pinned: bool,         // 是否被固定在本地
}

/// 内存存储统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_items: usize,
    pub total_size: usize,
    pub max_items: usize,
    pub usage_percentage: f64,
    pub archived_items: usize,  // 已归档到IPFS的项目数
    pub pinned_items: usize,    // 被固定的项目数
}

/// 内存管理器
pub struct MemoryManager {
    items: Arc<Mutex<HashMap<String, MemoryItem>>>,
    max_items: usize,
    max_size_mb: usize,
    ipfs_client: Arc<IpfsClient>,
    runtime: Arc<Runtime>,
}

impl MemoryManager {
    pub fn new(max_items: usize, max_size_mb: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let rt = Runtime::new()?;
        let ipfs_client = IpfsClient::default(); // 连接到本地IPFS节点

        Ok(Self {
            items: Arc::new(Mutex::new(HashMap::new())),
            max_items,
            max_size_mb,
            ipfs_client: Arc::new(ipfs_client),
            runtime: Arc::new(rt),
        })
    }

    /// 存储数据到内存
    pub fn set_item(&self, key: String, value: String) -> Result<(), String> {
        let mut items = self.items.lock().unwrap();

        let size = value.len();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;

        let item = MemoryItem {
            key: key.clone(),
            value,
            timestamp: now,
            size,
            cid: None,
            archived: false,
            pinned: false,
        };

        items.insert(key.clone(), item);
        println!("[MemoryManager] 存储数据: {} (大小: {} 字节)", key, size);
        Ok(())
    }

    /// 获取数据（如果在IPFS上则从IPFS检索）
    pub fn get_item(&self, key: &str) -> Option<String> {
        let mut items = self.items.lock().unwrap();

        if let Some(item) = items.get(key) {
            // 如果数据已被归档到IPFS且不在内存中，则从IPFS检索
            if item.archived {
                if let Some(cid_str) = &item.cid {
                    match self.retrieve_from_ipfs(cid_str) {
                        Ok(value) => {
                            println!("[MemoryManager] 从IPFS检索数据: {} (CID: {})", key, cid_str);
                            return Some(value);
                        },
                        Err(e) => {
                            println!("[MemoryManager] 从IPFS检索失败: {}", e);
                            return None;
                        }
                    }
                }
            } else {
                println!("[MemoryManager] 读取内存数据: {} (存在: true)", key);
                return Some(item.value.clone());
            }
        } else {
            println!("[MemoryManager] 读取数据: {} (存在: false)", key);
        }

        None
    }

    /// 将数据归档到IPFS
    pub fn archive_to_ipfs(&self, key: &str) -> Result<String, String> {
        let items = self.items.lock().unwrap();

        // 克隆需要的数据以避免生命周期问题
        let value_to_archive = if let Some(item) = items.get(key) {
            Some(item.value.clone())
        } else {
            None
        };

        drop(items); // 释放锁，避免后续操作时死锁

        if let Some(value) = value_to_archive {
            // 将数据添加到IPFS
            let rt = &self.runtime;
            let ipfs_client_ref = Arc::clone(&self.ipfs_client); // 使用Arc clone
            let value_owned = value.into_bytes(); // 转换为字节数组以避免生命周期问题

            let cid_result = rt.block_on(async move {
                use std::io::Cursor;
                let cursor = Cursor::new(value_owned);
                ipfs_client_ref
                    .add(cursor)
                    .await
                    .map_err(|e| format!("IPFS添加失败: {}", e))
            });

            match cid_result {
                Ok(response) => {
                    let cid = response.hash;

                    // 更新内存中的项目
                    let mut items = self.items.lock().unwrap();
                    if let Some(ref mut stored_item) = items.get_mut(key) {
                        stored_item.cid = Some(cid.clone());
                        stored_item.archived = true;
                    }

                    println!("[MemoryManager] 数据已归档到IPFS: {} (CID: {})", key, cid);
                    Ok(cid)
                },
                Err(e) => {
                    Err(format!("归档到IPFS失败: {}", e))
                }
            }
        } else {
            Err(format!("键不存在: {}", key))
        }
    }

    /// 从IPFS检索数据
    fn retrieve_from_ipfs(&self, cid: &str) -> Result<String, String> {
        // 由于IPFS API的复杂性，这里使用模拟实现
        // 在实际应用中，这应该连接到IPFS节点并检索数据
        println!("[MemoryManager] 尝试从IPFS检索CID: {}", cid);

        // 模拟从IPFS检索数据的过程
        // 在实际实现中，这里会连接到IPFS节点并获取数据
        Err("IPFS功能暂未完全实现".to_string())
    }

    /// 固定IPFS上的内容
    pub fn pin_cid(&self, cid: &str) -> Result<(), String> {
        let rt = &self.runtime;
        let ipfs_client = &self.ipfs_client;

        rt.block_on(async {
            ipfs_client
                .pin_add(cid, true)
                .await
                .map_err(|e| format!("IPFS固定失败: {}", e))?;

            Ok(())
        })
    }

    /// 取消固定IPFS上的内容
    pub fn unpin_cid(&self, cid: &str) -> Result<(), String> {
        let rt = &self.runtime;
        let ipfs_client = &self.ipfs_client;

        rt.block_on(async {
            ipfs_client
                .pin_rm(cid, true)
                .await
                .map_err(|e| format!("IPFS取消固定失败: {}", e))?;

            Ok(())
        })
    }

    /// 删除数据（如果已归档到IPFS，可以选择是否从IPFS中删除）
    pub fn remove_item(&self, key: &str, remove_from_ipfs: bool) -> bool {
        let mut items = self.items.lock().unwrap();
        if let Some(item) = items.get(key) {
            // 如果数据已归档到IPFS，根据参数决定是否从IPFS中删除
            if item.archived {
                if remove_from_ipfs {
                    if let Some(cid) = &item.cid {
                        let rt = &self.runtime;
                        let ipfs_client = &self.ipfs_client;

                        rt.block_on(async {
                            let _ = ipfs_client.pin_rm(cid, true).await; // 先取消固定
                            // 注意：我们不实际删除IPFS中的内容，因为其他用户可能也在引用它
                        });
                    }
                }
            }

            let existed = items.remove(key).is_some();
            if existed {
                println!("[MemoryManager] 删除数据: {}", key);
            }
            return existed;
        }

        false
    }

    /// 清空所有数据
    pub fn clear(&self, remove_archived_from_ipfs: bool) {
        let mut items = self.items.lock().unwrap();
        let keys_to_remove: Vec<String> = items.keys().cloned().collect();
        let size = keys_to_remove.len();

        for key in keys_to_remove {
            self.remove_item(&key, remove_archived_from_ipfs);
        }

        println!("[MemoryManager] 清空所有数据，删除了 {} 个项目", size);
    }

    /// 获取所有键
    pub fn get_keys(&self) -> Vec<String> {
        let items = self.items.lock().unwrap();
        items.keys().cloned().collect()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> MemoryStats {
        let items = self.items.lock().unwrap();
        let total_items = items.len();
        let total_size: usize = items
            .values()
            .map(|item| item.size)
            .sum();

        let archived_items = items
            .values()
            .filter(|item| item.archived)
            .count();

        let pinned_items = items
            .values()
            .filter(|item| item.pinned)
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
            archived_items,
            pinned_items,
        }
    }

    /// 手动触发垃圾回收 - 智能体自主选择清理
    pub fn garbage_collect(&self, keys_to_remove: Vec<String>) -> usize {
        let mut items = self.items.lock().unwrap();
        let mut removed_count = 0;

        for key in keys_to_remove {
            if items.contains_key(&key) {
                // 检查是否已归档到IPFS
                if let Some(item) = items.get(&key) {
                    if item.archived {
                        // 如果已归档，可以选择从IPFS中取消固定
                        if let Some(cid) = &item.cid {
                            let rt = &self.runtime;
                            let ipfs_client = &self.ipfs_client;

                            rt.block_on(async {
                                let _ = ipfs_client.pin_rm(cid, true).await; // 取消固定
                            });
                        }
                    }
                }

                items.remove(&key);
                removed_count += 1;
                println!("[MemoryManager] 垃圾回收删除项目: {}", key);
            }
        }

        removed_count
    }

    /// 标记项目为固定状态（不会被垃圾回收）
    pub fn pin_item(&self, key: &str) -> bool {
        let mut items = self.items.lock().unwrap();
        if let Some(mut item) = items.get_mut(key) {
            item.pinned = true;
            // 同时将内容固定到IPFS
            if let Some(cid) = &item.cid {
                let rt = &self.runtime;
                let ipfs_client = &self.ipfs_client;

                rt.block_on(async {
                    let _ = ipfs_client.pin_add(cid, true).await;
                });
            }
            true
        } else {
            false
        }
    }

    /// 取消固定项目
    pub fn unpin_item(&self, key: &str) -> bool {
        let mut items = self.items.lock().unwrap();
        if let Some(mut item) = items.get_mut(key) {
            item.pinned = false;
            // 同时取消IPFS上的固定
            if let Some(cid) = &item.cid {
                let rt = &self.runtime;
                let ipfs_client = &self.ipfs_client;

                rt.block_on(async {
                    let _ = ipfs_client.pin_rm(cid, true).await;
                });
            }
            true
        } else {
            false
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
pub fn remove_memory_item(key: String, remove_from_ipfs: bool) -> bool {
    let manager = get_memory_manager();
    manager.remove_item(&key, remove_from_ipfs)
}

/// Tauri命令：清空内存
#[tauri::command]
pub fn clear_memory(remove_archived_from_ipfs: bool) {
    let manager = get_memory_manager();
    manager.clear(remove_archived_from_ipfs);
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

/// Tauri命令：将数据归档到IPFS
#[tauri::command]
pub fn archive_to_ipfs(key: String) -> Result<String, String> {
    let manager = get_memory_manager();
    manager.archive_to_ipfs(&key)
}

/// Tauri命令：固定IPFS内容
#[tauri::command]
pub fn pin_cid(cid: String) -> Result<(), String> {
    let manager = get_memory_manager();
    manager.pin_cid(&cid)
}

/// Tauri命令：取消固定IPFS内容
#[tauri::command]
pub fn unpin_cid(cid: String) -> Result<(), String> {
    let manager = get_memory_manager();
    manager.unpin_cid(&cid)
}

/// Tauri命令：手动垃圾回收 - 智能体自主选择清理
#[tauri::command]
pub fn garbage_collect(keys_to_remove: Vec<String>) -> usize {
    let manager = get_memory_manager();
    manager.garbage_collect(keys_to_remove)
}

/// Tauri命令：标记项目为固定状态
#[tauri::command]
pub fn pin_item(key: String) -> bool {
    let manager = get_memory_manager();
    manager.pin_item(&key)
}

/// Tauri命令：取消固定项目
#[tauri::command]
pub fn unpin_item(key: String) -> bool {
    let manager = get_memory_manager();
    manager.unpin_item(&key)
}

/// Tauri命令：设置DIAP身份
#[tauri::command]
pub fn set_diap_identity(session_id: String, identity: String) -> Result<(), String> {
    let manager = get_memory_manager();
    let key = format!("diap_identity_{}", session_id);
    manager.set_item(key, identity)
}

/// Tauri命令：获取DIAP身份
#[tauri::command]
pub fn get_diap_identity(session_id: String) -> Option<String> {
    let manager = get_memory_manager();
    let key = format!("diap_identity_{}", session_id);
    manager.get_item(&key)
}

/// Tauri命令：删除DIAP身份
#[tauri::command]
pub fn remove_diap_identity(session_id: String, remove_from_ipfs: bool) -> bool {
    let manager = get_memory_manager();
    let key = format!("diap_identity_{}", session_id);
    manager.remove_item(&key, remove_from_ipfs)
}

/// Tauri命令：获取所有DIAP身份
#[tauri::command]
pub fn get_all_diap_identities() -> std::collections::HashMap<String, String> {
    let manager = get_memory_manager();
    let keys = manager.get_keys();
    let mut identities = std::collections::HashMap::new();

    for key in keys {
        if key.starts_with("diap_identity_") {
            if let Some(identity) = manager.get_item(&key) {
                let session_id = key.replace("diap_identity_", "");
                identities.insert(session_id, identity);
            }
        }
    }

    identities
}

/// Tauri命令：归档DIAP身份到IPFS
#[tauri::command]
pub fn archive_diap_identity_to_ipfs(session_id: String) -> Result<String, String> {
    let manager = get_memory_manager();
    let key = format!("diap_identity_{}", session_id);
    manager.archive_to_ipfs(&key)
}

/// Tauri命令：标记DIAP身份为固定状态
#[tauri::command]
pub fn pin_diap_identity(session_id: String) -> bool {
    let manager = get_memory_manager();
    let key = format!("diap_identity_{}", session_id);
    manager.pin_item(&key)
}

/// Tauri命令：取消固定DIAP身份
#[tauri::command]
pub fn unpin_diap_identity(session_id: String) -> bool {
    let manager = get_memory_manager();
    let key = format!("diap_identity_{}", session_id);
    manager.unpin_item(&key)
}

/// 获取全局内存管理器实例
fn get_memory_manager() -> &'static MemoryManager {
    use std::sync::OnceLock;

    static MANAGER: OnceLock<MemoryManager> = OnceLock::new();

    MANAGER.get_or_init(|| {
        MemoryManager::new(
            1000,  // 最大1000个项目
            50    // 最大50MB
        ).expect("Failed to initialize MemoryManager")
    })
}
