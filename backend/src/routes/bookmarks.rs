use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::bookmarks::{
    batch_update_bookmarks, create_bookmark, delete_bookmark, export_bookmarks, get_bookmark,
    get_bookmarks, import_bookmarks, increment_visit_count, update_bookmark,
};
use crate::state::AppState;

pub fn bookmark_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_bookmarks))
        .route("/", post(create_bookmark))
        .route("/batch", post(batch_update_bookmarks))
        .route("/import", post(import_bookmarks))
        .route("/export", get(export_bookmarks))
        .route("/{:id}", get(get_bookmark))
        .route("/{:id}", put(update_bookmark))
        .route("/{:id}", delete(delete_bookmark))
        .route("/{:id}/visit", post(increment_visit_count))
}
