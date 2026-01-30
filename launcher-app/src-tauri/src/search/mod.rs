//! Search engine abstraction layer using Tantivy.
//!
//! This module provides a clean abstraction over full-text search operations,
//! replacing SQLite FTS5 with Tantivy for better performance and features.

mod engine;
mod schema;
mod tantivy_engine;

pub use engine::{IndexStats, SearchEngine, SearchError};
pub use tantivy_engine::TantivySearchEngine;
