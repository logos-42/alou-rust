//! 任务执行器

use sqlx::SqlitePool;

pub struct TaskExecutor {
    db_pool: SqlitePool,
}

impl TaskExecutor {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }
}
