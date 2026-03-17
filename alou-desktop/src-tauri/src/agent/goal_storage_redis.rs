//! Redis 目标存储后端
//!
//! 适用于：
//! - 活跃目标的快速读写
//! - 目标状态变更的实时通知
//! - 自动过期的临时目标

use super::goal::{Goal, GoalId, GoalStatus, GoalStorage};
use async_trait::async_trait;
use redis::{AsyncCommands, Client, aio::MultiplexedConnection};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Redis 目标存储
pub struct RedisGoalStorage {
    conn: Arc<RwLock<MultiplexedConnection>>,
    ///  key 前缀
    key_prefix: String,
    /// 默认 TTL (秒)
    default_ttl: Option<usize>,
}

impl RedisGoalStorage {
    /// 创建新的 Redis 存储
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            conn: Arc::new(RwLock::new(conn)),
            key_prefix: "goal:".to_string(),
            default_ttl: Some(7 * 24 * 3600), // 默认 7 天过期
        })
    }

    /// 设置 key 前缀
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.key_prefix = prefix.to_string();
        self
    }

    /// 设置默认 TTL
    pub fn with_ttl(mut self, ttl_seconds: usize) -> Self {
        self.default_ttl = Some(ttl_seconds);
        self
    }

    /// 禁用 TTL
    pub fn without_ttl(mut self) -> Self {
        self.default_ttl = None;
        self
    }

    fn make_key(&self, goal_id: &GoalId) -> String {
        format!("{}{}", self.key_prefix, goal_id)
    }

    fn make_index_key(&self, agent_id: &str) -> String {
        format!("{}index:agent:{}", self.key_prefix, agent_id)
    }

    fn make_active_key(&self) -> String {
        format!("{}index:active", self.key_prefix)
    }

    /// 按 Agent 加载目标
    pub async fn load_by_agent(&self, agent_id: &str) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let index_key = self.make_index_key(agent_id);
        let mut conn = self.conn.write().await;

        let goal_ids: Vec<String> = conn.smembers(&index_key).await?;
        let mut goals = Vec::new();

        for goal_id in goal_ids {
            if let Ok(Some(goal)) = self.load(&goal_id).await {
                goals.push(goal);
            }
        }

        Ok(goals)
    }

    /// 加载活跃目标
    pub async fn load_active(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let active_key = self.make_active_key();
        let mut conn = self.conn.write().await;

        let goal_ids: Vec<String> = conn.smembers(&active_key).await?;
        let mut goals = Vec::new();

        for goal_id in goal_ids {
            if let Ok(Some(goal)) = self.load(&goal_id).await {
                // 双重检查状态
                if matches!(goal.status, GoalStatus::Active | GoalStatus::Pending | GoalStatus::Blocked { .. }) {
                    goals.push(goal);
                } else {
                    // 状态已变更，从活跃索引移除
                    let _: () = conn.srem(&active_key, &goal_id).await?;
                }
            }
        }

        Ok(goals)
    }
}

#[async_trait]
impl GoalStorage for RedisGoalStorage {
    async fn save(&self, goal: &Goal) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.make_key(&goal.id);
        let value = serde_json::to_string(goal)?;

        let mut conn = self.conn.write().await;

        // 保存目标数据
        if let Some(ttl) = self.default_ttl {
            redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl as i64)
                .arg(&value)
                .query_async(&mut *conn)
                .await?;
        } else {
            conn.set(&key, &value).await?;
        }

        // 更新索引
        if let Some(ref agent_id) = goal.agent_id {
            let index_key = self.make_index_key(agent_id);
            conn.sadd(&index_key, &goal.id).await?;
            
            // 索引也设置 TTL
            if let Some(ttl) = self.default_ttl {
                conn.expire(&index_key, ttl as i64).await?;
            }
        }

        // 活跃目标索引
        if matches!(goal.status, GoalStatus::Active | GoalStatus::Pending | GoalStatus::Blocked { .. }) {
            let active_key = self.make_active_key();
            conn.sadd(&active_key, &goal.id).await?;
            if let Some(ttl) = self.default_ttl {
                conn.expire(&active_key, ttl as i64).await?;
            }
        }

        Ok(())
    }

    async fn load(&self, goal_id: &GoalId) -> Result<Option<Goal>, Box<dyn std::error::Error>> {
        let key = self.make_key(goal_id);
        let mut conn = self.conn.write().await;

        let value: Option<String> = conn.get(&key).await?;

        match value {
            Some(v) => {
                let goal: Goal = serde_json::from_str(&v)?;
                Ok(Some(goal))
            }
            None => Ok(None),
        }
    }

    async fn load_all(&self) -> Result<Vec<Goal>, Box<dyn std::error::Error>> {
        let pattern = format!("{}*", self.key_prefix);
        let mut conn = self.conn.write().await;

        // 使用 SCAN 而不是 KEYS（生产环境更安全）
        let mut goals = Vec::new();
        let mut cursor = 0u64;

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut *conn)
                .await?;

            cursor = next_cursor;

            for key in keys {
                // 跳过索引 key
                if key.contains(":index:") {
                    continue;
                }

                if let Ok(Some(value)) = conn.get::<_, Option<String>>(&key).await {
                    if let Ok(goal) = serde_json::from_str::<Goal>(&value) {
                        goals.push(goal);
                    }
                }
            }

            if cursor == 0 {
                break;
            }
        }

        Ok(goals)
    }

    async fn delete(&self, goal_id: &GoalId) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.make_key(goal_id);
        
        // 先获取目标信息以清理索引（在获取锁之前）
        let goal = self.load(goal_id).await.ok().flatten();
        
        let mut conn = self.conn.write().await;

        // 清理索引
        if let Some(goal) = goal {
            // 清理 agent 索引
            if let Some(ref agent_id) = goal.agent_id {
                let index_key = self.make_index_key(agent_id);
                let _: () = conn.srem(&index_key, goal_id).await?;
            }

            // 清理活跃索引
            let active_key = self.make_active_key();
            let _: () = conn.srem(&active_key, goal_id).await?;
        }

        // 删除目标
        conn.del(&key).await?;

        Ok(())
    }
}
