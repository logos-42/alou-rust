//! 回滚工具 - 类型定义模块
//!
//! 定义快照、操作类型和相关数据结构

use serde::{Deserialize, Serialize};

/// 快照信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// 快照ID
    pub id: String,
    /// 快照名称/描述
    pub name: String,
    /// 创建时间戳
    pub created_at: i64,
    /// 原始路径
    pub source_path: String,
    /// 快照存储路径
    pub snapshot_path: String,
    /// 快照类型
    pub snapshot_type: SnapshotType,
    /// 文件数量（如果是目录）
    pub file_count: Option<usize>,
    /// 原始大小（字节）
    pub size_bytes: u64,
    /// 会话ID
    pub session_id: String,
    /// 标签（用于分类）
    pub tags: Vec<String>,
}

/// 快照类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotType {
    /// 单个文件
    File,
    /// 整个目录
    Directory,
    /// 批量多路径
    Batch,
}

/// 回滚操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RollbackAction {
    /// 创建快照
    CreateSnapshot {
        /// 目标路径（文件或目录）
        target_path: String,
        /// 快照名称/描述
        name: String,
        /// 标签（可选）
        #[serde(default)]
        tags: Vec<String>,
    },
    /// 批量创建快照
    CreateBatchSnapshot {
        /// 多个目标路径
        target_paths: Vec<String>,
        /// 快照名称前缀
        name_prefix: String,
    },
    /// 列出所有快照
    ListSnapshots {
        /// 按会话ID过滤（可选）
        #[serde(default)]
        session_id: Option<String>,
        /// 按标签过滤（可选）
        #[serde(default)]
        tags: Vec<String>,
    },
    /// 恢复快照
    RestoreSnapshot {
        /// 快照ID
        snapshot_id: String,
        /// 恢复到指定路径（可选，默认恢复到原路径）
        #[serde(default)]
        restore_path: Option<String>,
        /// 是否强制覆盖（默认false，如果目标存在则报错）
        #[serde(default)]
        force: bool,
    },
    /// 比较快照与当前状态
    CompareSnapshot {
        /// 快照ID
        snapshot_id: String,
    },
    /// 删除快照
    DeleteSnapshot {
        /// 快照ID
        snapshot_id: String,
    },
    /// 清理旧快照
    CleanupSnapshots {
        /// 保留最近N个快照
        #[serde(default)]
        keep_last: Option<usize>,
        /// 删除N天前的快照
        #[serde(default)]
        older_than_days: Option<i64>,
    },
    /// 获取快照详情
    GetSnapshotInfo {
        /// 快照ID
        snapshot_id: String,
    },
    /// 智能预检：为高风险操作自动创建快照
    AutoSnapshotBeforeOperation {
        /// 操作描述
        operation: String,
        /// 涉及的文件路径
        target_paths: Vec<String>,
        /// 操作风险等级（low/medium/high/critical）
        risk_level: String,
    },
}

/// 比较结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// 新增文件
    pub added: Vec<String>,
    /// 删除的文件
    pub removed: Vec<String>,
    /// 修改的文件
    pub modified: Vec<FileChange>,
    /// 未变动的文件
    pub unchanged: Vec<String>,
}

/// 文件变更详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub old_size: u64,
    pub new_size: u64,
    pub change_type: String,
}
