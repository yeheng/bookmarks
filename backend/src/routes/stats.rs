use axum::{routing::get, Router};

use crate::handlers::stats::get_user_stats;
use crate::state::AppState;

pub fn stats_routes() -> Router<AppState> {
    Router::new().route("/user", get(get_user_stats))
}
