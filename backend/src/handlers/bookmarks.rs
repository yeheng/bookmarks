use axum::{
    extract::{Json, Path, Query, State},
    response::Response,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::middleware::AuthenticatedUser;
use crate::models::{
    BookmarkBatchRequest, BookmarkBatchResult, BookmarkQuery, CreateBookmark, UpdateBookmark,
};
use crate::services::BookmarkService;
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::response::{
    success_message_response, success_response, success_response_with_message,
};

#[derive(Deserialize)]
pub struct BookmarkListQuery {
    pub collection_id: Option<i64>,
    pub tags: Option<String>, // Comma-separated
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_private: Option<bool>,
    pub is_read: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

pub async fn get_bookmarks(
    State(db_pool): State<SqlitePool>,
    Query(query): Query<BookmarkListQuery>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let tags: Vec<String> = query
        .tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let bookmark_query = BookmarkQuery {
        collection_id: query.collection_id,
        tags: if tags.is_empty() { None } else { Some(tags) },
        is_favorite: query.is_favorite,
        is_archived: query.is_archived,
        is_private: query.is_private,
        is_read: query.is_read,
        search: query.search,
        limit: query.limit,
        offset: query.offset,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
    };

    let bookmarks = BookmarkService::get_bookmarks(user_id, bookmark_query, &db_pool).await?;

    Ok(success_response(bookmarks))
}

pub async fn get_bookmark(
    State(db_pool): State<SqlitePool>,
    Path(bookmark_id): Path<i64>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let bookmark = BookmarkService::get_bookmark_by_id(user_id, bookmark_id, &db_pool).await?;

    Ok(success_response(bookmark))
}

pub async fn create_bookmark(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(bookmark_data): Json<CreateBookmark>,
) -> Result<Response, AppError> {
    let bookmark = BookmarkService::create_bookmark(user_id, bookmark_data, &app_state.db_pool).await?;

    Ok(success_response(bookmark))
}

pub async fn update_bookmark(
    State(app_state): State<AppState>,
    Path(bookmark_id): Path<i64>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(update_data): Json<UpdateBookmark>,
) -> Result<Response, AppError> {
    let bookmark =
        BookmarkService::update_bookmark(user_id, bookmark_id, update_data, &app_state.db_pool).await?;

    Ok(success_response(bookmark))
}

pub async fn delete_bookmark(
    State(app_state): State<AppState>,
    Path(bookmark_id): Path<i64>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let deleted = BookmarkService::delete_bookmark(user_id, bookmark_id, &app_state.db_pool).await?;

    if !deleted {
        return Err(AppError::NotFound("Bookmark not found".to_string()));
    }

    Ok(success_message_response("Bookmark deleted successfully"))
}
pub async fn batch_update_bookmarks(
    State(db_pool): State<SqlitePool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<BookmarkBatchRequest>,
) -> Result<Response, AppError> {
    if payload.bookmark_ids.is_empty() {
        return Err(AppError::BadRequest(
            "bookmark_ids cannot be empty".to_string(),
        ));
    }

    let result: BookmarkBatchResult =
        BookmarkService::batch_process(user_id, payload, &db_pool).await?;

    Ok(success_response_with_message(
        result,
        "Batch operation completed",
    ))
}
