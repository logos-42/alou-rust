//! 回滚工具 - 存储管理模块
//!
//! 负责快照的物理存储、恢复和比较

use super::types::{Snapshot, SnapshotType, ComparisonResult, FileChange};
use crate::tools::{ToolError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// 快照存储管理器
pub struct SnapshotStorage {
    storage_path: PathBuf,
}

impl SnapshotStorage {
    /// 创建新的存储管理器
    pub fn new() -> Self {
        let storage_path = Self::get_default_storage_path();
        Self { storage_path }
    }

    /// 获取默认存储路径
    fn get_default_storage_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".alou").join("snapshots")
    }

    /// 获取快照路径
    pub fn get_snapshot_path(&self, snapshot_id: &str) -> PathBuf {
        self.storage_path.join(snapshot_id)
    }

    /// 确保存储目录存在
    pub async fn ensure_storage(&self) -> Result<(), ToolError> {
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create storage: {}", e)))?;
        }
        Ok(())
    }

    /// 创建快照
    pub async fn create_snapshot(
        &self,
        source: &Path,
        snapshot_id: &str,
    ) -> Result<(u64, usize, SnapshotType), ToolError> {
        let snapshot_dir = self.storage_path.join(snapshot_id);
        fs::create_dir_all(&snapshot_dir).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create snapshot dir: {}", e)))?;

        let (size_bytes, file_count, snapshot_type) = if source.is_file() {
            let (size, count) = self.snapshot_file(source, &snapshot_dir).await?;
            (size, count, SnapshotType::File)
        } else {
            let (size, count) = self.snapshot_directory(source, &snapshot_dir).await?;
            (size, count, SnapshotType::Directory)
        };

        Ok((size_bytes, file_count, snapshot_type))
    }

    /// 创建单个文件快照
    async fn snapshot_file(&self, source: &Path, snapshot_dir: &Path) -> Result<(u64, usize), ToolError> {
        let file_name = source.file_name()
            .ok_or_else(|| ToolError::InvalidArguments("Invalid source path".to_string()))?;
        let dest = snapshot_dir.join(file_name);
        
        fs::copy(source, &dest)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to copy file: {}", e)))?;
        
        let metadata = fs::metadata(&dest).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get metadata: {}", e)))?;
        
        Ok((metadata.len(), 1))
    }

    /// 创建目录快照（递归复制）
    async fn snapshot_directory(&self, source: &Path, snapshot_dir: &Path) -> Result<(u64, usize), ToolError> {
        let mut total_size = 0u64;
        let mut file_count = 0usize;
        
        Self::copy_dir_recursive(source, snapshot_dir, &mut total_size, &mut file_count)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to snapshot directory: {}", e)))?;
        
        Ok((total_size, file_count))
    }

    /// 递归复制目录（使用迭代代替递归）
    async fn copy_dir_recursive(
        src: &Path,
        dst: &Path,
        total_size: &mut u64,
        file_count: &mut usize,
    ) -> std::io::Result<()> {
        fs::create_dir_all(dst).await?;

        // 使用栈代替递归
        let mut stack: Vec<(PathBuf, PathBuf)> = vec![(src.to_path_buf(), dst.to_path_buf())];

        while let Some((src_path, dst_path)) = stack.pop() {
            let mut entries = fs::read_dir(&src_path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let entry_src_path = entry.path();
                let entry_dst_path = dst_path.join(entry.file_name());

                let file_type = entry.file_type().await?;

                if file_type.is_dir() {
                    fs::create_dir_all(&entry_dst_path).await?;
                    stack.push((entry_src_path, entry_dst_path));
                } else if file_type.is_file() {
                    fs::copy(&entry_src_path, &entry_dst_path).await?;
                    let metadata = fs::metadata(&entry_dst_path).await?;
                    *total_size += metadata.len();
                    *file_count += 1;
                }
            }
        }

        Ok(())
    }

    /// 保存快照元数据
    pub async fn save_metadata(&self, snapshot_id: &str, snapshot: &Snapshot) -> Result<(), ToolError> {
        let snapshot_dir = self.storage_path.join(snapshot_id);
        let meta_path = snapshot_dir.join("_snapshot_meta.json");
        let meta_json = serde_json::to_string_pretty(snapshot).unwrap();
        fs::write(&meta_path, meta_json).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to save metadata: {}", e)))?;
        Ok(())
    }

    /// 恢复快照
    pub async fn restore_snapshot(
        &self,
        snapshot: &Snapshot,
        restore_path: &Path,
        force: bool,
    ) -> Result<(), ToolError> {
        if restore_path.exists() && !force {
            return Err(ToolError::ExecutionFailed(
                "Target path already exists. Use force=true to overwrite.".to_string()
            ));
        }

        let snapshot_path = PathBuf::from(&snapshot.snapshot_path);
        
        if !snapshot_path.exists() {
            return Err(ToolError::ExecutionFailed("Snapshot files not found".to_string()));
        }

        match snapshot.snapshot_type {
            SnapshotType::File => {
                if restore_path.exists() {
                    fs::remove_file(restore_path).await
                        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to remove existing: {}", e)))?;
                }
                fs::copy(&snapshot_path, restore_path).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to restore file: {}", e)))?;
            }
            SnapshotType::Directory => {
                if restore_path.exists() {
                    fs::remove_dir_all(restore_path).await
                        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to remove existing: {}", e)))?;
                }
                Self::copy_dir_recursive(&snapshot_path, restore_path, &mut 0, &mut 0).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to restore directory: {}", e)))?;
            }
            SnapshotType::Batch => {
                return Err(ToolError::ExecutionFailed("Batch restore not yet implemented".to_string()));
            }
        }

        Ok(())
    }

    /// 比较快照与当前状态
    pub async fn compare_with_current(&self, snapshot: &Snapshot) -> Result<ComparisonResult, ToolError> {
        let snapshot_path = PathBuf::from(&snapshot.snapshot_path);
        let current_path = PathBuf::from(&snapshot.source_path);
        
        let mut result = ComparisonResult {
            added: vec![],
            removed: vec![],
            modified: vec![],
            unchanged: vec![],
        };

        match snapshot.snapshot_type {
            SnapshotType::File => {
                self.compare_file(&snapshot_path, &current_path, "", &mut result).await?;
            }
            SnapshotType::Directory => {
                self.compare_directories(&snapshot_path, &current_path, "", &mut result).await?;
            }
            _ => {}
        }

        Ok(result)
    }

    async fn compare_file(
        &self,
        snapshot_path: &Path,
        current_path: &Path,
        rel_path: &str,
        result: &mut ComparisonResult,
    ) -> Result<(), ToolError> {
        let snap_exists = snapshot_path.exists();
        let curr_exists = current_path.exists();

        if snap_exists && !curr_exists {
            result.removed.push(rel_path.to_string());
        } else if !snap_exists && curr_exists {
            result.added.push(rel_path.to_string());
        } else if snap_exists && curr_exists {
            let snap_meta = fs::metadata(snapshot_path).await.unwrap();
            let curr_meta = fs::metadata(current_path).await.unwrap();
            
            if snap_meta.len() != curr_meta.len() {
                result.modified.push(FileChange {
                    path: rel_path.to_string(),
                    old_size: snap_meta.len(),
                    new_size: curr_meta.len(),
                    change_type: "size_changed".to_string(),
                });
            } else {
                result.unchanged.push(rel_path.to_string());
            }
        }

        Ok(())
    }

    async fn compare_directories(
        &self,
        snapshot_dir: &Path,
        current_dir: &Path,
        prefix: &str,
        result: &mut ComparisonResult,
    ) -> Result<(), ToolError> {
        let mut snapshot_files = HashMap::new();
        let mut current_files = HashMap::new();

        if snapshot_dir.exists() {
            Self::collect_files(snapshot_dir, prefix, &mut snapshot_files).await?;
        }
        if current_dir.exists() {
            Self::collect_files(current_dir, prefix, &mut current_files).await?;
        }

        for (path, size) in &snapshot_files {
            if let Some(&curr_size) = current_files.get(path) {
                if *size != curr_size {
                    result.modified.push(FileChange {
                        path: path.clone(),
                        old_size: *size,
                        new_size: curr_size,
                        change_type: "size_changed".to_string(),
                    });
                } else {
                    result.unchanged.push(path.clone());
                }
            } else {
                result.removed.push(path.clone());
            }
        }

        for path in current_files.keys() {
            if !snapshot_files.contains_key(path) {
                result.added.push(path.clone());
            }
        }

        Ok(())
    }

    /// 收集文件（使用迭代代替递归）
    async fn collect_files(dir: &Path, prefix: &str, files: &mut HashMap<String, u64>) -> Result<(), ToolError> {
        // 使用栈代替递归，存储 (目录路径, 相对前缀)
        let mut stack: Vec<(PathBuf, String)> = vec![(dir.to_path_buf(), prefix.to_string())];

        while let Some((current_dir, current_prefix)) = stack.pop() {
            let mut entries = fs::read_dir(&current_dir).await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read dir: {}", e)))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read entry: {}", e)))? {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let rel_path = if current_prefix.is_empty() { name.clone() } else { format!("{}/{}", current_prefix, name) };

                let file_type = entry.file_type().await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get file type: {}", e)))?;

                if file_type.is_dir() {
                    stack.push((path, rel_path));
                } else if file_type.is_file() {
                    let meta = entry.metadata().await
                        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get metadata: {}", e)))?;
                    files.insert(rel_path, meta.len());
                }
            }
        }

        Ok(())
    }
}
