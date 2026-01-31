//! SearchEngine trait definition and related types.

use crate::models::bookmark::BookmarkSearchResult;
use crate::models::file::FileSearchResult;
use thiserror::Error;

/// Errors that can occur during search operations
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Query parse error: {0}")]
    QueryError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

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

/// Search engine abstraction for full-text search operations.
///
/// This trait defines the interface for search operations, allowing
/// different implementations (e.g., Tantivy, in-memory mock for testing).
pub trait SearchEngine: Send + Sync {
    /// Search bookmarks with query string.
    ///
    /// Returns bookmarks matching the query, ranked by combined BM25 + frecency score.
    /// If query is empty, returns recently accessed bookmarks.
    fn search_bookmarks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<BookmarkSearchResult>, SearchError>;

    /// Search files with query string.
    ///
    /// Returns files matching the query with fuzzy matching support.
    /// If query is empty, returns recently accessed files.
    fn search_files(&self, query: &str, limit: usize) -> Result<Vec<FileSearchResult>, SearchError>;

    /// Index a single bookmark (add or update).
    fn index_bookmark(
        &self,
        id: i64,
        title: &str,
        url: &str,
        description: Option<&str>,
        tags: Option<&str>,
        last_accessed: Option<i64>,
        created_at: i64,
        updated_at: i64,
    ) -> Result<(), SearchError>;

    /// Index a single file (add or update).
    fn index_file(
        &self,
        id: i64,
        path: &str,
        name: &str,
        extension: Option<&str>,
        size: i64,
        modified_at: i64,
        directory_id: i64,
    ) -> Result<(), SearchError>;

    /// Delete a bookmark from the index.
    fn delete_bookmark(&self, id: i64) -> Result<(), SearchError>;

    /// Delete a file from the index.
    fn delete_file(&self, id: i64) -> Result<(), SearchError>;

    /// Delete all files for a directory.
    fn delete_directory_files(&self, directory_id: i64) -> Result<(), SearchError>;

    /// Rebuild the entire bookmark index from the database.
    ///
    /// Returns the number of bookmarks indexed.
    fn rebuild_bookmark_index(&self) -> Result<usize, SearchError>;

    /// Rebuild the entire file index from the database.
    ///
    /// Returns the number of files indexed.
    fn rebuild_file_index(&self) -> Result<usize, SearchError>;

    /// Get index statistics.
    fn get_stats(&self) -> Result<IndexStats, SearchError>;

    /// Commit any pending changes to the index.
    fn commit(&self) -> Result<(), SearchError>;
}
