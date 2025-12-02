use tower_http::cors::CorsLayer;
use std::env;

pub fn cors_layer() -> CorsLayer {
    // Get allowed origins from environment variable or use defaults
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string());
    
    let origins: Vec<&str> = allowed_origins.split(',').collect();
    
    CorsLayer::new()
        .allow_origin(origins.into_iter().map(|s| s.parse().unwrap()).collect::<Vec<_>>()) // 允许特定来源
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
