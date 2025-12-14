use axum::{
    extract::{Query, State},
    response::Response,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    middleware::AuthenticatedUser,
    models::{FilterCriteria, PaginationParams, SearchFilters, SearchResponse, SearchType},
    services::SearchService,
    state::AppState,
    utils::error::AppError,
    utils::response::success_response,
};

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub q: String,
    #[serde(rename = "type")]
    pub search_type: Option<String>,
    pub collection_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SuggestionQueryParams {
    pub q: String,
    pub limit: Option<i64>,
}

pub async fn search_resources(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Query(query): Query<SearchQueryParams>,
) -> Result<Response, AppError> {
    let filters = build_filters(&query)?;

    // 使用 FTS5 进行搜索
    let result: SearchResponse =
        SearchService::search_resources(user_id, filters, &app_state.db_pool).await?;

    Ok(success_response(result))
}

pub async fn get_search_suggestions(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Query(query): Query<SuggestionQueryParams>,
) -> Result<Response, AppError> {
    let suggestions =
        SearchService::get_search_suggestions(user_id, &query.q, query.limit, &app_state.db_pool)
            .await?;

    Ok(success_response(json!({
        "suggestions": suggestions
    })))
}

fn build_filters(query: &SearchQueryParams) -> Result<SearchFilters, AppError> {
    if query.q.trim().is_empty() {
        return Err(AppError::BadRequest("Search query cannot be empty".into()));
    }

    // 解析搜索类型
    let search_type = match query
        .search_type
        .as_deref()
        .unwrap_or("all")
        .to_lowercase()
        .as_str()
    {
        "title" => SearchType::Title,
        "content" => SearchType::Content,
        "url" => SearchType::Url,
        _ => SearchType::All,
    };

    // 构建过滤条件
    let filters = FilterCriteria {
        collection_id: query.collection_id,
        tags: query.tags.clone().unwrap_or_default(),
        date_from: query.date_from,
        date_to: query.date_to,
    };

    // 构建分页参数
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let pagination = PaginationParams::from_page(page, limit);

    Ok(SearchFilters {
        query: query.q.clone(),
        search_type,
        filters,
        pagination,
    })
}
