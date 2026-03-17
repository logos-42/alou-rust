//! PostgreSQL 目标存储后端
//!
//! 适用于：
//! - 长期可靠的数据持久化
//! - 复杂查询（按时间范围、标签、状态等）
//! - 数据分析与归档

use super::goal::{Goal, GoalId, GoalStorage, GoalStatus, Priority};
use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgRow};
use serde_json;
use std::sync::Arc;

/// PostgreSQL 目标存储
pub struct PostgresGoalStorage {
    pool: Arc<PgPool>,
    table_name: String,
}

impl PostgresGoalStorage {
    /// 创建新的 PostgreSQL 存储
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPool::connect(database_url).await?;

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
    async fn init_table(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR(64) PRIMARY KEY,
                parent_id VARCHAR(64),
                sub_goal_ids TEXT[] DEFAULT '{{}}',
                description TEXT NOT NULL,
                details TEXT,
                priority VARCHAR(16) NOT NULL,
                status VARCHAR(16) NOT NULL,
                progress FLOAT NOT NULL DEFAULT 0.0,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL,
                completed_at BIGINT,
                due_date BIGINT,
                agent_id VARCHAR(64),
                session_id VARCHAR(64),
                metadata JSONB DEFAULT '{{}}',
                progress_notes JSONB DEFAULT '[]',
                tags TEXT[] DEFAULT '{{}}'
            );

            -- 创建索引
            CREATE INDEX IF NOT EXISTS idx_{}_agent ON {}(agent_id);
            CREATE INDEX IF NOT EXISTS idx_{}_status ON {}(status);
            CREATE INDEX IF NOT EXISTS idx_{}_created ON {}(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_{}_tags ON {} USING GIN(tags);
            "#,
            self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
        );

        sqlx::query(&sql).execute(&*self.pool).await?;

        Ok(())
    }

    /// 执行数据库迁移（升级 schema）
    pub async fn migrate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 这里可以添加版本化的迁移逻辑
        log::info!("[PostgresGoalStorage] 数据库 schema 已初始化");
        Ok(())
    }
}

#[async_trait]
impl GoalStorage for PostgresGoalStorage {
    async fn save(&self, goal: &Goal) -> Result<(), Box<dyn std::error::Error>> {
        let sql = format!(
            r#"
            INSERT INTO {} (
                id, parent_id, sub_goal_ids, description, details, priority, status,
                progress, created_at, updated_at, completed_at, due_date,
                agent_id, session_id, metadata, progress_notes, tags
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (id) DO UPDATE SET
                parent_id = EXCLUDED.parent_id,
                sub_goal_ids = EXCLUDED.sub_goal_ids,
                description = EXCLUDED.description,
                details = EXCLUDED.details,
                priority = EXCLUDED.priority,
                status = EXCLUDED.status,
                progress = EXCLUDED.progress,
                updated_at = EXCLUDED.updated_at,
                completed_at = EXCLUDED.completed_at,
                due_date = EXCLUDED.due_date,
                agent_id = EXCLUDED.agent_id,
                session_id = EXCLUDED.session_id,
                metadata = EXCLUDED.metadata,
                progress_notes = EXCLUDED.progress_notes,
                tags = EXCLUDED.tags
            "#,
            self.table_name
        );

        sqlx::query(&sql)
            .bind(&goal.id)
            .bind(&goal.parent_id)
            .bind(&goal.sub_goal_ids)
            .bind(&goal.description)
            .bind(&goal.details)
            .bind(format!("{:?}", goal.priority))
            .bind(format!("{:?}", goal.status))
            .bind(goal.progress)
            .bind(goal.created_at)
            .bind(goal.updated_at)
            .bind(goal.completed_at)
            .bind(goal.due_date)
            .bind(&goal.agent_id)
            .bind(&goal.session_id)
            .bind(serde_json::to_value(&goal.metadata)?)
            .bind(serde_json::to_value(&goal.progress_notes)?)
            .bind(&goal.tags)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    async fn load(&self, goal_id: &GoalId) -> Result<Option<Goal>, Box<dyn std::error::Error>> {
        let sql = format!("SELECT * FROM {} WHERE id = $1", self.table_name);

        let row = sqlx::query(&sql)
            .bind(goal_id)
            .fetch_optional(&*self.pool)
            .await?;

        match row {
            Some(row) => {
                let goal = Self::row_to_goal(&row)?;
                Ok(Some(goal))
            }
            None => Ok(None),
        }
    }

    async fn load_all(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let sql = format!(
            "SELECT * FROM {} ORDER BY created_at DESC",
            self.table_name
        );

        let rows = sqlx::query(&sql).fetch_all(&*self.pool).await?;
        let goals = rows.iter().map(Self::row_to_goal).collect::<Result<Vec<_>, _>>()?;

        Ok(goals)
    }

    async fn delete(&self, goal_id: &GoalId) -> Result<(), Box<dyn std::error::Error>> {
        // 软删除：标记为 Cancelled 而不是物理删除
        let sql = format!(
            "UPDATE {} SET status = 'Cancelled', updated_at = $1 WHERE id = $2",
            self.table_name
        );

        sqlx::query(&sql)
            .bind(chrono::Utc::now().timestamp())
            .bind(goal_id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }
}

impl PostgresGoalStorage {
    /// 按 Agent ID 加载目标
    pub async fn load_by_agent(&self, agent_id: &str) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let sql = format!(
            "SELECT * FROM {} WHERE agent_id = $1 ORDER BY created_at DESC",
            self.table_name
        );

        let rows = sqlx::query(&sql)
            .bind(agent_id)
            .fetch_all(&*self.pool)
            .await?;

        let goals = rows.iter().map(Self::row_to_goal).collect::<Result<Vec<_>, _>>()?;
        Ok(goals)
    }

    /// 加载活跃目标
    pub async fn load_active(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let sql = format!(
            r#"SELECT * FROM {} WHERE status IN ('Active', 'Pending', 'Blocked') ORDER BY priority DESC, created_at DESC"#,
            self.table_name
        );

        let rows = sqlx::query(&sql).fetch_all(&*self.pool).await?;
        let goals = rows.iter().map(Self::row_to_goal).collect::<Result<Vec<_>, _>>()?;

        Ok(goals)
    }

    /// 按状态加载目标
    pub async fn load_by_status(&self, status: GoalStatus) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let sql = format!(
            "SELECT * FROM {} WHERE status = $1 ORDER BY created_at DESC",
            self.table_name
        );

        let rows = sqlx::query(&sql)
            .bind(format!("{:?}", status))
            .fetch_all(&*self.pool)
            .await?;

        let goals = rows.iter().map(Self::row_to_goal).collect::<Result<Vec<_>, _>>()?;
        Ok(goals)
    }

    /// 搜索目标
    pub async fn search(&self, query: &str) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let sql = format!(
            "SELECT * FROM {} WHERE description ILIKE $1 OR $2 = ANY(tags) ORDER BY created_at DESC",
            self.table_name
        );

        let rows = sqlx::query(&sql)
            .bind(format!("%{}%", query))
            .bind(query)
            .fetch_all(&*self.pool)
            .await?;

        let goals = rows.iter().map(Self::row_to_goal).collect::<Result<Vec<_>, _>>()?;
        Ok(goals)
    }

