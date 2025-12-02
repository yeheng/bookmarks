use axum::{
    extract::{Query, State},
    response::Response,
};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    middleware::AuthenticatedUser,
    models::{SearchFilters, SearchResponse, SearchType},
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
    pub collection_id: Option<String>,
    pub tags: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SuggestionQueryParams {
    pub q: String,
    pub limit: Option<i64>,
}

pub async fn search_bookmarks(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Query(query): Query<SearchQueryParams>,
) -> Result<Response, AppError> {
    let filters = build_filters(&query)?;
    let result: SearchResponse =
        SearchService::search_bookmarks(user_id, filters, &app_state.db_pool).await?;

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

    let tags = query
        .tags
        .as_ref()
        .map(|value| {
            value
                .split(',')
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let date_from = parse_date(query.date_from.as_deref())?;
    let date_to = parse_date(query.date_to.as_deref())?;

    let collection_id = match query.collection_id.as_deref() {
        Some(id) => Some(
            Uuid::parse_str(id)
                .map_err(|_| AppError::BadRequest("Invalid collection_id provided".to_string()))?,
        ),
        None => None,
    };

    Ok(SearchFilters {
        query: query.q.clone(),
        search_type,
        collection_id,
        tags,
        date_from,
        date_to,
        limit,
        offset,
    })
}

fn parse_date(value: Option<&str>) -> Result<Option<NaiveDate>, AppError> {
    if let Some(date_str) = value {
        if date_str.trim().is_empty() {
            return Ok(None);
        }

        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| AppError::BadRequest("Invalid date format (expected YYYY-MM-DD)".into()))
    } else {
        Ok(None)
    }
}
