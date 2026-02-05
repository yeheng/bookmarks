//! Search engine abstraction layer using Tantivy.
//!
//! This module provides a clean abstraction over full-text search operations,
//! replacing SQLite FTS5 with Tantivy for better performance and features.

mod engine;
mod frecency_worker;
mod schema;
mod tantivy_engine;

pub use engine::{IndexStats, SearchEngine};
pub use tantivy_engine::TantivySearchEngine;
