use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub async fn create_pool(&self) -> anyhow::Result<SqlitePool> {
        // 配置 SQLite 连接选项
        // 1. 开启 WAL 模式 - 允许并发读写，避免 SQLITE_BUSY
        // 2. 设置 busy_timeout - 失败前等待 5 秒
        let options = SqliteConnectOptions::from_str(&self.url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal) // Write-Ahead Logging
            .busy_timeout(std::time::Duration::from_secs(5)); // 等待锁释放

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
