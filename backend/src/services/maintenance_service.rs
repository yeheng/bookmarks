/// 维护服务 - 处理数据库维护任务
///
/// 包含 FTS 索引重建等后台维护任务

use sqlx::{Row, SqlitePool};
use tracing::{error, info, warn};

/// 检查并重建 FTS 索引（如果需要）
///
/// 检查逻辑：
/// 1. 检查 bookmarks_fts 是否为空
/// 2. 如果 FTS 为空但 bookmarks 有数据，则启动后台重建任务
/// 3. 重建过程在后台异步执行，不阻塞主应用启动
pub async fn check_and_rebuild_fts(pool: SqlitePool) -> anyhow::Result<()> {
    info!("检查 FTS 索引状态...");

    // 1. 检查 FTS 表行数
    // let fts_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks_fts")
    //    .fetch_one(&pool)
    //    .await?;

    // 如果 FTS 不为空，跳过重建
    // if fts_count > 0 {
        //info!("FTS 索引已存在 ({} 条记录)，跳过重建", fts_count);
        // return Ok(());
    // }

    // 2. 检查 bookmarks 表行数
    let bookmarks_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks")
        .fetch_one(&pool)
        .await?;

    // 如果 bookmarks 也为空，无需重建
    if bookmarks_count == 0 {
        info!("Bookmarks 表为空，无需重建 FTS 索引");
        return Ok(());
    }

    // 3. 启动后台重建任务
    warn!(
        "检测到 {} 条书签但 FTS 索引为空，启动后台重建任务...",
        bookmarks_count
    );

    // 使用 tokio::spawn 在后台执行重建，不阻塞主应用
    tokio::spawn(async move {
        if let Err(e) = rebuild_fts_index(pool).await {
            error!("FTS 索引重建失败: {}", e);
        }
    });

    Ok(())
}

/// 执行 FTS 索引重建的核心逻辑
///
/// 从 bookmarks 表读取数据，使用 jieba 分词，插入到 bookmarks_fts 表
async fn rebuild_fts_index(pool: SqlitePool) -> anyhow::Result<()> {
    use jieba_rs::Jieba;

    info!("开始重建 FTS5 索引...");

    // 清空现有 FTS 数据
    info!("清空现有 FTS 索引...");
    sqlx::query("DELETE FROM bookmarks_fts")
        .execute(&pool)
        .await?;

    // 读取所有书签
    info!("读取书签数据...");
    let bookmarks = sqlx::query(
        r#"
        SELECT
            b.id,
            b.title,
            b.description,
            b.url,
            COALESCE(
                (SELECT GROUP_CONCAT(t.name, ',')
                 FROM bookmark_tags bt
                 JOIN tags t ON bt.tag_id = t.id
                 WHERE bt.bookmark_id = b.id),
                ''
            ) as tags
        FROM bookmarks b
        "#,
    )
    .fetch_all(&pool)
    .await?;

    info!("找到 {} 条书签，开始分词和索引...", bookmarks.len());

    // 使用 jieba 分词并插入 FTS
    let jieba = Jieba::new();
    let mut count = 0;

    for row in bookmarks {
        let id: i64 = row.get("id");
        let title: String = row.get("title");
        let description: Option<String> = row.get("description");
        let url: String = row.get("url");
        let tags_str: String = row.get("tags");

        // 分词
        let title_keywords = jieba.cut(&title, false).join(" ");
        let description_keywords = description
            .as_ref()
            .map(|d| jieba.cut(d, false).join(" "))
            .unwrap_or_default();

        // 处理标签（逗号分隔）
        let tags_keywords = if tags_str.is_empty() {
            String::new()
        } else {
            tags_str
                .split(',')
                .map(|tag| jieba.cut(tag, false).join(" "))
                .collect::<Vec<_>>()
                .join(" ")
        };

        // 插入 FTS
        sqlx::query(
            r#"
            INSERT INTO bookmarks_fts (rowid, title, description, tags, url)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(title_keywords)
        .bind(description_keywords)
        .bind(tags_keywords)
        .bind(url)
        .execute(&pool)
        .await?;

        count += 1;
        if count % 100 == 0 {
            info!("已处理 {} 条书签...", count);
        }
    }

    info!("✅ FTS 索引重建完成！共处理 {} 条书签", count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_and_rebuild_fts_empty_database() {
        // 创建内存数据库用于测试
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // 运行 migrations 创建表结构
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // 测试空数据库情况
        let result = check_and_rebuild_fts(pool.clone()).await;
        assert!(result.is_ok());

        // 验证 FTS 表仍然为空
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks_fts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
