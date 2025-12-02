use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::{header, HeaderValue},
    response::Response,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthenticatedUser;
use crate::models::{
    BookmarkBatchRequest, BookmarkBatchResult, BookmarkExportFormat, BookmarkExportOptions,
    BookmarkExportPayload, BookmarkQuery, CreateBookmark, UpdateBookmark,
};
use crate::services::BookmarkService;
use crate::utils::error::AppError;
use crate::utils::response::{
    success_message_response, success_response, success_response_with_message,
};

#[derive(Deserialize)]
pub struct BookmarkListQuery {
    pub collection_id: Option<Uuid>,
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
    State(db_pool): State<PgPool>,
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
    State(db_pool): State<PgPool>,
    Path(bookmark_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let bookmark = BookmarkService::get_bookmark_by_id(user_id, bookmark_id, &db_pool).await?;

    Ok(success_response(bookmark))
}

pub async fn create_bookmark(
    State(db_pool): State<PgPool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(bookmark_data): Json<CreateBookmark>,
) -> Result<Response, AppError> {
    let bookmark = BookmarkService::create_bookmark(user_id, bookmark_data, &db_pool).await?;

    Ok(success_response(bookmark))
}

pub async fn update_bookmark(
    State(db_pool): State<PgPool>,
    Path(bookmark_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(update_data): Json<UpdateBookmark>,
) -> Result<Response, AppError> {
    let bookmark =
        BookmarkService::update_bookmark(user_id, bookmark_id, update_data, &db_pool).await?;

    Ok(success_response(bookmark))
}

pub async fn delete_bookmark(
    State(db_pool): State<PgPool>,
    Path(bookmark_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let deleted = BookmarkService::delete_bookmark(user_id, bookmark_id, &db_pool).await?;

    if !deleted {
        return Err(AppError::NotFound("Bookmark not found".to_string()));
    }

    Ok(success_message_response("Bookmark deleted successfully"))
}

pub async fn increment_visit_count(
    State(db_pool): State<PgPool>,
    Path(bookmark_id): Path<Uuid>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let visit_info = BookmarkService::increment_visit_count(user_id, bookmark_id, &db_pool).await?;

    Ok(success_response(visit_info))
}

pub async fn import_bookmarks(
    State(db_pool): State<PgPool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let mut format = BookmarkExportFormat::Json;
    let mut collection_id: Option<Uuid> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart payload: {}", e)))?
    {
        match field.name() {
            Some("format") => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Invalid format field: {}", e)))?;
                format = parse_export_format(&value)?;
            }
            Some("collection_id") => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Invalid collection_id: {}", e)))?;
                collection_id = Some(Uuid::parse_str(value.trim()).map_err(|_| {
                    AppError::BadRequest("collection_id must be a valid UUID".to_string())
                })?);
            }
            Some("file") => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Invalid file upload: {}", e)))?;
                file_bytes = Some(data.to_vec());
            }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        AppError::BadRequest("Missing bookmark file in multipart payload".to_string())
    })?;

    let mut bookmarks = parse_import_payload(&bytes, &format)?;

    let mut imported = 0usize;
    let mut duplicates = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for mut bookmark_data in bookmarks.drain(..) {
        if let Some(target_collection) = collection_id {
            bookmark_data.collection_id = Some(target_collection);
        }

        if BookmarkService::bookmark_exists(user_id, &bookmark_data.url, &db_pool).await? {
            duplicates += 1;
            continue;
        }

        match BookmarkService::create_bookmark(user_id, bookmark_data, &db_pool).await {
            Ok(_) => imported += 1,
            Err(err) => {
                skipped += 1;
                errors.push(format!("{}", err));
            }
        }
    }

    Ok(success_response_with_message(
        json!({
            "imported": imported,
            "skipped": skipped,
            "duplicates": duplicates,
            "errors": errors
        }),
        "Import completed",
    ))
}

pub async fn batch_update_bookmarks(
    State(db_pool): State<PgPool>,
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

#[derive(Debug, Deserialize)]
pub struct BookmarkExportQuery {
    #[serde(default)]
    pub format: BookmarkExportFormat,
    pub collection_id: Option<Uuid>,
    #[serde(default)]
    pub include_archived: bool,
}

pub async fn export_bookmarks(
    State(db_pool): State<PgPool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Query(query): Query<BookmarkExportQuery>,
) -> Result<Response, AppError> {
    let options = BookmarkExportOptions {
        format: query.format,
        collection_id: query.collection_id,
        include_archived: query.include_archived,
    };

    let payload: BookmarkExportPayload =
        BookmarkService::export_bookmarks(user_id, options, &db_pool).await?;

    let disposition =
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", payload.filename))
            .map_err(|_| AppError::Internal("Failed to build export headers".into()))?;

    let content_type = HeaderValue::from_str(&payload.content_type)
        .map_err(|_| AppError::Internal("Failed to build export headers".into()))?;

    let mut response = Response::new(payload.body.into());
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, disposition);

    Ok(response)
}

fn parse_export_format(raw: &str) -> Result<BookmarkExportFormat, AppError> {
    match raw.trim().to_lowercase().as_str() {
        "json" => Ok(BookmarkExportFormat::Json),
        "html" => Ok(BookmarkExportFormat::Html),
        "netscape" => Ok(BookmarkExportFormat::Netscape),
        other => Err(AppError::BadRequest(format!(
            "Unsupported format '{}'",
            other
        ))),
    }
}

fn parse_import_payload(
    bytes: &[u8],
    format: &BookmarkExportFormat,
) -> Result<Vec<CreateBookmark>, AppError> {
    match format {
        BookmarkExportFormat::Json => {
            let bookmarks: Vec<CreateBookmark> = serde_json::from_slice(bytes)
                .map_err(|e| AppError::BadRequest(format!("Invalid JSON import file: {}", e)))?;
            Ok(bookmarks)
        }
        BookmarkExportFormat::Html | BookmarkExportFormat::Netscape => {
            let content = String::from_utf8(bytes.to_vec())
                .map_err(|_| AppError::BadRequest("Import file is not valid UTF-8".into()))?;
            Ok(parse_netscape_bookmarks(&content))
        }
    }
}

fn parse_netscape_bookmarks(content: &str) -> Vec<CreateBookmark> {
    let link_regex = Regex::new(r#"(?i)<a[^>]*href="(?P<url>[^"]+)"[^>]*>(?P<title>[^<]+)"#)
        .expect("Failed to compile bookmark regex");
    let tag_regex =
        Regex::new(r#"(?i)tags="(?P<tags>[^"]+)""#).expect("Failed to compile tag regex");

    link_regex
        .captures_iter(content)
        .map(|caps| {
            let url = caps
                .name("url")
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let title = caps
                .name("title")
                .map(|m| m.as_str().trim().to_string())
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| "Untitled".to_string());

            let tags = tag_regex
                .captures(caps.get(0).map(|m| m.as_str()).unwrap_or_default())
                .and_then(|tag_caps| tag_caps.name("tags").map(|m| m.as_str().to_string()))
                .map(|value| {
                    value
                        .split(',')
                        .map(|tag| tag.trim().to_string())
                        .filter(|tag| !tag.is_empty())
                        .collect::<Vec<_>>()
                });

            CreateBookmark {
                title,
                url,
                description: None,
                collection_id: None,
                tags,
                is_favorite: None,
                is_private: None,
            }
        })
        .collect::<Vec<_>>()
}
