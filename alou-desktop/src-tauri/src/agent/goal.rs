//! 目标管理系统 (Goal Management System)
//!
//! 提供长期目标跟踪能力，支持：
//! - 创建/更新/完成目标
//! - 子目标分解
//! - 跨会话目标持久化
//! - 进度追踪

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

/// 目标 ID
pub type GoalId = String;

/// 目标优先级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl Priority {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "low" => Priority::Low,
            _ => Priority::Medium,
        }
    }
}

/// 目标状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalStatus {
    /// 待处理
    Pending,
    /// 进行中
    Active,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 被阻塞（需要外部输入）
    Blocked { reason: String },
}

/// 目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// 目标 ID
    pub id: GoalId,
    /// 父目标 ID（如果有）
    pub parent_id: Option<GoalId>,
    /// 子目标 IDs
    pub sub_goal_ids: Vec<GoalId>,
    /// 目标描述
    pub description: String,
    /// 详细说明
    pub details: Option<String>,
    /// 优先级
    pub priority: Priority,
    /// 状态
    pub status: GoalStatus,
    /// 进度 (0.0 - 1.0)
    pub progress: f32,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 完成时间
    pub completed_at: Option<i64>,
    /// 截止日期
    pub due_date: Option<i64>,
    /// 关联的 Agent ID
    pub agent_id: Option<String>,
    /// 关联的 Session ID
    pub session_id: Option<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 进度笔记
    pub progress_notes: Vec<ProgressNote>,
    /// 标签
    pub tags: Vec<String>,
}

/// 进度笔记
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNote {
    pub timestamp: i64,
    pub content: String,
    pub progress_at_note: f32,
}

/// 目标追踪器
pub struct GoalTracker {
    /// 活跃目标存储（内存）
    goals: Arc<RwLock<HashMap<GoalId, Goal>>>,
    /// 存储后端（持久化）
    storage: Arc<dyn GoalStorage>,
}

