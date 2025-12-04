use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{
    AllQuery, BooleanQuery, FuzzyTermQuery, Occur, Query, QueryParser, TermQuery,
};
use tantivy::schema::{
    Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, Value, FAST, INDEXED, STORED,
    TEXT,
};
use tantivy::snippet::SnippetGenerator;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};

use crate::models::BookmarkWithTags;

/// Tantivy 索引管理器
///
/// 负责管理书签的全文检索索引，包括索引的创建、更新、删除和搜索
#[derive(Clone)]
pub struct TantivyIndexManager {
    index: Index,
    reader: IndexReader,
    writer: Arc<Mutex<IndexWriter>>,
    _schema: Schema,
    // 缓存字段映射以提高性能
    field_cache: Arc<Mutex<HashMap<String, Field>>>,
}

/// Tantivy 搜索结果
#[derive(Debug, Clone)]
pub struct TantivySearchResult {
    pub bookmark_id: i64,
    pub score: f32,
}

/// Tantivy 搜索响应
#[derive(Debug)]
pub struct TantivySearchResponse {
    pub results: Vec<TantivySearchResult>,
    pub total: usize,
}

impl TantivyIndexManager {
    /// 创建新的索引管理器
    pub fn new(index_path: &str) -> Result<Self> {
        // 确保索引目录存在
        fs::create_dir_all(index_path)
            .with_context(|| format!("Failed to create index directory: {}", index_path))?;

        // 构建 Schema
        let schema = Self::build_schema();

        // 创建或打开索引
        let directory = MmapDirectory::open(Path::new(index_path))?;
        let index = Index::open_or_create(directory, schema.clone())?;

        // 创建 IndexWriter，使用 50MB 缓冲区
        let writer = index
            .writer(50_000_000)
            .with_context(|| "Failed to create index writer")?;

        // 创建 IndexReader
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .with_context(|| "Failed to create index reader")?;

        // 预构建字段缓存
        let mut field_cache = HashMap::new();
        field_cache.insert("id".to_string(), schema.get_field("id")?);
        field_cache.insert("title".to_string(), schema.get_field("title")?);
        field_cache.insert("url".to_string(), schema.get_field("url")?);
        field_cache.insert("description".to_string(), schema.get_field("description")?);
        field_cache.insert("tags".to_string(), schema.get_field("tags")?);
        field_cache.insert("user_id".to_string(), schema.get_field("user_id")?);
        field_cache.insert("created_at".to_string(), schema.get_field("created_at")?);

        Ok(Self {
            index,
            reader,
            writer: Arc::new(Mutex::new(writer)),
            _schema: schema,
            field_cache: Arc::new(Mutex::new(field_cache)),
        })
    }

    /// 构建 Tantivy Schema
    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();

        // ID 字段：索引并存储（用于精确查找）
        schema_builder.add_i64_field("id", INDEXED | STORED);

        // 标题字段：索引并存储，支持中文分词，包含位置信息用于高亮
        let title_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("title", title_options);

        // URL 字段：索引并存储
        schema_builder.add_text_field("url", TEXT | STORED);

        // 描述字段：索引并存储，包含位置信息用于高亮
        let description_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("description", description_options);

        // 标签字段：索引但不存储，将标签组合为单个文本
        schema_builder.add_text_field("tags", TEXT);

        // 用户ID字段：索引但不存储，用于过滤
        schema_builder.add_i64_field("user_id", INDEXED);

        // 创建时间字段：快速访问，用于排序
        schema_builder.add_i64_field("created_at", FAST);

