use sqlx::{query_as, query_scalar, Row, SqlitePool, QueryBuilder as SqlxQueryBuilder, sqlite::Sqlite};
use std::time::Instant;

use crate::models::{
    FilterCriteria, PaginationParams, ResourceWithTags, SearchFilters, SearchPagination, SearchResponse,
    SearchSuggestion, SearchType,
};
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

        // 根据配置对查询进行分词处理
        let search_keywords = prepare_for_search(Some(&filters.query));

        // 构建搜索查询
        let (search_sql, bind_values) = build_search_query(user_id, &search_keywords, &filters.search_type, &filters.filters, &filters.pagination);
        
        // 执行主查询
        let mut query = query_as::<_, ResourceWithTags>(&search_sql);
        for value in &bind_values {
            query = bind_query_value(query, value);
        }
        query = query.bind(filters.pagination.limit).bind(filters.pagination.offset);
        
        let resources = query.fetch_all(db_pool).await?;

        // 构建计数查询
        let (count_sql, count_bind_values) = build_count_query(user_id, &search_keywords, &filters.search_type, &filters.filters);
        
        let mut count_query = query_scalar::<_, i64>(&count_sql);
        for value in &count_bind_values {
            count_query = bind_query_value_scalar(count_query, value);
        }
        
        let total = count_query.fetch_one(db_pool).await?;

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
            highlights: None, // 如果需要高亮,可以使用 snippet() 函数提取
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

// Helper function to build search query using sqlx::QueryBuilder
fn build_search_query(
    user_id: i64,
    search_keywords: &str,
    search_type: &SearchType,
    filters: &FilterCriteria,
    pagination: &PaginationParams,
) -> (String, Vec<QueryValue>) {
    let mut bind_values = Vec::new();
    bind_values.push(QueryValue::I64(user_id));
    bind_values.push(QueryValue::Text(search_keywords.to_string()));

    let mut query: SqlxQueryBuilder<'_, Sqlite> = SqlxQueryBuilder::new(
        r#"
        SELECT
            r.id,
            r.user_id,
            r.collection_id,
            r.title,
            r.url,
            r.description,
            r.favicon_url,
            r.screenshot_url,
            r.thumbnail_url,
            r.is_favorite,
            r.is_archived,
            r.is_private,
            r.is_read,
            r.visit_count,
            r.last_visited,
            r.metadata,
            r.type,
            r.content,
            r.source,
            r.mime_type,
            r.created_at,
            r.updated_at,
            COALESCE(
                CASE
                    WHEN COUNT(t.name) > 0
                    THEN '[' || GROUP_CONCAT('"' || REPLACE(t.name, '"', '""') || '"', ',') || ']'
                    ELSE '[]'
                END,
                '[]'
            ) as tags,
            c.name as collection_name,
            c.color as collection_color,
            COALESCE(
                (SELECT COUNT(*) FROM resource_references rr
                 WHERE rr.source_id = r.id OR rr.target_id = r.id),
                0
            ) as reference_count
        FROM resources r
        JOIN resources_fts fts ON r.id = fts.rowid
        LEFT JOIN collections c ON r.collection_id = c.id
        LEFT JOIN resource_tags rt ON r.id = rt.resource_id
        LEFT JOIN tags t ON rt.tag_id = t.id
        "#,
    );

    // Build WHERE clause
    query.push(" WHERE r.user_id = ");
    query.push_bind(user_id);
    query.push(" AND ");

    // Add FTS5 MATCH condition
    let fts_match = match search_type {
        SearchType::Title => "resources_fts MATCH 'title:' || ",
        SearchType::Content => "resources_fts MATCH 'content:' || ",
        SearchType::Url => "resources_fts MATCH 'url:' || ",
        SearchType::All => "resources_fts MATCH ",
    };
    query.push(fts_match);
    query.push_bind(search_keywords);

    // Add additional filters
    if let Some(collection_id) = filters.collection_id {
        query.push(" AND r.collection_id = ");
        query.push_bind(collection_id);
        bind_values.push(QueryValue::I64(collection_id));
    }

    if !filters.tags.is_empty() {
        query.push(" AND r.id IN (");
        query.push(
            r#"
            SELECT rt.resource_id
            FROM resource_tags rt
            JOIN tags t2 ON rt.tag_id = t2.id
            WHERE t2.user_id = "#,
        );
        query.push_bind(user_id);
        query.push(" AND t2.name IN (");
        
        for (i, tag) in filters.tags.iter().enumerate() {
            if i > 0 {
                query.push(", ");
            }
            query.push_bind(tag);
            bind_values.push(QueryValue::Text(tag.clone()));
        }
        
        query.push(") GROUP BY rt.resource_id HAVING COUNT(DISTINCT t2.name) = ");
        query.push_bind(filters.tags.len() as i64);
        query.push(")");
        bind_values.push(QueryValue::I64(user_id));
        bind_values.push(QueryValue::I64(filters.tags.len() as i64));
    }

    if let Some(date_from) = filters.date_from {
        query.push(" AND r.created_at >= ");
        query.push_bind(date_from);
        bind_values.push(QueryValue::I64(date_from));
    }

    if let Some(date_to) = filters.date_to {
        query.push(" AND r.created_at <= ");
        query.push_bind(date_to);
        bind_values.push(QueryValue::I64(date_to));
    }

    query.push(
        r#"
        GROUP BY r.id, c.name, c.color
        ORDER BY rank
        LIMIT "#,
    );
    query.push_bind(pagination.limit);
    query.push(" OFFSET ");
    query.push_bind(pagination.offset);

    bind_values.push(QueryValue::I64(pagination.limit));
    bind_values.push(QueryValue::I64(pagination.offset));

    (query.sql().to_string(), bind_values)
}