impl GoalTracker {
    /// 创建新的目标追踪器
    pub fn new(storage: Arc<dyn GoalStorage>) -> Self {
        Self {
            goals: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    /// 创建目标
    pub async fn create_goal(
        &self,
        description: &str,
        priority: Priority,
        agent_id: Option<String>,
        session_id: Option<String>,
    ) -> Goal {
        let id = format!("goal_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
        let now = Utc::now().timestamp();

        let goal = Goal {
            id: id.clone(),
            parent_id: None,
            sub_goal_ids: vec![],
            description: description.to_string(),
            details: None,
            priority,
            status: GoalStatus::Active,
            progress: 0.0,
            created_at: now,
            updated_at: now,
            completed_at: None,
            due_date: None,
            agent_id,
            session_id,
            metadata: HashMap::new(),
            progress_notes: vec![],
            tags: vec!["auto_created".to_string()],
        };

        // 保存到内存
        {
            let mut goals = self.goals.write().await;
            goals.insert(id.clone(), goal.clone());
        }

        // 持久化
        if let Err(e) = self.storage.save(&goal).await {
            log::error!("[GoalTracker] 保存目标失败: {}", e);
        }

        log::info!("[GoalTracker] 创建目标: {} - {}", id, description);
        goal
    }

    /// 创建子目标
    pub async fn create_sub_goal(
        &self,
        parent_id: &GoalId,
        description: &str,
        priority: Priority,
    ) -> Result<Goal, GoalError> {
        // 检查父目标是否存在
        let parent_exists = {
            let goals = self.goals.read().await;
            goals.contains_key(parent_id)
        };

        if !parent_exists {
            return Err(GoalError::NotFound(parent_id.clone()));
        }

        let mut sub_goal = self.create_goal(
            description,
            priority,
            None,
            None,
        ).await;

        sub_goal.parent_id = Some(parent_id.to_string());

        // 更新父目标的子目标列表
        {
            let mut goals = self.goals.write().await;
            if let Some(parent) = goals.get_mut(parent_id) {
                parent.sub_goal_ids.push(sub_goal.id.clone());
            }
        }

        // 保存子目标
        if let Err(e) = self.storage.save(&sub_goal).await {
            log::error!("[GoalTracker] 保存子目标失败: {}", e);
        }

        log::info!(
            "[GoalTracker] 创建子目标: {} -> {}",
            parent_id,
            sub_goal.id
        );

        Ok(sub_goal)
    }

    /// 获取目标
    pub async fn get_goal(&self, goal_id: &GoalId) -> Option<Goal> {
        // 先从内存获取
        {
            let goals = self.goals.read().await;
            if let Some(goal) = goals.get(goal_id) {
                return Some(goal.clone());
            }
        }

        // 从存储获取
        match self.storage.load(goal_id).await {
            Ok(Some(goal)) => {
                // 缓存到内存
                let mut goals = self.goals.write().await;
                goals.insert(goal_id.to_string(), goal.clone());
                Some(goal)
            }
            _ => None,
        }
    }

    /// 更新目标进度
    pub async fn update_progress(
        &self,
        goal_id: &GoalId,
        progress: f32,
        note: Option<&str>,
    ) -> Result<(), GoalError> {
        let progress = progress.clamp(0.0, 1.0);
        let now = Utc::now().timestamp();

        {
            let mut goals = self.goals.write().await;
            let goal = goals.get_mut(goal_id)
                .ok_or_else(|| GoalError::NotFound(goal_id.clone()))?;

            goal.progress = progress;
            goal.updated_at = now;

            // 如果进度达到 100%，自动标记为完成
            if progress >= 1.0 && !matches!(goal.status, GoalStatus::Completed) {
                goal.status = GoalStatus::Completed;
                goal.completed_at = Some(now);
                log::info!("[GoalTracker] 目标自动完成: {}", goal_id);
            }

            // 添加进度笔记
            if let Some(note_content) = note {
                goal.progress_notes.push(ProgressNote {
                    timestamp: now,
                    content: note_content.to_string(),
                    progress_at_note: progress,
                });
            }
        }

        // 持久化
        if let Some(goal) = self.get_goal(goal_id).await {
            if let Err(e) = self.storage.save(&goal).await {
                log::error!("[GoalTracker] 更新进度失败: {}", e);
            }
        }

        // 更新父目标进度
        self.update_parent_progress(goal_id).await;

        Ok(())
    }

    /// 更新父目标进度
    async fn update_parent_progress(&self, goal_id: &GoalId) {
        let parent_id = {
            let goals = self.goals.read().await;
            goals.get(goal_id).and_then(|g| g.parent_id.clone())
        };

        if let Some(parent_id) = parent_id {
            let sub_goals = {
                let goals = self.goals.read().await;
                if let Some(parent) = goals.get(&parent_id) {
                    parent.sub_goal_ids.iter()
                        .filter_map(|id| goals.get(id).cloned())
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            };

            if !sub_goals.is_empty() {
                let avg_progress = sub_goals.iter().map(|g| g.progress).sum::<f32>() / sub_goals.len() as f32;
                // 直接更新而不递归调用 update_progress，避免递归
                let now = Utc::now().timestamp();
                {
                    let mut goals = self.goals.write().await;
                    if let Some(parent) = goals.get_mut(&parent_id) {
                        parent.progress = avg_progress;
                        parent.updated_at = now;
                    }
                }
                // 持久化
                if let Some(goal) = self.get_goal(&parent_id).await {
                    if let Err(e) = self.storage.save(&goal).await {
                        log::error!("[GoalTracker] 更新父目标进度失败：{}", e);
                    }
                }
                // 递归更新祖父目标
                Box::pin(self.update_parent_progress(&parent_id)).await;
            }
        }
    }

    /// 完成目标
    pub async fn complete_goal(&self, goal_id: &GoalId, final_note: Option<&str>) -> Result<(), GoalError> {
        let now = Utc::now().timestamp();

        {
            let mut goals = self.goals.write().await;
            let goal = goals.get_mut(goal_id)
                .ok_or_else(|| GoalError::NotFound(goal_id.clone()))?;

            goal.status = GoalStatus::Completed;
            goal.progress = 1.0;
            goal.completed_at = Some(now);
            goal.updated_at = now;

            if let Some(note) = final_note {
                goal.progress_notes.push(ProgressNote {
                    timestamp: now,
                    content: note.to_string(),
                    progress_at_note: 1.0,
                });
            }
        }

        // 持久化
        if let Some(goal) = self.get_goal(goal_id).await {
            if let Err(e) = self.storage.save(&goal).await {
                log::error!("[GoalTracker] 完成目标失败: {}", e);
            }
        }

        log::info!("[GoalTracker] 目标完成: {}", goal_id);
        Ok(())
    }

    /// 暂停目标
    pub async fn pause_goal(&self, goal_id: &GoalId, reason: &str) -> Result<(), GoalError> {
        {
            let mut goals = self.goals.write().await;
            let goal = goals.get_mut(goal_id)
                .ok_or_else(|| GoalError::NotFound(goal_id.clone()))?;

            goal.status = GoalStatus::Paused;
            goal.updated_at = Utc::now().timestamp();
            goal.progress_notes.push(ProgressNote {
                timestamp: Utc::now().timestamp(),
                content: format!("暂停: {}", reason),
                progress_at_note: goal.progress,
            });
        }

        if let Some(goal) = self.get_goal(goal_id).await {
            let _ = self.storage.save(&goal).await;
        }

        Ok(())
    }

    /// 取消目标
    pub async fn cancel_goal(&self, goal_id: &GoalId, reason: &str) -> Result<(), GoalError> {
        {
            let mut goals = self.goals.write().await;
            let goal = goals.get_mut(goal_id)
                .ok_or_else(|| GoalError::NotFound(goal_id.clone()))?;

            goal.status = GoalStatus::Cancelled;
            goal.updated_at = Utc::now().timestamp();
            goal.progress_notes.push(ProgressNote {
                timestamp: Utc::now().timestamp(),
                content: format!("取消: {}", reason),
                progress_at_note: goal.progress,
            });
        }

        if let Some(goal) = self.get_goal(goal_id).await {
            let _ = self.storage.save(&goal).await;
        }

        Ok(())
    }

    /// 获取活跃目标
    pub async fn get_active_goals(&self) -> Vec<Goal> {
        let goals = self.goals.read().await;
        goals.values()
            .filter(|g| matches!(g.status, GoalStatus::Active | GoalStatus::Pending | GoalStatus::Blocked { .. }))
            .cloned()
            .collect()
    }

    /// 获取指定 Agent 的活跃目标
    pub async fn get_active_goals_for_agent(&self, agent_id: &str) -> Vec<Goal> {
        let goals = self.goals.read().await;
        goals.values()
            .filter(|g| {
                g.agent_id.as_ref() == Some(&agent_id.to_string())
                    && matches!(g.status, GoalStatus::Active | GoalStatus::Pending | GoalStatus::Blocked { .. })
            })
            .cloned()
            .collect()
    }

    /// 获取所有目标（包括已完成）
    pub async fn get_all_goals(&self) -> Vec<Goal> {
        let goals = self.goals.read().await;
        goals.values().cloned().collect()
    }

    /// 搜索目标
    pub async fn search_goals(&self, keywords: &[String]) -> Vec<Goal> {
        let goals = self.goals.read().await;
        goals.values()
            .filter(|g| {
                keywords.iter().any(|kw| {
                    g.description.to_lowercase().contains(&kw.to_lowercase())
                        || g.tags.iter().any(|t| t.to_lowercase().contains(&kw.to_lowercase()))
                })
            })
            .cloned()
            .collect()
    }

    /// 获取目标摘要（用于显示）
    pub async fn get_goal_summary(&self, goal_id: &GoalId) -> Option<GoalSummary> {
        self.get_goal(goal_id).await.map(|g| GoalSummary {
            id: g.id,
            description: g.description,
            progress: g.progress,
            status: format!("{:?}", g.status),
            priority: format!("{:?}", g.priority),
        })
    }

    /// 加载所有目标（从存储）
    pub async fn load_all(&self) -> Result<(), GoalError> {
        match self.storage.load_all().await {
            Ok(goals) => {
                let mut map = self.goals.write().await;
                for goal in goals {
                    map.insert(goal.id.clone(), goal);
                }
                Ok(())
            }
            Err(e) => Err(GoalError::StorageError(e.to_string())),
        }
    }
}

/// 目标摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSummary {
    pub id: GoalId,
    pub description: String,
    pub progress: f32,
    pub status: String,
    pub priority: String,
}

/// 目标存储 trait
#[async_trait::async_trait]
pub trait GoalStorage: Send + Sync {
    async fn save(&self, goal: &Goal) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn load(&self, goal_id: &GoalId) -> Result<Option<Goal>, Box<dyn std::error::Error + Send + Sync>>;
    async fn load_all(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, goal_id: &GoalId) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 内存存储（用于测试）
pub struct InMemoryGoalStorage {
    data: Arc<RwLock<HashMap<GoalId, Goal>>>,
}

impl InMemoryGoalStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl GoalStorage for InMemoryGoalStorage {
    async fn save(&self, goal: &Goal) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self.data.write().await;
        data.insert(goal.id.clone(), goal.clone());
        Ok(())
    }

    async fn load(&self, goal_id: &GoalId) -> Result<Option<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        let data = self.data.read().await;
        Ok(data.get(goal_id).cloned())
    }

    async fn load_all(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        let data = self.data.read().await;
        Ok(data.values().cloned().collect())
    }

    async fn delete(&self, goal_id: &GoalId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut data = self.data.write().await;
        data.remove(goal_id);
        Ok(())
    }
}

/// 目标错误
#[derive(Debug)]
pub enum GoalError {
    NotFound(GoalId),
    StorageError(String),
    InvalidOperation(String),
}

impl std::fmt::Display for GoalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoalError::NotFound(id) => write!(f, "目标未找到: {}", id),
            GoalError::StorageError(msg) => write!(f, "存储错误: {}", msg),
            GoalError::InvalidOperation(msg) => write!(f, "无效操作: {}", msg),
        }
    }
}

