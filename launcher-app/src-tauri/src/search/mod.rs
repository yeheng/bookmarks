//! Search engine abstraction layer using Tantivy.
//!
//! This module provides full-text search operations using Tantivy,
//! replacing SQLite FTS5 with better performance and CJK support.
//!
//! Also provides the unified search provider system:
//! - `SearchProvider` trait for pluggable search sources
//! - `SearchAggregator` for coordinating multi-source search

pub mod aggregator;
pub mod bookmark_provider;
mod engine;
pub mod file_provider;
pub mod frecency;
pub mod plugin_provider;
pub mod provider;
pub mod query_parser;
mod schema;
mod tantivy_engine;

pub use aggregator::SearchAggregator;
pub use engine::IndexStats;
pub use frecency::FrecencyTracker;
pub use provider::SearchContext;
pub use tantivy_engine::TantivySearchEngine;
