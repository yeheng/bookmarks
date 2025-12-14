use sqlx::{Row, SqlitePool};
use std::time::Instant;

use crate::models::{SearchFilters, SearchPagination, SearchResponse, SearchSuggestion};
use crate::services::query_helper::{self, QueryOptions};
use crate::utils::error::AppResult;
use crate::utils::segmenter::prepare_for_search;

// 搜索验证常量
const MIN_SEARCH_QUERY_LENGTH: usize = 3;

pub struct SearchService;

impl SearchService {
    pub async fn search_resources(
        user_id: i64,
        filters: SearchFilters,
        db_pool: &SqlitePool,
    ) -> AppResult<SearchResponse> {
        // 验证搜索词最小长度
        if filters.query.trim().len() < MIN_SEARCH_QUERY_LENGTH {
            return Err(crate::utils::error::AppError::BadRequest(format!(
                "Search query must be at least {} characters",
                MIN_SEARCH_QUERY_LENGTH
            )));
        }

        let start = Instant::now();

        // 根据配置对查询进行分词处理 (虽然 FTS 会自己处理,但在 unified query 中我们还是需要它吗?
        // query_helper converts search_term to match query.
        // Wait, prepare_for_search does segmentation (jieba). We need that.
        let search_keywords = prepare_for_search(Some(&filters.query));

        let options = QueryOptions {
            user_id,
            collection_id: filters.filters.collection_id,
            resource_type: None, // SearchFilters doesn't have type filter? Wait, it should. `filters.filters` has tags etc.
            // Check FilterCriteria definition in search.rs. It doesn't have resource_type usually?
            // Let's check search.rs content again.
            // Yes, FilterCriteria struct (lines 37-43) has: collection_id, tags, date_from, date_to. No resource_type.
            tags: &filters.filters.tags,
            is_favorite: None, // Search usually doesn't filter specific flags, or maybe it should?
            is_archived: None,
            is_private: None,
            is_read: None,
            search_term: Some(&search_keywords),
            search_type: Some(&filters.search_type),
            date_from: filters.filters.date_from,
            date_to: filters.filters.date_to,
            limit: filters.pagination.limit,
            offset: filters.pagination.offset,
            sort_by: "rank", // Search defaults to rank
            sort_order: "desc",
        };

        // 执行主查询
        let resources = query_helper::fetch_resources(db_pool, &options).await?;

        // 执行计数查询
        let total = query_helper::count_resources(db_pool, &options).await?;

        // 构建响应
        let elapsed = start.elapsed().as_secs_f64();
        let page = filters.pagination.page();
        let total_pages = if total == 0 {
            0
        } else {
            (total + filters.pagination.limit - 1) / filters.pagination.limit
        };

        Ok(SearchResponse {
            items: resources,
            pagination: SearchPagination {
                page,
                limit: filters.pagination.limit,
                total,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
            search_time: elapsed,
            highlights: None,
        })
    }

    pub async fn get_search_suggestions(
        user_id: i64,
        query: &str,
        limit: Option<i64>,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<SearchSuggestion>> {
        let limit = limit.unwrap_or(10);

        let rows = sqlx::query(
            r#"
            SELECT suggestion, suggestion_type, usage_count, last_used_at
            FROM (
                SELECT r.title as suggestion,
                       'resource' as suggestion_type,
                       COUNT(*) as usage_count,
                       MAX(r.updated_at) as last_used_at
                FROM resources r
                WHERE r.user_id = $1
                  AND lower(r.title) LIKE lower($2 || '%')
                GROUP BY r.title

                UNION ALL

                SELECT t.name as suggestion,
                       'tag' as suggestion_type,
                       COUNT(rt.resource_id) as usage_count,
                       MAX(t.updated_at) as last_used_at
                FROM tags t
                LEFT JOIN resource_tags rt ON t.id = rt.tag_id
                WHERE t.user_id = $1
                  AND lower(t.name) LIKE lower($2 || '%')
                GROUP BY t.name
            ) combined
            WHERE suggestion IS NOT NULL AND suggestion <> ''
            ORDER BY usage_count DESC
            LIMIT $3"#,
        )
        .bind(user_id)
        .bind(query)
        .bind(limit)
        .fetch_all(db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| SearchSuggestion {
                text: row
                    .get::<Option<String>, _>("suggestion")
                    .unwrap_or_default(),
                suggestion_type: row
                    .get::<Option<String>, _>("suggestion_type")
                    .unwrap_or_else(|| "resource".to_string()),
                count: row.get::<Option<i64>, _>("usage_count").unwrap_or(0),
                last_used_at: row.get("last_used_at"),
            })
            .collect())
    }
}

// Helper functions removed
