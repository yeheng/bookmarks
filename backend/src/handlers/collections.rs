use axum::{
    extract::{Json, Path, Query, State},
    response::Response,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthenticatedUser;
use crate::models::{CollectionQuery, CreateCollection, UpdateCollection};
use crate::services::CollectionService;
use crate::utils::error::AppError;
use crate::utils::response::{success_message_response, success_response};

#[derive(Deserialize)]
pub struct CollectionListQuery {
    pub parent_id: Option<Uuid>,
    pub is_public: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_collections(
    State(db_pool): State<PgPool>,
    Query(query): Query<CollectionListQuery>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let collection_query = CollectionQuery {
        parent_id: query.parent_id,
        is_public: query.is_public,
        limit: query.limit,
        offset: query.offset,
    };

    let collections =
        CollectionService::get_collections(user_id, collection_query, &db_pool).await?;

    Ok(success_response(collections))
}

pub async fn get_collection(
    State(db_pool): State<PgPool>,
    Path(collection_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let collection = CollectionService::get_collection_by_id(user_id, collection_id, &db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Collection not found".to_string()))?;

    Ok(success_response(collection))
}

pub async fn create_collection(
    State(db_pool): State<PgPool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(collection_data): Json<CreateCollection>,
) -> Result<Response, AppError> {
    let collection =
        CollectionService::create_collection(user_id, collection_data, &db_pool).await?;

    Ok(success_response(collection))
}

pub async fn update_collection(
    State(db_pool): State<PgPool>,
    Path(collection_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(update_data): Json<UpdateCollection>,
) -> Result<Response, AppError> {
    let collection =
        CollectionService::update_collection(user_id, collection_id, update_data, &db_pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Collection not found".to_string()))?;

    Ok(success_response(collection))
}

pub async fn delete_collection(
    State(db_pool): State<PgPool>,
    Path(collection_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let deleted = CollectionService::delete_collection(user_id, collection_id, &db_pool).await?;

    if !deleted {
        return Err(AppError::NotFound("Collection not found".to_string()));
    }

    Ok(success_message_response("Collection deleted successfully"))
}
