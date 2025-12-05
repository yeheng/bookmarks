use sqlx::SqlitePool;

use crate::models::{Collection, CollectionQuery, CreateCollection, UpdateCollection};
use crate::utils::error::{AppError, AppResult};

pub struct CollectionService;

impl CollectionService {
    pub async fn create_collection(
        user_id: i64,
        collection_data: CreateCollection,
        db_pool: &SqlitePool,
    ) -> AppResult<Collection> {
        let collection = sqlx::query_as::<_, Collection>(
            r#"
            INSERT INTO collections (user_id, name, description, color, icon, parent_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, name, description, color, icon, sort_order,
                      is_default, is_public, parent_id,
                      bookmark_count, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(&collection_data.name)
        .bind(&collection_data.description)
        .bind(
            collection_data
                .color
                .unwrap_or_else(|| "#3b82f6".to_string()),
        )
        .bind(collection_data.icon.unwrap_or_else(|| "folder".to_string()))
        .bind(collection_data.parent_id)
        .fetch_one(db_pool)
        .await?;

        Ok(collection)
    }

    pub async fn get_collections(
        user_id: i64,
        query: CollectionQuery,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<Collection>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        // 构建动态查询，使用 sqlx::QueryBuilder 避免手动字符串拼接
        let mut query_builder =
            sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT * FROM collections WHERE user_id = ");

        // 添加 user_id 条件
        query_builder.push_bind(user_id);

        // 动态添加 parent_id 条件
        if let Some(parent_id) = query.parent_id {
            query_builder.push(" AND parent_id = ");
            query_builder.push_bind(parent_id);
        }

        // 动态添加 is_public 条件
        if let Some(is_public) = query.is_public {
            query_builder.push(" AND is_public = ");
            query_builder.push_bind(is_public);
        }

        // 添加排序
        query_builder.push(" ORDER BY sort_order, created_at");

        // 添加分页
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // 执行查询
        let collections = query_builder
            .build_query_as::<Collection>()
            .fetch_all(db_pool)
            .await?;

        Ok(collections)
    }

    pub async fn get_collection_by_id(
        user_id: i64,
        collection_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<Collection>> {
        let collection = sqlx::query_as::<_, Collection>(
            r#"
            SELECT id, user_id, name, description,
                   color, icon, sort_order,
                   is_default, is_public, parent_id,
                   bookmark_count, created_at,
                   updated_at
            FROM collections
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(collection_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        Ok(collection)
    }

    pub async fn update_collection(
        user_id: i64,
        collection_id: i64,
        update_data: UpdateCollection,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<Collection>> {
        // 检查是否有更新字段
        if update_data.name.is_none()
            && update_data.description.is_none()
            && update_data.color.is_none()
            && update_data.icon.is_none()
            && update_data.parent_id.is_none()
            && update_data.clear_parent_id.is_none()
            && update_data.sort_order.is_none()
        {
            return Err(AppError::BadRequest(
                "No update fields provided".to_string(),
            ));
        }

        // 使用 COALESCE 来只更新提供的字段
        let collection = sqlx::query_as::<_, Collection>(
            r#"
            UPDATE collections SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                color = COALESCE($3, color),
                icon = COALESCE($4, icon),
                parent_id = CASE WHEN $5 THEN NULL ELSE COALESCE($6, parent_id) END,
                sort_order = COALESCE($7, sort_order),
                updated_at = CAST(strftime('%s', 'now') AS INTEGER)
            WHERE id = $8 AND user_id = $9
            RETURNING id, user_id, name, description,
                      color, icon, sort_order,
                      is_default, is_public, parent_id,
                      bookmark_count, created_at,
                      updated_at
            "#,
        )
        .bind(update_data.name)
        .bind(update_data.description)
        .bind(update_data.color)
        .bind(update_data.icon)
        .bind(update_data.clear_parent_id.unwrap_or(false))
        .bind(update_data.parent_id)
        .bind(update_data.sort_order)
        .bind(collection_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        Ok(collection)
    }

    pub async fn delete_collection(
        user_id: i64,
        collection_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<bool> {
        // Check if collection is default
        let is_default = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT is_default
            FROM collections
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(collection_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?
        .unwrap_or(false);

        if is_default {
            return Err(AppError::BadRequest(
                "Cannot delete default collection".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM collections WHERE id = $1 AND user_id = $2")
            .bind(collection_id)
            .bind(user_id)
            .execute(db_pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
