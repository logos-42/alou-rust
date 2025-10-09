// ============================================
// Database Module - PostgreSQL connection pool management
// ============================================

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Initialize database connection pool
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Test database connection
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;
    
    Ok(())
}

