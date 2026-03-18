//! 媒体存档管理器 - 共享模块
//!
//! 被 media_api.rs 和 media_tools.rs 共用

/// 媒体存档管理器
pub struct MediaArchiveManager {
    storage_dir: std::path::PathBuf,
}

impl MediaArchiveManager {
    pub fn new() -> Result<Self, String> {
        let storage_dir = dirs::config_dir()
            .ok_or_else(|| "Failed to get config directory".to_string())?
            .join("alou")
            .join("media_archives");
        
        std::fs::create_dir_all(&storage_dir)
            .map_err(|e| format!("Failed to create archive directory: {}", e))?;
        
        Ok(Self { storage_dir })
    }
    
    pub async fn archive(&self, archive_id: &str, data: &serde_json::Value) -> Result<(), String> {
        let file_path = self.storage_dir.join(format!("{}.json", archive_id.replace(":", "_")));
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize archive data: {}", e))?;
        
        tokio::fs::write(&file_path, json)
            .await
            .map_err(|e| format!("Failed to write archive file: {}", e))?;
        
        log::info!("媒体存档已保存：{} -> {:?}", archive_id, file_path);
        Ok(())
    }
    
    pub async fn get(&self, archive_id: &str) -> Result<Option<serde_json::Value>, String> {
        let file_path = self.storage_dir.join(format!("{}.json", archive_id.replace(":", "_")));
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let content = tokio::fs::read_to_string(&file_path)
            .await
            .map_err(|e| format!("Failed to read archive file: {}", e))?;
        
        let data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse archive data: {}", e))?;
        
        Ok(Some(data))
    }
    
    pub async fn list(&self, limit: usize) -> Result<Vec<(String, serde_json::Value)>, String> {
        use std::fs;
        
        let mut archives = Vec::new();
        let entries = fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("Failed to read archive directory: {}", e))?;
        
        for entry in entries {
            if archives.len() >= limit {
                break;
            }
            
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(mut data) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(id) = path.file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.replace("_", ":"))
                            {
                                data["archive_id"] = serde_json::json!(id);
                                archives.push((id, data));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(archives)
    }
}
