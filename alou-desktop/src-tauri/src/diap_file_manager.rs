/**
 * DIAP 文件管理器
 * 独立处理 DIAP 身份的文件存储
 * 存储路径: {app_data_dir}/alou-desktop/diap/agents/{agent-id}/diap.json
 */

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

/// 获取 agent DIAP 存储目录
fn get_agent_diap_dir(app_handle: &AppHandle, agent_id: &str) -> Result<PathBuf, String> {
    let app_data_dir = match app_handle.path().app_data_dir() {
        Ok(dir) => dir.join("alou-desktop"),
        Err(e) => {
            log::warn!("[DiapFileManager] 无法从 Tauri 获取应用数据目录：{}", e);
            match dirs::data_dir() {
                Some(dir) => dir.join("alou-desktop"),
                None => {
                    return Err("无法获取应用数据目录".to_string());
                }
            }
        }
    };

    let agent_diap_dir = app_data_dir.join("diap").join("agents").join(agent_id);
    log::info!("[DiapFileManager] Agent DIAP 目录: {:?}", agent_diap_dir);
    Ok(agent_diap_dir)
}

/// 保存 DIAP 身份到文件
fn save_identity_to_file(app_handle: &AppHandle, agent_id: &str, identity: &str) -> Result<(), String> {
    // 获取 agent 存储目录
    let agent_diap_dir = get_agent_diap_dir(app_handle, agent_id)?;

    // 创建目录
    if let Err(e) = fs::create_dir_all(&agent_diap_dir) {
        log::error!("[DiapFileManager] 创建目录失败：{} - {:?}", e, agent_diap_dir);
        return Err(format!("创建目录失败：{}", e));
    }

    // 写入文件
    let diap_file = agent_diap_dir.join("diap.json");
    log::info!("[DiapFileManager] 写入文件: {:?}", diap_file);
    let temp_file = diap_file.with_extension("json.tmp");

    match fs::File::create(&temp_file) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(identity.as_bytes()) {
                log::error!("[DiapFileManager] 写入文件失败：{}", e);
                let _ = fs::remove_file(&temp_file);
                return Err(format!("写入文件失败：{}", e));
            }
            let _ = file.sync_all();
        },
        Err(e) => {
            log::error!("[DiapFileManager] 创建文件失败：{}", e);
            return Err(format!("创建文件失败：{}", e));
        }
    }

    // 原子重命名
    if let Err(e) = fs::rename(&temp_file, &diap_file) {
        log::error!("[DiapFileManager] 重命名文件失败：{}", e);
        let _ = fs::remove_file(&temp_file);
        return Err(format!("保存文件失败：{}", e));
    }

    log::info!("[DiapFileManager] ✅ DIAP 身份已保存到文件：{:?}", diap_file);
    Ok(())
}

/// 从文件读取 DIAP 身份
fn load_identity_from_file(app_handle: &AppHandle, agent_id: &str) -> Option<String> {
    // 获取 agent 存储目录
    let agent_diap_dir = match get_agent_diap_dir(app_handle, agent_id) {
        Ok(dir) => dir,
        Err(e) => {
            log::warn!("[DiapFileManager] 获取目录失败：{}", e);
            return None;
        }
    };

    let diap_file = agent_diap_dir.join("diap.json");
    log::info!("[DiapFileManager] 读取文件: {:?}", diap_file);

    // 如果文件不存在
    if !diap_file.exists() {
        log::debug!("[DiapFileManager] 文件不存在：{:?}", diap_file);
        return None;
    }

    // 从文件读取
    match fs::read_to_string(&diap_file) {
        Ok(content) => {
            log::info!("[DiapFileManager] ✅ 从文件加载 DIAP 身份：{:?}", diap_file);
            Some(content)
        }
        Err(e) => {
            log::error!("[DiapFileManager] 读取文件失败：{} - {:?}", e, diap_file);
            None
        }
    }
}