// Helper function to build count query
fn build_count_query(
    user_id: i64,
    search_keywords: &str,
    search_type: &SearchType,
    filters: &FilterCriteria,
) -> (String, Vec<QueryValue>) {
    let mut bind_values = Vec::new();
    bind_values.push(QueryValue::I64(user_id));
    bind_values.push(QueryValue::Text(search_keywords.to_string()));

    let mut query: SqlxQueryBuilder<'_, Sqlite> = SqlxQueryBuilder::new(
        "SELECT COUNT(*) FROM resources r JOIN resources_fts fts ON r.id = fts.rowid",
    );

    query.push(" WHERE r.user_id = ");
    query.push_bind(user_id);
    query.push(" AND ");

    // Add FTS5 MATCH condition
    let fts_match = match search_type {
        SearchType::Title => "resources_fts MATCH 'title:' || ",
        SearchType::Content => "resources_fts MATCH 'content:' || ",
        SearchType::Url => "resources_fts MATCH 'url:' || ",
        SearchType::All => "resources_fts MATCH ",
    };
    query.push(fts_match);
    query.push_bind(search_keywords);

    // Add additional filters (same as search query)
    if let Some(collection_id) = filters.collection_id {
        query.push(" AND r.collection_id = ");
        query.push_bind(collection_id);
        bind_values.push(QueryValue::I64(collection_id));
    }

    if !filters.tags.is_empty() {
        query.push(" AND r.id IN (");
        query.push(
            r#"
            SELECT rt.resource_id
            FROM resource_tags rt
            JOIN tags t2 ON rt.tag_id = t2.id
            WHERE t2.user_id = "#,
        );
        query.push_bind(user_id);
        query.push(" AND t2.name IN (");
        
        for tag in &filters.tags {
            query.push_bind(tag);
            bind_values.push(QueryValue::Text(tag.clone()));
        }
        
        query.push(") GROUP BY rt.resource_id HAVING COUNT(DISTINCT t2.name) = ");
        query.push_bind(filters.tags.len() as i64);
        query.push(")");
        bind_values.push(QueryValue::I64(user_id));
        bind_values.push(QueryValue::I64(filters.tags.len() as i64));
    }

    if let Some(date_from) = filters.date_from {
        query.push(" AND r.created_at >= ");
        query.push_bind(date_from);
        bind_values.push(QueryValue::I64(date_from));
    }

    if let Some(date_to) = filters.date_to {
        query.push(" AND r.created_at <= ");
        query.push_bind(date_to);
        bind_values.push(QueryValue::I64(date_to));
    }

    (query.sql().to_string(), bind_values)
}

// Helper functions to bind values to queries
fn bind_query_value<'a>(
    query: sqlx::query::QueryAs<'a, sqlx::Sqlite, ResourceWithTags, sqlx::sqlite::SqliteArguments<'a>>,
    value: &'a QueryValue,
) -> sqlx::query::QueryAs<'a, sqlx::Sqlite, ResourceWithTags, sqlx::sqlite::SqliteArguments<'a>> {
    match value {
        QueryValue::I64(v) => query.bind(*v),
        QueryValue::Text(v) => query.bind(v),
    }
}

fn bind_query_value_scalar<'a>(
    query: sqlx::query::QueryScalar<'a, sqlx::Sqlite, i64, sqlx::sqlite::SqliteArguments<'a>>,
    value: &'a QueryValue,
) -> sqlx::query::QueryScalar<'a, sqlx::Sqlite, i64, sqlx::sqlite::SqliteArguments<'a>> {
    match value {
        QueryValue::I64(v) => query.bind(*v),
        QueryValue::Text(v) => query.bind(v),
    }
}

#[derive(Clone)]
enum QueryValue {
    I64(i64),
    Text(String),
}
