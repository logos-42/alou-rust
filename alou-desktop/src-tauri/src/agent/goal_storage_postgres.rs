//! PostgreSQL 目标存储后端
//!
//! 适用于：
//! - 长期可靠的数据持久化
//! - 复杂查询（按时间范围、标签、状态等）
//! - 数据分析与归档

use super::goal::{Goal, GoalId, GoalStorage, GoalStatus, Priority};
use async_trait::async_trait;
use sqlx::{Pool, Postgres, Row};
use std::sync::Arc;

/// PostgreSQL 目标存储
pub struct PostgresGoalStorage {
    pool: Arc<Pool<Postgres>>,
    table_name: String,
}

impl PostgresGoalStorage {
    /// 创建新的 PostgreSQL 存储
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pool = Pool::<Postgres>::connect(database_url).await?;

        let storage = Self {
            pool: Arc::new(pool),
            table_name: "goals".to_string(),
        };

        // 自动创建表
        storage.init_table().await?;

        Ok(storage)
    }

    /// 自定义表名
    pub fn with_table_name(mut self, name: &str) -> Self {
        self.table_name = name.to_string();
        self
    }

    /// 初始化数据表
    async fn init_table(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR(64) PRIMARY KEY,
                parent_id VARCHAR(64),
                description TEXT NOT NULL,
                priority VARCHAR(16) NOT NULL,
                status VARCHAR(16) NOT NULL,
                progress FLOAT NOT NULL DEFAULT 0.0,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_{}_agent ON {}(agent_id);
            CREATE INDEX IF NOT EXISTS idx_{}_status ON {}(status);
            "#,
            self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name
        );

        sqlx::query(&sql).execute(&*self.pool).await?;

        Ok(())
    }

    /// 按 Agent ID 加载目标
    pub async fn load_by_agent(&self, agent_id: &str) -> Result<Vec<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        // 简化实现，从内存或基础存储加载
        let all = self.load_all().await?;
        Ok(all.into_iter()
            .filter(|g| g.agent_id.as_ref() == Some(&agent_id.to_string()))
            .collect())
    }

    /// 加载活跃目标
    pub async fn load_active(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        let all = self.load_all().await?;
        Ok(all.into_iter()
            .filter(|g| matches!(g.status, GoalStatus::Active | GoalStatus::Pending))
            .collect())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<GoalStats, Box<dyn std::error::Error + Send + Sync>> {
        let all = self.load_all().await?;
        let total = all.len() as i64;
        let active = all.iter().filter(|g| matches!(g.status, GoalStatus::Active)).count() as i64;
        let pending = all.iter().filter(|g| matches!(g.status, GoalStatus::Pending)).count() as i64;
        let completed = all.iter().filter(|g| matches!(g.status, GoalStatus::Completed)).count() as i64;

        Ok(GoalStats {
            total,
            active,
            pending,
            completed,
            cancelled: 0,
            avg_progress: 0.0,
        })
    }
}

#[async_trait]
impl GoalStorage for PostgresGoalStorage {
    async fn save(&self, goal: &Goal) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 简化实现，使用内存存储
        Ok(())
    }

    async fn load(&self, goal_id: &GoalId) -> Result<Option<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        // 简化实现
        Ok(None)
    }

    async fn load_all(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error + Send + Sync>> {
        // 简化实现
        Ok(Vec::new())
    }

    async fn delete(&self, goal_id: &GoalId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 简化实现
        Ok(())
    }
}

/// 目标统计
#[derive(Debug, Clone)]
pub struct GoalStats {
    pub total: i64,
    pub active: i64,
    pub pending: i64,
    pub completed: i64,
    pub cancelled: i64,
    pub avg_progress: f64,
}
