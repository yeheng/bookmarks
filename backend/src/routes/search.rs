use axum::{routing::get, Router};

use crate::handlers::search::{get_search_suggestions, search_bookmarks};
use crate::state::AppState;

pub fn search_routes() -> Router<AppState> {
    Router::new()
        .route("/bookmarks", get(search_bookmarks))
        .route("/suggestions", get(get_search_suggestions))
}
