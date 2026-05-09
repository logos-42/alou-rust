//! 工作流状态存储
//!
//! 提供工作流执行状态的持久化存储功能

use super::{WorkflowExecution, ExecutionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 存储配置
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// 存储目录
    pub storage_dir: PathBuf,
    /// 最大存储的执行数量
    pub max_executions: usize,
    /// 清理间隔（秒）
    pub cleanup_interval_secs: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_dir: PathBuf::from("./workflow_storage"),
            max_executions: 1000,
            cleanup_interval_secs: 3600, // 1小时
        }
    }
}

/// 存储的执行元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub execution_id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub file_path: PathBuf,
}

/// 工作流状态存储器
pub struct WorkflowStorage {
    config: StorageConfig,
    /// 元数据缓存
    metadata_cache: Arc<RwLock<HashMap<String, ExecutionMetadata>>>,
    /// 存储目录
    storage_dir: PathBuf,
}

impl WorkflowStorage {
    /// 创建新的存储器
    pub fn new(config: StorageConfig) -> Result<Self, String> {
        // 确保存储目录存在
        if !config.storage_dir.exists() {
            fs::create_dir_all(&config.storage_dir)
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        let storage = Self {
            config,
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            storage_dir: PathBuf::from("./workflow_storage"),
        };

        // 加载现有元数据
        storage.load_metadata_cache()?;

        Ok(storage)
    }

    /// 保存执行状态
    pub async fn save_execution(&self, execution: &WorkflowExecution) -> Result<(), String> {
        let file_name = format!("{}.json", execution.execution_id);
        let file_path = self.storage_dir.join(file_name);

        // 序列化执行状态
        let json = serde_json::to_string_pretty(execution)
            .map_err(|e| format!("Failed to serialize execution: {}", e))?;

        // 写入文件
        tokio::fs::write(&file_path, json).await
            .map_err(|e| format!("Failed to write execution file: {}", e))?;

        // 更新元数据缓存
        let metadata = ExecutionMetadata {
            execution_id: execution.execution_id.clone(),
            workflow_id: execution.workflow_id.clone(),
            status: execution.status.clone(),
            created_at: execution.started_at,
            updated_at: chrono::Utc::now().timestamp(),
            file_path: file_path.clone(),
        };

        let mut cache = self.metadata_cache.write().await;
        cache.insert(execution.execution_id.clone(), metadata);

        Ok(())
    }

    /// 加载执行状态
    pub async fn load_execution(&self, execution_id: &str) -> Result<Option<WorkflowExecution>, String> {
        let cache = self.metadata_cache.read().await;
        let metadata = match cache.get(execution_id) {
            Some(meta) => meta,
            None => return Ok(None),
        };

        if !metadata.file_path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&metadata.file_path).await
            .map_err(|e| format!("Failed to read execution file: {}", e))?;

        let execution: WorkflowExecution = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to deserialize execution: {}", e))?;

        Ok(Some(execution))
    }

    /// 删除执行状态
    pub async fn delete_execution(&self, execution_id: &str) -> Result<(), String> {
        let mut cache = self.metadata_cache.write().await;
        if let Some(metadata) = cache.remove(execution_id) {
            if metadata.file_path.exists() {
                tokio::fs::remove_file(&metadata.file_path).await
                    .map_err(|e| format!("Failed to delete execution file: {}", e))?;
            }
        }

        Ok(())
    }

    /// 列出所有执行
    pub async fn list_executions(&self, status_filter: Option<ExecutionStatus>) -> Result<Vec<ExecutionMetadata>, String> {
        let cache = self.metadata_cache.read().await;
        let mut executions: Vec<ExecutionMetadata> = cache.values().cloned().collect();

        // 按创建时间排序（最新的在前）
        executions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // 应用状态过滤
        if let Some(status) = status_filter {
            executions.retain(|e| e.status == status);
        }

        Ok(executions)
    }

    /// 清理旧的执行记录
    pub async fn cleanup_old_executions(&self) -> Result<(), String> {
        let cache = self.metadata_cache.read().await;
        let mut executions: Vec<ExecutionMetadata> = cache.values().cloned().collect();

        // 按更新时间排序（最旧的在前）
        executions.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));

        // 如果超过最大数量，删除最旧的
        if executions.len() > self.config.max_executions {
            let to_delete = executions.len() - self.config.max_executions;
            let delete_list: Vec<String> = executions.iter()
                .take(to_delete)
                .map(|e| e.execution_id.clone())
                .collect();

            drop(cache); // 释放读锁

            for execution_id in delete_list {
                self.delete_execution(&execution_id).await?;
            }
        }

        Ok(())
    }

    /// 获取存储统计信息
    pub async fn get_stats(&self) -> Result<StorageStats, String> {
        let cache = self.metadata_cache.read().await;

        let mut stats = StorageStats {
            total_executions: cache.len(),
            status_counts: HashMap::new(),
            total_size_bytes: 0,
        };

        for metadata in cache.values() {
            *stats.status_counts.entry(metadata.status.clone()).or_insert(0) += 1;

            if metadata.file_path.exists() {
                if let Ok(metadata_fs) = tokio::fs::metadata(&metadata.file_path).await {
                    stats.total_size_bytes += metadata_fs.len();
                }
            }
        }

        Ok(stats)
    }

    /// 加载元数据缓存
    fn load_metadata_cache(&self) -> Result<(), String> {
        let mut cache = HashMap::new();

        // 扫描存储目录中的所有JSON文件
        let entries = fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("Failed to read storage directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    let execution_id = file_name.to_string();

                    // 读取文件内容获取基本信息
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(execution) = serde_json::from_str::<WorkflowExecution>(&content) {
                            let metadata = ExecutionMetadata {
                                execution_id: execution.execution_id.clone(),
                                workflow_id: execution.workflow_id.clone(),
                                status: execution.status.clone(),
                                created_at: execution.started_at,
                                updated_at: execution.completed_at.unwrap_or(execution.started_at),
                                file_path: path,
                            };
                            cache.insert(execution_id, metadata);
                        }
                    }
                }
            }
        }

        // 更新缓存
        let mut metadata_cache = self.metadata_cache.try_write()
            .map_err(|_| "Failed to acquire metadata cache lock".to_string())?;
        *metadata_cache = cache;

        Ok(())
    }
}

/// 存储统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_executions: usize,
    pub status_counts: HashMap<ExecutionStatus, usize>,
    pub total_size_bytes: u64,
}

impl Default for WorkflowStorage {
    fn default() -> Self {
        Self::new(StorageConfig::default()).unwrap()
    }
}