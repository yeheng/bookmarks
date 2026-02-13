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
        let results = self.engine.search_files(&ctx.query, ctx.limit)?;

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
                icon: None,
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
