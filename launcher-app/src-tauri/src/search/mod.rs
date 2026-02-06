//! Search engine abstraction layer using Tantivy.
//!
//! This module provides full-text search operations using Tantivy,
//! replacing SQLite FTS5 with better performance and CJK support.

mod engine;
mod frecency_worker;
mod schema;
mod tantivy_engine;

pub use engine::{IndexStats, SearchError};
pub use tantivy_engine::TantivySearchEngine;
