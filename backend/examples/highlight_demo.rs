//! Tantivy 高亮功能演示
//!
//! 展示如何使用内置的 SnippetGenerator 生成搜索结果高亮

use bookmarks_api::config::AppConfig;
use bookmarks_api::services::TantivyIndexManager;
use bookmarks_api::models::BookmarkWithTags;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_target(true)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("🎨 Tantivy 高亮功能演示");
    println!("==================");

    let start_time = Instant::now();

    // 1. 加载配置
    let config = AppConfig::load()?;

    // 2. 创建临时索引目录用于演示
    let index_path = "./temp_highlight_index";
    std::fs::create_dir_all(index_path)?;

    // 3. 初始化 Tantivy 索引管理器
    println!("📁 初始化索引管理器...");
    let index_manager = TantivyIndexManager::new(index_path)?;

    // 4. 创建演示书签
    let demo_bookmarks = vec![
        create_demo_bookmark(
            1,
            1,
            "Rust 编程语言完全指南",
            "https://www.rust-lang.org/",
            Some("Rust 是一门系统编程语言，注重安全、速度和并发性。它没有垃圾回收器，这使其在资源受限的环境中表现出色。"),
            vec!["rust", "programming", "systems"],
        ),
        create_demo_bookmark(
            2,
            1,
            "JavaScript 异步编程教程",
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise",
            Some("深入学习 JavaScript 中的 Promise、async/await 和事件循环，掌握现代异步编程模式。"),
            vec!["javascript", "async", "tutorial"],
        ),
        create_demo_bookmark(
            3,
            1,
            "Python 数据科学入门",
            "https://pandas.pydata.org/docs/",
            Some("使用 Python 进行数据分析和可视化，包括 NumPy、Pandas 和 Matplotlib 的使用方法。"),
            vec!["python", "data-science", "pandas"],
        ),
    ];

    // 5. 添加书签到索引
    println!("📚 添加演示书签到索引...");
    for bookmark in &demo_bookmarks {
        index_manager.add_bookmark(bookmark)?;
        println!("  ✅ 添加: {}", bookmark.bookmark.title);
    }
    index_manager.commit()?;
    index_manager.reload()?;

    let index_time = start_time.elapsed();
    println!("⏱️  索引构建耗时: {:?}", index_time);

    // 6. 演示高亮功能
    println!("\n🔍 高亮功能演示");
    println!("==================");

    let search_queries = vec!["rust", "编程", "数据", "async", "python"];

    for query in search_queries {
        println!("\n🔎 搜索词: '{}'", query);
        println!("----------------------------");

        // 搜索结果
        let search_start = Instant::now();
        let results = index_manager.search(query, 1, 10, 0)?;
        let search_time = search_start.elapsed();

        if !results.results.is_empty() {
            for result in &results.results {
                println!("📄 书签 ID: {} (评分: {:.2})", result.bookmark_id, result.score);

                // 生成高亮
                let highlight_start = Instant::now();
                let highlights = index_manager.generate_highlights(result.bookmark_id, query)?;
                let highlight_time = highlight_start.elapsed();

                if !highlights.is_empty() {
                    for (field, snippets) in highlights {
                        println!("  🎨 {}:", field);
                        for snippet in snippets {
                            println!("    {}", snippet);
                        }
                    }
                } else {
                    println!("  ⚪ 无高亮片段");
                }
                println!("  ⚡ 高亮生成耗时: {:?}", highlight_time);
            }
        } else {
            println!("  ⚪ 无搜索结果");
        }
        println!("  ⚡ 搜索耗时: {:?}", search_time);
    }

    // 7. 统计信息
    println!("\n📊 索引统计信息");
    println!("==================");
    let stats = index_manager.get_stats()?;
    println!("  📚 总书签数: {}", stats);
    println!("  ⏱️  总耗时: {:?}", start_time.elapsed());

    // 8. 清理临时文件
    std::fs::remove_dir_all(index_path)?;
    println!("🧹 清理完成");

    println!("\n✅ 演示完成！");
    Ok(())
}

fn create_demo_bookmark(
    id: i64,
    user_id: i64,
    title: &str,
    url: &str,
    description: Option<&str>,
    tags: Vec<&str>,
) -> BookmarkWithTags {
    use bookmarks_api::models::Bookmark;
    use chrono::Utc;

    BookmarkWithTags {
        bookmark: Bookmark {
            id,
            user_id,
            collection_id: None,
            title: title.to_string(),
            url: url.to_string(),
            description: description.map(|s| s.to_string()),
            favicon_url: None,
            screenshot_url: None,
            thumbnail_url: None,
            is_favorite: false,
            is_archived: false,
            is_private: false,
            is_read: false,
            visit_count: 0,
            last_visited: None,
            reading_time: None,
            difficulty_level: None,
            metadata: serde_json::Value::Null,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        },
        tags: tags.into_iter().map(|s| s.to_string()).collect(),
        collection_name: None,
        collection_color: None,
    }
}