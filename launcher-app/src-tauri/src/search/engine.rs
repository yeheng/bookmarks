//! Search engine types and error definitions.
//!
//! Note: The SearchEngine trait has been removed in favor of direct
//! TantivySearchEngine usage. This simplifies the codebase since we
//! only have one implementation.

use thiserror::Error;

/// Errors that can occur during search operations
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Query parse error: {0}")]
    QueryError(String),

    #[error("Lock error: {0}")]
    LockError(&'static str),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Statistics about search indexes
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexStats {
    pub bookmark_count: usize,
    pub file_count: usize,
    pub bookmark_index_size_bytes: u64,
    pub file_index_size_bytes: u64,
}