    /// 加载目标树（包含子目标）
    pub async fn load_goal_tree(&self, root_goal_id: &GoalId) -> Result<Option<(Goal, Vec<Goal>)>, Box<dyn std::error::Error>> {
        // 加载根目标
        let root = match self.load(root_goal_id).await? {
            Some(g) => g,
            None => return Ok(None),
        };

        // 加载所有子目标
        let sql = format!(
            "SELECT * FROM {} WHERE parent_id = $1",
            self.table_name
        );

        let rows = sqlx::query(&sql)
            .bind(root_goal_id)
            .fetch_all(&*self.pool)
            .await?;

        let children = rows.iter().map(Self::row_to_goal).collect::<Result<Vec<_>, _>>()?;

        Ok(Some((root, children)))
    }

    /// 归档已完成的目标（移动到归档表）
    pub async fn archive_completed(&self, older_than_days: i64) -> Result<u64, Box<dyn std::error::Error>> {
        let cutoff = chrono::Utc::now().timestamp() - older_than_days * 24 * 3600;

        // 创建归档表（如果不存在）
        let create_archive_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}_archive AS 
            SELECT * FROM {} WHERE 1=0
            "#,
            self.table_name, self.table_name
        );
        sqlx::query(&create_archive_sql).execute(&*self.pool).await?;

        // 移动数据
        let archive_sql = format!(
            r#"
            WITH moved AS (
                DELETE FROM {}
                WHERE status = 'Completed' AND completed_at < $1
                RETURNING *
            )
            INSERT INTO {}_archive SELECT * FROM moved
            "#,
            self.table_name, self.table_name
        );

        let result = sqlx::query(&archive_sql)
            .bind(cutoff)
            .execute(&*self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<GoalStats, Box<dyn std::error::Error>> {
        let sql = format!(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'Active') as active,
                COUNT(*) FILTER (WHERE status = 'Pending') as pending,
                COUNT(*) FILTER (WHERE status = 'Completed') as completed,
                COUNT(*) FILTER (WHERE status = 'Cancelled') as cancelled,
                AVG(progress) FILTER (WHERE status = 'Active') as avg_progress
            FROM {}
            "#,
            self.table_name
        );

        let row = sqlx::query(&sql).fetch_one(&*self.pool).await?;

        Ok(GoalStats {
            total: row.get("total"),
            active: row.get("active"),
            pending: row.get("pending"),
            completed: row.get("completed"),
            cancelled: row.get("cancelled"),
            avg_progress: row.get::<Option<f64>, _>("avg_progress").unwrap_or(0.0),
        })
    }

    /// 将数据库行转换为 Goal
    fn row_to_goal(row: &PgRow) -> Result<Goal, Box<dyn std::error::Error>> {
        let priority_str: String = row.try_get("priority")?;
        let status_str: String = row.try_get("status")?;

        Ok(Goal {
            id: row.try_get("id")?,
            parent_id: row.try_get("parent_id")?,
            sub_goal_ids: row.try_get("sub_goal_ids")?,
            description: row.try_get("description")?,
            details: row.try_get("details")?,
            priority: match priority_str.as_str() {
                "Critical" => Priority::Critical,
                "High" => Priority::High,
                "Low" => Priority::Low,
                _ => Priority::Medium,
            },
            status: match status_str.as_str() {
                "Pending" => GoalStatus::Pending,
                "Completed" => GoalStatus::Completed,
                "Paused" => GoalStatus::Paused,
                "Cancelled" => GoalStatus::Cancelled,
                "Blocked" => GoalStatus::Blocked { reason: String::new() },
                _ => GoalStatus::Active,
            },
            progress: row.try_get("progress")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            completed_at: row.try_get("completed_at")?,
            due_date: row.try_get("due_date")?,
            agent_id: row.try_get("agent_id")?,
            session_id: row.try_get("session_id")?,
            metadata: serde_json::from_value(row.try_get::<serde_json::Value, _>("metadata")?)?,
            progress_notes: serde_json::from_value(row.try_get::<serde_json::Value, _>("progress_notes")?)?,
            tags: row.try_get("tags")?,
        })
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
