use sqlx::{Row, SqlitePool};

use crate::models::{CreateTag, Tag, TagQuery, UpdateTag};
use crate::services::IndexerService;
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

        // 开启事务 - 确保标签更新和 FTS 索引更新的 ACID 一致性
        let mut tx = db_pool.begin().await?;

        // 检查标签名是否被更新（如果是，需要重建相关书签的 FTS 索引）
        let name_changed = update_data.name.is_some();

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
        .fetch_optional(&mut *tx)
        .await?;

        // 如果标签不存在，回滚并返回
        let Some(_tag) = tag.as_ref() else {
            tx.rollback().await?;
            return Ok(None);
        };

        // ⚠️ 核心修复：如果标签名被更新，必须重建所有关联书签的 FTS 索引
        if name_changed {
            // 查询所有使用该标签的书签 ID
            let bookmark_ids = sqlx::query(
                r#"
                SELECT bookmark_id
                FROM bookmark_tags
                WHERE tag_id = $1
                "#,
            )
            .bind(tag_id)
            .fetch_all(&mut *tx)
            .await?;

            // 对每个受影响的书签，重建 FTS 索引
            for row in bookmark_ids {
                let bookmark_id: i64 = row.get("bookmark_id");
                IndexerService::index_bookmark(&mut tx, bookmark_id, user_id).await?;
            }
        }

        // 提交事务 - ACID 保证：标签更新和 FTS 更新要么都成功，要么都失败
        tx.commit().await?;

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
