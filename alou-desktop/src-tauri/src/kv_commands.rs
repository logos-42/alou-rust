// KV commands module - 本地KV存储命令
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// KV存储状态
#[derive(Clone)]
pub struct KvState {
    data: Arc<Mutex<HashMap<String, String>>>,
    storage_path: PathBuf,
}

impl KvState {
    pub fn new() -> Self {
        let storage_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("alou-desktop")
            .join("kv_storage.json");

        // 确保目录存在
        if let Some(parent) = storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let data = Arc::new(Mutex::new(HashMap::new()));

        // 尝试从文件加载数据
        if storage_path.exists() {
            if let Ok(content) = fs::read_to_string(&storage_path) {
                if let Ok(loaded_data) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    let len = loaded_data.len();
                    *data.lock().unwrap() = loaded_data;
                    println!("[KV] 从文件加载了 {} 个项目", len);
                }
            }
        }

        Self { data, storage_path }
    }

    /// 保存数据到文件
    fn save_to_file(&self) -> Result<(), String> {
        let data = self.data.lock().unwrap();
        let json = serde_json::to_string_pretty(&*data)
            .map_err(|e| format!("序列化KV数据失败: {}", e))?;

        fs::write(&self.storage_path, json)
            .map_err(|e| format!("写入KV文件失败: {}", e))?;

        Ok(())
    }
}

/// 设置KV项
#[tauri::command]
pub async fn kv_set(
    key: String,
    value: String,
    kv_state: State<'_, KvState>,
) -> Result<(), String> {
    {
        let mut data = kv_state.data.lock().unwrap();
        data.insert(key.clone(), value);
    }

    // 保存到文件
    kv_state.save_to_file()?;

    println!("[KV] 设置项目: {}", key);
    Ok(())
}

/// 获取KV项
#[tauri::command]
pub async fn kv_get(
    key: String,
    kv_state: State<'_, KvState>,
) -> Result<Option<String>, String> {
    let data = kv_state.data.lock().unwrap();
    let value = data.get(&key).cloned();
    println!("[KV] 获取项目: {} (存在: {})", key, value.is_some());
    Ok(value)
}

/// 删除KV项
#[tauri::command]
pub async fn kv_remove(
    key: String,
    kv_state: State<'_, KvState>,
) -> Result<bool, String> {
    let existed = {
        let mut data = kv_state.data.lock().unwrap();
        data.remove(&key).is_some()
    };

    if existed {
        // 保存到文件
        kv_state.save_to_file()?;
        println!("[KV] 删除项目: {}", key);
    }

    Ok(existed)
}

/// 检查KV项是否存在
#[tauri::command]
pub async fn kv_exists(
    key: String,
    kv_state: State<'_, KvState>,
) -> Result<bool, String> {
    let data = kv_state.data.lock().unwrap();
    let exists = data.contains_key(&key);
    Ok(exists)
}

/// 获取所有KV键
#[tauri::command]
pub async fn kv_keys(
    kv_state: State<'_, KvState>,
) -> Result<Vec<String>, String> {
    let data = kv_state.data.lock().unwrap();
    let keys = data.keys().cloned().collect();
    Ok(keys)
}

/// 清空所有KV数据
#[tauri::command]
pub async fn kv_clear(
    kv_state: State<'_, KvState>,
) -> Result<(), String> {
    {
        let mut data = kv_state.data.lock().unwrap();
        data.clear();
    }

    // 保存到文件
    kv_state.save_to_file()?;

    println!("[KV] 清空所有数据");
    Ok(())
}

/// 获取KV统计信息
#[derive(Serialize)]
pub struct KvStats {
    pub total_items: usize,
    pub storage_path: String,
}

#[tauri::command]
pub async fn kv_stats(
    kv_state: State<'_, KvState>,
) -> Result<KvStats, String> {
    let data = kv_state.data.lock().unwrap();
    let stats = KvStats {
        total_items: data.len(),
        storage_path: kv_state.storage_path.to_string_lossy().to_string(),
    };
    Ok(stats)
}

/// 批量设置KV项
#[tauri::command]
pub async fn kv_set_batch(
    items: HashMap<String, String>,
    kv_state: State<'_, KvState>,
) -> Result<(), String> {
    let item_count = items.len();
    {
        let mut data = kv_state.data.lock().unwrap();
        for (key, value) in items {
            data.insert(key.clone(), value);
        }
    }

    // 保存到文件
    kv_state.save_to_file()?;

    println!("[KV] 批量设置了 {} 个项目", item_count);
    Ok(())
}

/// 批量获取KV项
#[tauri::command]
pub async fn kv_get_batch(
    keys: Vec<String>,
    kv_state: State<'_, KvState>,
) -> Result<HashMap<String, Option<String>>, String> {
    let data = kv_state.data.lock().unwrap();
    let mut result = HashMap::new();

    for key in keys {
        let value = data.get(&key).cloned();
        result.insert(key, value);
    }

    Ok(result)
}