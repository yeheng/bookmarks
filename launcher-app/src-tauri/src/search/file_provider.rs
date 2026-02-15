//! File search provider — wraps TantivySearchEngine for file search.

use async_trait::async_trait;
use std::sync::Arc;

use super::engine::SearchError;
use super::provider::{ProviderResult, SearchContext, SearchProvider, SourceType};
use super::TantivySearchEngine;

/// Provides file search results via Tantivy.
pub struct FileSearchProvider {
    engine: Arc<TantivySearchEngine>,
}

impl FileSearchProvider {
    pub fn new(engine: Arc<TantivySearchEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl SearchProvider for FileSearchProvider {
    fn source_id(&self) -> &str {
        "files"
    }

    fn source_label(&self) -> &str {
        "Files"
    }

    fn source_type(&self) -> SourceType {
        SourceType::File
    }

    async fn search(&self, ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError> {
        let extension_filter = ctx
            .structured_query
            .filters
            .get("type")
            .or_else(|| ctx.structured_query.filters.get("ext"))
            .map(|s| s.as_str());

        let results = self
            .engine
            .search_files(&ctx.query, ctx.limit, extension_filter)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|r| ProviderResult {
                id: r.id.to_string(),
                title: r.name,
                subtitle: r.path.clone(),
                source_type: SourceType::File,
                source_id: "files".to_string(),
                score: r.score,
                frecency_score: r.frecency_score,
                icon: Some("file".to_string()),
                url: None,
                path: Some(r.path),
                favicon_url: None,
                description: None,
                extension: r.extension,
                size: Some(r.size),
                modified_at: Some(r.modified_at),
                plugin_actions: None,
                plugin_badge: None,
                plugin_keyword: None,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::provider::SearchContext;
    use super::super::query_parser::StructuredQuery;
    use tempfile::TempDir;

    fn create_test_engine() -> (TempDir, Arc<TantivySearchEngine>) {
        let tmp = TempDir::new().unwrap();
        let engine = TantivySearchEngine::new(tmp.path().to_path_buf()).unwrap();
        let engine_arc = Arc::new(engine);
        (tmp, engine_arc)
    }

    #[tokio::test]
    async fn test_source_id() {
        let (_tmp, engine) = create_test_engine();
        let provider = FileSearchProvider::new(engine);
        assert_eq!(provider.source_id(), "files");
    }

    #[tokio::test]
    async fn test_source_label() {
        let (_tmp, engine) = create_test_engine();
        let provider = FileSearchProvider::new(engine);
        assert_eq!(provider.source_label(), "Files");
    }

    #[tokio::test]
    async fn test_source_type() {
        let (_tmp, engine) = create_test_engine();
        let provider = FileSearchProvider::new(engine);
        assert_eq!(provider.source_type(), SourceType::File);
    }

    #[tokio::test]
    async fn test_search_returns_mapped_results() {
        let (_tmp, engine) = create_test_engine();

        // Index a test file
        engine
            .index_file(1, "/home/user/document.pdf", "document.pdf", Some("pdf"), 1024, 1700000000, 1)
            .unwrap();

        let provider = FileSearchProvider::new(engine);
        let ctx = SearchContext {
            query: "document".to_string(),
            structured_query: StructuredQuery::parse("document"),
            limit: 10,
            fuzzy: true,
            sources: None,
        };

        let results = provider.search(&ctx).await.unwrap();
        assert!(!results.is_empty());

        let first = &results[0];
        assert_eq!(first.id, "1");
        assert_eq!(first.title, "document.pdf");
        assert_eq!(first.path, Some("/home/user/document.pdf".to_string()));
        assert_eq!(first.source_type, SourceType::File);
        assert_eq!(first.source_id, "files");
        assert!(first.score > 0.0);
        assert_eq!(first.size, Some(1024));
        assert!(first.url.is_none());
        assert!(first.plugin_actions.is_none());
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let (_tmp, engine) = create_test_engine();
        let provider = FileSearchProvider::new(engine);
        let ctx = SearchContext {
            query: "".to_string(),
            structured_query: StructuredQuery::parse(""),
            limit: 10,
            fuzzy: true,
            sources: None,
        };

        let results = provider.search(&ctx).await;
        assert!(results.is_ok());
    }

    #[tokio::test]
    async fn test_search_no_matches() {
        let (_tmp, engine) = create_test_engine();

        engine
            .index_file(1, "/home/user/readme.txt", "readme.txt", Some("txt"), 512, 1700000000, 1)
            .unwrap();

        let provider = FileSearchProvider::new(engine);
        let ctx = SearchContext {
            query: "zzzznonexistent".to_string(),
            structured_query: StructuredQuery::parse("zzzznonexistent"),
            limit: 10,
            fuzzy: true,
            sources: None,
        };

        let results = provider.search(&ctx).await.unwrap();
        assert!(results.is_empty());
    }
}

