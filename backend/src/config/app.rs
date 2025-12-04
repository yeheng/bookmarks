use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: super::DatabaseConfig,
    pub auth: super::AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[allow(dead_code)]
impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Self::load()
    }

    /// 获取 Tantivy 索引文件路径
    ///
    /// 默认存储在 ./data/tantivy_index 目录下
    pub fn tantivy_index_path(&self) -> String {
        std::env::var("TANTIVY_INDEX_PATH")
            .unwrap_or_else(|_| "./data/tantivy_index".to_string())
    }
}
