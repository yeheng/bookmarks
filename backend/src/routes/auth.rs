use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::auth::{
    change_password, get_current_user, login, logout, refresh_token, register,
};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
        .route("/change-password", post(change_password))
}

pub fn ano_routes() -> Router<AppState> {
    Router::new()
        .route("/refresh", post(refresh_token))
        .route("/register", post(register))
        .route("/login", post(login))
}
