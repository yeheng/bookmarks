use std::time::Instant;

use crate::models::{
    BookmarkWithTags, SearchFilters, SearchPagination, SearchResponse, SearchType,
};
use crate::services::TantivyIndexManager;
use crate::utils::error::AppResult;
use sqlx::SqlitePool;

/// Tantivy 搜索服务
///
/// 提供基于 Tantivy 的全文搜索功能，支持相关性排序和高亮显示
pub struct TantivySearchService;

impl TantivySearchService {
    /// 主搜索方法
    ///
    /// 结合 Tantivy 全文搜索和 SQLite 数据查询，提供完整的搜索结果
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户ID，用于数据隔离
    /// * `filters` - 搜索过滤器
    /// * `index_manager` - Tantivy 索引管理器
    /// * `db_pool` - SQLite 数据库连接池
    ///
    /// # 返回
    ///
    /// 返回包含搜索结果和分页信息的 SearchResponse
    pub async fn search_bookmarks(
        user_id: i64,
        filters: SearchFilters,
        index_manager: &TantivyIndexManager,
        db_pool: &SqlitePool,
    ) -> AppResult<SearchResponse> {
        let start = Instant::now();

        // 1. 使用 Tantivy 搜索获取 bookmark_ids + scores
        let search_results = if filters.query.trim().is_empty() {
            // 如果没有搜索词，返回所有结果（按时间排序）
            Self::search_without_text(&filters, user_id, db_pool).await?
        } else {
            // 有搜索词时使用全文搜索
            Self::tantivy_search(&filters, user_id, index_manager).await?
        };

        // 2. 如果 Tantivy 搜索没有结果，直接返回空响应
        if search_results.results.is_empty() {
            return Ok(SearchResponse {
                items: vec![],
                pagination: SearchPagination {
                    page: (filters.offset / filters.limit) + 1,
                    limit: filters.limit,
                    total: 0,
                    total_pages: 0,
                    has_next: false,
                    has_prev: false,
                },
                search_time: start.elapsed().as_secs_f64(),
                highlights: None,
            });
        }

        // 3. 使用 SQLite 查询完整书签信息(JOIN tags, collections)
        let bookmark_ids: Vec<i64> = search_results.results
            .iter()
            .map(|result| result.bookmark_id)
            .collect();

        let bookmarks = Self::fetch_bookmarks_by_ids(&bookmark_ids, user_id, db_pool).await?;

        // 4. 合并结果（添加 score, highlights）
        let mut enhanced_bookmarks = Vec::new();
        for bookmark in bookmarks {
            // 查找对应的分数
            if search_results.results.iter()
                .any(|r| r.bookmark_id == bookmark.bookmark.id) {
                // TODO: 添加分数和高亮信息到返回结构中
                // 暂时不修改 BookmarkWithTags 结构，后续可以考虑扩展
                enhanced_bookmarks.push(bookmark);
            }
        }

        // 5. 计算分页信息
        let page = (filters.offset / filters.limit) + 1;
        let total_pages = if search_results.total == 0 {
            0
        } else {
            (search_results.total + filters.limit as usize - 1) / filters.limit as usize
        };

        Ok(SearchResponse {
            items: enhanced_bookmarks,
            pagination: SearchPagination {
                page,
                limit: filters.limit,
                total: search_results.total as i64,
                total_pages: total_pages as i64,
                has_next: page < total_pages as i64,
                has_prev: page > 1,
            },
            search_time: start.elapsed().as_secs_f64(),
            highlights: None,
        })
    }

    /// Tantivy 全文搜索
    ///
    /// # 参数
    ///
    /// * `filters` - 搜索过滤器
    /// * `user_id` - 用户ID
    /// * `index_manager` - Tantivy 索引管理器
    ///
    /// # 返回
    ///
    /// 返回 Tantivy 搜索结果
    async fn tantivy_search(
        filters: &SearchFilters,
        user_id: i64,
        index_manager: &TantivyIndexManager,
    ) -> AppResult<crate::services::tantivy_index::TantivySearchResponse> {
        // 构建搜索查询
        let search_query = Self::build_search_query(filters);

        // 执行 Tantivy 搜索
        index_manager.search(
            &search_query,
            user_id,
            filters.limit as usize,
            filters.offset as usize,
        ).map_err(|e| crate::utils::error::AppError::Internal(format!("Tantivy search failed: {}", e)))
    }

