use axum::{middleware as mw, Router};
use axum_jwt_auth::Decoder;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{self, EnvFilter};

mod config;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod state;
mod utils;

use config::AppConfig;
use middleware::{auth_middleware, logging_middleware};
use routes::{
    auth_routes, bookmark_routes, collection_routes, search_routes, stats_routes, tag_routes,
};
use state::AppState;
use utils::jwt::{JWTService, JwtClaims};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing with a sensible default when RUST_LOG isn't set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_target(true)
        .with_env_filter(env_filter)
        .init();

    // Load configuration
    let config = AppConfig::from_env()?;

    // Initialize database connection pool
    let db_pool = config.database.create_pool().await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    // Initialize shared JWT decoder for middleware
    let jwt_decoder: Decoder<JwtClaims> = Arc::new(JWTService::new(config.auth.jwt_secret.clone()));
    let app_state = AppState::new(db_pool.clone(), jwt_decoder);

    // Protected routes requiring authentication
    let protected_routes = Router::new()
        .nest("/api/bookmarks", bookmark_routes())
        .nest("/api/collections", collection_routes())
        .nest("/api/tags", tag_routes())
        .nest("/api/search", search_routes())
        .nest("/api/stats", stats_routes())
        .layer(mw::from_fn_with_state(app_state.clone(), auth_middleware));

    // Build application router
    let app = Router::new()
        .nest("/api/auth", auth_routes())
        .merge(protected_routes)
        .layer(middleware::cors::cors_layer())
        .layer(mw::from_fn(logging_middleware))
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
