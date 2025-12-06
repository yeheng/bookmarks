//! FTS5 搜索功能配置验证集成测试
//!
//! 验证不同分词模式下的搜索效果和配置
//! 运行方式：
//! - 默认模式：cargo test test_fts_config_integration
//! - jieba 模式：cargo test test_fts_config_integration --features jieba

use bookmarks_api::utils::segmenter::{prepare_for_search, prepare_tags_for_search};

#[test]
fn test_fts_config_integration() {
    println!("=== FTS5 分词器配置验证 ===\n");

    // 模拟书签数据
    let bookmarks = vec![
        (
            "Linux内核开发指南",
            "深入讲解Linux内核的开发技术和调试方法",
            vec!["Linux".to_string(), "内核".to_string(), "技术".to_string()],
        ),
        (
            "Rust编程语言",
            "Rust系统编程语言的核心特性",
            vec!["Rust".to_string(), "编程".to_string(), "语言".to_string()],
        ),
        (
            "数据库设计与优化",
            "MySQL和PostgreSQL数据库性能优化",
            vec!["数据库".to_string(), "SQL".to_string(), "优化".to_string()],
        ),
    ];

    println!("1. 默认模式（SQLite FTS5 unicode61 分词器）:");
    println!("   - 特点：基于 Unicode 6.1 标准的分词");
    println!("   - 优势：原生支持，无需外部依赖");
    println!("   - 适用：英文为主，中文简单的场景\n");

    for (i, (title, desc, tags)) in bookmarks.iter().enumerate() {
        let title_processed = prepare_for_search(Some(title));
        let desc_processed = prepare_for_search(Some(desc));
        let tags_processed = prepare_tags_for_search(&tags);

        println!("   书签 {}: {}", i + 1, title);
        println!("   标题索引: '{}'", title_processed);
        println!("   描述索引: '{}'", desc_processed);
        println!("   标签索引: '{}'", tags_processed);
        println!();

        // 验证处理结果
        assert!(!title_processed.is_empty(), "标题处理结果不应为空");
        assert!(!desc_processed.is_empty(), "描述处理结果不应为空");
        assert!(!tags_processed.is_empty(), "标签处理结果不应为空");

        // 验证关键词包含
        assert!(
            title_processed.contains("Linux")
                || title_processed.contains("Rust")
                || title_processed.contains("数据库")
        );
        assert!(
            tags_processed.contains("Linux")
                || tags_processed.contains("Rust")
                || tags_processed.contains("数据库")
        );
    }

    // 测试不同模式下的行为差异
    #[cfg(feature = "jieba")]
    {
        println!("2. jieba 模式（启用 jieba feature）:");
        println!("   - 特点：基于词典的中文分词");
        println!("   - 优势：中文分词更精确，搜索体验更好");
        println!("   - 适用：中文内容较多的场景\n");

        // jieba 模式下的特定验证
        let test_text = "Linux内核开发指南";
        let result = prepare_for_search(Some(test_text));
        let words: Vec<&str> = result.split_whitespace().collect();

        println!("   测试文本: {}", test_text);
        println!("   jieba 分词结果: '{}'", result);
        println!("   分词数量: {}\n", words.len());

        // 验证分词效果
        assert!(words.len() >= 2, "jieba 应该将中文文本分词为多个词");
        assert!(words.iter().any(|w| w.contains("Linux")), "应该包含 Linux");
    }

    #[cfg(not(feature = "jieba"))]
    {
        println!("2. 默认模式验证:");
        println!("   - 当前使用 SQLite FTS5 的 unicode61 分词器");
        println!("   - 中文文本将保持原样，由数据库处理分词\n");

        // 默认模式下的特定验证
        let test_text = "Linux内核开发指南";
        let result = prepare_for_search(Some(test_text));

        println!("   测试文本: {}", test_text);
        println!("   处理结果: '{}'", result);
        println!("   验证: 原始文本保持不变\n");

        assert_eq!(result, test_text, "默认模式应该保持原始文本不变");
    }

    println!("3. FTS5 分词器配置说明:");
    println!("   当前配置: tokenize = 'unicode61 remove_diacritics 2'");
    println!("   - unicode61: 支持 Unicode 6.1 标准的字符分割");
    println!("   - remove_diacritics 2: 移除变音符号，提高匹配率");
    println!("   - 适用于中英文混合文本的基本分词需求\n");

    println!("4. 搜索效果对比:");
    println!("   搜索词: '内核开发'");
    println!("   - 默认模式: 需要完整匹配 '内核开发' 或 FTS5 自动分割");
    println!("   - jieba 模式: 可以匹配 '内核' 和 '开发' 的组合\n");

    println!("5. 推荐配置:");
    println!("   - 生产环境（中文为主）: 启用 jieba feature");
    println!("   - 开发环境（快速测试）: 使用默认模式");
    println!("   - 英文为主: 默认模式已足够\n");

    // 验证标签处理的一致性
    let test_tags = vec!["技术".to_string(), "编程".to_string(), "开发".to_string()];
    let tags_result = prepare_tags_for_search(&test_tags);

    println!("6. 标签处理验证:");
    println!("   原始标签: {:?}", test_tags);
    println!("   处理结果: '{}'", tags_result);

    assert!(!tags_result.is_empty(), "标签处理结果不应为空");
    for tag in &test_tags {
        assert!(tags_result.contains(tag), "结果应包含所有标签: {}", tag);
    }

    println!("\n=== 配置验证完成 ===");
}

