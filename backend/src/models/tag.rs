use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTag {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct TagWithUsage {
    #[serde(flatten)]
    pub tag: Tag,
    pub bookmark_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct TagQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
