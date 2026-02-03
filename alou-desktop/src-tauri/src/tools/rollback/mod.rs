//! 回滚工具 (Rollback Tool)
//!
//! 提供文件和目录的快照备份与恢复功能，防止智能体的鲁莽变动产生不可逆影响。
//!
//! 模块结构：
//! - types: 类型定义（Snapshot, RollbackAction等）
//! - storage: 快照存储管理
//! - executor: 工具执行器实现

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::fs;
use chrono::Utc;

pub mod types;
pub mod storage;

use types::{Snapshot, SnapshotType, RollbackAction, ComparisonResult, FileChange};
use storage::SnapshotStorage;

/// 回滚工具主结构
pub struct RollbackTool {
    metadata: ToolMetadata,
    storage: SnapshotStorage,
    snapshots: Arc<Mutex<HashMap<String, Vec<Snapshot>>>>,
}

impl RollbackTool {
    /// 创建新的回滚工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "rollback".to_string(),
                name: "Rollback Tool".to_string(),
                description: "文件备份与回滚系统，防止不可逆的变动".to_string(),
                category: ToolCategory::FileSystem,
                priority: ToolPriority::Critical,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
            },
            storage: SnapshotStorage::new(),
            snapshots: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 生成快照ID
    fn generate_snapshot_id(&self) -> String {
        format!("snap_{}_{}", Utc::now().timestamp_millis(), uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or(""))
    }
}

#[async_trait]
impl ToolExecutor for RollbackTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let action: RollbackAction = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        self.storage.ensure_storage().await?;

        match action {
            RollbackAction::CreateSnapshot { target_path, name, tags } => {
                self.handle_create_snapshot(target_path, name, tags, &context.session_id).await
            }
            RollbackAction::CreateBatchSnapshot { target_paths, name_prefix } => {
                self.handle_batch_snapshot(target_paths, name_prefix, &context.session_id).await
            }
            RollbackAction::ListSnapshots { session_id, tags } => {
                self.handle_list_snapshots(session_id, tags).await
            }
            RollbackAction::RestoreSnapshot { snapshot_id, restore_path, force } => {
                self.handle_restore_snapshot(snapshot_id, restore_path, force).await
            }
            RollbackAction::CompareSnapshot { snapshot_id } => {
                self.handle_compare_snapshot(snapshot_id).await
            }
            RollbackAction::DeleteSnapshot { snapshot_id } => {
                self.handle_delete_snapshot(snapshot_id).await
            }
            RollbackAction::CleanupSnapshots { keep_last, older_than_days } => {
                self.handle_cleanup(keep_last, older_than_days).await
            }
            RollbackAction::GetSnapshotInfo { snapshot_id } => {
                self.handle_get_info(snapshot_id).await
            }
            RollbackAction::AutoSnapshotBeforeOperation { operation, target_paths, risk_level } => {
                self.handle_auto_snapshot(operation, target_paths, risk_level, &context.session_id).await
            }
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if serde_json::from_value::<RollbackAction>(args.clone()).is_ok() {
            Ok(())
        } else {
            Err(ToolError::InvalidArguments("Invalid rollback action arguments".to_string()))
        }
    }

    fn help(&self) -> String {
        include_str!("./help.md").to_string()
    }
}

// 实现具体的处理函数
impl RollbackTool {
    async fn handle_create_snapshot(
        &self,
        target_path: String,
        name: String,
        tags: Vec<String>,
        session_id: &str,
    ) -> Result<ToolResult, ToolError> {
        let source = PathBuf::from(&target_path);
        
        if !source.exists() {
            return Err(ToolError::InvalidArguments(format!("Path does not exist: {}", target_path)));
        }

        let snapshot_id = self.generate_snapshot_id();
        let (size_bytes, file_count, snapshot_type) = self.storage
            .create_snapshot(&source, &snapshot_id).await?;
        
        let snapshot = Snapshot {
            id: snapshot_id.clone(),
            name,
            created_at: Utc::now().timestamp(),
            source_path: target_path.clone(),
            snapshot_path: self.storage.get_snapshot_path(&snapshot_id).to_string_lossy().to_string(),
            snapshot_type,
            file_count: Some(file_count),
            size_bytes,
            session_id: session_id.to_string(),
            tags,
        };

        // 保存到索引
        {
            let mut snapshots = self.snapshots.lock().unwrap();
            snapshots.entry(session_id.to_string())
                .or_insert_with(Vec::new)
                .push(snapshot.clone());
        }

        // 保存元数据文件
        self.storage.save_metadata(&snapshot_id, &snapshot).await?;

        Ok(ToolResult {
            success: true,
            data: serde_json::json!(snapshot),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Created snapshot '{}' for {}", snapshot_id, target_path)),
            warnings: vec![],
            context: None,
        })
    }

