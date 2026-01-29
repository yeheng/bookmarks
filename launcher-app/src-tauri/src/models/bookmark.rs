use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: Option<i64>,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub tags: Option<String>,
    pub source: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_accessed: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookmarkSearchResult {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub score: f64,
    pub frecency_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}
