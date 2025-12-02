use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub async fn create_pool(&self) -> anyhow::Result<SqlitePool> {
        let pool = SqlitePool::connect(&self.url).await?;
        Ok(pool)
    }
}
