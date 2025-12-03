use crate::config::AppConfig;
use config::{Config, Environment, File};

impl AppConfig {
    /// Load configuration from multiple sources in order of precedence:
    /// 1. config/default.toml (base configuration)
    /// 2. config/{environment}.toml (environment-specific overrides)
    /// 3. config/local.toml (local overrides, should not be committed)
    /// 4. Environment variables with APP_ prefix
    pub fn load_with_env(env: Option<&str>) -> anyhow::Result<Self> {
        let mut builder =
            Config::builder().add_source(File::with_name("config/default").required(false));

        // Add environment-specific config if provided
        if let Some(environment) = env {
            builder = builder
                .add_source(File::with_name(&format!("config/{}", environment)).required(false));
        }

        // Add local overrides (for development)
        builder = builder.add_source(File::with_name("config/local").required(false));

        // Environment variables override config files
        builder = builder.add_source(Environment::with_prefix("APP").separator("__"));

        let config = builder.build()?;
        let mut app_config: AppConfig = config.try_deserialize()?;

        // Ensure required fields are set
        Self::ensure_required_fields(&mut app_config)?;

        Ok(app_config)
    }

    /// Load configuration automatically detecting environment
    pub fn load() -> anyhow::Result<Self> {
        let env = std::env::var("APP_ENV")
            .ok()
            .or_else(|| match std::env::var("RUST_ENV").ok() {
                Some(rust_env) if rust_env == "production" => Some("production".to_string()),
                Some(rust_env) if rust_env == "development" => Some("development".to_string()),
                _ => None,
            });

        Self::load_with_env(env.as_deref())
    }

    fn ensure_required_fields(config: &mut AppConfig) -> anyhow::Result<()> {
        // Database URL
        if config.database.url.is_empty() {
            config.database.url = std::env::var("DATABASE_URL")
                .or_else(|_| std::env::var("APP_DATABASE__URL"))
                .map_err(|_| {
                    anyhow::anyhow!(
                        "Database URL must be set via DATABASE_URL, APP_DATABASE__URL, or config file"
                    )
                })?;
        }

        // JWT Secret
        if config.auth.jwt_secret.is_empty() {
            config.auth.jwt_secret = std::env::var("JWT_SECRET")
                .or_else(|_| std::env::var("APP_AUTH__JWT_SECRET"))
                .map_err(|_| {
                    anyhow::anyhow!(
                        "JWT secret must be set via JWT_SECRET, APP_AUTH__JWT_SECRET, or config file"
                    )
                })?;
        }

        Ok(())
    }
}
