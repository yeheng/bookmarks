use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Collection, CollectionQuery, CreateCollection, UpdateCollection};
use crate::utils::error::{AppError, AppResult};

pub struct CollectionService;

impl CollectionService {
    pub async fn create_collection(
        user_id: Uuid,
        collection_data: CreateCollection,
        db_pool: &PgPool,
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
        user_id: Uuid,
        query: CollectionQuery,
        db_pool: &PgPool,
    ) -> AppResult<Vec<Collection>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut sql = "SELECT * FROM collections WHERE user_id = $1".to_string();
        let mut param_count = 1;

        if query.parent_id.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND parent_id = ${}", param_count));
        }

        if query.is_public.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND is_public = ${}", param_count));
        }

        sql.push_str(" ORDER BY sort_order, created_at");

        let mut query_builder = sqlx::query_as::<_, Collection>(&sql).bind(user_id);

        if let Some(parent_id) = query.parent_id {
            query_builder = query_builder.bind(parent_id);
        }
        if let Some(is_public) = query.is_public {
            query_builder = query_builder.bind(is_public);
        }

        let collections = query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        Ok(collections)
    }

    pub async fn get_collection_by_id(
        user_id: Uuid,
        collection_id: Uuid,
        db_pool: &PgPool,
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
        user_id: Uuid,
        collection_id: Uuid,
        update_data: UpdateCollection,
        db_pool: &PgPool,
    ) -> AppResult<Option<Collection>> {
        // 检查是否有更新字段
        if update_data.name.is_none()
            && update_data.description.is_none()
            && update_data.color.is_none()
            && update_data.icon.is_none()
            && update_data.parent_id.is_none()
            && update_data.sort_order.is_none()
        {
            return Err(AppError::BadRequest("No update fields provided".to_string()).into());
        }

        // 使用 COALESCE 来只更新提供的字段
        let collection = sqlx::query_as::<_, Collection>(
            r#"
            UPDATE collections SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                color = COALESCE($3, color),
                icon = COALESCE($4, icon),
                parent_id = CASE WHEN $5::boolean THEN $6 ELSE parent_id END,
                sort_order = COALESCE($7, sort_order),
                updated_at = NOW()
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
        .bind(update_data.parent_id.is_some())
        .bind(update_data.parent_id.flatten())
        .bind(update_data.sort_order)
        .bind(collection_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        Ok(collection)
    }

    pub async fn delete_collection(
        user_id: Uuid,
        collection_id: Uuid,
        db_pool: &PgPool,
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
            return Err(
                AppError::BadRequest("Cannot delete default collection".to_string()).into(),
            );
        }

        let result = sqlx::query("DELETE FROM collections WHERE id = $1 AND user_id = $2")
            .bind(collection_id)
            .bind(user_id)
            .execute(db_pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
