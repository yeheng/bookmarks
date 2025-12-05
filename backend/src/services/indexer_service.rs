use sqlx::{Row, SqlitePool};

use crate::models::Bookmark;
use crate::utils::error::{AppError, AppResult};
use crate::utils::segmenter::{prepare_for_search, prepare_tags_for_search};

/// IndexerService - FTS 索引管理服务
///
/// 这个服务封装了所有与 FTS5 全文搜索索引相关的逻辑
/// 确保系统中关于"如何索引一个书签"的逻辑只存在于一个地方 (DRY 原则)
///
/// 分词策略：
/// - 默认模式：存储原始文本，让 SQLite FTS5 的 unicode61 分词器处理分词
/// - jieba 模式：使用 jieba 进行中文分词预处理，存储空格分隔的关键词
///
/// 使用场景：
/// - BookmarkService 在创建/更新书签时调用
/// - TagService 在重命名标签时调用（遍历受影响的书签）
/// - MaintenanceService 在重建索引时调用
pub struct IndexerService;

impl IndexerService {
    /// 为单个书签建立或更新 FTS 索引
    ///
    /// # 参数
    /// - tx: 数据库事务，确保与书签操作的 ACID 一致性
    /// - bookmark_id: 要索引的书签 ID
    /// - db_pool: 数据库连接池（用于查询书签和标签数据）
    ///
    /// # 工作流程
    /// 1. 查询书签的完整数据（title, description, url）
    /// 2. 查询书签关联的所有标签名称
    /// 3. 根据配置处理文本（默认：原始文本，jieba：分词预处理）
    /// 4. 更新或插入 FTS 索引记录
    pub async fn index_bookmark(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        bookmark_id: i64,
        user_id: i64,
    ) -> AppResult<()> {
        // 1. 查询书签数据
        let bookmark = sqlx::query_as::<_, Bookmark>(
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
        .fetch_optional(&mut **tx)
        .await?;

        // 如果书签不存在，返回错误
        let bookmark =
            bookmark.ok_or_else(|| AppError::NotFound("Bookmark not found".to_string()))?;

        // 2. 查询书签关联的标签
        let tag_rows = sqlx::query(
            r#"
            SELECT t.name
            FROM tags t
            JOIN bookmark_tags bt ON t.id = bt.tag_id
            WHERE bt.bookmark_id = $1 AND t.user_id = $2
            "#,
        )
        .bind(bookmark_id)
        .bind(user_id)
        .fetch_all(&mut **tx)
        .await?;

        let tag_names: Vec<String> = tag_rows
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();

        // 3. 准备 FTS 数据（根据配置选择分词策略）
        let title_keywords = prepare_for_search(Some(&bookmark.title));
        let description_keywords = prepare_for_search(bookmark.description.as_deref());
        let tags_keywords = prepare_tags_for_search(&tag_names);
        let url_text = bookmark.url.clone();

        // 4. 检查 FTS 记录是否存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM bookmarks_fts WHERE rowid = $1)",
        )
        .bind(bookmark_id)
        .fetch_one(&mut **tx)
        .await?;

        if exists {
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
            .bind(bookmark_id)
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
            .bind(bookmark_id)
            .bind(title_keywords)
            .bind(description_keywords)
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
    /// - user_id: 用户 ID（可选，None 表示重建所有用户的索引）
    /// - db_pool: 数据库连接池
    pub async fn rebuild_index(
        user_id: Option<i64>,
        db_pool: &SqlitePool,
    ) -> AppResult<u64> {
        let mut tx = db_pool.begin().await?;

        // 清空 FTS 表
        if let Some(uid) = user_id {
            sqlx::query(
                r#"
                DELETE FROM bookmarks_fts
                WHERE rowid IN (SELECT id FROM bookmarks WHERE user_id = $1)
                "#,
            )
            .bind(uid)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query("DELETE FROM bookmarks_fts")
                .execute(&mut *tx)
                .await?;
        }

        // 查询所有需要重建索引的书签
        let bookmark_ids: Vec<(i64, i64)> = if let Some(uid) = user_id {
            sqlx::query_as("SELECT id, user_id FROM bookmarks WHERE user_id = $1")
                .bind(uid)
                .fetch_all(&mut *tx)
                .await?
        } else {
            sqlx::query_as("SELECT id, user_id FROM bookmarks")
                .fetch_all(&mut *tx)
                .await?
        };

        let count = bookmark_ids.len() as u64;

        // 逐个重建索引
        for (bookmark_id, bookmark_user_id) in bookmark_ids {
            Self::index_bookmark(&mut tx, bookmark_id, bookmark_user_id).await?;
        }

        tx.commit().await?;

        Ok(count)
    }
}