    async fn handle_batch_snapshot(
        &self,
        target_paths: Vec<String>,
        name_prefix: String,
        session_id: &str,
    ) -> Result<ToolResult, ToolError> {
        let mut created_snapshots = vec![];
        let mut errors = vec![];

        for (idx, path) in target_paths.iter().enumerate() {
            let name = format!("{} - Item {}", name_prefix, idx + 1);
            match self.handle_create_snapshot(path.clone(), name, vec!["batch".to_string()], session_id).await {
                Ok(result) => {
                    if let Ok(snap) = serde_json::from_value::<Snapshot>(result.data) {
                        created_snapshots.push(snap);
                    }
                }
                Err(e) => errors.push(format!("{}: {:?}", path, e)),
            }
        }

        let output = if errors.is_empty() {
            format!("Successfully created {} snapshots", created_snapshots.len())
        } else {
            format!("Created {} snapshots, {} errors", created_snapshots.len(), errors.len())
        };

        Ok(ToolResult {
            success: errors.is_empty() || !created_snapshots.is_empty(),
            data: serde_json::json!({
                "snapshots": created_snapshots,
                "errors": errors,
                "total_requested": target_paths.len(),
                "successful": created_snapshots.len(),
                "failed": errors.len(),
            }),
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            execution_time_ms: 0,
            output: Some(output),
            warnings: errors,
            context: None,
        })
    }

