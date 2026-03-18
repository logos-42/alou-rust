//! 混合目标存储（Redis + PostgreSQL）
//!
//! 架构：
//! - Redis: 活跃目标的快速读写 + 实时通知
//! - PostgreSQL: 完整持久化 + 复杂查询 + 历史归档

use super::goal::{Goal, GoalId, GoalStorage};
use super::goal_storage_postgres::PostgresGoalStorage;
use super::goal_storage_redis::RedisGoalStorage;
use std::sync::Arc;

/// 混合存储配置
#[derive(Debug, Clone)]
pub struct HybridStorageConfig {
    /// Redis TTL（秒）
    pub redis_ttl: usize,
    /// 自动归档天数（0 表示不自动归档）
    pub auto_archive_days: i64,
}

impl Default for HybridStorageConfig {
    fn default() -> Self {
        Self {
            redis_ttl: 7 * 24 * 3600, // 7 天
            auto_archive_days: 30,
        }
    }
}

/// 混合目标存储
pub struct HybridGoalStorage {
    /// PostgreSQL - 持久层
    postgres: Arc<PostgresGoalStorage>,
    /// Redis - 缓存层
    redis: Arc<RedisGoalStorage>,
    /// 配置
    config: HybridStorageConfig,
}

impl HybridGoalStorage {
    /// 创建新的混合存储
    pub async fn new(
        postgres: Arc<PostgresGoalStorage>,
        redis: Arc<RedisGoalStorage>,
    ) -> Self {
        Self {
            postgres,
            redis,
            config: HybridStorageConfig::default(),
        }
    }

    /// 自定义配置
    pub fn with_config(mut self, config: HybridStorageConfig) -> Self {
        self.config = config;
        self
    }

    /// 保存目标（写穿透模式）
    pub async fn save(&self, goal: &Goal) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. 先写 PostgreSQL（持久化）
        self.postgres.save(goal).await?;

        // 2. 再写 Redis（缓存）
        if let Err(e) = self.redis.save(goal).await {
            log::warn!("[HybridGoalStorage] Redis 写入失败（非关键）: {}", e);
        }

        Ok(())
    }

    /// 加载目标（读穿透模式）
    pub async fn load(&self, goal_id: &GoalId) -> Result<Option<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        // 1. 先读 Redis
        match self.redis.load(goal_id).await {
            Ok(Some(goal)) => {
                log::debug!("[HybridGoalStorage] Redis 命中: {}", goal_id);
                return Ok(Some(goal));
            }
            _ => {
                // 缓存未命中，继续读 PostgreSQL
            }
        }

        // 2. 读 PostgreSQL
        match self.postgres.load(goal_id).await? {
            Some(goal) => {
                log::debug!("[HybridGoalStorage] PostgreSQL 命中: {}", goal_id);
                
                // 回填 Redis（异步，不阻塞）
                let redis = self.redis.clone();
                let goal_clone = goal.clone();
                tokio::spawn(async move {
                    if let Err(e) = redis.save(&goal_clone).await {
                        log::debug!("[HybridGoalStorage] Redis 回填失败: {}", e);
                    }
                });

                Ok(Some(goal))
            }
            None => Ok(None),
        }
    }

    /// 加载所有目标
    pub async fn load_all(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        // 优先从 PostgreSQL 加载完整数据
        self.postgres.load_all().await
    }

    /// 加载活跃目标（优先 Redis）
    pub async fn load_active(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        // 从 PostgreSQL 加载
        let goals = self.postgres.load_active().await?;
        Ok(goals)
    }

    /// 删除目标
    pub async fn delete(&self, goal_id: &GoalId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 先删 PostgreSQL
        self.postgres.delete(goal_id).await?;

        // 再删 Redis
        if let Err(e) = self.redis.delete(goal_id).await {
            log::debug!("[HybridGoalStorage] Redis 删除失败: {}", e);
        }

        Ok(())
    }

    /// 更新进度
    pub async fn update_progress(
        &self,
        goal_id: &GoalId,
        progress: f32,
        note: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. 读取完整目标
        let mut goal = match self.load(goal_id).await? {
            Some(g) => g,
            None => return Err(format!("目标 {} 不存在", goal_id).into()),
        };

        // 2. 更新进度
        goal.progress = progress.clamp(0.0, 1.0);
        goal.updated_at = chrono::Utc::now().timestamp();

        // 3. 检查是否完成
        if goal.progress >= 1.0 {
            goal.status = super::goal::GoalStatus::Completed;
            goal.completed_at = Some(chrono::Utc::now().timestamp());
        }

        // 4. 保存
        self.save(&goal).await?;

        Ok(())
    }

    /// 预热缓存（从 PostgreSQL 加载活跃目标到 Redis）
    pub async fn warm_cache(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let active_goals = self.postgres.load_active().await?;
        let count = active_goals.len();

        for goal in active_goals {
            if let Err(e) = self.redis.save(&goal).await {
                log::warn!("[HybridGoalStorage] 缓存预热失败: {}", e);
            }
        }

        log::info!("[HybridGoalStorage] 缓存预热完成: {} 个目标", count);
        Ok(count)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<super::goal_storage_postgres::GoalStats, Box<dyn std::error::Error + Send + Sync>> {
        self.postgres.get_stats().await
    }
}

#[async_trait::async_trait]
impl super::goal::GoalStorage for HybridGoalStorage {
    async fn save(&self, goal: &super::goal::Goal) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.save(goal).await
    }

    async fn load(&self, goal_id: &super::goal::GoalId) -> Result<Option<super::goal::Goal>, Box<dyn std::error::Error + Send + Sync>> {
        self.load(goal_id).await
    }

    async fn load_all(&self) -> Result<Vec<super::goal::Goal>, Box<dyn std::error::Error + Send + Sync>> {
        self.load_all().await
    }

    async fn delete(&self, goal_id: &super::goal::GoalId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.delete(goal_id).await
    }
}