        schema_builder.build()
    }

    /// 添加书签到索引
    pub fn add_bookmark(&self, bookmark: &BookmarkWithTags) -> Result<()> {
        let writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire writer lock"))?;

        // 获取字段（使用缓存避免重复查询）
        let cache = self
            .field_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire field cache lock"))?;

        // 创建文档并添加字段
        let mut doc = TantivyDocument::default();
        doc.add_i64(cache["id"], bookmark.bookmark.id);
        doc.add_text(cache["title"], &bookmark.bookmark.title);
        doc.add_text(cache["url"], &bookmark.bookmark.url);
        doc.add_i64(cache["user_id"], bookmark.bookmark.user_id);
        doc.add_i64(cache["created_at"], bookmark.bookmark.created_at);

        // 添加可选字段
        if let Some(description) = &bookmark.bookmark.description {
            doc.add_text(cache["description"], description);
        }

        // 将标签组合为单个文本，用空格分隔
        if !bookmark.tags.is_empty() {
            doc.add_text(cache["tags"], &bookmark.tags.join(" "));
        }

        writer.add_document(doc)?;
        Ok(())
    }

    /// 更新书签索引
    pub fn update_bookmark(&self, bookmark: &BookmarkWithTags) -> Result<()> {
        self.delete_bookmark_internal(bookmark.bookmark.id)?;
        self.add_bookmark(bookmark)?;
        Ok(())
    }

    /// 删除书签索引
    pub fn delete_bookmark(&self, bookmark_id: i64) -> Result<()> {
        self.delete_bookmark_internal(bookmark_id)
    }

    /// 内部删除方法
    fn delete_bookmark_internal(&self, bookmark_id: i64) -> Result<()> {
        let writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire writer lock"))?;

        // 创建查询：匹配指定ID的文档
        let id_field = self.get_field_cached("id")?;
        let term = Term::from_field_i64(id_field, bookmark_id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        writer.delete_query(Box::new(query))?;
        Ok(())
    }

    /// 按用户ID删除所有书签索引
    pub fn delete_by_user(&self, user_id: i64) -> Result<()> {
        let writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire writer lock"))?;

        let user_id_field = self.get_field_cached("user_id")?;
        let term = Term::from_field_i64(user_id_field, user_id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        writer.delete_query(Box::new(query))?;
        Ok(())
    }

    /// 提交索引更改
    pub fn commit(&self) -> Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire writer lock"))?;

        writer.commit()?;
        Ok(())
    }

    /// 搜索书签
    pub fn search(
        &self,
        query_str: &str,
        user_id: i64,
        limit: usize,
        offset: usize,
    ) -> Result<TantivySearchResponse> {
        let searcher = self.reader.searcher();

        // 构建查询
        let query = self.build_query(query_str, user_id)?;

        // 执行搜索
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit + offset))?;
        let total = searcher.search(&query, &Count)?;

        // 提取结果
        let results = top_docs
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(score, doc_address)| {
                let doc: TantivyDocument = searcher.doc(doc_address)?;
                let bookmark_id = doc
                    .get_first(self.get_field_cached("id")?)
                    .and_then(|value| value.as_i64())
                    .ok_or_else(|| anyhow::anyhow!("Document missing id field"))?;

                Ok(TantivySearchResult { bookmark_id, score })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(TantivySearchResponse { results, total })
    }

    /// 构建搜索查询
    fn build_query(&self, query_str: &str, user_id: i64) -> Result<Box<dyn Query>> {
        let mut sub_queries: Vec<Box<dyn Query>> = Vec::new();

        // 用户ID过滤（必须匹配）
        let user_id_field = self.get_field_cached("user_id")?;
        let user_term = Term::from_field_i64(user_id_field, user_id);
        sub_queries.push(Box::new(TermQuery::new(
            user_term,
            IndexRecordOption::Basic,
        )));

        // 全文搜索查询（如果提供了搜索词）
        if !query_str.is_empty() {
            let text_query = self.build_text_query(query_str)?;
            sub_queries.push(text_query);
        }

        Ok(Box::new(BooleanQuery::new(
            sub_queries.into_iter().map(|q| (Occur::Must, q)).collect(),
        )))
    }

    /// 构建文本搜索查询
    fn build_text_query(&self, query_str: &str) -> Result<Box<dyn Query>> {
        let title_field = self.get_field_cached("title")?;
        let url_field = self.get_field_cached("url")?;
        let description_field = self.get_field_cached("description")?;
        let tags_field = self.get_field_cached("tags")?;

        // 为每个字段创建模糊查询，支持拼写纠错
        let title_term = Term::from_field_text(title_field, query_str);
        let title_query = Box::new(FuzzyTermQuery::new(title_term, 1, true));

        let url_term = Term::from_field_text(url_field, query_str);
        let url_query = Box::new(FuzzyTermQuery::new(url_term, 1, true));

        let description_term = Term::from_field_text(description_field, query_str);
        let description_query = Box::new(FuzzyTermQuery::new(description_term, 1, true));

        let tags_term = Term::from_field_text(tags_field, query_str);
        let tags_query = Box::new(FuzzyTermQuery::new(tags_term, 1, true));

        // 构建布尔查询：任意字段匹配即可（OR 操作）
        Ok(Box::new(BooleanQuery::new(vec![
            (Occur::Should, title_query),
            (Occur::Should, url_query),
            (Occur::Should, description_query),
            (Occur::Should, tags_query),
        ])))
    }

    /// 清空所有索引
    pub fn clear_all(&self) -> Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire writer lock"))?;

        writer.delete_query(Box::new(AllQuery))?;
        writer.commit()?;
        Ok(())
    }

    /// 获取缓存的 Schema 字段
    fn get_field_cached(&self, name: &str) -> Result<Field> {
        let cache = self
            .field_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire field cache lock"))?;

        cache
            .get(name)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Field '{}' not found in cache", name))
    }

    /// 获取索引统计信息
    pub fn get_stats(&self) -> Result<usize> {
        let searcher = self.reader.searcher();
        let total = searcher.search(&AllQuery, &Count)?;
        Ok(total)
    }

    /// 重新加载索引reader，用于测试
    pub fn reload(&self) -> Result<()> {
        self.reader
            .reload()
            .with_context(|| "Failed to reload index reader")?;
        Ok(())
    }

    /// 生成搜索高亮片段（使用 Tantivy 内置高亮功能）
    pub fn generate_highlights(
        &self,
        bookmark_id: i64,
        query_str: &str,
    ) -> Result<HashMap<String, Vec<String>>> {
        let searcher = self.reader.searcher();

        // 创建 QueryParser 用于解析查询
        let title_field = self.get_field_cached("title")?;
        let description_field = self.get_field_cached("description")?;
        let tags_field = self.get_field_cached("tags")?;

        let query_parser = QueryParser::for_index(
            &self.index,
            vec![title_field, description_field, tags_field],
        );

        // 解析查询
        let parsed_query = query_parser
            .parse_query(query_str)
            .with_context(|| format!("Failed to parse query: {}", query_str))?;

        // 查找特定文档
        let id_field = self.get_field_cached("id")?;
        let id_term = Term::from_field_i64(id_field, bookmark_id);
        let id_query = TermQuery::new(id_term, IndexRecordOption::Basic);

        // 组合查询：文档ID + 文本查询
        let combined_query = BooleanQuery::new(vec![
            (Occur::Must, Box::new(id_query)),
            (Occur::Should, parsed_query),
        ]);

        let docs = searcher.search(&combined_query, &TopDocs::with_limit(1))?;

        if let Some((_, doc_address)) = docs.first() {
            let doc: TantivyDocument = searcher.doc(*doc_address)?;
            let mut highlights = HashMap::new();

            // 为每个字段生成高亮
            for (field_name, field) in [("title", title_field), ("description", description_field)]
            {
                // 重新解析查询（因为每个字段需要独立的查询）
                let field_parser = QueryParser::for_index(&self.index, vec![field]);
                let field_query = field_parser
                    .parse_query(query_str)
                    .with_context(|| format!("Failed to parse query for field: {}", field_name))?;

                // 创建高亮生成器
                let mut snippet_generator =
                    SnippetGenerator::create(&searcher, &*field_query, field).with_context(
                        || {
                            format!(
                                "Failed to create snippet generator for field: {}",
                                field_name
                            )
                        },
                    )?;

                // 设置高亮参数
                snippet_generator.set_max_num_chars(200); // 最多200个字符

                // 生成高亮片段
                let snippet = snippet_generator.snippet_from_doc(&doc);

                if !snippet.is_empty() {
                    // 生成HTML格式的高亮
                    let html_highlight = snippet.to_html();
                    if !html_highlight.trim().is_empty() {
                        highlights.insert(field_name.to_string(), vec![html_highlight]);
                    }
                }
            }

            Ok(highlights)
        } else {
            Ok(HashMap::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_bookmark() -> BookmarkWithTags {
        BookmarkWithTags {
            bookmark: crate::models::Bookmark {
                id: 1,
                user_id: 1,
                collection_id: None,
                title: "Rust Programming Language".to_string(),
                url: "https://www.rust-lang.org".to_string(),
                description: Some(
                    "A systems programming language that runs blazingly fast".to_string(),
                ),
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
                created_at: 1640995200, // 2022-01-01
                updated_at: 1640995200,
            },
            tags: vec!["rust".to_string(), "programming".to_string()],
            collection_name: None,
            collection_color: None,
        }
    }

    #[test]
    fn test_index_manager_crud() -> Result<()> {
        // 创建临时目录
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().to_str().unwrap();

        // 创建索引管理器
        let manager = TantivyIndexManager::new(index_path)?;

        // 测试添加书签
        let bookmark = create_test_bookmark();
        manager.add_bookmark(&bookmark)?;
        manager.commit()?;
        manager.reload()?;

        // 验证索引统计
        let stats = manager.get_stats()?;
        assert_eq!(stats, 1);

        // 测试搜索
        let results = manager.search("rust", 1, 10, 0)?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].bookmark_id, 1);

        // 测试更新书签
        let mut updated_bookmark = bookmark.clone();
        updated_bookmark.bookmark.title = "Rust Programming Language (Updated)".to_string();
        updated_bookmark.tags.push("systems".to_string());
        manager.update_bookmark(&updated_bookmark)?;
        manager.commit()?;

        // 验证更新后的搜索结果
        let results = manager.search("systems", 1, 10, 0)?;
        assert_eq!(results.results.len(), 1);

        // 测试删除书签
        manager.delete_bookmark(1)?;
        manager.commit()?;
        manager.reload()?;

        // 验证删除后的索引统计
        let stats = manager.get_stats()?;
        assert_eq!(stats, 0);

        Ok(())
    }

    #[test]
    fn test_user_isolation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().to_str().unwrap();

        let manager = TantivyIndexManager::new(index_path)?;

        // 创建两个用户的书签
        let mut bookmark1 = create_test_bookmark();
        bookmark1.bookmark.user_id = 1;

        let mut bookmark2 = create_test_bookmark();
        bookmark2.bookmark.id = 2;
        bookmark2.bookmark.user_id = 2;

        // 添加书签
        manager.add_bookmark(&bookmark1)?;
        manager.add_bookmark(&bookmark2)?;
        manager.commit()?;
        manager.reload()?;

        // 用户1搜索应该只能看到自己的书签
        let results = manager.search("rust", 1, 10, 0)?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].bookmark_id, 1);

        // 用户2搜索应该只能看到自己的书签
        let results = manager.search("rust", 2, 10, 0)?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].bookmark_id, 2);

        Ok(())
    }

    #[test]
    fn test_fuzzy_search() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().to_str().unwrap();

        let manager = TantivyIndexManager::new(index_path)?;

        let bookmark = create_test_bookmark();
        manager.add_bookmark(&bookmark)?;
        manager.commit()?;
        manager.reload()?;

        // 测试拼写纠错（模糊匹配）
        let results = manager.search("rusty", 1, 10, 0)?;
        assert_eq!(results.results.len(), 1); // 应该能匹配到 "rust"

        Ok(())
    }

    #[test]
    fn test_field_cache() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().to_str().unwrap();

        let manager = TantivyIndexManager::new(index_path)?;

        // 测试字段缓存
        let id_field = manager.get_field_cached("id")?;
        let title_field = manager.get_field_cached("title")?;

        // 缓存的字段应该是有效的
        assert_eq!(id_field.field_id(), 0);
        assert_eq!(title_field.field_id(), 1);

        Ok(())
    }

    #[test]
    fn test_generate_highlights() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().to_str().unwrap();

        let manager = TantivyIndexManager::new(index_path)?;

        // 创建测试书签
        let bookmark = create_test_bookmark();
        manager.add_bookmark(&bookmark)?;
        manager.commit()?;
        manager.reload()?;

        // 测试高亮生成
        let highlights = manager.generate_highlights(1, "rust")?;

        // 应该包含标题和描述的高亮
        assert!(!highlights.is_empty());
        assert!(highlights.contains_key("title"));

        Ok(())
    }
}
