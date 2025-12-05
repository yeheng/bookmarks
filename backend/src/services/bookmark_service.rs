use sqlx::{Row, SqlitePool};

use crate::models::{
    Bookmark, BookmarkBatchAction, BookmarkBatchError, BookmarkBatchRequest, BookmarkBatchResult,
    BookmarkExportFormat, BookmarkExportOptions, BookmarkExportPayload, BookmarkQuery,
    BookmarkVisitInfo, BookmarkWithTags, CreateBookmark, UpdateBookmark,
};
use crate::utils::error::{AppError, AppResult};
use crate::utils::segmenter::{prepare_for_search, prepare_tags_for_search};
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

        // 收集标签名称用于 FTS
        let mut tag_names = Vec::new();

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

                tag_names.push(tag_name);
            }
        }

        // 同步 FTS 索引 - 使用封装的私有方法
        Self::sync_fts_index(&mut tx, &bookmark, &tag_names, false).await?;

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

        // Validate sort_by and sort_order
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

        let mut sql = r#"
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
            WHERE b.user_id = $1
        "#
        .to_string();

        let mut param_count = 1;

        // Add filter conditions
        if let Some(_collection_id) = query.collection_id {
            param_count += 1;
            sql.push_str(&format!(" AND b.collection_id = ${}", param_count));
        }

        if let Some(_is_favorite) = query.is_favorite {
            param_count += 1;
            sql.push_str(&format!(" AND b.is_favorite = ${}", param_count));
        }

        if let Some(_is_archived) = query.is_archived {
            param_count += 1;
            sql.push_str(&format!(" AND b.is_archived = ${}", param_count));
        }

        if let Some(_is_private) = query.is_private {
            param_count += 1;
            sql.push_str(&format!(" AND b.is_private = ${}", param_count));
        }

        if let Some(_is_read) = query.is_read {
            param_count += 1;
            sql.push_str(&format!(" AND b.is_read = ${}", param_count));
        }

        if let Some(ref _search_term) = query.search {
            param_count += 1;
            sql.push_str(&format!(
                " AND (b.title LIKE '%' || ${} || '%' OR COALESCE(b.description, '') LIKE '%' || ${} || '%')",
                param_count, param_count
            ));
        }

        if let Some(ref tags) = query.tags {
            if !tags.is_empty() {
                param_count += 1;
                let tag_placeholders = tags
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("${}", param_count + 1 + i))
                    .collect::<Vec<_>>()
                    .join(",");
                param_count += tags.len();

                sql.push_str(&format!(
                    " AND b.id IN (
                        SELECT bookmark_id
                        FROM bookmark_tags
                        JOIN tags ON bookmark_tags.tag_id = tags.id
                        WHERE tags.name IN ({})
                        GROUP BY bookmark_id
                        HAVING COUNT(DISTINCT tags.id) = {}
                    )",
                    tag_placeholders,
                    tags.len()
                ));
            }
        }

        // Add ordering with validated sort field
        let sort_field = match sort_by {
            "title" => "b.title",
            "created_at" => "b.created_at",
            "updated_at" => "b.updated_at",
            "visit_count" => "b.visit_count",
            "last_visited" => "b.last_visited",
            _ => "b.created_at", // Default safe fallback
        };

        let sort_direction = match sort_order {
            "ASC" | "asc" => "ASC",
            "DESC" | "desc" => "DESC",
            _ => "DESC", // Default safe fallback
        };

        sql.push_str(&format!(
            " GROUP BY b.id, c.name, c.color ORDER BY {} {} LIMIT ${} OFFSET ${}",
            sort_field,
            sort_direction,
            param_count + 1,
            param_count + 2
        ));

        let mut query_builder = sqlx::query_as::<_, BookmarkWithTags>(&sql).bind(user_id);

        // Bind parameters
        if let Some(collection_id) = query.collection_id {
            query_builder = query_builder.bind(collection_id);
        }
        if let Some(is_favorite) = query.is_favorite {
            query_builder = query_builder.bind(is_favorite);
        }
        if let Some(is_archived) = query.is_archived {
            query_builder = query_builder.bind(is_archived);
        }
        if let Some(is_private) = query.is_private {
            query_builder = query_builder.bind(is_private);
        }
        if let Some(is_read) = query.is_read {
            query_builder = query_builder.bind(is_read);
        }
        if let Some(search_term) = query.search {
            query_builder = query_builder.bind(search_term);
        }
        if let Some(tags) = query.tags {
            for tag in tags {
                query_builder = query_builder.bind(tag);
            }
        }

        let bookmarks = query_builder
            .bind(limit)
            .bind(offset)
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
    ) -> AppResult<Option<Bookmark>> {
        // Validate URL if provided
        if let Some(ref url) = update_data.url {
            validate_url(url)
                .then_some(())
                .ok_or_else(|| AppError::BadRequest(format!("Invalid URL format: {}", url)))?;
        }

        // Start transaction - 同时更新 bookmarks 和 bookmarks_fts
        let mut tx = db_pool.begin().await?;

        // 构建更新查询 - 使用Option字段逐个更新
        let bookmark = if update_data.title.is_some()
            || update_data.url.is_some()
            || update_data.description.is_some()
            || update_data.collection_id.is_some()
            || update_data.clear_collection_id.is_some()
            || update_data.is_favorite.is_some()
            || update_data.is_archived.is_some()
            || update_data.is_private.is_some()
            || update_data.is_read.is_some()
            || update_data.reading_time.is_some()
            || update_data.difficulty_level.is_some()
        {
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
            return Err(AppError::BadRequest(
                "No update fields provided".to_string(),
            ));
        };

        // 如果 bookmark 不存在，提前返回
        let Some(ref updated_bookmark) = bookmark else {
            tx.rollback().await?;
            return Ok(None);
        };

        // 收集标签名称用于 FTS
        let mut tag_names = Vec::new();

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

                tag_names.push(tag_name);
            }
        } else {
            // 如果没有更新标签，获取现有标签用于 FTS 更新
            let existing_tags = sqlx::query(
                r#"
                SELECT t.name
                FROM tags t
                JOIN bookmark_tags bt ON t.id = bt.tag_id
                WHERE bt.bookmark_id = $1 AND t.user_id = $2
                "#,
            )
            .bind(bookmark_id)
            .bind(user_id)
            .fetch_all(&mut *tx)
            .await?;

            tag_names = existing_tags
                .into_iter()
                .map(|row| row.get::<String, _>("name"))
                .collect();
        }

        // 同步 FTS 索引 - 使用封装的私有方法
        Self::sync_fts_index(&mut tx, updated_bookmark, &tag_names, true).await?;

        // Commit transaction - ACID 保证
        tx.commit().await?;

        Ok(bookmark)
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

    pub async fn increment_visit_count(
        user_id: i64,
        bookmark_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<BookmarkVisitInfo> {
        let record = sqlx::query(
            r#"
            UPDATE bookmarks
            SET visit_count = visit_count + 1, last_visited = CAST(strftime('%s', 'now') AS INTEGER)
            WHERE id = $1 AND user_id = $2
            RETURNING visit_count, last_visited
            "#,
        )
        .bind(bookmark_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        let record = record.ok_or_else(|| AppError::NotFound("Bookmark not found".to_string()))?;
        let visit_count: i64 = record.get("visit_count");
        let last_visited = record.get("last_visited");

        Ok(BookmarkVisitInfo {
            visit_count: visit_count as i64,
            last_visited,
        })
    }

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

    pub async fn export_bookmarks(
        user_id: i64,
        options: BookmarkExportOptions,
        db_pool: &SqlitePool,
    ) -> AppResult<BookmarkExportPayload> {
        let query = BookmarkQuery {
            collection_id: options.collection_id,
            tags: None,
            is_favorite: None,
            is_archived: if options.include_archived {
                None
            } else {
                Some(false)
            },
            is_private: None,
            is_read: None,
            search: None,
            limit: Some(5000),
            offset: Some(0),
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
        };

        let bookmarks = Self::get_bookmarks(user_id, query, db_pool).await?;
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");

        match options.format {
            BookmarkExportFormat::Json => {
                let body = serde_json::to_vec(&bookmarks)?;
                Ok(BookmarkExportPayload {
                    filename: format!("bookmarks-{}.json", timestamp),
                    content_type: "application/json".to_string(),
                    body,
                })
            }
            BookmarkExportFormat::Html | BookmarkExportFormat::Netscape => {
                let mut body = String::from(
                    "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n<TITLE>Bookmarks</TITLE>\n<H1>Bookmarks</H1>\n<DL><p>\n",
                );

                for bookmark in bookmarks {
                    let tags = if bookmark.tags.is_empty() {
                        "".to_string()
                    } else {
                        bookmark.tags.join(",")
                    };
                    body.push_str(&format!(
                        r#"<DT><A HREF="{url}" ADD_DATE="{ts}" TAGS="{tags}">{title}</A>"#,
                        url = bookmark.bookmark.url,
                        ts = bookmark.bookmark.created_at,
                        tags = tags,
                        title = bookmark.bookmark.title
                    ));
                    body.push('\n');
                    if let Some(description) = bookmark.bookmark.description {
                        body.push_str(&format!("<DD>{}\n", description));
                    }
                }

                body.push_str("</DL><p>");

                Ok(BookmarkExportPayload {
                    filename: format!("bookmarks-{}.html", timestamp),
                    content_type: "text/html; charset=utf-8".to_string(),
                    body: body.into_bytes(),
                })
            }
        }
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

    /// 同步书签的 FTS 索引
    ///
    /// 这个私有方法封装了从书签数据生成分词并更新 FTS 索引的完整逻辑
    /// 用于 create_bookmark 和 update_bookmark 中，消除代码重复
    ///
    /// # 参数
    /// - tx: 数据库事务
    /// - bookmark: 书签数据（包含 id, title, description, url）
    /// - tags: 标签名称列表（用于分词）
    /// - is_update: 是否为更新操作（true = UPDATE，false = INSERT）
    async fn sync_fts_index(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        bookmark: &Bookmark,
        tags: &[String],
        is_update: bool,
    ) -> AppResult<()> {
        // 使用 jieba 分词，准备 FTS5 数据
        let title_keywords = prepare_for_search(Some(&bookmark.title));
        let description_keywords = prepare_for_search(bookmark.description.as_deref());
        let tags_keywords = prepare_tags_for_search(tags);
        let url_text = bookmark.url.clone();

        if is_update {
            // 更新现有 FTS 索引
            sqlx::query(
                r#"
                UPDATE bookmarks_fts
                SET title = $1, description = $2, tags = $3, url = $4
                WHERE rowid = $5
                "#,
            )
            .bind(title_keywords)
            .bind(description_keywords)
            .bind(tags_keywords)
            .bind(url_text)
            .bind(bookmark.id)
            .execute(&mut **tx)
            .await?;
        } else {
            // 插入新的 FTS 索引
            sqlx::query(
                r#"
                INSERT INTO bookmarks_fts (rowid, title, description, tags, url)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(bookmark.id)
            .bind(title_keywords)
            .bind(description_keywords)
            .bind(tags_keywords)
            .bind(url_text)
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        
        // Create tables for testing
        sqlx::query(
            r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                avatar_url TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                email_verified INTEGER NOT NULL DEFAULT 0,
                email_verification_token TEXT,
                password_reset_token TEXT,
                password_reset_expires_at INTEGER,
                last_login_at INTEGER,
                created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER))
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE collections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT,
                icon TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                collection_id INTEGER,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                description TEXT,
                favicon_url TEXT,
                screenshot_url TEXT,
                thumbnail_url TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                is_archived INTEGER NOT NULL DEFAULT 0,
                is_private INTEGER NOT NULL DEFAULT 0,
                is_read INTEGER NOT NULL DEFAULT 0,
                visit_count INTEGER NOT NULL DEFAULT 0,
                last_visited INTEGER,
                reading_time INTEGER,
                difficulty_level INTEGER,
                metadata TEXT DEFAULT '{}',
                created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE bookmark_tags (
                bookmark_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (bookmark_id, tag_id),
                FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // 创建 FTS5 虚拟表用于全文搜索
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
                title,
                description,
                tags,
                url,
                content='',
                tokenize='unicode61'
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert test user
        sqlx::query(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)"
        )
        .bind("testuser")
        .bind("test@example.com")
        .bind("hashed_password")
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_bookmark_success() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let bookmark_data = CreateBookmark {
            collection_id: None,
            title: "Test Bookmark".to_string(),
            url: "https://example.com".to_string(),
            description: Some("Test description".to_string()),
            is_favorite: Some(true),
            is_private: Some(false),
            tags: Some(vec!["test".to_string(), "example".to_string()]),
        };

        let result = BookmarkService::create_bookmark(user_id, bookmark_data, &pool).await;
        if let Err(e) = &result {
            println!("Error creating bookmark: {:?}", e);
            panic!("Failed to create bookmark: {:?}", e);
        }
        assert!(result.is_ok());

        let bookmark = result.unwrap();
        assert_eq!(bookmark.title, "Test Bookmark");
        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.user_id, user_id);
        assert!(bookmark.is_favorite);
        assert!(!bookmark.is_private);
    }

    #[tokio::test]
    async fn test_create_bookmark_invalid_url() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let bookmark_data = CreateBookmark {
            collection_id: None,
            title: "Test Bookmark".to_string(),
            url: "invalid-url".to_string(),
            description: None,
            is_favorite: None,
            is_private: None,
            tags: None,
        };

        let result = BookmarkService::create_bookmark(user_id, bookmark_data, &pool).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_bookmarks_empty() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let query = BookmarkQuery::default();
        let result = BookmarkService::get_bookmarks(user_id, query, &pool).await;
        assert!(result.is_ok());

        let bookmarks = result.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_get_bookmark_by_id_not_found() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let bookmark_id = 999;
        
        let result = BookmarkService::get_bookmark_by_id(user_id, bookmark_id, &pool).await;
        assert!(result.is_ok());

        let bookmark = result.unwrap();
        assert!(bookmark.is_none());
    }

    #[tokio::test]
    async fn test_update_bookmark_not_found() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let bookmark_id = 999;
        
        let update_data = UpdateBookmark {
            title: Some("Updated Title".to_string()),
            url: None,
            description: None,
            collection_id: None,
            clear_collection_id: None,
            is_favorite: None,
            is_archived: None,
            is_private: None,
            is_read: None,
            reading_time: None,
            difficulty_level: None,
            tags: None,
        };

        let result = BookmarkService::update_bookmark(user_id, bookmark_id, update_data, &pool).await;
        assert!(result.is_ok());

        let bookmark = result.unwrap();
        assert!(bookmark.is_none());
    }

    #[tokio::test]
    async fn test_delete_bookmark_not_found() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let bookmark_id = 999;
        
        let result = BookmarkService::delete_bookmark(user_id, bookmark_id, &pool).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_bookmark_exists_false() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let url = "https://example.com";
        
        let result = BookmarkService::bookmark_exists(user_id, url, &pool).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_increment_visit_count_not_found() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let bookmark_id = 999;
        
        let result = BookmarkService::increment_visit_count(user_id, bookmark_id, &pool).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_process_empty() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let request = BookmarkBatchRequest {
            action: BookmarkBatchAction::Delete,
            bookmark_ids: vec![],
            data: None,
        };

        let result = BookmarkService::batch_process(user_id, request, &pool).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.processed, 0);
        assert_eq!(batch_result.failed, 0);
    }

    #[tokio::test]
    async fn test_export_bookmarks_empty() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let options = BookmarkExportOptions {
            collection_id: None,
            format: BookmarkExportFormat::Json,
            include_archived: false,
        };

        let result = BookmarkService::export_bookmarks(user_id, options, &pool).await;
        assert!(result.is_ok());

        let payload = result.unwrap();
        assert!(payload.filename.starts_with("bookmarks-"));
        assert!(payload.filename.ends_with(".json"));
        assert_eq!(payload.content_type, "application/json");
        assert!(!payload.body.is_empty());
    }
}
