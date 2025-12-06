use std::env;
use tower_http::cors::CorsLayer;
use tracing::warn;

pub fn cors_layer() -> CorsLayer {
    // Get allowed origins from environment variable or use defaults
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string());

    let origins: Vec<&str> = allowed_origins.split(',').collect();

    // Parse origins safely, filtering out invalid ones
    let parsed_origins: Vec<_> = origins
        .into_iter()
        .filter_map(|s| match s.trim().parse() {
            Ok(origin) => {
                tracing::debug!("CORS: Allowing origin: {:?}", origin);
                Some(origin)
            }
            Err(e) => {
                warn!("CORS: Invalid origin '{}' skipped: {}", s, e);
                None
            }
        })
        .collect();

    if parsed_origins.is_empty() {
        warn!("CORS: No valid origins configured, using localhost defaults");
        // Fallback to localhost origins if none are valid
        return CorsLayer::new()
            .allow_origin([
                "http://localhost:3000".parse().unwrap(),
                "http://localhost:5173".parse().unwrap(),
            ])
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
                axum::http::Method::PATCH,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::ACCEPT,
                axum::http::header::CONTENT_TYPE,
            ])
            .allow_credentials(true)
            .expose_headers([
                axum::http::header::CONTENT_LENGTH,
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
            .max_age(std::time::Duration::from_secs(86400));
    }

    CorsLayer::new()
        .allow_origin(parsed_origins) // 允许特定来源
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
            axum::http::Method::PATCH,
        ]) // 允许特定方法
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
        ]) // 允许特定头部
        .allow_credentials(true)
        .expose_headers([
            axum::http::header::CONTENT_LENGTH,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]) // 暴露特定头部
        .max_age(std::time::Duration::from_secs(86400)) // 预检请求缓存24小时
}
