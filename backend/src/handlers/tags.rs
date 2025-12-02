use axum::{
    extract::{Json, Path, Query, State},
    response::Response,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::middleware::AuthenticatedUser;
use crate::models::{CreateTag, TagQuery, UpdateTag};
use crate::services::TagService;
use crate::utils::error::AppError;
use crate::utils::response::{success_message_response, success_response};

#[derive(Deserialize)]
pub struct TagListQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_tags(
    State(db_pool): State<SqlitePool>,
    Query(query): Query<TagListQuery>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let tag_query = TagQuery {
        search: query.search,
        limit: query.limit,
        offset: query.offset,
    };

    let tags = TagService::get_tags(user_id, tag_query, &db_pool).await?;

    Ok(success_response(tags))
}

pub async fn get_popular_tags(
    State(db_pool): State<SqlitePool>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let limit = params.get("limit").and_then(|s| s.parse::<i64>().ok());

    let tags = TagService::get_popular_tags(user_id, limit, &db_pool).await?;

    Ok(success_response(tags))
}

pub async fn get_tag(
    State(db_pool): State<SqlitePool>,
    Path(tag_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let tag = TagService::get_tag_by_id(user_id, tag_id, &db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Tag not found".to_string()))?;

    Ok(success_response(tag))
}

pub async fn create_tag(
    State(db_pool): State<SqlitePool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(tag_data): Json<CreateTag>,
) -> Result<Response, AppError> {
    let tag = TagService::create_tag(user_id, tag_data, &db_pool).await?;

    Ok(success_response(tag))
}

pub async fn update_tag(
    State(db_pool): State<SqlitePool>,
    Path(tag_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(update_data): Json<UpdateTag>,
) -> Result<Response, AppError> {
    let tag = TagService::update_tag(user_id, tag_id, update_data, &db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Tag not found".to_string()))?;

    Ok(success_response(tag))
}

pub async fn delete_tag(
    State(db_pool): State<SqlitePool>,
    Path(tag_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let deleted = TagService::delete_tag(user_id, tag_id, &db_pool).await?;

    if !deleted {
        return Err(AppError::NotFound("Tag not found".to_string()));
    }

    Ok(success_message_response("Tag deleted successfully"))
}
