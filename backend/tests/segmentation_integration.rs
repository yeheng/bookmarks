//! 分词功能集成测试
//! 
//! 测试不同分词模式下的文本处理效果
//! 运行方式：
//! - 默认模式：cargo test test_segmentation_integration
//! - jieba 模式：cargo test test_segmentation_integration --features jieba

use bookmarks_api::utils::segmenter::{prepare_for_search, prepare_tags_for_search};

#[test]
fn test_segmentation_integration() {
    println!("=== 分词功能集成测试 ===\n");

    // 测试中文文本
    let chinese_text = "Linux内核开发与调试技术";
    let result = prepare_for_search(Some(chinese_text));
    println!("中文文本: {}", chinese_text);
    println!("分词结果: {}\n", result);
    
    assert!(!result.is_empty());

    // 测试英文文本
    let english_text = "Rust programming language";
    let result = prepare_for_search(Some(english_text));
    println!("英文文本: {}", english_text);
    println!("分词结果: {}\n", result);
    
    assert!(!result.is_empty());
    assert!(result.contains("Rust"));
    assert!(result.contains("programming"));
    assert!(result.contains("language"));

    // 测试混合文本
    let mixed_text = "Rust语言开发Linux驱动程序";
    let result = prepare_for_search(Some(mixed_text));
    println!("混合文本: {}", mixed_text);
    println!("分词结果: {}\n", result);
    
    assert!(!result.is_empty());
    assert!(result.contains("Rust"));

    // 测试标签
    let tags = vec![
        "技术".to_string(),
        "编程".to_string(),
        "Rust语言".to_string(),
        "Linux内核".to_string(),
    ];
    let result = prepare_tags_for_search(&tags);
    println!("标签: {:?}", tags);
    println!("标签分词: {}\n", result);
    
    assert!(!result.is_empty());
    assert!(result.contains("技术"));
    assert!(result.contains("编程"));

    // 测试带标点的文本
    let punctuated_text = "Linux，内核开发。调试技术！";
    let result = prepare_for_search(Some(punctuated_text));
    println!("带标点文本: {}", punctuated_text);
    println!("分词结果: {}\n", result);
    
    assert!(!result.is_empty());

    // 根据不同的 feature 进行特定验证
    #[cfg(feature = "jieba")]
    {
        println!("当前模式: jieba 分词模式");
        println!("特点: 使用 jieba-rs 进行中文分词，提供更精确的中文搜索体验");
        
        // jieba 模式下的特定验证
        let chinese_result = prepare_for_search(Some("Linux内核开发"));
        let words: Vec<&str> = chinese_result.split_whitespace().collect();
        assert!(words.len() >= 2, "中文应该被分词为多个词");
        assert!(words.iter().any(|w| w.contains("Linux")));
    }
    
    #[cfg(not(feature = "jieba"))]
    {
        println!("当前模式: SQLite FTS 默认模式");
        println!("特点: 使用 SQLite FTS5 的 unicode61 分词器，依赖数据库内置分词能力");
        
        // 默认模式下的特定验证
        let original_text = "Linux内核开发";
        let result = prepare_for_search(Some(original_text));
        assert_eq!(result, original_text, "默认模式应该返回原始文本");
    }

    println!("\n=== 集成测试完成 ===");
}

#[test]
fn test_segmentation_edge_cases() {
    // 测试空值
    assert_eq!(prepare_for_search(None), "");
    assert_eq!(prepare_for_search(Some("")), "");
    
    // 测试只有空格的文本
    let spaces_result = prepare_for_search(Some("   "));
    #[cfg(feature = "jieba")]
    {
        // jieba 模式：空格可能被处理为空字符串或保持原样
        assert!(spaces_result.is_empty() || spaces_result.trim().is_empty());
    }
    #[cfg(not(feature = "jieba"))]
    {
        // 默认模式：trim() 会移除空格
        assert_eq!(spaces_result, "");
    }
    
    // 测试标签边界情况
    assert_eq!(prepare_tags_for_search(&[]), "");
    
    let empty_tags = vec!["".to_string(), "   ".to_string()];
    let tags_result = prepare_tags_for_search(&empty_tags);
    assert!(tags_result.trim().is_empty(), "只包含空字符串和空格的标签应该返回空或空白字符串");
}

#[test]
fn test_segmentation_performance() {
    // 性能测试：处理大量文本
    let large_text = "Linux内核开发与调试技术".repeat(100);
    let start = std::time::Instant::now();
    let _result = prepare_for_search(Some(&large_text));
    let duration = start.elapsed();
    
    // 根据不同模式设置不同的性能阈值
    #[cfg(feature = "jieba")]
    {
        // jieba 模式：由于需要加载词典和进行复杂分词，允许更长的处理时间
        assert!(duration.as_millis() < 2000, "jieba 分词处理时间过长: {:?}", duration);
    }
    #[cfg(not(feature = "jieba"))]
    {
        // 默认模式：简单的字符串处理，应该很快
        assert!(duration.as_millis() < 100, "默认分词处理时间过长: {:?}", duration);
    }
}

#[test]
fn test_different_languages() {
    let test_cases = vec![
        ("中文测试", "中文文本处理"),
        ("English Test", "English text processing"),
        ("Mixed 中英文 Test", "Mixed Chinese English text"),
        ("数字123测试", "Numbers 123 test"),
        ("特殊符号!@#测试", "Special symbols !@# test"),
    ];
    
    for (input, description) in test_cases {
        let result = prepare_for_search(Some(input));
        println!("{}: '{}' -> '{}'", description, input, result);
        assert!(!result.is_empty(), "处理结果不应为空: {}", input);
    }
}