/// 从文件删除 DIAP 身份
fn remove_identity_from_file(app_handle: &AppHandle, agent_id: &str) -> bool {
    // 获取 agent 存储目录
    let agent_diap_dir = match get_agent_diap_dir(app_handle, agent_id) {
        Ok(dir) => dir,
        Err(e) => {
            log::warn!("[DiapFileManager] 获取目录失败：{}", e);
            return false;
        }
    };

    let diap_file = agent_diap_dir.join("diap.json");

    // 删除文件
    if diap_file.exists() {
        match fs::remove_file(&diap_file) {
            Ok(_) => {
                log::info!("[DiapFileManager] ✅ 已删除 DIAP 身份文件：{:?}", diap_file);
                true
            }
            Err(e) => {
                log::error!("[DiapFileManager] 删除文件失败：{} - {:?}", e, diap_file);
                false
            }
        }
    } else {
        log::debug!("[DiapFileManager] 文件不存在，无需删除：{:?}", diap_file);
        true
    }
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// Tauri 命令：保存 DIAP 身份到文件
#[tauri::command]
pub fn set_diap_identity_for_agent(agent_id: String, identity: String) -> Result<(), String> {
    let app_handle = APP_HANDLE.get()
        .ok_or_else(|| "APP_HANDLE 未初始化".to_string())?;

    save_identity_to_file(&app_handle, &agent_id, &identity)
}

/// Tauri 命令：从文件读取 DIAP 身份
#[tauri::command]
pub fn get_diap_identity_for_agent(agent_id: String) -> Option<String> {
    let app_handle = APP_HANDLE.get()?;
    load_identity_from_file(&app_handle, &agent_id)
}

/// Tauri 命令：从文件删除 DIAP 身份
#[tauri::command]
pub fn remove_diap_identity_for_agent(agent_id: String, _remove_from_ipfs: bool) -> bool {
    let app_handle = match APP_HANDLE.get() {
        Some(h) => h,
        None => {
            log::warn!("[DiapFileManager] APP_HANDLE 未初始化");
            return false;
        }
    };

    remove_identity_from_file(&app_handle, &agent_id)
}

/// Tauri 命令：获取所有 DIAP 身份文件列表
#[tauri::command]
pub fn get_all_diap_identities_for_agent() -> std::collections::HashMap<String, String> {
    let app_handle = match APP_HANDLE.get() {
        Some(h) => h,
        None => {
            log::warn!("[DiapFileManager] APP_HANDLE 未初始化");
            return std::collections::HashMap::new();
        }
    };

    let app_data_dir = match app_handle.path().app_data_dir() {
        Ok(dir) => dir.join("alou-desktop").join("diap").join("agents"),
        Err(e) => {
            log::warn!("[DiapFileManager] 获取应用数据目录失败：{}", e);
            return std::collections::HashMap::new();
        }
    };

    let mut identities = std::collections::HashMap::new();

    // 遍历 agents 目录
    if let Ok(entries) = fs::read_dir(&app_data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(agent_id) = path.file_name().and_then(|n| n.to_str()) {
                    let diap_file = path.join("diap.json");
                    if diap_file.exists() {
                        if let Ok(content) = fs::read_to_string(&diap_file) {
                            identities.insert(agent_id.to_string(), content);
                        }
                    }
                }
            }
        }
    }

    log::info!("[DiapFileManager] 找到 {} 个 DIAP 身份文件", identities.len());
    identities
}

// 静态 APP_HANDLE 引用
use std::sync::OnceLock;
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// 初始化 APP_HANDLE
pub fn init_app_handle(app_handle: AppHandle) {
    let _ = APP_HANDLE.set(app_handle);
}


/// 启动时自动加载所有 DIAP 身份到内存
pub fn load_all_identities_to_memory(app_handle: &AppHandle) {
    log::info!("[DiapFileManager] 开始启动时自动加载所有 DIAP 身份...");
    
    let identities = get_all_diap_identities_for_agent();
    
    if identities.is_empty() {
        log::info!("[DiapFileManager] 未找到任何 DIAP 身份文件，跳过加载");
        return;
    }
    
    // 将每个身份加载到 memory_manager 的内存缓存中
    for (agent_id, identity_json) in identities.iter() {
        let memory_key = format!("diap_agent_{}", agent_id);
        
        // 使用 memory_manager 的 set_item 方法存储到内存
        let manager = crate::memory_manager::get_memory_manager();
        let _ = manager.set_item(memory_key, identity_json.clone());
        
        log::debug!("[DiapFileManager] ✅ 已加载 agent {} 的 DIAP 身份到内存", agent_id);
    }
    
    log::info!("[DiapFileManager] ✅ 启动时加载完成，共加载 {} 个 DIAP 身份", identities.len());
}
