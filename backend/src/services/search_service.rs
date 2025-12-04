use sqlx::{Row, SqlitePool};
use std::time::Instant;

use crate::models::{
    BookmarkWithTags, SearchFilters, SearchPagination, SearchResponse, SearchSuggestion, SearchType,
};
use crate::utils::error::AppResult;
use crate::utils::segmenter::prepare_for_search;

pub struct SearchService;

impl SearchService {
    pub async fn search_bookmarks(
        user_id: i64,
        filters: SearchFilters,
        db_pool: &SqlitePool,
    ) -> AppResult<SearchResponse> {
        let start = Instant::now();

        // 使用 jieba 对查询进行分词
        let search_keywords = prepare_for_search(Some(&filters.query));

        // 构建过滤条件 SQL
        let (filter_sql, binds, last_param) = Self::build_filter_sql(user_id, &filters);
        let limit_param = last_param + 1;
        let offset_param = last_param + 2;

        // 构建 FTS5 搜索查询
        // snippet() 函数用于生成高亮片段: snippet(table, column, prefix, suffix, ellipsis, max_tokens)
        let search_sql = format!(
            r#"
            SELECT
                b.id,
                b.user_id,
                b.collection_id,
                b.title,
                b.url,
                b.description,
                b.favicon_url,
                b.screenshot_url,
                b.thumbnail_url,
                b.is_favorite,
                b.is_archived,
                b.is_private,
                b.is_read,
                b.visit_count,
                b.last_visited,
                b.reading_time,
                b.difficulty_level,
                b.metadata,
                b.created_at,
                b.updated_at,
                COALESCE(
                    CASE
                        WHEN COUNT(t.name) > 0
                        THEN '[' || GROUP_CONCAT('"' || REPLACE(t.name, '"', '""') || '"', ',') || ']'
                        ELSE '[]'
                    END,
                    '[]'
                ) as tags,
                c.name as collection_name,
                c.color as collection_color
            FROM bookmarks b
            JOIN bookmarks_fts fts ON b.id = fts.rowid
            LEFT JOIN collections c ON b.collection_id = c.id
            LEFT JOIN bookmark_tags bt ON b.id = bt.bookmark_id
            LEFT JOIN tags t ON bt.tag_id = t.id
            {filter_sql}
            GROUP BY b.id, c.name, c.color
            ORDER BY rank
            LIMIT ${} OFFSET ${}
            "#,
            limit_param, offset_param
        );

        let mut query_builder = sqlx::query_as::<_, BookmarkWithTags>(&search_sql).bind(user_id);

        // 添加 FTS5 MATCH 参数（始终在第一个过滤参数之前）
        query_builder = query_builder.bind(&search_keywords);

        // 添加其他过滤器参数
        for bind in &binds {
            query_builder = match bind {
                BindValue::I64(value) => query_builder.bind(*value),
                BindValue::Text(value) => query_builder.bind(value.clone()),
            };
        }

        let bookmarks = query_builder
            .bind(filters.limit)
            .bind(filters.offset)
            .fetch_all(db_pool)
            .await?;

        // 统计总数（也使用 FTS5 过滤）
        let count_sql = format!("SELECT COUNT(*) FROM bookmarks b JOIN bookmarks_fts fts ON b.id = fts.rowid {filter_sql}");
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(user_id);
        count_query = count_query.bind(&search_keywords);
        for bind in &binds {
            count_query = match bind {
                BindValue::I64(value) => count_query.bind(*value),
                BindValue::Text(value) => count_query.bind(value.clone()),
            };
        }

        let total = count_query.fetch_one(db_pool).await?;

        let elapsed = start.elapsed().as_secs_f64();
        let page = (filters.offset / filters.limit) + 1;
        let total_pages = if total == 0 {
            0
        } else {
            (total + filters.limit - 1) / filters.limit
        };

        Ok(SearchResponse {
            items: bookmarks,
            pagination: SearchPagination {
                page,
                limit: filters.limit,
                total,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
            search_time: elapsed,
            highlights: None, // 如果需要高亮，可以使用 snippet() 函数提取
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
                SELECT b.title as suggestion,
                       'bookmark' as suggestion_type,
                       COUNT(*) as usage_count,
                       MAX(b.updated_at) as last_used_at
                FROM bookmarks b
                WHERE b.user_id = $1
                  AND lower(b.title) LIKE lower($2 || '%')
                GROUP BY b.title

                UNION ALL

                SELECT t.name as suggestion,
                       'tag' as suggestion_type,
                       COUNT(bt.bookmark_id) as usage_count,
                       MAX(t.updated_at) as last_used_at
                FROM tags t
                LEFT JOIN bookmark_tags bt ON t.id = bt.tag_id
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
                    .unwrap_or_else(|| "bookmark".to_string()),
                count: row.get::<Option<i64>, _>("usage_count").unwrap_or(0),
                last_used_at: row.get("last_used_at"),
            })
            .collect())
    }

