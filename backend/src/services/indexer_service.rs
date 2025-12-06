use sqlx::{Row, SqlitePool};

use crate::models::Resource;
use crate::utils::error::{AppError, AppResult};
use crate::utils::segmenter::{prepare_for_search, prepare_tags_for_search};

/// IndexerService - FTS 索引管理服务
///
/// 这个服务封装了所有与 FTS5 全文搜索索引相关的逻辑
/// 确保系统中关于"如何索引一个资源"的逻辑只存在于一个地方 (DRY 原则)
///
/// 分词策略:
/// - 默认模式: 存储原始文本,让 SQLite FTS5 的 unicode61 分词器处理分词
/// - jieba 模式: 使用 jieba 进行中文分词预处理,存储空格分隔的关键词
///
/// 使用场景:
/// - ResourceService 在创建/更新资源时调用
/// - TagService 在重命名标签时调用(遍历受影响的资源)
/// - MaintenanceService 在重建索引时调用
pub struct IndexerService;

impl IndexerService {
    /// 为单个资源建立或更新 FTS 索引
    ///
    /// # 参数
    /// - tx: 数据库事务,确保与资源操作的 ACID 一致性
    /// - resource_id: 要索引的资源 ID
    /// - user_id: 用户 ID
    ///
    /// # 工作流程
    /// 1. 查询资源的完整数据 (title, description, url, content)
    /// 2. 查询资源关联的所有标签名称
    /// 3. 根据配置处理文本 (默认: 原始文本, jieba: 分词预处理)
    /// 4. 更新或插入 FTS 索引记录
    pub async fn index_resource(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        resource_id: i64,
        user_id: i64,
    ) -> AppResult<()> {
        // 1. 查询资源数据
        let resource = sqlx::query_as::<_, Resource>(
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
        .fetch_optional(&mut **tx)
        .await?;

        // 如果资源不存在,返回错误
        let resource =
            resource.ok_or_else(|| AppError::NotFound("Resource not found".to_string()))?;

        // 2. 查询资源关联的标签
        let tag_rows = sqlx::query(
            r#"
            SELECT t.name
            FROM tags t
            JOIN resource_tags rt ON t.id = rt.tag_id
            WHERE rt.resource_id = $1 AND t.user_id = $2
            "#,
        )
        .bind(resource_id)
        .bind(user_id)
        .fetch_all(&mut **tx)
        .await?;

        let tag_names: Vec<String> = tag_rows
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();

        // 3. 准备 FTS 数据 (根据配置选择分词策略)
        let title_keywords = prepare_for_search(Some(&resource.title));
        let description_keywords = prepare_for_search(resource.description.as_deref());
        let content_keywords = prepare_for_search(resource.content.as_deref());
        let tags_keywords = prepare_tags_for_search(&tag_names);
        let url_text = resource.url.unwrap_or_default();

        // 4. 检查 FTS 记录是否存在
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM resources_fts WHERE rowid = $1)")
                .bind(resource_id)
                .fetch_one(&mut **tx)
                .await?;

        if exists {
            // 更新现有 FTS 索引
            sqlx::query(
                r#"
                UPDATE resources_fts
                SET title = $1, description = $2, content = $3, tags = $4, url = $5
                WHERE rowid = $6
                "#,
            )
            .bind(title_keywords)
            .bind(description_keywords)
            .bind(content_keywords)
            .bind(tags_keywords)
            .bind(url_text)
            .bind(resource_id)
            .execute(&mut **tx)
            .await?;
        } else {
            // 插入新的 FTS 索引
            sqlx::query(
                r#"
                INSERT INTO resources_fts (rowid, title, description, content, tags, url)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(resource_id)
            .bind(title_keywords)
            .bind(description_keywords)
            .bind(content_keywords)
            .bind(tags_keywords)
            .bind(url_text)
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    /// 批量重建 FTS 索引
    ///
    /// 用于维护服务或数据迁移场景
    ///
    /// # 参数
    /// - user_id: 用户 ID (可选, None 表示重建所有用户的索引)
    /// - db_pool: 数据库连接池
    pub async fn rebuild_index(user_id: Option<i64>, db_pool: &SqlitePool) -> AppResult<u64> {
        let mut tx = db_pool.begin().await?;

        // 清空 FTS 表
        if let Some(uid) = user_id {
            sqlx::query(
                r#"
                DELETE FROM resources_fts
                WHERE rowid IN (SELECT id FROM resources WHERE user_id = $1)
                "#,
            )
            .bind(uid)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query("DELETE FROM resources_fts")
                .execute(&mut *tx)
                .await?;
        }

        // 查询所有需要重建索引的资源
        let resource_ids: Vec<(i64, i64)> = if let Some(uid) = user_id {
            sqlx::query_as("SELECT id, user_id FROM resources WHERE user_id = $1")
                .bind(uid)
                .fetch_all(&mut *tx)
                .await?
        } else {
            sqlx::query_as("SELECT id, user_id FROM resources")
                .fetch_all(&mut *tx)
                .await?
        };

        let count = resource_ids.len() as u64;

        // 逐个重建索引
        for (resource_id, resource_user_id) in resource_ids {
            Self::index_resource(&mut tx, resource_id, resource_user_id).await?;
        }

        tx.commit().await?;

        Ok(count)
    }
}
