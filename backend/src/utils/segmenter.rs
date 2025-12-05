/// 文本分词工具模块
/// 默认使用 SQLite FTS5 内置分词器，可选启用 jieba-rs 进行中文分词
/// 
/// 分词策略：
/// - 默认：依赖 SQLite FTS5 的 unicode61 分词器，不对文本进行预处理
/// - 启用 jieba feature：使用 jieba 进行中文分词，提供更好的中文搜索体验
#[cfg(feature = "jieba")]
use jieba_rs::Jieba;
#[cfg(feature = "jieba")]
use once_cell::sync::Lazy;

#[cfg(feature = "jieba")]
/// 全局静态 Jieba 实例
/// 使用 Lazy 初始化避免重复加载字典，提高性能
static JIEBA: Lazy<Jieba> = Lazy::new(|| Jieba::new());

/// 将文本转换为适合 FTS5 索引的格式
///
/// # 参数
/// * `text` - 可选的输入文本
///
/// # 返回
/// * 处理后的文本字符串，如果输入为 None 则返回空字符串
///
/// # 分词策略
/// - 默认模式（无 jieba feature）：返回原始文本，让 SQLite FTS5 的 unicode61 分词器处理
/// - jieba 模式：使用 jieba 进行中文分词，返回空格分隔的关键词
///
/// # 示例
/// ```
/// use bookmarks_api::utils::segmenter::prepare_for_search;
///
/// // 默认模式：返回原始文本
/// let result = prepare_for_search(Some("Linux内核开发"));
/// assert_eq!(result, "Linux内核开发");
/// 
/// // jieba 模式：返回分词结果
/// // 返回类似: "Linux 内核 开发"
/// ```
pub fn prepare_for_search(text: Option<&str>) -> String {
    match text {
        Some(t) if !t.is_empty() => {
            #[cfg(feature = "jieba")]
            {
                // 使用精确模式分词（cut 方法的第二个参数为 false）
                // 这样可以获得更准确的分词结果，适合搜索场景
                JIEBA.cut(t, false).join(" ")
            }
            #[cfg(not(feature = "jieba"))]
            {
                // 默认模式：返回原始文本，让 SQLite FTS5 的 unicode61 分词器处理
                // 这样可以充分利用 SQLite FTS5 内置的分词能力
                t.trim().to_string()
            }
        }
        _ => String::new(),
    }
}

/// 处理标签数组，将其转换为适合 FTS5 索引的格式
///
/// # 参数
/// * `tags` - 标签字符串的切片
///
/// # 返回
/// * 空格分隔的所有标签
///
/// # 分词策略
/// - 默认模式：直接用空格连接标签，让 SQLite FTS5 处理分词
/// - jieba 模式：对每个标签进行分词，然后合并
pub fn prepare_tags_for_search(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }

    #[cfg(feature = "jieba")]
    {
        // 对每个标签进行分词，然后合并
        tags.iter()
            .filter(|tag| !tag.trim().is_empty())
            .map(|tag| JIEBA.cut(tag, false).join(" "))
            .collect::<Vec<_>>()
            .join(" ")
    }
    #[cfg(not(feature = "jieba"))]
    {
        // 默认模式：直接用空格连接标签，让 SQLite FTS5 的 unicode61 分词器处理
        tags.iter()
            .filter(|tag| !tag.trim().is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// 注意：simple_text_processing 函数已被移除
// 现在默认使用 SQLite FTS5 的内置分词器，不需要在 Rust 层面进行预处理
// 如果需要自定义预处理逻辑，可以在启用 jieba feature 时实现

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_for_search_chinese() {
        let result = prepare_for_search(Some("Linux内核开发"));
        // 验证输出不为空
        assert!(!result.is_empty());
        
        #[cfg(feature = "jieba")]
        {
            // jieba 模式：应该包含分词结果
            assert!(result.contains("Linux"));
        }
        #[cfg(not(feature = "jieba"))]
        {
            // 默认模式：应该返回原始文本
            assert_eq!(result, "Linux内核开发");
        }
    }

    #[test]
    fn test_prepare_for_search_empty() {
        assert_eq!(prepare_for_search(None), "");
        assert_eq!(prepare_for_search(Some("")), "");
    }

    #[test]
    fn test_prepare_for_search_english() {
        let result = prepare_for_search(Some("Rust programming language"));
        // 英文文本应该被正确处理
        assert!(!result.is_empty());
        
        #[cfg(not(feature = "jieba"))]
        {
            // 默认模式：应该返回原始文本
            assert_eq!(result, "Rust programming language");
        }
    }

    #[test]
    fn test_prepare_tags_for_search() {
        let tags = vec![
            "技术".to_string(),
            "编程".to_string(),
            "Rust".to_string(),
        ];
        let result = prepare_tags_for_search(&tags);
        assert!(!result.is_empty());
        // 验证所有标签都被处理
        assert!(result.contains("技术"));
        assert!(result.contains("编程"));
        assert!(result.contains("Rust"));
    }

    #[test]
    fn test_prepare_tags_empty() {
        let tags: Vec<String> = vec![];
        assert_eq!(prepare_tags_for_search(&tags), "");
    }

    #[test]
    fn test_prepare_tags_mixed() {
        let tags = vec![
            "技术".to_string(),
            "programming".to_string(),
            "Rust语言".to_string(),
        ];
        let result = prepare_tags_for_search(&tags);
        assert!(!result.is_empty());
        
        #[cfg(not(feature = "jieba"))]
        {
            // 默认模式：应该直接用空格连接
            assert_eq!(result, "技术 programming Rust语言");
        }
        
        #[cfg(feature = "jieba")]
        {
            // jieba 模式：应该包含分词结果
            assert!(result.contains("技术"));
            assert!(result.contains("programming"));
            assert!(result.contains("Rust") || result.contains("语言"));
        }
    }

    #[cfg(not(feature = "jieba"))]
    #[test]
    fn test_default_fts_processing() {
        let result = prepare_for_search(Some("Linux，内核开发。"));
        // 默认模式：应该返回原始文本（去除首尾空格）
        assert_eq!(result, "Linux，内核开发。");
        // SQLite FTS5 的 unicode61 分词器会在搜索时处理标点符号
    }

    #[cfg(feature = "jieba")]
    #[test]
    fn test_jieba_segmentation() {
        let result = prepare_for_search(Some("Linux内核开发"));
        // 在启用 jieba 的情况下，应该能正确分词
        assert!(!result.is_empty());
        // 验证分词效果（具体分词结果取决于 jieba 的词典）
        assert!(result.contains("Linux"));
        // 验证中文被正确分词（通常会被分为 "Linux", "内核", "开发"）
        let words: Vec<&str> = result.split_whitespace().collect();
        assert!(words.len() >= 2); // 至少应该分成两个词
    }
}
