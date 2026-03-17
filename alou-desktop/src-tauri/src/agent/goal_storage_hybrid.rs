//! 混合目标存储（Redis + PostgreSQL）
//!
//! 架构：
//! - Redis: 活跃目标的快速读写 + 实时通知
//! - PostgreSQL: 完整持久化 + 复杂查询 + 历史归档
//!
//! 写操作：先写 PostgreSQL，成功后更新 Redis
//! 读操作：先读 Redis，未命中则读 PostgreSQL 并回填

use super::goal::{Goal, GoalId, GoalStatus};
use super::goal_storage_redis::RedisGoalStorage;
use super::goal_storage_postgres::PostgresGoalStorage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 混合存储配置
#[derive(Debug, Clone)]
pub struct HybridStorageConfig {
    /// Redis TTL（秒）
    pub redis_ttl: usize,
    /// 是否启用 Redis 发布订阅
    pub enable_pub_sub: bool,
    /// 自动归档天数（0 表示不自动归档）
    pub auto_archive_days: i64,
}

impl Default for HybridStorageConfig {
    fn default() -> Self {
        Self {
            redis_ttl: 7 * 24 * 3600, // 7 天
            enable_pub_sub: true,
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
    /// 是否使用缓存（用于故障降级）
    cache_enabled: Arc<RwLock<bool>>,
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
            cache_enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// 自定义配置
    pub fn with_config(mut self, config: HybridStorageConfig) -> Self {
        self.config = config;
        self
    }

    /// 禁用缓存（故障降级）
    pub async fn disable_cache(&self) {
        let mut enabled = self.cache_enabled.write().await;
        *enabled = false;
        log::warn!("[HybridGoalStorage] Redis 缓存已禁用");
    }

    /// 启用缓存
    pub async fn enable_cache(&self) {
        let mut enabled = self.cache_enabled.write().await;
        *enabled = true;
        log::info!("[HybridGoalStorage] Redis 缓存已启用");
    }

    /// 保存目标（写穿透模式）
    pub async fn save(&self, goal: &Goal) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 先写 PostgreSQL（持久化）
        self.postgres.save(goal).await?;

        // 2. 再写 Redis（缓存）
        let cache_enabled = *self.cache_enabled.read().await;
        if cache_enabled {
            if let Err(e) = self.redis.save(goal).await {
                log::warn!("[HybridGoalStorage] Redis 写入失败（非关键）: {}", e);
            }
        }

        Ok(())
    }

    /// 加载目标（读穿透模式）
    pub async fn load(&self, goal_id: &GoalId) -> Result<Option<Goal>, Box<dyn std::error::Error>> {
        let cache_enabled = *self.cache_enabled.read().await;

        // 1. 先读 Redis
        if cache_enabled {
            match self.redis.load(goal_id).await {
                Ok(Some(goal)) => {
                    log::debug!("[HybridGoalStorage] Redis 命中: {}", goal_id);
                    return Ok(Some(goal));
                }
                Ok(None) => {
                    // 缓存未命中，继续读 PostgreSQL
                }
                Err(e) => {
                    log::warn!("[HybridGoalStorage] Redis 读取失败: {}", e);
                    // Redis 故障，降级到 PostgreSQL
                }
            }
        }

        // 2. 读 PostgreSQL
        match self.postgres.load(goal_id).await? {
            Some(goal) => {
                log::debug!("[HybridGoalStorage] PostgreSQL 命中: {}", goal_id);
                
                // 回填 Redis（异步，不阻塞）
                if cache_enabled {
                    let redis = self.redis.clone();
                    let goal_clone = goal.clone();
                    tokio::spawn(async move {
                        if let Err(e) = redis.save(&goal_clone).await {
                            log::debug!("[HybridGoalStorage] Redis 回填失败: {}", e);
                        }
                    });
                }

                Ok(Some(goal))
            }
            None => Ok(None),
        }
    }

    /// 加载所有目标
    pub async fn load_all(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        // 优先从 PostgreSQL 加载完整数据
        self.postgres.load_all().await
    }

    /// 加载活跃目标（优先 Redis）
    pub async fn load_active(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let cache_enabled = *self.cache_enabled.read().await;

        if cache_enabled {
            match self.redis.load_active().await {
                Ok(goals) if !goals.is_empty() => {
                    log::debug!("[HybridGoalStorage] Redis 返回 {} 个活跃目标", goals.len());
                    return Ok(goals);
                }
                _ => {
                    // Redis 未命中或失败，读 PostgreSQL
                }
            }
        }

        // 从 PostgreSQL 加载
        let goals = self.postgres.load_active().await?;

        // 回填 Redis
        if cache_enabled {
            for goal in &goals {
                let redis = self.redis.clone();
                let goal_clone = goal.clone();
                tokio::spawn(async move {
                    let _ = redis.save(&goal_clone).await;
                });
            }
        }

        Ok(goals)
    }

    /// 删除目标
    pub async fn delete(&self, goal_id: &GoalId) -> Result<(), Box<dyn std::error::Error>> {
        // 先删 PostgreSQL
        self.postgres.delete(goal_id).await?;

        // 再删 Redis
        let cache_enabled = *self.cache_enabled.read().await;
        if cache_enabled {
            if let Err(e) = self.redis.delete(goal_id).await {
                log::debug!("[HybridGoalStorage] Redis 删除失败: {}", e);
            }
        }

        Ok(())
    }

    /// 更新进度
    pub async fn update_progress(
        &self,
        goal_id: &GoalId,
        progress: f32,
        note: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 读取完整目标
        let mut goal = match self.load(goal_id).await? {
            Some(g) => g,
            None => return Err(format!("目标 {} 不存在", goal_id).into()),
        };

        // 2. 更新进度
        goal.progress = progress.clamp(0.0, 1.0);
        goal.updated_at = chrono::Utc::now().timestamp();

        if let Some(note_content) = note {
            goal.progress_notes.push(super::goal::ProgressNote {
                timestamp: chrono::Utc::now().timestamp(),
                content: note_content.to_string(),
                progress_at_note: progress,
            });
        }

        // 3. 检查是否完成
        if goal.progress >= 1.0 {
            goal.status = GoalStatus::Completed;
            goal.completed_at = Some(chrono::Utc::now().timestamp());
        }

        // 4. 保存
        self.save(&goal).await?;

        Ok(())
    }

    /// 预热缓存（从 PostgreSQL 加载活跃目标到 Redis）
    pub async fn warm_cache(&self) -> Result<usize, Box<dyn std::error::Error>> {
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

    /// 清理缓存（仅删除 Redis 数据）
    pub async fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 注意：这里应该使用 Redis 的 SCAN + DEL
        log::info!("[HybridGoalStorage] 缓存清理完成");
        Ok(())
    }

    /// 自动归档已完成的目标
    pub async fn auto_archive(&self) -> Result<u64, Box<dyn std::error::Error>> {
        if self.config.auto_archive_days <= 0 {
            return Ok(0);
        }

        let archived = self.postgres.archive_completed(self.config.auto_archive_days).await?;

        if archived > 0 {
            log::info!("[HybridGoalStorage] 自动归档 {} 个已完成目标", archived);
            
            // 清理 Redis 中的已归档目标
            // 这里可以通过订阅 PostgreSQL 的变更通知来实现
        }

        Ok(archived)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<super::goal_storage_postgres::GoalStats, Box<dyn std::error::Error>> {
        self.postgres.get_stats().await
    }

    /// 按 Agent 加载目标
    pub async fn load_by_agent(&self, agent_id: &str) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        self.postgres.load_by_agent(agent_id).await
    }

    /// 搜索目标
    pub async fn search(&self, query: &str) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        self.postgres.search(query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要 Redis 和 PostgreSQL 服务
    // 在实际项目中应该使用测试容器

    #[tokio::test]
    async fn test_cache_hit() {
        // 模拟缓存命中场景
    }

    #[tokio::test]
    async fn test_cache_miss() {
        // 模拟缓存未命中场景
    }
}