    async fn handle_list_snapshots(
        &self,
        session_filter: Option<String>,
        tag_filter: Vec<String>,
    ) -> Result<ToolResult, ToolError> {
        let snapshots = self.snapshots.lock().unwrap();
        
        let mut all_snapshots: Vec<&Snapshot> = vec![];
        
        for (session_id, snaps) in snapshots.iter() {
            if let Some(ref filter) = session_filter {
                if session_id != filter {
                    continue;
                }
            }
            
            for snap in snaps {
                if !tag_filter.is_empty() {
                    let has_tag = tag_filter.iter().any(|tag| snap.tags.contains(tag));
                    if !has_tag {
                        continue;
                    }
                }
                all_snapshots.push(snap);
            }
        }

        all_snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "snapshots": all_snapshots,
                "total": all_snapshots.len(),
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Found {} snapshots", all_snapshots.len())),
            warnings: vec![],
            context: None,
        })
    }

    async fn handle_restore_snapshot(
        &self,
        snapshot_id: String,
        restore_path: Option<String>,
        force: bool,
    ) -> Result<ToolResult, ToolError> {
        let snapshot = {
            let snapshots = self.snapshots.lock().unwrap();
            let mut found = None;
            for snaps in snapshots.values() {
                if let Some(snap) = snaps.iter().find(|s| s.id == snapshot_id) {
                    found = Some(snap.clone());
                    break;
                }
            }
            found
        };

        let snapshot = snapshot
            .ok_or_else(|| ToolError::InvalidArguments(format!("Snapshot not found: {}", snapshot_id)))?;

        let restore_to = restore_path.map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&snapshot.source_path));

        self.storage.restore_snapshot(&snapshot, &restore_to, force).await?;

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "snapshot_id": snapshot_id,
                "restored_to": restore_to.to_string_lossy().to_string(),
                "original_path": snapshot.source_path,
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Restored snapshot {} to {}", snapshot_id, restore_to.display())),
            warnings: vec![],
            context: None,
        })
    }

    async fn handle_compare_snapshot(&self, snapshot_id: String) -> Result<ToolResult, ToolError> {
        let snapshot = {
            let snapshots = self.snapshots.lock().unwrap();
            let mut found = None;
            for snaps in snapshots.values() {
                if let Some(snap) = snaps.iter().find(|s| s.id == snapshot_id) {
                    found = Some(snap.clone());
                    break;
                }
            }
            found
        };

        let snapshot = snapshot
            .ok_or_else(|| ToolError::InvalidArguments(format!("Snapshot not found: {}", snapshot_id)))?;

        let comparison = self.storage.compare_with_current(&snapshot).await?;

        Ok(ToolResult {
            success: true,
            data: serde_json::json!(comparison),
            error: None,
            execution_time_ms: 0,
            output: Some(format!(
                "Comparison: {} added, {} removed, {} modified, {} unchanged",
                comparison.added.len(),
                comparison.removed.len(),
                comparison.modified.len(),
                comparison.unchanged.len()
            )),
            warnings: vec![],
            context: None,
        })
    }

    async fn handle_delete_snapshot(&self, snapshot_id: String) -> Result<ToolResult, ToolError> {
        // 先在锁内找到快照并从内存中移除
        let snapshot_path_opt = {
            let mut snapshots = self.snapshots.lock().unwrap();
            let mut removed = false;
            let mut snapshot_path_opt = None;

            for snaps in snapshots.values_mut() {
                if let Some(pos) = snaps.iter().position(|s| s.id == snapshot_id) {
                    snapshot_path_opt = Some(snaps[pos].snapshot_path.clone());
                    snaps.remove(pos);
                    removed = true;
                    break;
                }
            }

            if !removed {
                return Err(ToolError::InvalidArguments(format!("Snapshot not found: {}", snapshot_id)));
            }

            snapshot_path_opt
        };

        // 释放锁后再执行异步操作
        if let Some(snapshot_path) = snapshot_path_opt {
            let path = PathBuf::from(snapshot_path);
            if path.exists() {
                let _ = fs::remove_dir_all(&path).await;
            }
        }

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({ "deleted": snapshot_id }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Deleted snapshot {}", snapshot_id)),
            warnings: vec![],
            context: None,
        })
    }

    async fn handle_cleanup(
        &self,
        keep_last: Option<usize>,
        older_than_days: Option<i64>,
    ) -> Result<ToolResult, ToolError> {
        // 先在锁内收集需要删除的快照和路径
        let mut paths_to_delete = vec![];
        let mut deleted_ids = vec![];
        let deleted_count = {
            let mut snapshots = self.snapshots.lock().unwrap();
            let now = Utc::now().timestamp();
            let day_seconds = 24 * 60 * 60;
            let mut deleted_count = 0;

            for snaps in snapshots.values_mut() {
                snaps.sort_by(|a, b| b.created_at.cmp(&a.created_at));

                if let Some(keep) = keep_last {
                    while snaps.len() > keep {
                        if let Some(snap) = snaps.pop() {
                            deleted_ids.push(snap.id.clone());
                            paths_to_delete.push(snap.snapshot_path.clone());
                            deleted_count += 1;
                        }
                    }
                }

                if let Some(days) = older_than_days {
                    snaps.retain(|snap| {
                        let age_days = (now - snap.created_at) / day_seconds;
                        if age_days > days {
                            deleted_ids.push(snap.id.clone());
                            paths_to_delete.push(snap.snapshot_path.clone());
                            deleted_count += 1;
                            false
                        } else {
                            true
                        }
                    });
                }
            }

            deleted_count
        };

        // 释放锁后再执行异步删除
        for path in paths_to_delete {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                let _ = fs::remove_dir_all(&path_buf).await;
            }
        }

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "deleted_count": deleted_count,
                "deleted_ids": deleted_ids,
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Cleaned up {} snapshots", deleted_count)),
            warnings: vec![],
            context: None,
        })
    }

    async fn handle_get_info(&self, snapshot_id: String) -> Result<ToolResult, ToolError> {
        let snapshots = self.snapshots.lock().unwrap();
        
        for snaps in snapshots.values() {
            if let Some(snap) = snaps.iter().find(|s| s.id == snapshot_id) {
                return Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(snap),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Snapshot: {} - {}", snap.id, snap.name)),
                    warnings: vec![],
                    context: None,
                });
            }
        }

        Err(ToolError::InvalidArguments(format!("Snapshot not found: {}", snapshot_id)))
    }

    async fn handle_auto_snapshot(
        &self,
        operation: String,
        target_paths: Vec<String>,
        risk_level: String,
        session_id: &str,
    ) -> Result<ToolResult, ToolError> {
        let should_snapshot = match risk_level.as_str() {
            "critical" => true,
            "high" => true,
            "medium" => target_paths.len() > 3,
            "low" => false,
            _ => true,
        };

        if !should_snapshot {
            return Ok(ToolResult {
                success: true,
                data: serde_json::json!({
                    "skipped": true,
                    "reason": "Risk level too low for auto-snapshot",
                }),
                error: None,
                execution_time_ms: 0,
                output: Some("Auto-snapshot skipped (low risk)".to_string()),
                warnings: vec![],
                context: None,
            });
        }

        let name_prefix = format!("[Auto] Before: {}", operation);
        self.handle_batch_snapshot(target_paths, name_prefix, session_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rollback_tool_creation() {
        let tool = RollbackTool::new();
        assert_eq!(tool.metadata().id, "rollback");
    }

    #[tokio::test]
    async fn test_snapshot_id_generation() {
        let tool = RollbackTool::new();
        let id1 = tool.generate_snapshot_id();
        let id2 = tool.generate_snapshot_id();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("snap_"));
    }
}
