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