    /// 无文本搜索（返回所有书签，按时间排序）
    ///
    /// 当没有搜索词时使用 SQLite 查询所有书签
    async fn search_without_text(
        filters: &SearchFilters,
        user_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<crate::services::tantivy_index::TantivySearchResponse> {
        // 获取总数
        let total_sql = "SELECT COUNT(*) FROM bookmarks WHERE user_id = $1";
        let total = sqlx::query_scalar::<_, i64>(total_sql)
            .bind(user_id)
            .fetch_one(db_pool)
            .await?;

        // 获取分页数据（仅ID，用于后续完整查询）
        let ids_sql = r#"
            SELECT id FROM bookmarks
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
        "#;
        let bookmark_ids = sqlx::query_scalar::<_, i64>(ids_sql)
            .bind(user_id)
            .bind(filters.limit)
            .bind(filters.offset)
            .fetch_all(db_pool)
            .await?;

        // 转换为 TantivySearchResult 格式，使用创建时间作为排序依据
        let results = bookmark_ids
            .into_iter()
            .map(|id| crate::services::tantivy_index::TantivySearchResult {
                bookmark_id: id,
                score: 1.0, // 无搜索时所有结果分数相同
            })
            .collect();

        Ok(crate::services::tantivy_index::TantivySearchResponse {
            results,
            total: total as usize,
        })
    }

    /// 根据 ID 列表批量获取书签详细信息
    ///
    /// # 参数
    ///
    /// * `bookmark_ids` - 书签ID列表
    /// * `user_id` - 用户ID
    /// * `db_pool` - 数据库连接池
    ///
    /// # 返回
    ///
    /// 返回包含标签和集合信息的书签列表
    async fn fetch_bookmarks_by_ids(
        bookmark_ids: &[i64],
        user_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<BookmarkWithTags>> {
        if bookmark_ids.is_empty() {
            return Ok(vec![]);
        }

        // 构建 IN 子句的占位符
        let placeholders: Vec<String> = (0..bookmark_ids.len())
            .map(|i| format!("${}", i + 3)) // 从 $3 开始，因为 $1 是 user_id, $2 是 count(*)
            .collect();

        let sql = format!(
            r#"
            SELECT
                b.id, b.user_id, b.collection_id, b.title, b.url, b.description,
                b.favicon_url, b.screenshot_url, b.thumbnail_url,
                b.is_favorite, b.is_archived, b.is_private, b.is_read,
                b.visit_count, b.last_visited, b.reading_time, b.difficulty_level,
                b.metadata, b.created_at, b.updated_at,
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
            LEFT JOIN collections c ON b.collection_id = c.id
            LEFT JOIN bookmark_tags bt ON b.id = bt.bookmark_id
            LEFT JOIN tags t ON bt.tag_id = t.id
            WHERE b.user_id = $1 AND b.id IN ({})
            GROUP BY b.id, c.name, c.color
            ORDER BY b.created_at DESC
            "#,
            placeholders.join(",")
        );

        let mut query = sqlx::query_as::<_, BookmarkWithTags>(&sql).bind(user_id);
        for id in bookmark_ids {
            query = query.bind(id);
        }

        let bookmarks = query.fetch_all(db_pool).await?;
        Ok(bookmarks)
    }

    /// 构建搜索查询字符串
    ///
    /// 根据搜索类型和查询内容构建适合 Tantivy 的查询字符串
    fn build_search_query(filters: &SearchFilters) -> String {
        match filters.search_type {
            SearchType::Title => format!("title:{}", filters.query),
            SearchType::Content => format!("description:{}", filters.query),
            SearchType::Url => format!("url:{}", filters.query),
            SearchType::All => filters.query.clone(), // 默认在所有字段中搜索
        }
    }

    /// 生成搜索结果高亮片段
    ///
    /// # 参数
    ///
    /// * `bookmark_id` - 书签ID
    /// * `query` - 搜索查询
    /// * `index_manager` - Tantivy 索引管理器
    ///
    /// # 返回
    ///
    /// 返回包含高亮片段的 HashMap，键为字段名，值为高亮片段列表
    pub fn generate_highlights(
        bookmark_id: i64,
        query: &str,
        index_manager: &crate::services::tantivy_index::TantivyIndexManager,
    ) -> std::collections::HashMap<String, Vec<String>> {
        match index_manager.generate_highlights(bookmark_id, query) {
            Ok(highlights) => highlights,
            Err(e) => {
                tracing::warn!("Failed to generate highlights for bookmark {}: {}", bookmark_id, e);
                std::collections::HashMap::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_without_query() {
        // TODO: 实现测试
        // 需要设置测试数据库和索引
    }

    #[tokio::test]
    async fn test_search_with_query() {
        // TODO: 实现测试
        // 需要设置测试数据库和索引
    }

    #[test]
    fn test_build_search_query() {
        let filters = SearchFilters {
            query: "rust programming".to_string(),
            search_type: SearchType::Title,
            collection_id: None,
            tags: vec![],
            date_from: None,
            date_to: None,
            limit: 10,
            offset: 0,
        };

        let query = TantivySearchService::build_search_query(&filters);
        assert_eq!(query, "title:rust programming");

        let filters_all = SearchFilters {
            search_type: SearchType::All,
            ..filters
        };

        let query_all = TantivySearchService::build_search_query(&filters_all);
        assert_eq!(query_all, "rust programming");
    }
}