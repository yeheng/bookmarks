/// 中文分词工具模块
/// 使用 jieba-rs 进行中文分词，为 FTS5 全文搜索提供支持
use jieba_rs::Jieba;
use once_cell::sync::Lazy;

/// 全局静态 Jieba 实例
/// 使用 Lazy 初始化避免重复加载字典，提高性能
static JIEBA: Lazy<Jieba> = Lazy::new(|| Jieba::new());

/// 将文本转换为空格分隔的关键词，供 FTS5 索引使用
///
/// # 参数
/// * `text` - 可选的输入文本
///
/// # 返回
/// * 空格分隔的关键词字符串，如果输入为 None 则返回空字符串
///
/// # 示例
/// ```
/// let result = prepare_for_search(Some("Linux内核开发"));
/// // 返回类似: "Linux 内核 开发"
/// ```
pub fn prepare_for_search(text: Option<&str>) -> String {
    match text {
        Some(t) if !t.is_empty() => {
            // 使用精确模式分词（cut 方法的第二个参数为 false）
            // 这样可以获得更准确的分词结果，适合搜索场景
            JIEBA.cut(t, false).join(" ")
        }
        _ => String::new(),
    }
}

/// 处理标签数组，将其转换为空格分隔的文本供 FTS5 索引
///
/// # 参数
/// * `tags` - 标签字符串的切片
///
/// # 返回
/// * 空格分隔的所有标签（经过分词）
pub fn prepare_tags_for_search(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }

    // 对每个标签进行分词，然后合并
    tags.iter()
        .filter(|tag| !tag.is_empty())
        .map(|tag| JIEBA.cut(tag, false).join(" "))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_for_search_chinese() {
        let result = prepare_for_search(Some("Linux内核开发"));
        // 验证输出包含分词结果
        assert!(!result.is_empty());
        assert!(result.contains("Linux"));
    }

    #[test]
    fn test_prepare_for_search_empty() {
        assert_eq!(prepare_for_search(None), "");
        assert_eq!(prepare_for_search(Some("")), "");
    }

    #[test]
    fn test_prepare_for_search_english() {
        let result = prepare_for_search(Some("Rust programming language"));
        // 英文也应该被正确处理
        assert!(!result.is_empty());
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
}
