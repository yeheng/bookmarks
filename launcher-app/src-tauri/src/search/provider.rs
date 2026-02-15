//! Search provider trait and supporting types.
//!
//! Defines the `SearchProvider` abstraction that all search sources implement,
//! along with `SearchContext` (query parameters) and `ProviderResult` (unified result).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::engine::SearchError;
use super::query_parser::StructuredQuery;

/// The type of source a result comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Bookmark,
    File,
    Plugin,
}

/// Query context passed to every provider.
#[derive(Debug, Clone)]
pub struct SearchContext {
    /// The raw query string.
    pub query: String,
    /// Parsed structured query.
    pub structured_query: StructuredQuery,
    /// Maximum results per provider.
    pub limit: usize,
    /// Whether to prefer fuzzy matching.
    pub fuzzy: bool,
    /// Optional list of source IDs to include. `None` means all sources.
    pub sources: Option<Vec<String>>,
}

/// A single result returned by a provider before aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResult {
    /// Unique ID of the item within its source.
    pub id: String,
    /// Primary display text.
    pub title: String,
    /// Secondary display text (URL for bookmarks, path for files, etc.).
    pub subtitle: String,
    /// Which source produced this result.
    pub source_type: SourceType,
    /// Source-specific identifier (e.g. "bookmarks", "files", "plugin:github-search").
    pub source_id: String,
    /// Raw relevance score from the provider (not yet normalized).
    pub score: f64,
    /// Frecency score from the provider.
    pub frecency_score: f64,
    /// Optional icon identifier or path.
    pub icon: Option<String>,
    /// Optional URL (bookmarks).
    pub url: Option<String>,
    /// Optional file path.
    pub path: Option<String>,
    /// Optional favicon URL.
    pub favicon_url: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Optional file extension.
    pub extension: Option<String>,
    /// Optional file size.
    pub size: Option<i64>,
    /// Optional file modified_at.
    pub modified_at: Option<i64>,
    /// Plugin-specific: actions available.
    pub plugin_actions: Option<Vec<serde_json::Value>>,
    /// Plugin-specific: badge text.
    pub plugin_badge: Option<String>,
    /// Plugin-specific: keyword that triggered this result.
    pub plugin_keyword: Option<String>,
}

/// Trait that every search source must implement.
///
/// Providers are registered with the `SearchAggregator` and queried in parallel.
/// Each provider is responsible for searching its own data source and returning
/// `ProviderResult`s with raw scores — the aggregator handles normalization
/// and cross-source ranking.
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Unique identifier for this source (e.g. "bookmarks", "files", "plugin:my-plugin").
    fn source_id(&self) -> &str;

    /// Human-readable label for UI grouping.
    fn source_label(&self) -> &str;

    /// The type of source.
    fn source_type(&self) -> SourceType;

    /// Execute a search against this source.
    async fn search(&self, ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError>;
}
