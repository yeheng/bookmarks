use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Collection {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub icon: String,
    pub sort_order: i32,
    pub is_default: bool,
    pub is_public: bool,
    pub parent_id: Option<i64>,
    pub bookmark_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateCollection {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollection {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<i64>,
    pub clear_parent_id: Option<bool>,
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
    pub parent_id: Option<i64>,
    pub is_public: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
