use serde::{Deserialize, Serialize};

use super::BookmarkWithTags;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SearchType {
    #[default]
    All,
    Title,
    Content,
    Url,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub query: String,
    pub search_type: SearchType,
    pub collection_id: Option<i64>,
    pub tags: Vec<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub items: Vec<BookmarkWithTags>,
    pub pagination: SearchPagination,
    pub search_time: f64,
    pub highlights: Option<std::collections::HashMap<i64, std::collections::HashMap<String, Vec<String>>>>, // bookmark_id -> field -> snippets
}

/// 带高亮的搜索结果项
/// 保留此结构以备将来实现 FTS5 snippet() 高亮功能
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct BookmarkWithHighlights {
    #[serde(flatten)]
    pub bookmark: BookmarkWithTags,
    pub highlights: std::collections::HashMap<String, Vec<String>>, // field -> snippets
}

#[derive(Debug, Serialize)]
pub struct SearchPagination {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchSuggestion {
    pub text: String,
    pub suggestion_type: String,
    pub count: i64,
    pub last_used_at: Option<i64>,
}