    /// 构建过滤条件 SQL
    ///
    /// 返回: (filter_sql, binds, last_param_index)
    /// 注意：FTS5 MATCH 参数始终是第 2 个参数（第 1 个是 user_id）
    fn build_filter_sql(user_id: i64, filters: &SearchFilters) -> (String, Vec<BindValue>, i32) {
        let mut sql = String::from("WHERE b.user_id = $1");
        // FTS5 MATCH 参数是 $2，所以从 $3 开始计数其他过滤器
        let mut param_index = 2;
        let mut binds = Vec::new();

        // 添加 FTS5 MATCH 条件（根据搜索类型）
        match filters.search_type {
            SearchType::Title => {
                sql.push_str(" AND bookmarks_fts MATCH 'title:' || $2");
            }
            SearchType::Content => {
                sql.push_str(" AND bookmarks_fts MATCH 'description:' || $2");
            }
            SearchType::Url => {
                sql.push_str(" AND bookmarks_fts MATCH 'url:' || $2");
            }
            SearchType::All => {
                // 搜索所有字段（title, description, tags, url）
                sql.push_str(" AND bookmarks_fts MATCH $2");
            }
        }

        // 其他过滤条件
        if let Some(collection_id) = filters.collection_id {
            param_index += 1;
            sql.push_str(&format!(" AND b.collection_id = ${}", param_index));
            binds.push(BindValue::I64(collection_id));
        }

        if !filters.tags.is_empty() {
            param_index += 1;
            let tag_user_param = param_index;
            let tag_placeholders = filters
                .tags
                .iter()
                .enumerate()
                .map(|(i, _)| format!("${}", param_index + 1 + (i as i32)))
                .collect::<Vec<_>>()
                .join(",");
            param_index += filters.tags.len() as i32;

            sql.push_str(&format!(
                " AND b.id IN (
                    SELECT bt.bookmark_id
                    FROM bookmark_tags bt
                    JOIN tags t2 ON bt.tag_id = t2.id
                    WHERE t2.user_id = ${} AND t2.name IN ({})
                    GROUP BY bt.bookmark_id
                    HAVING COUNT(DISTINCT t2.name) = {}
                )",
                tag_user_param,
                tag_placeholders,
                filters.tags.len()
            ));
            binds.push(BindValue::I64(user_id));
            for tag in &filters.tags {
                binds.push(BindValue::Text(tag.clone()));
            }
        }

        if let Some(date_from) = filters.date_from {
            param_index += 1;
            sql.push_str(&format!(" AND b.created_at >= ${}", param_index));
            binds.push(BindValue::I64(date_from));
        }

        if let Some(date_to) = filters.date_to {
            param_index += 1;
            sql.push_str(&format!(" AND b.created_at <= ${}", param_index));
            binds.push(BindValue::I64(date_to));
        }

        (sql, binds, param_index)
    }
}

#[derive(Clone)]
enum BindValue {
    I64(i64),
    Text(String),
}