#[test]
fn test_fts_search_scenarios() {
    println!("=== FTS 搜索场景测试 ===\n");

    // 模拟各种搜索场景
    let search_scenarios = vec![
        ("精确匹配", "Linux内核", "应该能匹配包含完整短语的内容"),
        ("部分匹配", "内核", "应该能匹配包含关键词的内容"),
        ("英文搜索", "programming", "应该能匹配英文内容"),
        ("混合搜索", "Rust开发", "应该能匹配中英文混合内容"),
        ("技术术语", "数据库优化", "应该能匹配专业术语"),
    ];

    for (scenario, query, description) in search_scenarios {
        println!("场景: {}", scenario);
        println!("搜索词: '{}'", query);
        println!("说明: {}", description);

        let processed_query = prepare_for_search(Some(query));
        println!("处理后: '{}'", processed_query);

        // 验证查询处理
        assert!(!processed_query.is_empty(), "搜索词处理结果不应为空");

        println!();
    }

    println!("=== 搜索场景测试完成 ===");
}

#[test]
fn test_fts_performance_considerations() {
    println!("=== FTS 性能考虑测试 ===\n");

    // 测试大量数据的处理性能
    let large_bookmarks: Vec<(String, String, Vec<String>)> = (0..1000)
        .map(|i| {
            let title = format!("书签标题_{}", i);
            let desc = format!("书签描述内容_{}", i);
            let tags = vec![format!("标签_{}", i), "通用标签".to_string()];
            (title, desc, tags)
        })
        .collect();

    let start = std::time::Instant::now();

    for (title, desc, tags) in &large_bookmarks {
        let _title_processed = prepare_for_search(Some(title));
        let _desc_processed = prepare_for_search(Some(desc));
        let _tags_processed = prepare_tags_for_search(tags);
    }

    let duration = start.elapsed();
    let avg_time_per_item = duration.as_millis() as f64 / large_bookmarks.len() as f64;

    println!("处理 {} 个书签耗时: {:?}", large_bookmarks.len(), duration);
    println!("平均每个书签处理时间: {:.3} ms", avg_time_per_item);

    // 验证性能在合理范围内
    #[cfg(feature = "jieba")]
    {
        // jieba 模式：允许更长的处理时间
        assert!(
            avg_time_per_item < 5.0,
            "jieba 模式平均处理时间过长: {:.3} ms",
            avg_time_per_item
        );
    }
    #[cfg(not(feature = "jieba"))]
    {
        // 默认模式：应该很快
        assert!(
            avg_time_per_item < 1.0,
            "默认模式平均处理时间过长: {:.3} ms",
            avg_time_per_item
        );
    }

    println!("\n=== 性能测试完成 ===");
}
