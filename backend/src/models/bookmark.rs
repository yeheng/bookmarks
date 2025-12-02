use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bookmark {
    pub id: Uuid,
    pub user_id: Uuid,
    pub collection_id: Option<Uuid>,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub screenshot_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_private: bool,
    pub is_read: bool,
    pub visit_count: i32,
    pub last_visited: Option<DateTime<Utc>>,
    pub reading_time: Option<i32>,
    pub difficulty_level: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmark {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookmark {
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub collection_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_private: Option<bool>,
    pub is_read: Option<bool>,
    pub reading_time: Option<i32>,
    pub difficulty_level: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct BookmarkWithTags {
    #[serde(flatten)]
    pub bookmark: Bookmark,
    pub tags: Vec<String>,
    pub collection_name: Option<String>,
    pub collection_color: Option<String>,
}

// 手动实现 FromRow，因为使用了 flatten
impl<'r> FromRow<'r, sqlx::postgres::PgRow> for BookmarkWithTags {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(BookmarkWithTags {
            bookmark: Bookmark {
                id: row.try_get("id")?,
                user_id: row.try_get("user_id")?,
                collection_id: row.try_get("collection_id")?,
                title: row.try_get("title")?,
                url: row.try_get("url")?,
                description: row.try_get("description")?,
                favicon_url: row.try_get("favicon_url")?,
                screenshot_url: row.try_get("screenshot_url")?,
                thumbnail_url: row.try_get("thumbnail_url")?,
                is_favorite: row.try_get("is_favorite")?,
                is_archived: row.try_get("is_archived")?,
                is_private: row.try_get("is_private")?,
                is_read: row.try_get("is_read")?,
                visit_count: row.try_get("visit_count")?,
                last_visited: row.try_get("last_visited")?,
                reading_time: row.try_get("reading_time")?,
                difficulty_level: row.try_get("difficulty_level")?,
                metadata: row.try_get("metadata")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            },
            tags: row.try_get("tags")?,
            collection_name: row.try_get("collection_name")?,
            collection_color: row.try_get("collection_color")?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct BookmarkQuery {
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_private: Option<bool>,
    pub is_read: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>, // "created_at", "updated_at", "title", "visit_count"
    pub sort_order: Option<String>, // "asc", "desc"
}

#[derive(Debug, Deserialize)]
pub struct ImportBookmarks {
    pub bookmarks: Vec<CreateBookmark>,
    pub collection_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookmarkBatchAction {
    Delete,
    Move,
    AddTags,
    RemoveTags,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkBatchData {
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkBatchRequest {
    pub action: BookmarkBatchAction,
    pub bookmark_ids: Vec<Uuid>,
    pub data: Option<BookmarkBatchData>,
}

#[derive(Debug, Serialize)]
pub struct BookmarkBatchResult {
    pub processed: usize,
    pub failed: usize,
    pub errors: Vec<BookmarkBatchError>,
}

#[derive(Debug, Serialize)]
pub struct BookmarkBatchError {
    pub bookmark_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct BookmarkVisitInfo {
    pub visit_count: i64,
    pub last_visited: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum BookmarkExportFormat {
    #[default]
    Json,
    Html,
    Netscape,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkExportOptions {
    #[serde(default)]
    pub format: BookmarkExportFormat,
    pub collection_id: Option<Uuid>,
    #[serde(default)]
    pub include_archived: bool,
}

#[derive(Debug)]
pub struct BookmarkExportPayload {
    pub filename: String,
    pub content_type: String,
    pub body: Vec<u8>,
}
