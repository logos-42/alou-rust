//! Storage - SQLite 存储（sqlx）

use sqlx::{SqlitePool, Row};
use serde::{Deserialize, Serialize};

/// 存储
pub struct Storage {
    pool: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub group_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
}

impl Storage {
    pub async fn new(path: &str) -> Result<Self, String> {
        let pool = SqlitePool::connect(path)
            .await
            .map_err(|e| format!("连接数据库失败：{}", e))?;
        
        // 创建表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                group_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| format!("创建表失败：{}", e))?;
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                config TEXT,
                created_at INTEGER NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| format!("创建表失败：{}", e))?;
        
        log::info!("Storage 初始化完成：{}", path);
        
        Ok(Self { pool })
    }
    
    /// 保存消息
    pub async fn save_message(&self, message: &StoredMessage) -> Result<(), String> {
        sqlx::query(
            "INSERT OR REPLACE INTO messages (id, group_id, sender_id, content, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(&message.id)
        .bind(&message.group_id)
        .bind(&message.sender_id)
        .bind(&message.content)
        .bind(message.timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("保存消息失败：{}", e))?;
        
        Ok(())
    }
    
    /// 获取群聊历史消息
    pub async fn get_group_history(&self, group_id: &str, limit: i32) -> Result<Vec<StoredMessage>, String> {
        let messages = sqlx::query(
            "SELECT id, group_id, sender_id, content, timestamp 
             FROM messages 
             WHERE group_id = ?1 
             ORDER BY timestamp DESC 
             LIMIT ?2"
        )
        .bind(group_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("查询消息失败：{}", e))?;
        
        let result = messages
            .iter()
            .map(|row| StoredMessage {
                id: row.get(0),
                group_id: row.get(1),
                sender_id: row.get(2),
                content: row.get(3),
                timestamp: row.get(4),
            })
            .collect();
        
        Ok(result)
    }
    
    /// 保存 agent 配置
    pub async fn save_agent(&self, agent_id: &str, name: &str, config: &str) -> Result<(), String> {
        sqlx::query(
            "INSERT OR REPLACE INTO agents (id, name, config, created_at)
             VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(agent_id)
        .bind(name)
        .bind(config)
        .bind(chrono::Utc::now().timestamp_millis())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("保存 agent 失败：{}", e))?;
        
        Ok(())
    }
    
    /// 获取所有 agent
    pub async fn get_all_agents(&self) -> Result<Vec<(String, String, String)>, String> {
        let agents = sqlx::query(
            "SELECT id, name, config FROM agents"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("查询 agent 失败：{}", e))?;
        
        let result = agents
            .iter()
            .map(|row| {
                (
                    row.get::<String, _>(0),
                    row.get::<String, _>(1),
                    row.get::<String, _>(2),
                )
            })
            .collect();
        
        Ok(result)
    }
}
