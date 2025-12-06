//! 维护服务 - 处理数据库维护任务
//! 包含 FTS 索引重建等后台维护任务
use sqlx::SqlitePool;
use tracing::{error, info, warn};

use crate::services::IndexerService;

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

    // 2. 检查 resources 表行数
    let resources_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources")
        .fetch_one(&pool)
        .await?;

    // 如果 resources 也为空，无需重建
    if resources_count == 0 {
        info!("Resources 表为空，无需重建 FTS 索引");
        return Ok(());
    }

    // 3. 启动后台重建任务
    warn!(
        "检测到 {} 条资源但 FTS 索引为空，启动后台重建任务...",
        resources_count
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
/// 委托给 IndexerService::rebuild_index 统一处理
async fn rebuild_fts_index(pool: SqlitePool) -> anyhow::Result<()> {
    info!("开始重建 FTS5 索引...");

    let count = IndexerService::rebuild_index(None, &pool).await?;

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

        // 验证 FTS 表记录数（如果resources表为空，FTS表也应该为空）
        let fts_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources_fts")
            .fetch_one(&pool)
            .await
            .unwrap();
        let resources_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(fts_count, resources_count); // FTS记录数应该等于resources记录数
    }
}
