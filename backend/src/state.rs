use axum::extract::FromRef;
use axum_jwt_auth::Decoder;
use sqlx::SqlitePool;

use crate::services::TantivyIndexManager;
use crate::utils::jwt::JwtClaims;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub jwt_decoder: Decoder<JwtClaims>,
    pub tantivy_index: TantivyIndexManager,
}

impl AppState {
    pub fn new(
        db_pool: SqlitePool,
        jwt_decoder: Decoder<JwtClaims>,
        tantivy_index: TantivyIndexManager,
    ) -> Self {
        Self {
            db_pool,
            jwt_decoder,
            tantivy_index,
        }
    }
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

impl FromRef<AppState> for Decoder<JwtClaims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_decoder.clone()
    }
}

impl FromRef<AppState> for TantivyIndexManager {
    fn from_ref(state: &AppState) -> Self {
        state.tantivy_index.clone()
    }
}
