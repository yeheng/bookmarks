use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub async fn create_pool(&self) -> anyhow::Result<PgPool> {
        let pool = PgPool::connect(&self.url).await?;
        Ok(pool)
    }
}
