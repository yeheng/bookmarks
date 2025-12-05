use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Unauthorized(String),

    #[error("Conflict error: {0}")]
    Conflict(String),

    #[error("Bad request error: {0}")]
    BadRequest(String),

    #[error("Not found error: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Password hashing error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        match err.downcast::<AppError>() {
            Ok(app_error) => app_error,
            Err(err) => AppError::Anyhow(err),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 检查是否为生产环境
        let is_production = std::env::var("APP_ENV")
            .or_else(|_| std::env::var("RUST_ENV"))
            .map(|env| env == "production")
            .unwrap_or(false);

        let (status, error_message) = match self {
            AppError::Database(ref err) => {
                // 记录详细错误到日志
                tracing::error!("Database error: {:?}", err);
                // 生产环境返回通用错误信息
                if is_production {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                }
            }
            AppError::Unauthorized(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            AppError::Conflict(ref msg) => (StatusCode::CONFLICT, msg.as_str()),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::Internal(ref msg) => {
                // 记录详细错误到日志
                tracing::error!("Internal error: {}", msg);
                // 生产环境隐藏详细错误
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Jwt(ref err) => {
                // 记录 JWT 错误详情
                tracing::error!("JWT error: {:?}", err);
                (StatusCode::UNAUTHORIZED, "Invalid authentication token")
            }
            AppError::Bcrypt(ref err) => {
                // 记录 Bcrypt 错误详情
                tracing::error!("Bcrypt error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Password processing error",
                )
            }
            AppError::Config(ref err) => {
                // 记录配置错误详情
                tracing::error!("Config error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error")
            }
            AppError::Serialization(ref err) => {
                // 记录序列化错误详情
                tracing::error!("Serialization error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Data serialization error",
                )
            }
            AppError::Anyhow(ref err) => {
                // 记录 Anyhow 错误详情
                tracing::error!("Anyhow error: {:?}", err);
                // 生产环境返回通用错误
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
