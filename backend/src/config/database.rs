use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub async fn create_pool(&self) -> anyhow::Result<SqlitePool> {
        let options = SqliteConnectOptions::from_str(&self.url)?.create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;
        Ok(pool)
    }

    #[allow(dead_code)]
    pub fn new(url: String) -> Self {
        Self {
            url,
            max_connections: 10,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:bookmarks.db".to_string(),
            max_connections: 10,
        }
    }
}
