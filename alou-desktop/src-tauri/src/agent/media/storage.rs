//! 媒体文件存储
//!
//! 保存到本地文件系统

use std::path::PathBuf;
use crate::agent::media_config::MediaApiConfig;

/// 保存媒体文件到本地
pub fn save_media(bytes: &[u8], media_type: &str, format: &str) -> Result<String, String> {
    // 获取存储目录
    let storage_dir = get_media_directory()?;
    
    // 创建目录
    std::fs::create_dir_all(&storage_dir)
        .map_err(|e| format!("Failed to create media directory: {}", e))?;
    
    // 生成文件名
    let filename = format!("{}_{}.{}", 
        media_type,
        chrono::Utc::now().timestamp(),
        format
    );
    
    let file_path = storage_dir.join(&filename);
    
    // 写入文件
    std::fs::write(&file_path, bytes)
        .map_err(|e| format!("Failed to write media file: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}

/// 获取媒体存储目录
pub fn get_media_directory() -> Result<PathBuf, String> {
    // 默认目录
    let mut path = dirs::home_dir()
        .ok_or_else(|| "Failed to get home directory".to_string())?;
    path.push("Alou");
    path.push("media");

    Ok(path)
}

/// 删除媒体文件
pub fn delete_media(file_path: &str) -> Result<(), String> {
    std::fs::remove_file(file_path)
        .map_err(|e| format!("Failed to delete media file: {}", e))
}

/// 获取媒体文件元数据
pub fn get_media_metadata(file_path: &str) -> Result<MediaFileMetadata, String> {
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    Ok(MediaFileMetadata {
        size_bytes: metadata.len(),
        created_at: metadata.created()
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
            .unwrap_or(0),
        modified_at: metadata.modified()
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
            .unwrap_or(0),
    })
}

#[derive(Debug, Clone)]
pub struct MediaFileMetadata {
    pub size_bytes: u64,
    pub created_at: i64,
    pub modified_at: i64,
}
