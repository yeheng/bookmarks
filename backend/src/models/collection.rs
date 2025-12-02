use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Collection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub icon: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub is_public: bool,
    pub parent_id: Option<Uuid>,
    pub bookmark_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCollection {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollection {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<Option<Uuid>>,
    pub sort_order: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct CollectionWithBookmarkCount {
    #[serde(flatten)]
    pub collection: Collection,
    pub bookmark_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CollectionQuery {
    pub parent_id: Option<Uuid>,
    pub is_public: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
