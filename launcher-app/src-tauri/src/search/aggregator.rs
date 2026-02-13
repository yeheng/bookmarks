//! Search aggregator — coordinates parallel search across all providers.
//!
//! The `SearchAggregator` holds registered `SearchProvider`s and orchestrates
//! unified search: dispatching queries in parallel, normalizing scores per-source,
//! computing a global ranking, and returning a merged result list.

use super::engine::SearchError;
use super::provider::{ProviderResult, SearchContext, SearchProvider};

/// Orchestrates multi-source unified search.
pub struct SearchAggregator {
    providers: Vec<Box<dyn SearchProvider>>,
}

impl SearchAggregator {
    /// Create an empty aggregator.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a search provider.
    pub fn register(&mut self, provider: Box<dyn SearchProvider>) {
        self.providers.push(provider);
    }

    /// Execute a unified search across all (or filtered) providers.
    ///
    /// Steps:
    /// 1. Filter providers by `ctx.sources` if specified.
    /// 2. Query all enabled providers in parallel.
    /// 3. Apply min-max score normalization per provider.
    /// 4. Compute `global_score = normalized_score * 0.7 + normalized_frecency * 0.3`.
    /// 5. Sort by global_score descending, truncate to `ctx.limit`.
    pub async fn search(&self, ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError> {
        // 1. Filter providers
        let active_providers: Vec<&Box<dyn SearchProvider>> = self
            .providers
            .iter()
            .filter(|p| {
                ctx.sources
                    .as_ref()
                    .map(|sources| sources.iter().any(|s| s == p.source_id()))
                    .unwrap_or(true)
            })
            .collect();

        if active_providers.is_empty() {
            return Ok(Vec::new());
        }

        // 2. Query all providers in parallel
        let futures: Vec<_> = active_providers
            .iter()
            .map(|p| p.search(ctx))
            .collect();

        let results_per_provider = futures::future::join_all(futures).await;

        // 3. Collect and normalize per-provider
        let mut all_results: Vec<ProviderResult> = Vec::new();

        for provider_results in results_per_provider {
            let mut results = match provider_results {
                Ok(r) => r,
                Err(e) => {
                    // Log error but don't fail the entire search
                    eprintln!("Search provider error: {}", e);
                    continue;
                }
            };

            if results.is_empty() {
                continue;
            }

            // Min-max normalization for scores within this provider
            let (min_score, max_score) = results.iter().fold((f64::MAX, f64::MIN), |(min, max), r| {
                (min.min(r.score), max.max(r.score))
            });
            let score_range = max_score - min_score;

            let (min_frec, max_frec) = results.iter().fold((f64::MAX, f64::MIN), |(min, max), r| {
                (min.min(r.frecency_score), max.max(r.frecency_score))
            });
            let frec_range = max_frec - min_frec;

            for r in &mut results {
                let norm_score = if score_range > 0.0 {
                    (r.score - min_score) / score_range
                } else {
                    1.0 // single result or all same score
                };

                let norm_frec = if frec_range > 0.0 {
                    (r.frecency_score - min_frec) / frec_range
                } else if r.frecency_score > 0.0 {
                    1.0
                } else {
                    0.0
                };

                // Store the global score back in `score` field
                r.score = norm_score * 0.7 + norm_frec * 0.3;
            }

            all_results.extend(results);
        }

        // 4. Sort by global score descending
        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 5. Truncate to limit
        all_results.truncate(ctx.limit);

        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::provider::SourceType;
    use async_trait::async_trait;

    /// A mock provider for testing.
    struct MockProvider {
        id: String,
        label: String,
        source: SourceType,
        results: Vec<ProviderResult>,
        should_error: bool,
    }

    impl MockProvider {
        fn new(id: &str, results: Vec<ProviderResult>) -> Self {
            Self {
                id: id.to_string(),
                label: id.to_string(),
                source: SourceType::Bookmark,
                results,
                should_error: false,
            }
        }

        fn with_error(id: &str) -> Self {
            Self {
                id: id.to_string(),
                label: id.to_string(),
                source: SourceType::Bookmark,
                results: Vec::new(),
                should_error: true,
            }
        }

        fn make_result(id: &str, score: f64, frecency: f64) -> ProviderResult {
            ProviderResult {
                id: id.to_string(),
                title: format!("Result {}", id),
                subtitle: String::new(),
                source_type: SourceType::Bookmark,
                source_id: "mock".to_string(),
                score,
                frecency_score: frecency,
                icon: None,
                url: None,
                path: None,
                favicon_url: None,
                description: None,
                extension: None,
                size: None,
                modified_at: None,
                plugin_actions: None,
                plugin_badge: None,
                plugin_keyword: None,
            }
        }
    }

    #[async_trait]
    impl SearchProvider for MockProvider {
        fn source_id(&self) -> &str {
            &self.id
        }
        fn source_label(&self) -> &str {
            &self.label
        }
        fn source_type(&self) -> SourceType {
            self.source.clone()
        }
        async fn search(&self, _ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError> {
            if self.should_error {
                return Err(SearchError::IndexError("mock error".to_string()));
            }
            Ok(self.results.clone())
        }
    }

    fn make_ctx(query: &str) -> SearchContext {
        SearchContext {
            query: query.to_string(),
            limit: 10,
            fuzzy: false,
            sources: None,
        }
    }

    #[tokio::test]
    async fn test_empty_aggregator_returns_empty() {
        let agg = SearchAggregator::new();
        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_single_provider_normalization() {
        let mut agg = SearchAggregator::new();
        agg.register(Box::new(MockProvider::new(
            "bookmarks",
            vec![
                MockProvider::make_result("1", 10.0, 0.0),
                MockProvider::make_result("2", 5.0, 0.0),
                MockProvider::make_result("3", 0.0, 0.0),
            ],
        )));

        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert_eq!(results.len(), 3);

        // Highest score should be first (score=10 → normalized=1.0 → global=0.7)
        assert_eq!(results[0].id, "1");
        assert!((results[0].score - 0.7).abs() < 0.001);

        // Middle (score=5 → normalized=0.5 → global=0.35)
        assert_eq!(results[1].id, "2");
        assert!((results[1].score - 0.35).abs() < 0.001);

        // Lowest (score=0 → normalized=0.0 → global=0.0)
        assert_eq!(results[2].id, "3");
        assert!((results[2].score - 0.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_multi_provider_merge() {
        let mut agg = SearchAggregator::new();

        // Provider A with high scores
        agg.register(Box::new(MockProvider::new(
            "a",
            vec![MockProvider::make_result("a1", 100.0, 0.0)],
        )));

        // Provider B with high scores
        agg.register(Box::new(MockProvider::new(
            "b",
            vec![MockProvider::make_result("b1", 50.0, 0.0)],
        )));

        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert_eq!(results.len(), 2);

        // Both single results normalize to 1.0, so both get global score 0.7
        assert!((results[0].score - 0.7).abs() < 0.001);
        assert!((results[1].score - 0.7).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_source_filtering() {
        let mut agg = SearchAggregator::new();
        agg.register(Box::new(MockProvider::new(
            "bookmarks",
            vec![MockProvider::make_result("b1", 10.0, 0.0)],
        )));
        agg.register(Box::new(MockProvider::new(
            "files",
            vec![MockProvider::make_result("f1", 10.0, 0.0)],
        )));

        // Filter to only bookmarks
        let ctx = SearchContext {
            query: "test".to_string(),
            limit: 10,
            fuzzy: false,
            sources: Some(vec!["bookmarks".to_string()]),
        };

        let results = agg.search(&ctx).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "b1");
    }

    #[tokio::test]
    async fn test_limit_truncation() {
        let mut agg = SearchAggregator::new();
        agg.register(Box::new(MockProvider::new(
            "test",
            (0..20)
                .map(|i| MockProvider::make_result(&i.to_string(), i as f64, 0.0))
                .collect(),
        )));

        let ctx = SearchContext {
            query: "test".to_string(),
            limit: 5,
            fuzzy: false,
            sources: None,
        };

        let results = agg.search(&ctx).await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_provider_error_does_not_fail_search() {
        let mut agg = SearchAggregator::new();
        agg.register(Box::new(MockProvider::with_error("broken")));
        agg.register(Box::new(MockProvider::new(
            "working",
            vec![MockProvider::make_result("w1", 10.0, 0.0)],
        )));

        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "w1");
    }

    #[tokio::test]
    async fn test_frecency_affects_ranking() {
        let mut agg = SearchAggregator::new();
        agg.register(Box::new(MockProvider::new(
            "test",
            vec![
                MockProvider::make_result("low_score_high_frec", 1.0, 10.0),
                MockProvider::make_result("high_score_low_frec", 10.0, 0.0),
            ],
        )));

        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert_eq!(results.len(), 2);

        // high_score_low_frec: norm_score=1.0, norm_frec=0.0 → 0.7
        // low_score_high_frec: norm_score=0.0, norm_frec=1.0 → 0.3
        assert_eq!(results[0].id, "high_score_low_frec");
        assert!((results[0].score - 0.7).abs() < 0.001);

        assert_eq!(results[1].id, "low_score_high_frec");
        assert!((results[1].score - 0.3).abs() < 0.001);
    }

    /// Integration test: Real BookmarkSearchProvider + FileSearchProvider → SearchAggregator
    #[tokio::test]
    async fn test_integration_mixed_results() {
        use super::super::bookmark_provider::BookmarkSearchProvider;
        use super::super::file_provider::FileSearchProvider;
        use super::super::TantivySearchEngine;
        use std::sync::Arc;

        let tmp = tempfile::TempDir::new().unwrap();
        let engine = Arc::new(TantivySearchEngine::new(tmp.path().to_path_buf()).unwrap());

        // Index a bookmark and a file that both match "rust"
        engine
            .index_bookmark(
                1, "Rust Language", "https://rust-lang.org",
                Some("Official Rust website"), Some("rust"),
                None, 1000, 1000,
            )
            .unwrap();

        engine
            .index_file(1, "/docs/rust_guide.pdf", "rust_guide.pdf", Some("pdf"), 2048, 1700000000, 1)
            .unwrap();

        let mut agg = SearchAggregator::new();
        agg.register(Box::new(BookmarkSearchProvider::new(engine.clone())));
        agg.register(Box::new(FileSearchProvider::new(engine)));

        let ctx = SearchContext {
            query: "rust".to_string(),
            limit: 10,
            fuzzy: true,
            sources: None,
        };

        let results = agg.search(&ctx).await.unwrap();

        // Both providers should return results
        assert!(results.len() >= 2);

        // Verify we have results from both sources
        let has_bookmark = results.iter().any(|r| r.source_id == "bookmarks");
        let has_file = results.iter().any(|r| r.source_id == "files");
        assert!(has_bookmark, "Should have bookmark results");
        assert!(has_file, "Should have file results");

        // Verify results are sorted by score descending
        for w in results.windows(2) {
            assert!(w[0].score >= w[1].score, "Results should be sorted by score desc");
        }
    }

    /// Integration test: Settings-based source filtering
    #[tokio::test]
    async fn test_integration_settings_filtering() {
        use super::super::bookmark_provider::BookmarkSearchProvider;
        use super::super::file_provider::FileSearchProvider;
        use super::super::TantivySearchEngine;
        use std::sync::Arc;

        let tmp = tempfile::TempDir::new().unwrap();
        let engine = Arc::new(TantivySearchEngine::new(tmp.path().to_path_buf()).unwrap());

        engine
            .index_bookmark(1, "Rust", "https://rust-lang.org", None, None, None, 1000, 1000)
            .unwrap();
        engine
            .index_file(1, "/docs/rust.pdf", "rust.pdf", Some("pdf"), 1024, 1700000000, 1)
            .unwrap();

        let mut agg = SearchAggregator::new();
        agg.register(Box::new(BookmarkSearchProvider::new(engine.clone())));
        agg.register(Box::new(FileSearchProvider::new(engine)));

        // Only bookmarks enabled (simulating show_files = false)
        let ctx = SearchContext {
            query: "rust".to_string(),
            limit: 10,
            fuzzy: true,
            sources: Some(vec!["bookmarks".to_string()]),
        };

        let results = agg.search(&ctx).await.unwrap();
        assert!(results.iter().all(|r| r.source_id == "bookmarks"));

        // Only files enabled (simulating show_bookmarks = false)
        let ctx = SearchContext {
            query: "rust".to_string(),
            limit: 10,
            fuzzy: true,
            sources: Some(vec!["files".to_string()]),
        };

        let results = agg.search(&ctx).await.unwrap();
        assert!(results.iter().all(|r| r.source_id == "files"));
    }
}

