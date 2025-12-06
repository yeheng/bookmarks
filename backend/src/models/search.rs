use serde::{Deserialize, Serialize};

use super::ResourceWithTags;

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

/// 分页参数
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub limit: i64,
    pub offset: i64,
}

impl PaginationParams {
    /// 从页码和每页数量创建分页参数
    pub fn from_page(page: i64, limit: i64) -> Self {
        let offset = (page - 1) * limit;
        Self { limit, offset }
    }

    /// 获取当前页码
    pub fn page(&self) -> i64 {
        (self.offset / self.limit) + 1
    }
}

/// 过滤条件
#[derive(Debug, Clone)]
pub struct FilterCriteria {
    pub collection_id: Option<i64>,
    pub tags: Vec<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
}

impl Default for FilterCriteria {
    fn default() -> Self {
        Self {
            collection_id: None,
            tags: Vec::new(),
            date_from: None,
            date_to: None,
        }
    }
}

/// 搜索参数（组合了查询文本、搜索类型、过滤条件和分页）
#[derive(Debug)]
pub struct SearchFilters {
    pub query: String,
    pub search_type: SearchType,
    pub filters: FilterCriteria,
    pub pagination: PaginationParams,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub items: Vec<ResourceWithTags>,
    pub pagination: SearchPagination,
    pub search_time: f64,
    pub highlights: Option<std::collections::HashMap<i64, std::collections::HashMap<String, Vec<String>>>>, // resource_id -> field -> snippets
}

/// 带高亮的搜索结果项
/// 保留此结构以备将来实现 FTS5 snippet() 高亮功能
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct ResourceWithHighlights {
    #[serde(flatten)]
    pub resource: ResourceWithTags,
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
