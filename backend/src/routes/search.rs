use axum::{routing::get, Router};

use crate::handlers::search::{get_search_suggestions, search_resources};
use crate::state::AppState;

pub fn search_routes() -> Router<AppState> {
    Router::new()
        .route("/resources", get(search_resources))
        .route("/suggestions", get(get_search_suggestions))
}