impl std::error::Error for GoalError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_goal() {
        let storage = Arc::new(InMemoryGoalStorage::new());
        let tracker = GoalTracker::new(storage);

        let goal = tracker.create_goal(
            "测试目标",
            Priority::High,
            Some("agent_1".to_string()),
            Some("session_1".to_string()),
        ).await;

        assert_eq!(goal.description, "测试目标");
        assert_eq!(goal.priority, Priority::High);
        assert_eq!(goal.progress, 0.0);
        assert!(matches!(goal.status, GoalStatus::Active));
    }

    #[tokio::test]
    async fn test_update_progress() {
        let storage = Arc::new(InMemoryGoalStorage::new());
        let tracker = GoalTracker::new(storage);

        let goal = tracker.create_goal("测试", Priority::Medium, None, None).await;
        let _ = tracker.update_progress(&goal.id, 0.5, Some("一半完成")).await;

        let updated = tracker.get_goal(&goal.id).await.unwrap();
        assert_eq!(updated.progress, 0.5);
        assert_eq!(updated.progress_notes.len(), 1);
    }

    #[tokio::test]
    async fn test_auto_complete() {
        let storage = Arc::new(InMemoryGoalStorage::new());
        let tracker = GoalTracker::new(storage);

        let goal = tracker.create_goal("测试", Priority::Medium, None, None).await;
        let _ = tracker.update_progress(&goal.id, 1.0, None).await;

        let updated = tracker.get_goal(&goal.id).await.unwrap();
        assert!(matches!(updated.status, GoalStatus::Completed));
        assert!(updated.completed_at.is_some());
    }
}
