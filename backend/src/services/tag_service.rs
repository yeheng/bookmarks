use sqlx::SqlitePool;

use crate::models::{CreateTag, Tag, TagQuery, UpdateTag};
use crate::utils::error::{AppError, AppResult};

pub struct TagService;

impl TagService {
    pub async fn create_tag(
        user_id: i64,
        tag_data: CreateTag,
        db_pool: &SqlitePool,
    ) -> AppResult<Tag> {
        let tag = sqlx::query_as::<_, Tag>(
            r#"
            INSERT INTO tags (user_id, name, color, description, usage_count)
            VALUES ($1, $2, $3, $4, 0)
            RETURNING
                id,
                user_id,
                name,
                color,
                description,
                usage_count,
                created_at,
                updated_at
            "#,
        )
        .bind(user_id)
        .bind(&tag_data.name)
        .bind(tag_data.color.unwrap_or_else(|| "#64748b".to_string()))
        .bind(&tag_data.description)
        .fetch_one(db_pool)
        .await?;

        Ok(tag)
    }

    pub async fn get_tags(
        user_id: i64,
        query: TagQuery,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<Tag>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut sql = r#"
            SELECT
                id,
                user_id,
                name,
                color,
                description,
                COALESCE(usage_count, 0) as usage_count,
                created_at,
                updated_at
            FROM tags WHERE user_id = $1
        "#
        .to_string();
        let mut param_count = 1;

        if query.search.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND lower(name) LIKE lower(${})", param_count));
        }

        sql.push_str(" ORDER BY usage_count DESC, name");

        let mut query_builder = sqlx::query_as::<_, Tag>(&sql).bind(user_id);

        if let Some(search) = &query.search {
            query_builder = query_builder.bind(format!("%{}%", search));
        }

        let tags = query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        Ok(tags)
    }

    pub async fn get_tag_by_id(
        user_id: i64,
        tag_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<Tag>> {
        let tag = sqlx::query_as::<_, Tag>(
            r#"
            SELECT
                id,
                user_id,
                name,
                color,
                description,
                COALESCE(usage_count, 0) as usage_count,
                created_at,
                updated_at
            FROM tags
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(tag_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        Ok(tag)
    }

    pub async fn update_tag(
        user_id: i64,
        tag_id: i64,
        update_data: UpdateTag,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<Tag>> {
        // 检查是否有更新字段
        if update_data.name.is_none()
            && update_data.color.is_none()
            && update_data.description.is_none()
        {
            return Err(AppError::BadRequest(
                "No update fields provided".to_string(),
            ));
        }

        // 使用 COALESCE 来只更新提供的字段
        let tag = sqlx::query_as::<_, Tag>(
            r#"
            UPDATE tags SET
                name = COALESCE($1, name),
                color = COALESCE($2, color),
                description = COALESCE($3, description),
                updated_at = CAST(strftime('%s', 'now') AS INTEGER)
            WHERE id = $4 AND user_id = $5
            RETURNING id, user_id, name, color,
                      description, usage_count,
                      created_at, updated_at
            "#,
        )
        .bind(update_data.name)
        .bind(update_data.color)
        .bind(update_data.description)
        .bind(tag_id)
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        Ok(tag)
    }

    pub async fn delete_tag(user_id: i64, tag_id: i64, db_pool: &SqlitePool) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM tags WHERE id = $1 AND user_id = $2")
            .bind(tag_id)
            .bind(user_id)
            .execute(db_pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_popular_tags(
        user_id: i64,
        limit: Option<i64>,
        db_pool: &SqlitePool,
    ) -> AppResult<Vec<Tag>> {
        let limit = limit.unwrap_or(20);

        let tags = sqlx::query_as::<_, Tag>(
            r#"
            SELECT
                id,
                user_id,
                name,
                color,
                description,
                COALESCE(usage_count, 0) as usage_count,
                created_at,
                updated_at
            FROM tags
            WHERE user_id = $1
            ORDER BY usage_count DESC, name
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(db_pool)
        .await?;

        Ok(tags)
    }
}
