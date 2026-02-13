//! Bookmark search provider — wraps TantivySearchEngine for bookmark search.

use async_trait::async_trait;
use std::sync::Arc;

use super::engine::SearchError;
use super::provider::{ProviderResult, SearchContext, SearchProvider, SourceType};
use super::TantivySearchEngine;

/// Provides bookmark search results via Tantivy.
pub struct BookmarkSearchProvider {
    engine: Arc<TantivySearchEngine>,
}

impl BookmarkSearchProvider {
    pub fn new(engine: Arc<TantivySearchEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl SearchProvider for BookmarkSearchProvider {
    fn source_id(&self) -> &str {
        "bookmarks"
    }

    fn source_label(&self) -> &str {
        "Bookmarks"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Bookmark
    }

    async fn search(&self, ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError> {
        let results = self.engine.search_bookmarks(&ctx.query, ctx.limit)?;

        Ok(results
            .into_iter()
            .map(|r| ProviderResult {
                id: r.id.to_string(),
                title: r.title,
                subtitle: r.url.clone(),
                source_type: SourceType::Bookmark,
                source_id: "bookmarks".to_string(),
                score: r.score,
                frecency_score: r.frecency_score,
                icon: None,
                url: Some(r.url),
                path: None,
                favicon_url: r.favicon_url,
                description: r.description,
                extension: None,
                size: None,
                modified_at: None,
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
        let provider = BookmarkSearchProvider::new(engine);
        assert_eq!(provider.source_id(), "bookmarks");
    }

    #[tokio::test]
    async fn test_source_label() {
        let (_tmp, engine) = create_test_engine();
        let provider = BookmarkSearchProvider::new(engine);
        assert_eq!(provider.source_label(), "Bookmarks");
    }

    #[tokio::test]
    async fn test_source_type() {
        let (_tmp, engine) = create_test_engine();
        let provider = BookmarkSearchProvider::new(engine);
        assert_eq!(provider.source_type(), SourceType::Bookmark);
    }

    #[tokio::test]
    async fn test_search_returns_mapped_results() {
        let (_tmp, engine) = create_test_engine();

        // Index a test bookmark
        engine
            .index_bookmark(
                1,
                "Rust Programming",
                "https://rust-lang.org",
                Some("The Rust programming language"),
                Some("rust,programming"),
                None,
                1000,
                1000,
            )
            .unwrap();

        let provider = BookmarkSearchProvider::new(engine);
        let ctx = SearchContext {
            query: "Rust".to_string(),
            limit: 10,
            fuzzy: true,
            sources: None,
        };

        let results = provider.search(&ctx).await.unwrap();
        assert!(!results.is_empty());

        let first = &results[0];
        assert_eq!(first.id, "1");
        assert_eq!(first.title, "Rust Programming");
        assert_eq!(first.url, Some("https://rust-lang.org".to_string()));
        assert_eq!(first.source_type, SourceType::Bookmark);
        assert_eq!(first.source_id, "bookmarks");
        assert!(first.score > 0.0);
        assert!(first.path.is_none());
        assert!(first.plugin_actions.is_none());
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let (_tmp, engine) = create_test_engine();
        let provider = BookmarkSearchProvider::new(engine);
        let ctx = SearchContext {
            query: "".to_string(),
            limit: 10,
            fuzzy: true,
            sources: None,
        };

        // Should not error, may return empty or recent bookmarks
        let results = provider.search(&ctx).await;
        assert!(results.is_ok());
    }

    #[tokio::test]
    async fn test_search_no_matches() {
        let (_tmp, engine) = create_test_engine();

        engine
            .index_bookmark(1, "Hello World", "https://example.com", None, None, None, 1000, 1000)
            .unwrap();

        let provider = BookmarkSearchProvider::new(engine);
        let ctx = SearchContext {
            query: "zzzznonexistent".to_string(),
            limit: 10,
            fuzzy: true,
            sources: None,
        };

        let results = provider.search(&ctx).await.unwrap();
        assert!(results.is_empty());
    }
}

