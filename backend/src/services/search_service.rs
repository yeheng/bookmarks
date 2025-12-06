use sqlx::{Row, SqlitePool};
use std::time::Instant;

use crate::models::{
    FilterCriteria, ResourceWithTags, SearchFilters, SearchPagination, SearchResponse,
    SearchSuggestion, SearchType,
};
use crate::utils::error::AppResult;
use crate::utils::segmenter::prepare_for_search;

pub struct SearchService;

impl SearchService {
    pub async fn search_resources(
        user_id: i64,
        filters: SearchFilters,
        db_pool: &SqlitePool,
    ) -> AppResult<SearchResponse> {
        let start = Instant::now();

        // 根据配置对查询进行分词处理
        let search_keywords = prepare_for_search(Some(&filters.query));

        // 构建查询
        let query_builder = QueryBuilder::new(user_id, &search_keywords);
        let (search_sql, count_sql, bind_values) = query_builder
            .with_search_type(&filters.search_type)
            .with_filters(&filters.filters)
            .build();

        // 执行主查询
        let mut query = sqlx::query_as::<_, ResourceWithTags>(&search_sql);
        query = query.bind(user_id).bind(&search_keywords);
        for bind in &bind_values {
            query = match bind {
                BindValue::I64(value) => query.bind(*value),
                BindValue::Text(value) => query.bind(value),
            };
        }

        let resources = query
            .bind(filters.pagination.limit)
            .bind(filters.pagination.offset)
            .fetch_all(db_pool)
            .await?;

        // 执行计数查询
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        count_query = count_query.bind(user_id).bind(&search_keywords);
        for bind in &bind_values {
            count_query = match bind {
                BindValue::I64(value) => count_query.bind(*value),
                BindValue::Text(value) => count_query.bind(value),
            };
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

/// FTS5 查询构建器 - 消除字符串拼接,提高可维护性
struct QueryBuilder {
    user_id: i64,
    search_type: Option<SearchType>,
    filters: Option<FilterCriteria>,
}

impl QueryBuilder {
    fn new(user_id: i64, _search_keywords: &str) -> Self {
        Self {
            user_id,
            search_type: None,
            filters: None,
        }
    }

    fn with_search_type(mut self, search_type: &SearchType) -> Self {
        self.search_type = Some(match search_type {
            SearchType::Title => SearchType::Title,
            SearchType::Content => SearchType::Content,
            SearchType::Url => SearchType::Url,
            SearchType::All => SearchType::All,
        });
        self
    }

    fn with_filters(mut self, filters: &FilterCriteria) -> Self {
        self.filters = Some(filters.clone());
        self
    }

    /// 构建完整的 SQL 查询
    /// 返回: (search_sql, count_sql, bind_values)
    fn build(self) -> (String, String, Vec<BindValue>) {
        let (where_clause, bind_values) = self.build_where_clause();

        // 计算 LIMIT 和 OFFSET 的参数位置
        // $1 = user_id, $2 = search_keywords, $3+ = filter bind_values
        let limit_param_index = 3 + bind_values.len();
        let offset_param_index = limit_param_index + 1;

        let search_sql = format!(
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
                r.reading_time,
                r.difficulty_level,
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
            {}
            GROUP BY r.id, c.name, c.color
            ORDER BY rank
            LIMIT ${} OFFSET ${}
            "#,
            where_clause, limit_param_index, offset_param_index
        );

        let count_sql = format!(
            "SELECT COUNT(*) FROM resources r JOIN resources_fts fts ON r.id = fts.rowid {}",
            where_clause
        );

        (search_sql, count_sql, bind_values)
    }

    /// 构建 WHERE 子句
    /// 返回: (where_clause, bind_values)
    fn build_where_clause(&self) -> (String, Vec<BindValue>) {
        let mut conditions = vec!["r.user_id = $1".to_string()];
        let mut bind_values = Vec::new();

        // 添加 FTS5 MATCH 条件 (根据搜索类型)
        let fts_condition = match self.search_type.as_ref().unwrap_or(&SearchType::All) {
            SearchType::Title => "resources_fts MATCH 'title:' || $2",
            SearchType::Content => "resources_fts MATCH 'content:' || $2",
            SearchType::Url => "resources_fts MATCH 'url:' || $2",
            SearchType::All => "resources_fts MATCH $2",
        };
        conditions.push(fts_condition.to_string());

        // 添加其他过滤条件
        if let Some(filters) = &self.filters {
            let mut param_index = 2; // 从 $3 开始 ($1 是 user_id, $2 是 FTS MATCH)

            if let Some(collection_id) = filters.collection_id {
                param_index += 1;
                conditions.push(format!("r.collection_id = ${}", param_index));
                bind_values.push(BindValue::I64(collection_id));
            }

            if !filters.tags.is_empty() {
                param_index += 1;
                let tag_user_param = param_index;
                let tag_placeholders: Vec<String> = filters
                    .tags
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("${}", param_index + 1 + (i as i32)))
                    .collect();
                param_index += filters.tags.len() as i32;

                conditions.push(format!(
                    "r.id IN (
                        SELECT rt.resource_id
                        FROM resource_tags rt
                        JOIN tags t2 ON rt.tag_id = t2.id
                        WHERE t2.user_id = ${} AND t2.name IN ({})
                        GROUP BY rt.resource_id
                        HAVING COUNT(DISTINCT t2.name) = {}
                    )",
                    tag_user_param,
                    tag_placeholders.join(","),
                    filters.tags.len()
                ));
                bind_values.push(BindValue::I64(self.user_id));
                for tag in &filters.tags {
                    bind_values.push(BindValue::Text(tag.clone()));
                }
            }

            if let Some(date_from) = filters.date_from {
                param_index += 1;
                conditions.push(format!("r.created_at >= ${}", param_index));
                bind_values.push(BindValue::I64(date_from));
            }

            if let Some(date_to) = filters.date_to {
                param_index += 1;
                conditions.push(format!("r.created_at <= ${}", param_index));
                bind_values.push(BindValue::I64(date_to));
            }
        }

        let where_clause = format!("WHERE {}", conditions.join(" AND "));
        (where_clause, bind_values)
    }
}

#[derive(Clone)]
enum BindValue {
    I64(i64),
    Text(String),
}
