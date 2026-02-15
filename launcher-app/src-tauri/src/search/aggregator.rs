//! Search aggregator — coordinates parallel search across all providers.
//!
//! The `SearchAggregator` holds registered `SearchProvider`s and orchestrates
//! unified search: dispatching queries in parallel, normalizing scores per-source,
//! computing a global ranking, and returning a merged result list.

use super::engine::SearchError;
use super::provider::{ProviderResult, SearchContext, SearchProvider};

/// Weight for normalized relevance score in global ranking.
/// Higher values prioritize text match relevance over user behavior.
const SCORE_WEIGHT: f64 = 0.7;

/// Weight for frecency (user behavior) in global ranking.
/// Higher values prioritize frequently accessed items.
const FRECENCY_WEIGHT: f64 = 0.3;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use super::frecency::FrecencyTracker;

/// Orchestrates multi-source unified search.
pub struct SearchAggregator {
    providers: RwLock<HashMap<String, Arc<dyn SearchProvider>>>,
    frecency_tracker: Arc<FrecencyTracker>,
}

impl SearchAggregator {
    /// Create an empty aggregator.
    pub fn new(frecency_tracker: Arc<FrecencyTracker>) -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            frecency_tracker,
        }
    }

    /// Register a search provider.
    pub fn register(&self, id: String, provider: Box<dyn SearchProvider>) {
        if let Ok(mut providers) = self.providers.write() {
            // Convert Box to Arc for shared ownership
            providers.insert(id, Arc::from(provider));
        }
    }

    /// Unregister a search provider.
    pub fn unregister(&self, id: &str) {
        if let Ok(mut providers) = self.providers.write() {
            providers.remove(id);
        }
    }

    /// Execute a unified search across all (or filtered) providers.
    ///
    /// Steps:
    /// 1. Filter providers by `ctx.sources` if specified.
    /// 2. Query all enabled providers in parallel.
    /// 3. Apply min-max score normalization per source.
    /// 4. Compute `global_score = normalized_score * SCORE_WEIGHT + normalized_frecency * FRECENCY_WEIGHT`.
    /// 5. Sort by global_score descending, truncate to `ctx.limit`.
    pub async fn search(&self, ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError> {
        // Handle empty query - return early
        if ctx.query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // 1. Identify active providers
        let active_providers: Vec<Arc<dyn SearchProvider>> = {
            let lock = self.providers.read().map_err(|_| SearchError::IndexError("Failed to acquire lock".into()))?;
            
            if let Some(scope) = &ctx.structured_query.scope {
                // Scope routing: If scope is present, only select that provider
                // Look for ID == scope OR ID == "plugin:<scope>"
                let mut matches = Vec::new();
                
                // Direct match (e.g. "bookmarks" if scope is "bookmarks")
                if let Some(p) = lock.get(scope) {
                    matches.push(p.clone());
                } else {
                    // Plugin match (e.g. "plugin:gh" if scope is "gh")
                    // We need to scan keys for "plugin:<scope>" or similar?
                    // Actually the design doc says:
                    // "Query gh: match provider with ID 'gh' or 'plugin:gh'"
                    
                    // Optimisation: Construct the key "plugin:<scope>"
                    let plugin_key = format!("plugin:{}", scope);
                    if let Some(p) = lock.get(&plugin_key) {
                        matches.push(p.clone());
                    }
                    
                    // Fallback: Check if any provider explicitly handles this scope?
                    // Design doc mentions "If scope miss behavior... return empty".
                }
                
                matches
            } else {
                // Broadcast to filtered sources or all
                lock.iter()
                    .filter(|(id, _)| {
                        ctx.sources
                            .as_ref()
                            .map(|sources| sources.contains(id))
                            .unwrap_or(true)
                    })
                    .map(|(_, p)| p.clone())
                    .collect()
            }
        };

        if active_providers.is_empty() {
            // Should we return error if scope was found but no provider?
            // "If query.scope == Some("gh") but no provider matches... Return empty results"
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
                r.score = norm_score * SCORE_WEIGHT + norm_frec * FRECENCY_WEIGHT;
            }

            all_results.extend(results);
        }

        // 4. Sort by global score descending
        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 4.5 Deduplicate: keep highest-scoring result for each (source_id, id) pair
        let mut seen = HashSet::new();
        let mut deduplicated: Vec<ProviderResult> = Vec::new();
        for result in all_results {
            let key = format!("{}:{}", result.source_id, result.id);
            if !seen.contains(&key) {
                seen.insert(key);
                deduplicated.push(result);
            }
        }
        all_results = deduplicated;

        // 5. Truncate to limit
        all_results.truncate(ctx.limit);

        Ok(all_results)
    }

    /// Record usage for an item to boost its ranking.
    pub fn record_usage(&self, source_id: &str, item_id: &str) -> Result<(), SearchError> {
        self.frecency_tracker
            .record_usage(source_id, item_id)
            .map_err(|e| SearchError::IndexError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::provider::SourceType;
    use super::super::query_parser::StructuredQuery;
    use crate::db::Database;
    use crate::services::data_service::DataService;
    use crate::search::TantivySearchEngine;
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

    fn setup_tracker() -> Arc<FrecencyTracker> {
        let db = Database::new_in_memory().unwrap();
        db.initialize().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(TantivySearchEngine::new(temp_dir.path().to_path_buf()).unwrap());
        let ds = Arc::new(DataService::new(db, engine));
        Arc::new(FrecencyTracker::new(ds))
    }

    fn make_ctx(query: &str) -> SearchContext {
        SearchContext {
            query: query.to_string(),
            structured_query: StructuredQuery::parse(query),
            limit: 10,
            fuzzy: false,
            sources: None,
        }
    }

    #[tokio::test]
    async fn test_empty_aggregator_returns_empty() {
        let agg = SearchAggregator::new(setup_tracker());
        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_single_provider_normalization() {
        let agg = SearchAggregator::new(setup_tracker());
        agg.register("bookmarks".into(), Box::new(MockProvider::new(
            "bookmarks",
            vec![
                MockProvider::make_result("1", 10.0, 0.0),
                MockProvider::make_result("2", 5.0, 0.0),
                MockProvider::make_result("3", 0.0, 0.0),
            ],
        )));

        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert_eq!(results.len(), 3);

        // Highest score should be first (score=10 → normalized=1.0 → global=SCORE_WEIGHT)
        assert_eq!(results[0].id, "1");
        assert!((results[0].score - SCORE_WEIGHT).abs() < 0.001);

        // Middle (score=5 → normalized=0.5 → global=0.35)
        assert_eq!(results[1].id, "2");
        assert!((results[1].score - SCORE_WEIGHT * 0.5).abs() < 0.001);

        // Lowest (score=0 → normalized=0.0 → global=0.0)
        assert_eq!(results[2].id, "3");
        assert!((results[2].score - 0.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_multi_provider_merge() {
        let agg = SearchAggregator::new(setup_tracker());

        // Provider A with high scores
        agg.register("a".into(), Box::new(MockProvider::new(
            "a",
            vec![MockProvider::make_result("a1", 100.0, 0.0)],
        )));

        // Provider B with high scores
        agg.register("b".into(), Box::new(MockProvider::new(
            "b",
            vec![MockProvider::make_result("b1", 50.0, 0.0)],
        )));

        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert_eq!(results.len(), 2);

        // Both single results normalize to 1.0, so both get global score SCORE_WEIGHT
        assert!((results[0].score - SCORE_WEIGHT).abs() < 0.001);
        assert!((results[1].score - SCORE_WEIGHT).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_source_filtering() {
        let agg = SearchAggregator::new(setup_tracker());
        agg.register("bookmarks".into(), Box::new(MockProvider::new(
            "bookmarks",
            vec![MockProvider::make_result("b1", 10.0, 0.0)],
        )));
        agg.register("files".into(), Box::new(MockProvider::new(
            "files",
            vec![MockProvider::make_result("f1", 10.0, 0.0)],
        )));

        // Filter to only bookmarks
        let ctx = SearchContext {
            query: "test".to_string(),
            structured_query: StructuredQuery::parse("test"),
            limit: 10,
            fuzzy: false,
            sources: Some(vec!["bookmarks".to_string()]),
        };

        let results = agg.search(&ctx).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "b1");
    }
    
    #[tokio::test]
    async fn test_scoped_search() {
        let agg = SearchAggregator::new(setup_tracker());
        agg.register("plugin:gh".into(), Box::new(MockProvider::new(
            "plugin:gh", 
            vec![MockProvider::make_result("gh1", 10.0, 0.0)]
        )));
        agg.register("bookmarks".into(), Box::new(MockProvider::new(
            "bookmarks",
            vec![MockProvider::make_result("b1", 10.0, 0.0)]
        )));

        // Scope "gh" should map to "plugin:gh"
        let ctx = SearchContext {
            query: "gh: test".to_string(),
            structured_query: StructuredQuery::parse("gh: test"),
            limit: 10,
            fuzzy: false,
            sources: None, // should ignore this if scope is present? Or strict intersection? Logic above ignores `sources` if scope is matched.
        };

        let results = agg.search(&ctx).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "gh1");
    }

    #[tokio::test]
    async fn test_limit_truncation() {
        let agg = SearchAggregator::new(setup_tracker());
        agg.register("test".into(), Box::new(MockProvider::new(
            "test",
            (0..20)
                .map(|i| MockProvider::make_result(&i.to_string(), i as f64, 0.0))
                .collect(),
        )));

        let ctx = SearchContext {
            query: "test".to_string(),
            structured_query: StructuredQuery::parse("test"),
            limit: 5,
            fuzzy: false,
            sources: None,
        };

        let results = agg.search(&ctx).await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_provider_error_does_not_fail_search() {
        let agg = SearchAggregator::new(setup_tracker());
        agg.register("broken".into(), Box::new(MockProvider::with_error("broken")));
        agg.register("working".into(), Box::new(MockProvider::new(
            "working",
            vec![MockProvider::make_result("w1", 10.0, 0.0)],
        )));

        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "w1");
    }

    #[tokio::test]
    async fn test_frecency_affects_ranking() {
        let agg = SearchAggregator::new(setup_tracker());
        agg.register("test".into(), Box::new(MockProvider::new(
            "test",
            vec![
                MockProvider::make_result("low_score_high_frec", 1.0, 10.0),
                MockProvider::make_result("high_score_low_frec", 10.0, 0.0),
            ],
        )));

        let results = agg.search(&make_ctx("test")).await.unwrap();
        assert_eq!(results.len(), 2);

        // high_score_low_frec: norm_score=1.0, norm_frec=0.0 → SCORE_WEIGHT
        // low_score_high_frec: norm_score=0.0, norm_frec=1.0 → FRECENCY_WEIGHT
        assert_eq!(results[0].id, "high_score_low_frec");
        assert!((results[0].score - SCORE_WEIGHT).abs() < 0.001);

        assert_eq!(results[1].id, "low_score_high_frec");
        assert!((results[1].score - FRECENCY_WEIGHT).abs() < 0.001);
    }
}
