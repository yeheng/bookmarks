use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::collections::{
    create_collection, delete_collection, get_collection, get_collections, update_collection,
};
use crate::state::AppState;

pub fn collection_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_collections))
        .route("/", post(create_collection))
        .route("/{:id}", get(get_collection))
        .route("/{:id}", put(update_collection))
        .route("/{:id}", delete(delete_collection))
}
