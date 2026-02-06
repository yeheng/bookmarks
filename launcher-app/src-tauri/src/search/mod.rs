//! Search engine abstraction layer using Tantivy.
//!
//! This module provides full-text search operations using Tantivy,
//! replacing SQLite FTS5 with better performance and CJK support.

mod engine;
mod schema;
mod tantivy_engine;

pub use engine::IndexStats;
pub use tantivy_engine::TantivySearchEngine;
