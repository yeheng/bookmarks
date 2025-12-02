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

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        let mut app_config: AppConfig = config.try_deserialize()?;

        // Override with environment variables if not set
        if app_config.database.url.is_empty() {
            app_config.database.url = std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
        }

        if app_config.auth.jwt_secret.is_empty() {
            app_config.auth.jwt_secret = std::env::var("JWT_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_SECRET must be set"))?;
        }

        Ok(app_config)
    }
}
