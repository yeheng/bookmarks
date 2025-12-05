use sqlx::{Row, SqlitePool};

use crate::models::{
    Bookmark, BookmarkBatchAction, BookmarkBatchError, BookmarkBatchRequest, BookmarkBatchResult, BookmarkQuery,
    BookmarkWithTags, CreateBookmark, UpdateBookmark,
};
use crate::services::IndexerService;
use crate::utils::error::{AppError, AppResult};
use crate::utils::validation::validate_url;

pub struct BookmarkService;

impl BookmarkService {
    pub async fn create_bookmark(
        user_id: i64,
        bookmark_data: CreateBookmark,
        db_pool: &SqlitePool,
    ) -> AppResult<Bookmark> {
        // Validate URL
        validate_url(&bookmark_data.url)
            .then_some(())
            .ok_or_else(|| {
                AppError::BadRequest(format!("Invalid URL format: {}", bookmark_data.url))
            })?;

        // Start transaction - 事务内同时更新 bookmarks 和 bookmarks_fts
        let mut tx = db_pool.begin().await?;

        // Create bookmark
        let bookmark = sqlx::query_as::<_, Bookmark>(
            r#"
            INSERT INTO bookmarks (user_id, collection_id, title, url, description, is_favorite, is_private)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, collection_id, title, url, description, favicon_url, screenshot_url,
                      thumbnail_url, is_favorite, is_archived, is_private, is_read, visit_count,
                      last_visited, reading_time, difficulty_level, metadata, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(bookmark_data.collection_id)
        .bind(&bookmark_data.title)
        .bind(&bookmark_data.url)
        .bind(&bookmark_data.description)
        .bind(bookmark_data.is_favorite.unwrap_or(false))
        .bind(bookmark_data.is_private.unwrap_or(false))
        .fetch_one(&mut *tx)
        .await?;

        // Handle tags
        if let Some(tags) = bookmark_data.tags {
            for tag_name in tags {
                // Ensure tag exists (SQLite compatible)
                let tag_row = sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO tags (user_id, name)
                    VALUES ($1, $2);
                    SELECT id FROM tags WHERE user_id = $1 AND name = $2
                    "#,
                )
                .bind(user_id)
                .bind(&tag_name)
                .fetch_one(&mut *tx)
                .await?;
                let tag_id: i64 = tag_row.get("id");

                // Associate bookmark with tag
                sqlx::query(
                    "INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag_id) VALUES ($1, $2)",
                )
                .bind(bookmark.id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 同步 FTS 索引 - 使用 IndexerService
        IndexerService::index_bookmark(&mut tx, bookmark.id, user_id).await?;

        // Commit transaction - ACID 保证：要么 bookmarks 和 bookmarks_fts 都成功，要么都失败
        tx.commit().await?;

        Ok(bookmark)
    }

    pub async fn get_bookmarks(
        user_id: i64,
        query: BookmarkQuery,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<BookmarkWithTags>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("desc");

        // 验证排序字段和顺序，防止 SQL 注入
        let valid_sort_fields = [
            "created_at",
            "updated_at",
            "title",
            "visit_count",
            "last_visited",
        ];
        let valid_sort_orders = ["asc", "desc"];

        if !valid_sort_fields.contains(&sort_by) {
            return Err(AppError::BadRequest(format!(
                "Invalid sort field: '{}'. Valid fields are: {}",
                sort_by,
                valid_sort_fields.join(", ")
            )));
        }

        if !valid_sort_orders.contains(&sort_order) {
            return Err(AppError::BadRequest(format!(
                "Invalid sort order: '{}'. Valid orders are: {}",
                sort_order,
                valid_sort_orders.join(", ")
            )));
        }

        // 使用 QueryBuilder 构建动态查询 - 自动管理参数绑定
        let mut query_builder = sqlx::QueryBuilder::new(
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
            WHERE b.user_id =
            "#,
        );

        // 自动绑定 user_id - 不需要手动管理参数索引
        query_builder.push_bind(user_id);

        // 动态添加过滤条件 - QueryBuilder 自动管理参数绑定
        if let Some(collection_id) = query.collection_id {
            query_builder.push(" AND b.collection_id = ");
            query_builder.push_bind(collection_id);
        }

        if let Some(is_favorite) = query.is_favorite {
            query_builder.push(" AND b.is_favorite = ");
            query_builder.push_bind(is_favorite);
        }

        if let Some(is_archived) = query.is_archived {
            query_builder.push(" AND b.is_archived = ");
            query_builder.push_bind(is_archived);
        }

        if let Some(is_private) = query.is_private {
            query_builder.push(" AND b.is_private = ");
            query_builder.push_bind(is_private);
        }

        if let Some(is_read) = query.is_read {
            query_builder.push(" AND b.is_read = ");
            query_builder.push_bind(is_read);
        }

        // 搜索条件 - 简化为单次绑定
        if let Some(ref search_term) = query.search {
            query_builder.push(" AND (b.title LIKE '%' || ");
            query_builder.push_bind(search_term);
            query_builder.push(" || '%' OR COALESCE(b.description, '') LIKE '%' || ");
            query_builder.push_bind(search_term);
            query_builder.push(" || '%')");
        }

        // Tags 过滤 - 使用 QueryBuilder 处理 IN 子句
        if let Some(ref tags) = query.tags {
            if !tags.is_empty() {
                query_builder.push(
                    " AND b.id IN (
                        SELECT bookmark_id
                        FROM bookmark_tags
                        JOIN tags ON bookmark_tags.tag_id = tags.id
                        WHERE tags.name IN (",
                );

                // 使用 separated 方法处理 IN 子句中的多个值
                let mut separated = query_builder.separated(", ");
                for tag in tags {
                    separated.push_bind(tag);
                }

                query_builder.push(") GROUP BY bookmark_id HAVING COUNT(DISTINCT tags.id) = ");
                query_builder.push_bind(tags.len() as i64);
                query_builder.push(")");
            }
        }

        // 添加 GROUP BY
        query_builder.push(" GROUP BY b.id, c.name, c.color");

        // 添加排序 - 已验证的字段名可以安全拼接
        let sort_field = match sort_by {
            "title" => "b.title",
            "created_at" => "b.created_at",
            "updated_at" => "b.updated_at",
            "visit_count" => "b.visit_count",
            "last_visited" => "b.last_visited",
            _ => "b.created_at", // 安全回退
        };

        let sort_direction = match sort_order {
            "ASC" | "asc" => "ASC",
            "DESC" | "desc" => "DESC",
            _ => "DESC", // 安全回退
        };

        query_builder.push(format!(" ORDER BY {} {}", sort_field, sort_direction));

        // 添加分页 - QueryBuilder 自动管理参数
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // 执行查询
        let bookmarks = query_builder
            .build_query_as::<BookmarkWithTags>()
            .fetch_all(db_pool)
            .await?;

        Ok(bookmarks)
    }

    pub async fn get_bookmark_by_id(
        user_id: i64,
        bookmark_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<BookmarkWithTags>> {
        let bookmark = sqlx::query_as::<_, BookmarkWithTags>(
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
            WHERE b.id = $1 AND b.user_id = $2
            GROUP BY b.id, c.name, c.color
            "#,
        )
        .bind(bookmark_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        Ok(bookmark)
    }

    pub async fn update_bookmark(
        user_id: i64,
        bookmark_id: i64,
        update_data: UpdateBookmark,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<BookmarkWithTags>> {
        // Validate URL if provided
        if let Some(ref url) = update_data.url {
            validate_url(url)
                .then_some(())
                .ok_or_else(|| AppError::BadRequest(format!("Invalid URL format: {}", url)))?;
        }

        // Start transaction - 同时更新 bookmarks 和 bookmarks_fts
        let mut tx = db_pool.begin().await?;

        // 检查是否有书签字段需要更新（不包括标签）
        let has_bookmark_updates = update_data.title.is_some()
            || update_data.url.is_some()
            || update_data.description.is_some()
            || update_data.collection_id.is_some()
            || update_data.clear_collection_id.is_some()
            || update_data.is_favorite.is_some()
            || update_data.is_archived.is_some()
            || update_data.is_private.is_some()
            || update_data.is_read.is_some()
            || update_data.reading_time.is_some()
            || update_data.difficulty_level.is_some();

        // 检查是否有任何字段需要更新（包括标签）
        let has_any_updates = has_bookmark_updates || update_data.tags.is_some();

        if !has_any_updates {
            return Err(AppError::BadRequest(
                "No update fields provided".to_string(),
            ));
        }

        // 获取或更新书签
        let bookmark = if has_bookmark_updates {
            // 使用 COALESCE 来只更新提供的字段
            sqlx::query_as::<_, Bookmark>(
                r#"
                UPDATE bookmarks SET
                    title = COALESCE($1, title),
                    url = COALESCE($2, url),
                    description = COALESCE($3, description),
                    collection_id = CASE WHEN $4 THEN NULL ELSE COALESCE($5, collection_id) END,
                    is_favorite = COALESCE($6, is_favorite),
                    is_archived = COALESCE($7, is_archived),
                    is_private = COALESCE($8, is_private),
                    is_read = COALESCE($9, is_read),
                    reading_time = COALESCE($10, reading_time),
                    difficulty_level = COALESCE($11, difficulty_level),
                    updated_at = CAST(strftime('%s', 'now') AS INTEGER)
                WHERE id = $12 AND user_id = $13
                RETURNING id, user_id, collection_id, title, url, description, favicon_url,
                          screenshot_url, thumbnail_url, is_favorite,
                          is_archived, is_private, is_read, visit_count, last_visited,
                          reading_time, difficulty_level, metadata, created_at, updated_at
                "#,
            )
            .bind(update_data.title.as_ref())
            .bind(update_data.url.as_ref())
            .bind(update_data.description.as_ref())
            .bind(update_data.clear_collection_id.unwrap_or(false))
            .bind(update_data.collection_id)
            .bind(update_data.is_favorite)
            .bind(update_data.is_archived)
            .bind(update_data.is_private)
            .bind(update_data.is_read)
            .bind(update_data.reading_time)
            .bind(update_data.difficulty_level)
            .bind(bookmark_id)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
        } else {
            // 如果只更新标签，先获取现有书签
            sqlx::query_as::<_, Bookmark>(
                r#"
                SELECT id, user_id, collection_id, title, url, description, favicon_url,
                       screenshot_url, thumbnail_url, is_favorite, is_archived, is_private, 
                       is_read, visit_count, last_visited, reading_time, difficulty_level, 
                       metadata, created_at, updated_at
                FROM bookmarks 
                WHERE id = $1 AND user_id = $2
                "#,
            )
            .bind(bookmark_id)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
        };

        // 如果 bookmark 不存在，提前返回
        let Some(ref updated_bookmark) = bookmark else {
            tx.rollback().await?;
            return Ok(None);
        };

        // Handle tags update
        if let Some(tags) = update_data.tags {
            // Delete existing tag associations
            sqlx::query("DELETE FROM bookmark_tags WHERE bookmark_id = $1")
                .bind(bookmark_id)
                .execute(&mut *tx)
                .await?;

            // Add new tag associations
            for tag_name in tags {
                let tag_row = sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO tags (user_id, name)
                    VALUES ($1, $2);
                    SELECT id FROM tags WHERE user_id = $1 AND name = $2
                    "#,
                )
                .bind(user_id)
                .bind(&tag_name)
                .fetch_one(&mut *tx)
                .await?;
                let tag_id: i64 = tag_row.get("id");

                sqlx::query(
                    "INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag_id) VALUES ($1, $2)",
                )
                .bind(bookmark_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 同步 FTS 索引 - 使用 IndexerService
        IndexerService::index_bookmark(&mut tx, updated_bookmark.id, user_id).await?;

        // Commit transaction - ACID 保证
        tx.commit().await?;

        // 构建包含 tags 和 collection 信息的完整响应
        let bookmark_with_tags = if let Some(bookmark) = bookmark {
            // 获取标签
            let tags = sqlx::query_scalar(
                "SELECT t.name FROM tags t 
                 JOIN bookmark_tags bt ON t.id = bt.tag_id 
                 WHERE bt.bookmark_id = $1 
                 ORDER BY t.name"
            )
            .bind(bookmark.id)
            .fetch_all(db_pool)
            .await?;

            // 获取收藏夹信息
            let collection_info = if let Some(collection_id) = bookmark.collection_id {
                sqlx::query_as::<_, (String, String)>(
                    "SELECT name, color FROM collections WHERE id = $1"
                )
                .bind(collection_id)
                .fetch_optional(db_pool)
                .await?
                .map(|(name, color)| (name, Some(color)))
            } else {
                None
            };

            Some(BookmarkWithTags {
                bookmark,
                tags,
                collection_name: collection_info.as_ref().map(|(name, _)| name.clone()),
                collection_color: collection_info.as_ref().and_then(|(_, color)| color.clone()),
            })
        } else {
            None
        };

        Ok(bookmark_with_tags)
    }

    pub async fn delete_bookmark(
        user_id: i64,
        bookmark_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        // Start transaction - 同时删除 bookmarks 和 bookmarks_fts
        let mut tx = db_pool.begin().await?;

        // 首先从 FTS 索引中删除（必须在 bookmarks 删除之前，因为需要 rowid 关联）
        sqlx::query("DELETE FROM bookmarks_fts WHERE rowid = $1")
            .bind(bookmark_id)
            .execute(&mut *tx)
            .await?;

        // 删除 bookmark（CASCADE 会自动删除 bookmark_tags）
        let result = sqlx::query("DELETE FROM bookmarks WHERE id = $1 AND user_id = $2")
            .bind(bookmark_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        let was_deleted = result.rows_affected() > 0;

        // Commit transaction - ACID 保证
        tx.commit().await?;

        Ok(was_deleted)
    }

    #[allow(dead_code)]
    pub async fn bookmark_exists(user_id: i64, url: &str, db_pool: &SqlitePool) -> AppResult<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE user_id = $1 AND url = $2)",
        )
        .bind(user_id)
        .bind(url)
        .fetch_one(db_pool)
        .await?;

        Ok(exists)
    }

    pub async fn batch_process(
        user_id: i64,
        request: BookmarkBatchRequest,
        db_pool: &SqlitePool,
    ) -> AppResult<BookmarkBatchResult> {
        let BookmarkBatchRequest {
            action,
            bookmark_ids,
            data,
        } = request;

        let mut result = BookmarkBatchResult {
            processed: 0,
            failed: 0,
            errors: Vec::new(),
        };

        for bookmark_id in bookmark_ids {
            let operation = match action {
                BookmarkBatchAction::Delete => {
                    Self::delete_bookmark(user_id, bookmark_id, db_pool).await
                }
                BookmarkBatchAction::Move => {
                    let collection_id = data
                        .as_ref()
                        .and_then(|data| data.collection_id)
                        .ok_or_else(|| {
                            AppError::BadRequest(
                                "Batch move operation requires collection_id".to_string(),
                            )
                        })?;

                    Self::move_bookmark(user_id, bookmark_id, collection_id, db_pool).await
                }
                BookmarkBatchAction::AddTags => {
                    let tags = data
                        .as_ref()
                        .and_then(|data| data.tags.clone())
                        .filter(|tags| !tags.is_empty())
                        .ok_or_else(|| {
                            AppError::BadRequest(
                                "Batch add_tags operation requires a non-empty tags list"
                                    .to_string(),
                            )
                        })?;

                    Self::add_tags(user_id, bookmark_id, tags, db_pool).await
                }
                BookmarkBatchAction::RemoveTags => {
                    let tags = data
                        .as_ref()
                        .and_then(|data| data.tags.clone())
                        .filter(|tags| !tags.is_empty())
                        .ok_or_else(|| {
                            AppError::BadRequest(
                                "Batch remove_tags operation requires a non-empty tags list"
                                    .to_string(),
                            )
                        })?;

                    Self::remove_tags(user_id, bookmark_id, tags, db_pool).await
                }
            };

            match operation {
                Ok(true) => result.processed += 1,
                Ok(false) => {
                    result.failed += 1;
                    result.errors.push(BookmarkBatchError {
                        bookmark_id,
                        reason: "Bookmark not found".to_string(),
                    });
                }
                Err(err) => {
                    result.failed += 1;
                    result.errors.push(BookmarkBatchError {
                        bookmark_id,
                        reason: err.to_string(),
                    });
                }
            }
        }

        Ok(result)
    }

    async fn move_bookmark(
        user_id: i64,
        bookmark_id: i64,
        collection_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE bookmarks
            SET collection_id = $1, updated_at = CAST(strftime('%s', 'now') AS INTEGER)
            WHERE id = $2 AND user_id = $3
            "#,
        )
        .bind(collection_id)
        .bind(bookmark_id)
        .bind(user_id)
        .execute(db_pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn add_tags(
        user_id: i64,
        bookmark_id: i64,
        tags: Vec<String>,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        let mut tx = db_pool.begin().await?;

        for tag_name in tags {
            let tag_row = sqlx::query(
                r#"
                INSERT OR IGNORE INTO tags (user_id, name)
                VALUES ($1, $2);
                SELECT id FROM tags WHERE user_id = $1 AND name = $2
                "#,
            )
            .bind(user_id)
            .bind(&tag_name)
            .fetch_one(&mut *tx)
            .await?;
            let tag_id: i64 = tag_row.get("id");

            sqlx::query(
                "INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag_id) VALUES ($1, $2)",
            )
            .bind(bookmark_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(true)
    }

    async fn remove_tags(
        user_id: i64,
        bookmark_id: i64,
        tags: Vec<String>,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        // SQLite doesn't support USING with DELETE, so we need a subquery
        let mut result = 0;
        for tag_name in tags {
            let delete_result = sqlx::query(
                r#"
                DELETE FROM bookmark_tags
                WHERE bookmark_id = $1
                  AND tag_id IN (
                    SELECT id FROM tags WHERE user_id = $2 AND name = $3
                  )
                "#,
            )
            .bind(bookmark_id)
            .bind(user_id)
            .bind(&tag_name)
            .execute(db_pool)
            .await?;
            result += delete_result.rows_affected();
        }

        Ok(result > 0)
    }
}
