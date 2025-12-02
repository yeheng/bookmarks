use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::tags::{
    create_tag, delete_tag, get_popular_tags, get_tag, get_tags, update_tag,
};
use crate::state::AppState;

pub fn tag_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_tags))
        .route("/", post(create_tag))
        .route("/popular", get(get_popular_tags))
        .route("/{:id}", get(get_tag))
        .route("/{:id}", put(update_tag))
        .route("/{:id}", delete(delete_tag))
}
