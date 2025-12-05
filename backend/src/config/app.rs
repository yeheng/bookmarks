use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: super::DatabaseConfig,
    pub auth: super::AuthConfig,
    #[serde(default)]
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
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

    /// 检查当前是否为生产环境
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// 检查当前是否为开发环境
    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }
}
