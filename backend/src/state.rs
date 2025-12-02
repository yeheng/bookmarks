use axum::extract::FromRef;
use axum_jwt_auth::Decoder;
use sqlx::PgPool;

use crate::utils::jwt::JwtClaims;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_decoder: Decoder<JwtClaims>,
}

impl AppState {
    pub fn new(db_pool: PgPool, jwt_decoder: Decoder<JwtClaims>) -> Self {
        Self {
            db_pool,
            jwt_decoder,
        }
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

impl FromRef<AppState> for Decoder<JwtClaims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_decoder.clone()
    }
}
