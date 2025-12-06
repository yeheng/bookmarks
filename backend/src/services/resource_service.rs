use sqlx::{Row, SqlitePool};

use crate::models::{
    Resource, ResourceBatchAction, ResourceBatchError, ResourceBatchRequest, ResourceBatchResult,
    ResourceQuery, ResourceWithTags, CreateResource, UpdateResource,
    ResourceReferenceQuery, ResourceReferenceList, ResourceType,
};
use crate::services::IndexerService;
use crate::utils::error::{AppError, AppResult};
use crate::utils::validation::validate_url;

pub struct ResourceService;

impl ResourceService {
    /// 创建资源 - 支持 Link, Note, Snippet, File 四种类型
    /// 类型验证规则:
    /// - Link: 必须有 url
    /// - Note: 必须有 content
    /// - Snippet: 必须有 content
    /// - File: 必须有 source
    pub async fn create_resource(
        user_id: i64,
        resource_data: CreateResource,
        db_pool: &SqlitePool,
    ) -> AppResult<Resource> {
        // 解析资源类型
        let resource_type = ResourceType::from_str(&resource_data.resource_type)
            .map_err(|e| AppError::BadRequest(e))?;

        // 类型感知验证
        match resource_type {
            ResourceType::Link => {
                let url = resource_data.url.as_ref()
                    .ok_or_else(|| AppError::BadRequest("Link type requires url field".to_string()))?;
                validate_url(url)
                    .then_some(())
                    .ok_or_else(|| AppError::BadRequest(format!("Invalid URL format: {}", url)))?;
            }
            ResourceType::Note | ResourceType::Snippet => {
                if resource_data.content.is_none() {
                    return Err(AppError::BadRequest(
                        format!("{} type requires content field", resource_type.as_str())
                    ));
                }
            }
            ResourceType::File => {
                if resource_data.source.is_none() {
                    return Err(AppError::BadRequest("File type requires source field".to_string()));
                }
            }
        }

        // 开始事务 - 同时更新 resources 和 resources_fts
        let mut tx = db_pool.begin().await?;

        // 创建资源
        let resource = sqlx::query_as::<_, Resource>(
            r#"
            INSERT INTO resources (user_id, collection_id, title, url, description, is_favorite, is_private, type, content, source, mime_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, user_id, collection_id, title, url, description, favicon_url, screenshot_url,
                      thumbnail_url, is_favorite, is_archived, is_private, is_read, visit_count,
                      last_visited, reading_time, difficulty_level, metadata, type, content, source, mime_type,
                      created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(resource_data.collection_id)
        .bind(&resource_data.title)
        .bind(&resource_data.url)
        .bind(&resource_data.description)
        .bind(resource_data.is_favorite.unwrap_or(false))
        .bind(resource_data.is_private.unwrap_or(false))
        .bind(&resource_data.resource_type)
        .bind(&resource_data.content)
        .bind(&resource_data.source)
        .bind(&resource_data.mime_type)
        .fetch_one(&mut *tx)
        .await?;

        // 处理标签
        if let Some(tags) = resource_data.tags {
            for tag_name in tags {
                // 确保标签存在 (SQLite compatible)
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

                // 关联资源与标签
                sqlx::query(
                    "INSERT OR IGNORE INTO resource_tags (resource_id, tag_id) VALUES ($1, $2)",
                )
                .bind(resource.id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 同步 FTS 索引 - 使用 IndexerService
        IndexerService::index_resource(&mut tx, resource.id, user_id).await?;

        // 提交事务 - ACID 保证: resources 和 resources_fts 同时成功或失败
        tx.commit().await?;

        Ok(resource)
    }

    /// 获取资源列表 - 支持类型过滤
    pub async fn get_resources(
        user_id: i64,
        query: ResourceQuery,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<ResourceWithTags>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("desc");

        // 验证排序字段和顺序,防止 SQL 注入
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
                r.id, r.user_id, r.collection_id, r.title, r.url, r.description,
                r.favicon_url, r.screenshot_url, r.thumbnail_url,
                r.is_favorite, r.is_archived, r.is_private, r.is_read,
                r.visit_count, r.last_visited, r.reading_time, r.difficulty_level,
                r.metadata, r.type, r.content, r.source, r.mime_type,
                r.created_at, r.updated_at,
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
            FROM resources r
            LEFT JOIN collections c ON r.collection_id = c.id
            LEFT JOIN resource_tags rt ON r.id = rt.resource_id
            LEFT JOIN tags t ON rt.tag_id = t.id
            WHERE r.user_id =
            "#,
        );

        // 自动绑定 user_id
        query_builder.push_bind(user_id);

        // 动态添加过滤条件 - QueryBuilder 自动管理参数绑定
        if let Some(collection_id) = query.collection_id {
            query_builder.push(" AND r.collection_id = ");
            query_builder.push_bind(collection_id);
        }

        if let Some(ref resource_type) = query.resource_type {
            query_builder.push(" AND r.type = ");
            query_builder.push_bind(resource_type);
        }

        if let Some(is_favorite) = query.is_favorite {
            query_builder.push(" AND r.is_favorite = ");
            query_builder.push_bind(is_favorite);
        }

        if let Some(is_archived) = query.is_archived {
            query_builder.push(" AND r.is_archived = ");
            query_builder.push_bind(is_archived);
        }

        if let Some(is_private) = query.is_private {
            query_builder.push(" AND r.is_private = ");
            query_builder.push_bind(is_private);
        }

        if let Some(is_read) = query.is_read {
            query_builder.push(" AND r.is_read = ");
            query_builder.push_bind(is_read);
        }

        // 搜索条件 - 支持 title, description, content 字段
        if let Some(ref search_term) = query.search {
            query_builder.push(" AND (r.title LIKE '%' || ");
            query_builder.push_bind(search_term);
            query_builder.push(" || '%' OR COALESCE(r.description, '') LIKE '%' || ");
            query_builder.push_bind(search_term);
            query_builder.push(" || '%' OR COALESCE(r.content, '') LIKE '%' || ");
            query_builder.push_bind(search_term);
            query_builder.push(" || '%')");
        }

        // Tags 过滤 - 使用 QueryBuilder 处理 IN 子句
        if let Some(ref tags) = query.tags {
            if !tags.is_empty() {
                query_builder.push(
                    " AND r.id IN (
                        SELECT resource_id
                        FROM resource_tags
                        JOIN tags ON resource_tags.tag_id = tags.id
                        WHERE tags.name IN (",
                );

                // 使用 separated 方法处理 IN 子句中的多个值
                let mut separated = query_builder.separated(", ");
                for tag in tags {
                    separated.push_bind(tag);
                }

                query_builder.push(") GROUP BY resource_id HAVING COUNT(DISTINCT tags.id) = ");
                query_builder.push_bind(tags.len() as i64);
                query_builder.push(")");
            }
        }

        // 添加 GROUP BY
        query_builder.push(" GROUP BY r.id, c.name, c.color");

        // 添加排序 - 已验证的字段名可以安全拼接
        let sort_field = match sort_by {
            "title" => "r.title",
            "created_at" => "r.created_at",
            "updated_at" => "r.updated_at",
            "visit_count" => "r.visit_count",
            "last_visited" => "r.last_visited",
            _ => "r.created_at", // 安全回退
        };

        let sort_direction = match sort_order {
            "ASC" | "asc" => "ASC",
            "DESC" | "desc" => "DESC",
            _ => "DESC", // 安全回退
        };

        query_builder.push(format!(" ORDER BY {} {}", sort_field, sort_direction));

        // 添加分页
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // 执行查询
        let resources = query_builder
            .build_query_as::<ResourceWithTags>()
            .fetch_all(db_pool)
            .await?;

        Ok(resources)
    }

    /// 根据 ID 获取单个资源
    pub async fn get_resource_by_id(
        user_id: i64,
        resource_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<ResourceWithTags>> {
        let resource = sqlx::query_as::<_, ResourceWithTags>(
            r#"
            SELECT
                r.id, r.user_id, r.collection_id, r.title, r.url, r.description,
                r.favicon_url, r.screenshot_url, r.thumbnail_url,
                r.is_favorite, r.is_archived, r.is_private, r.is_read,
                r.visit_count, r.last_visited, r.reading_time, r.difficulty_level,
                r.metadata, r.type, r.content, r.source, r.mime_type,
                r.created_at, r.updated_at,
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
            FROM resources r
            LEFT JOIN collections c ON r.collection_id = c.id
            LEFT JOIN resource_tags rt ON r.id = rt.resource_id
            LEFT JOIN tags t ON rt.tag_id = t.id
            WHERE r.id = $1 AND r.user_id = $2
            GROUP BY r.id, c.name, c.color
            "#,
        )
        .bind(resource_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        Ok(resource)
    }

    /// 更新资源 - 支持类型感知验证
    pub async fn update_resource(
        user_id: i64,
        resource_id: i64,
        update_data: UpdateResource,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<ResourceWithTags>> {
        // 类型感知验证 (如果更新了类型)
        if let Some(ref resource_type_str) = update_data.resource_type {
            let resource_type = ResourceType::from_str(resource_type_str)
                .map_err(|e| AppError::BadRequest(e))?;

            match resource_type {
                ResourceType::Link => {
                    if let Some(ref url) = update_data.url {
                        validate_url(url)
                            .then_some(())
                            .ok_or_else(|| AppError::BadRequest(format!("Invalid URL format: {}", url)))?;
                    }
                }
                ResourceType::Note | ResourceType::Snippet => {
                    // Note: 这里只验证新提供的 content,如果不提供则保留原值
                }
                ResourceType::File => {
                    // File: source 字段验证
                }
            }
        }

        // 如果提供了 URL,验证格式
        if let Some(ref url) = update_data.url {
            validate_url(url)
                .then_some(())
                .ok_or_else(|| AppError::BadRequest(format!("Invalid URL format: {}", url)))?;
        }

        // 开始事务 - 同时更新 resources 和 resources_fts
        let mut tx = db_pool.begin().await?;

        // 检查是否有资源字段需要更新(不包括标签)
        let has_resource_updates = update_data.title.is_some()
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
            || update_data.resource_type.is_some()
            || update_data.content.is_some()
            || update_data.source.is_some()
            || update_data.mime_type.is_some();

        // 检查是否有任何字段需要更新(包括标签)
        let has_any_updates = has_resource_updates || update_data.tags.is_some();

        if !has_any_updates {
            return Err(AppError::BadRequest(
                "No update fields provided".to_string(),
            ));
        }

        // 获取或更新资源
        let resource = if has_resource_updates {
            // 使用 COALESCE 来只更新提供的字段
            sqlx::query_as::<_, Resource>(
                r#"
                UPDATE resources SET
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
                    type = COALESCE($12, type),
                    content = COALESCE($13, content),
                    source = COALESCE($14, source),
                    mime_type = COALESCE($15, mime_type),
                    updated_at = CAST(strftime('%s', 'now') AS INTEGER)
                WHERE id = $16 AND user_id = $17
                RETURNING id, user_id, collection_id, title, url, description, favicon_url,
                          screenshot_url, thumbnail_url, is_favorite,
                          is_archived, is_private, is_read, visit_count, last_visited,
                          reading_time, difficulty_level, metadata, type, content, source, mime_type,
                          created_at, updated_at
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
            .bind(update_data.resource_type.as_ref())
            .bind(update_data.content.as_ref())
            .bind(update_data.source.as_ref())
            .bind(update_data.mime_type.as_ref())
            .bind(resource_id)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
        } else {
            // 如果只更新标签,先获取现有资源
            sqlx::query_as::<_, Resource>(
                r#"
                SELECT id, user_id, collection_id, title, url, description, favicon_url,
                       screenshot_url, thumbnail_url, is_favorite, is_archived, is_private,
                       is_read, visit_count, last_visited, reading_time, difficulty_level,
                       metadata, type, content, source, mime_type, created_at, updated_at
                FROM resources
                WHERE id = $1 AND user_id = $2
                "#,
            )
            .bind(resource_id)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
        };

        // 如果资源不存在,提前返回
        let Some(ref updated_resource) = resource else {
            tx.rollback().await?;
            return Ok(None);
        };

        // 处理标签更新
        if let Some(tags) = update_data.tags {
            // 删除现有标签关联
            sqlx::query("DELETE FROM resource_tags WHERE resource_id = $1")
                .bind(resource_id)
                .execute(&mut *tx)
                .await?;

            // 添加新的标签关联
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
                    "INSERT OR IGNORE INTO resource_tags (resource_id, tag_id) VALUES ($1, $2)",
                )
                .bind(resource_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 同步 FTS 索引 - 使用 IndexerService
        IndexerService::index_resource(&mut tx, updated_resource.id, user_id).await?;

        // 提交事务 - ACID 保证
        tx.commit().await?;

        // 构建包含 tags 和 collection 信息的完整响应
        let resource_with_tags = if let Some(resource) = resource {
            // 获取标签
            let tags = sqlx::query_scalar(
                "SELECT t.name FROM tags t
                 JOIN resource_tags rt ON t.id = rt.tag_id
                 WHERE rt.resource_id = $1
                 ORDER BY t.name"
            )
            .bind(resource.id)
            .fetch_all(db_pool)
            .await?;

            // 获取收藏夹信息
            let collection_info = if let Some(collection_id) = resource.collection_id {
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

            Some(ResourceWithTags {
                resource,
                tags,
                collection_name: collection_info.as_ref().map(|(name, _)| name.clone()),
                collection_color: collection_info.as_ref().and_then(|(_, color)| color.clone()),
                reference_count: None,
            })
        } else {
            None
        };

        Ok(resource_with_tags)
    }

    /// 删除资源
    pub async fn delete_resource(
        user_id: i64,
        resource_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        // 开始事务 - 同时删除 resources 和 resources_fts
        let mut tx = db_pool.begin().await?;

        // 首先从 FTS 索引中删除(必须在 resources 删除之前,因为需要 rowid 关联)
        sqlx::query("DELETE FROM resources_fts WHERE rowid = $1")
            .bind(resource_id)
            .execute(&mut *tx)
            .await?;

        // 删除资源(CASCADE 会自动删除 resource_tags 和 resource_references)
        let result = sqlx::query("DELETE FROM resources WHERE id = $1 AND user_id = $2")
            .bind(resource_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        let was_deleted = result.rows_affected() > 0;

        // 提交事务 - ACID 保证
        tx.commit().await?;

        Ok(was_deleted)
    }

    /// 检查资源是否存在
    #[allow(dead_code)]
    pub async fn resource_exists(user_id: i64, url: &str, db_pool: &SqlitePool) -> AppResult<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM resources WHERE user_id = $1 AND url = $2)",
        )
        .bind(user_id)
        .bind(url)
        .fetch_one(db_pool)
        .await?;

        Ok(exists)
    }

    /// 批量操作资源
    pub async fn batch_process(
        user_id: i64,
        request: ResourceBatchRequest,
        db_pool: &SqlitePool,
    ) -> AppResult<ResourceBatchResult> {
        let ResourceBatchRequest {
            action,
            resource_ids,
            data,
        } = request;

        let mut result = ResourceBatchResult {
            processed: 0,
            failed: 0,
            errors: Vec::new(),
        };

        for resource_id in resource_ids {
            let operation = match action {
                ResourceBatchAction::Delete => {
                    Self::delete_resource(user_id, resource_id, db_pool).await
                }
                ResourceBatchAction::Move => {
                    let collection_id = data
                        .as_ref()
                        .and_then(|data| data.collection_id)
                        .ok_or_else(|| {
                            AppError::BadRequest(
                                "Batch move operation requires collection_id".to_string(),
                            )
                        })?;

                    Self::move_resource(user_id, resource_id, collection_id, db_pool).await
                }
                ResourceBatchAction::AddTags => {
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

                    Self::add_tags(user_id, resource_id, tags, db_pool).await
                }
                ResourceBatchAction::RemoveTags => {
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

                    Self::remove_tags(user_id, resource_id, tags, db_pool).await
                }
            };

            match operation {
                Ok(true) => result.processed += 1,
                Ok(false) => {
                    result.failed += 1;
                    result.errors.push(ResourceBatchError {
                        resource_id,
                        reason: "Resource not found".to_string(),
                    });
                }
                Err(err) => {
                    result.failed += 1;
                    result.errors.push(ResourceBatchError {
                        resource_id,
                        reason: err.to_string(),
                    });
                }
            }
        }

        Ok(result)
    }

    // ============================================================
    // 资源引用关系 CRUD 方法
    // ============================================================

    /// 创建资源引用关系
    pub async fn create_resource_reference(
        source_id: i64,
        target_id: i64,
        reference_type: Option<String>,
        user_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<i64> {
        let ref_type = reference_type.unwrap_or_else(|| "related".to_string());

        // 验证 source_id 和 target_id 都属于该用户
        let source_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1 AND user_id = $2)",
        )
        .bind(source_id)
        .bind(user_id)
        .fetch_one(db_pool)
        .await?;

        let target_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1 AND user_id = $2)",
        )
        .bind(target_id)
        .bind(user_id)
        .fetch_one(db_pool)
        .await?;

        if !source_exists {
            return Err(AppError::NotFound("Source resource not found".to_string()));
        }

        if !target_exists {
            return Err(AppError::NotFound("Target resource not found".to_string()));
        }

        // 创建引用关系
        let result = sqlx::query(
            r#"
            INSERT INTO resource_references (source_id, target_id, type)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(source_id)
        .bind(target_id)
        .bind(&ref_type)
        .fetch_one(db_pool)
        .await?;

        let reference_id: i64 = result.get("id");
        Ok(reference_id)
    }

    /// 删除资源引用关系
    pub async fn delete_resource_reference(
        source_id: i64,
        target_id: i64,
        reference_type: Option<String>,
        user_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        // 验证资源归属
        let source_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1 AND user_id = $2)",
        )
        .bind(source_id)
        .bind(user_id)
        .fetch_one(db_pool)
        .await?;

        if !source_exists {
            return Err(AppError::NotFound("Source resource not found".to_string()));
        }

        // 构建删除查询
        let result = if let Some(ref_type) = reference_type {
            sqlx::query(
                "DELETE FROM resource_references WHERE source_id = $1 AND target_id = $2 AND type = $3",
            )
            .bind(source_id)
            .bind(target_id)
            .bind(&ref_type)
            .execute(db_pool)
            .await?
        } else {
            sqlx::query(
                "DELETE FROM resource_references WHERE source_id = $1 AND target_id = $2",
            )
            .bind(source_id)
            .bind(target_id)
            .execute(db_pool)
            .await?
        };

        Ok(result.rows_affected() > 0)
    }

    /// 获取资源的引用列表
    pub async fn get_resource_references(
        resource_id: i64,
        query: ResourceReferenceQuery,
        user_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<ResourceReferenceList> {
        // 验证资源归属
        let resource_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1 AND user_id = $2)",
        )
        .bind(resource_id)
        .bind(user_id)
        .fetch_one(db_pool)
        .await?;

        if !resource_exists {
            return Err(AppError::NotFound("Resource not found".to_string()));
        }

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        let direction = query.direction.as_deref().unwrap_or("both");

        // 构建查询
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT DISTINCT
                r.id, r.user_id, r.collection_id, r.title, r.url, r.description,
                r.favicon_url, r.screenshot_url, r.thumbnail_url,
                r.is_favorite, r.is_archived, r.is_private, r.is_read,
                r.visit_count, r.last_visited, r.reading_time, r.difficulty_level,
                r.metadata, r.type, r.content, r.source, r.mime_type,
                r.created_at, r.updated_at,
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
            FROM resources r
            LEFT JOIN collections c ON r.collection_id = c.id
            LEFT JOIN resource_tags rt ON r.id = rt.resource_id
            LEFT JOIN tags t ON rt.tag_id = t.id
            WHERE r.user_id = "#,
        );
        query_builder.push_bind(user_id);

        query_builder.push(" AND (");

        // 根据方向构建条件
        match direction {
            "source" => {
                query_builder.push(
                    "r.id IN (SELECT target_id FROM resource_references WHERE source_id = ",
                );
                query_builder.push_bind(resource_id);
                if let Some(ref ref_type) = query.reference_type {
                    query_builder.push(" AND type = ");
                    query_builder.push_bind(ref_type);
                }
                query_builder.push(")");
            }
            "target" => {
                query_builder.push(
                    "r.id IN (SELECT source_id FROM resource_references WHERE target_id = ",
                );
                query_builder.push_bind(resource_id);
                if let Some(ref ref_type) = query.reference_type {
                    query_builder.push(" AND type = ");
                    query_builder.push_bind(ref_type);
                }
                query_builder.push(")");
            }
            _ => {
                // both
                query_builder.push(
                    "r.id IN (SELECT target_id FROM resource_references WHERE source_id = ",
                );
                query_builder.push_bind(resource_id);
                if let Some(ref ref_type) = query.reference_type {
                    query_builder.push(" AND type = ");
                    query_builder.push_bind(ref_type);
                }
                query_builder.push(") OR r.id IN (SELECT source_id FROM resource_references WHERE target_id = ");
                query_builder.push_bind(resource_id);
                if let Some(ref ref_type) = query.reference_type {
                    query_builder.push(" AND type = ");
                    query_builder.push_bind(ref_type);
                }
                query_builder.push(")");
            }
        }

        query_builder.push(")");
        query_builder.push(" GROUP BY r.id, c.name, c.color");
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit + 1); // 多取一条检测 has_more
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // 执行查询
        let mut items = query_builder
            .build_query_as::<ResourceWithTags>()
            .fetch_all(db_pool)
            .await?;

        // 检测是否有更多数据
        let has_more = items.len() > limit as usize;
        if has_more {
            items.pop(); // 移除多取的那一条
        }

        Ok(ResourceReferenceList {
            items,
            limit,
            offset,
            has_more,
        })
    }

    // ============================================================
    // 内部辅助方法
    // ============================================================

    /// 移动资源到指定收藏夹
    async fn move_resource(
        user_id: i64,
        resource_id: i64,
        collection_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE resources
            SET collection_id = $1, updated_at = CAST(strftime('%s', 'now') AS INTEGER)
            WHERE id = $2 AND user_id = $3
            "#,
        )
        .bind(collection_id)
        .bind(resource_id)
        .bind(user_id)
        .execute(db_pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 为资源添加标签
    async fn add_tags(
        user_id: i64,
        resource_id: i64,
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
                "INSERT OR IGNORE INTO resource_tags (resource_id, tag_id) VALUES ($1, $2)",
            )
            .bind(resource_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(true)
    }

    /// 从资源中移除标签
    async fn remove_tags(
        user_id: i64,
        resource_id: i64,
        tags: Vec<String>,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        // SQLite 不支持 USING 语法,需要使用子查询
        let mut result = 0;
        for tag_name in tags {
            let delete_result = sqlx::query(
                r#"
                DELETE FROM resource_tags
                WHERE resource_id = $1
                  AND tag_id IN (
                    SELECT id FROM tags WHERE user_id = $2 AND name = $3
                  )
                "#,
            )
            .bind(resource_id)
            .bind(user_id)
            .bind(&tag_name)
            .execute(db_pool)
            .await?;
            result += delete_result.rows_affected();
        }

        Ok(result > 0)
    }
}
