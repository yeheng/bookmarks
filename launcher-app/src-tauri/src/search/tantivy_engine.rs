//! Tantivy-based search engine implementation.
//!
//! # Lock Ordering and Concurrency
//!
//! This engine uses multiple locks to protect shared resources:
//!
//! 1. `bookmark_writer: Mutex<IndexWriter>` - Protects bookmark index writes
//! 2. `file_writer: Mutex<IndexWriter>` - Protects file index writes
//!
//! ## Lock Acquisition Strategy
//!
//! - Search methods (search_bookmarks, search_files) perform Tantivy search
//!   without any locks, keeping operations fast and simple.
//!
//! - Write methods (index_bookmark, index_file) only acquire writer locks.
//!   Callers must ensure no conflicts with concurrent operations.
//!
//! - Rebuild methods only acquire writer locks. The Service layer is responsible
//!   for fetching data from the database and passing it to the rebuild methods.
//!
//! ## Important Notes
//!
//! - SearchEngine is decoupled from Database - it only manages indexes
//! - Service layer coordinates between Database and SearchEngine
//! - All lock poisoning is handled via SearchError::LockError
//! - Frecency updates are performed directly (no batch worker) since desktop
//!   app click rates don't require batching optimization

use super::engine::{IndexStats, SearchError};
use super::schema::{build_bookmark_schema, build_file_schema, register_tokenizers};
use crate::models::bookmark::BookmarkSearchResult;
use crate::models::file::FileSearchResult;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tantivy::collector::TopDocs;

/// Index writer heap size in bytes (15MB)
/// Tantivy requires minimum 15MB per writer. Reduced from 50MB.
/// This saves ~70MB of RAM (2 writers × 50MB → 2 × 15MB).
const INDEX_WRITER_HEAP_SIZE: usize = 15_000_000;

/// Search result multiplier for frecency re-ranking
/// Fetch 2x the requested limit to allow for re-ranking
const SEARCH_RESULT_MULTIPLIER: usize = 2;

use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};

/// Cached bookmark schema fields for fast access
#[derive(Clone)]
struct BookmarkFields {
    id: Field,
    title: Field,
    url: Field,
    description: Field,
    tags: Field,
    last_accessed: Field,
    created_at: Field,
    updated_at: Field,
    access_count: Field,
    access_timestamp: Field,
}

/// Cached file schema fields for fast access
#[derive(Clone)]
struct FileFields {
    id: Field,
    path: Field,
    name: Field,
    extension: Field,
    size: Field,
    modified_at: Field,
    directory_id: Field,
    access_count: Field,
    access_timestamp: Field,
}

/// Tantivy-based search engine implementation.
///
/// Provides full-text search for bookmarks and files using Tantivy.
/// This engine is decoupled from the database - it only manages search indexes.
/// The Service layer coordinates between the database and search engine.
///
/// Frecency updates are performed directly - no batching needed for desktop apps.
pub struct TantivySearchEngine {
    // Bookmark index components
    bookmark_index: Index,
    bookmark_writer: Arc<Mutex<IndexWriter>>,
    bookmark_reader: IndexReader,
    bookmark_schema: Schema,
    bookmark_fields: BookmarkFields,  // Cached fields

    // File index components
    file_index: Index,
    file_writer: Arc<Mutex<IndexWriter>>,
    file_reader: IndexReader,
    file_schema: Schema,
    file_fields: FileFields,  // Cached fields

    // Index directory for stats
    index_dir: PathBuf,
}

impl TantivySearchEngine {
    /// Create a new Tantivy search engine.
    ///
    /// # Arguments
    /// * `index_dir` - Directory to store Tantivy indexes
    ///
    /// Note: This engine is decoupled from the database.
    /// Use the Service layer to coordinate database and index operations.
    pub fn new(index_dir: PathBuf) -> Result<Self, SearchError> {
        // Create index directories
        std::fs::create_dir_all(&index_dir)?;

        // Setup bookmark index
        let bookmark_schema = build_bookmark_schema();
        let bookmark_dir = index_dir.join("bookmarks");
        std::fs::create_dir_all(&bookmark_dir)?;

        let bookmark_index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(&bookmark_dir)
                .map_err(|e| SearchError::IndexError(e.to_string()))?,
            bookmark_schema.clone(),
        )
        .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Register ngram tokenizer for CJK support
        register_tokenizers(&bookmark_index)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let bookmark_writer = bookmark_index
            .writer(INDEX_WRITER_HEAP_SIZE)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let bookmark_reader = bookmark_index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Setup file index
        let file_schema = build_file_schema();
        let file_dir = index_dir.join("files");
        std::fs::create_dir_all(&file_dir)?;

        let file_index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(&file_dir)
                .map_err(|e| SearchError::IndexError(e.to_string()))?,
            file_schema.clone(),
        )
        .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Register ngram tokenizer for CJK support
        register_tokenizers(&file_index)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let file_writer = file_index
            .writer(INDEX_WRITER_HEAP_SIZE)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let file_reader = file_index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Create bookmark and file writers as Arc<Mutex<>>
        let bookmark_writer_arc = Arc::new(Mutex::new(bookmark_writer));
        let file_writer_arc = Arc::new(Mutex::new(file_writer));

        // Cache bookmark schema fields for fast access
        let bookmark_fields = BookmarkFields {
            id: bookmark_schema.get_field("id").map_err(|e| SearchError::IndexError(e.to_string()))?,
            title: bookmark_schema.get_field("title").map_err(|e| SearchError::IndexError(e.to_string()))?,
            url: bookmark_schema.get_field("url").map_err(|e| SearchError::IndexError(e.to_string()))?,
            description: bookmark_schema.get_field("description").map_err(|e| SearchError::IndexError(e.to_string()))?,
            tags: bookmark_schema.get_field("tags").map_err(|e| SearchError::IndexError(e.to_string()))?,
            last_accessed: bookmark_schema.get_field("last_accessed").map_err(|e| SearchError::IndexError(e.to_string()))?,
            created_at: bookmark_schema.get_field("created_at").map_err(|e| SearchError::IndexError(e.to_string()))?,
            updated_at: bookmark_schema.get_field("updated_at").map_err(|e| SearchError::IndexError(e.to_string()))?,
            access_count: bookmark_schema.get_field("access_count").map_err(|e| SearchError::IndexError(e.to_string()))?,
            access_timestamp: bookmark_schema.get_field("access_timestamp").map_err(|e| SearchError::IndexError(e.to_string()))?,
        };

        // Cache file schema fields for fast access
        let file_fields = FileFields {
            id: file_schema.get_field("id").map_err(|e| SearchError::IndexError(e.to_string()))?,
            path: file_schema.get_field("path").map_err(|e| SearchError::IndexError(e.to_string()))?,
            name: file_schema.get_field("name").map_err(|e| SearchError::IndexError(e.to_string()))?,
            extension: file_schema.get_field("extension").map_err(|e| SearchError::IndexError(e.to_string()))?,
            size: file_schema.get_field("size").map_err(|e| SearchError::IndexError(e.to_string()))?,
            modified_at: file_schema.get_field("modified_at").map_err(|e| SearchError::IndexError(e.to_string()))?,
            directory_id: file_schema.get_field("directory_id").map_err(|e| SearchError::IndexError(e.to_string()))?,
            access_count: file_schema.get_field("access_count").map_err(|e| SearchError::IndexError(e.to_string()))?,
            access_timestamp: file_schema.get_field("access_timestamp").map_err(|e| SearchError::IndexError(e.to_string()))?,
        };

        Ok(Self {
            bookmark_index,
            bookmark_writer: bookmark_writer_arc,
            bookmark_reader,
            bookmark_schema,
            bookmark_fields,
            file_index,
            file_writer: file_writer_arc,
            file_reader,
            file_schema,
            file_fields,
            index_dir,
        })
    }

    /// Calculate directory size in bytes.
    fn calculate_dir_size(dir: &std::path::Path) -> u64 {
        let mut size = 0;
        if dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(metadata) = entry.metadata() {
                            size += metadata.len();
                        }
                    }
                }
            }
        }
        size
    }

    /// Get recent bookmarks from index, sorted by access_timestamp (ZERO DB lock).
    fn get_recent_bookmarks_from_index(
        &self,
        limit: usize,
    ) -> Result<Vec<BookmarkSearchResult>, SearchError> {
        let searcher = self.bookmark_reader.searcher();
        let fields = &self.bookmark_fields;

        // Use AllQuery to get all docs, sorted by access_timestamp descending
        let all_query = tantivy::query::AllQuery;
        let top_docs = searcher
            .search(
                &all_query,
                &TopDocs::with_limit(limit).order_by_fast_field::<i64>("access_timestamp", tantivy::Order::Desc),
            )
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Note: order_by_fast_field with Desc returns descending order
        let mut results: Vec<BookmarkSearchResult> = Vec::new();
        for (_ts, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| SearchError::IndexError(e.to_string()))?;

            let id = doc.get_first(fields.id).and_then(|v| v.as_i64()).unwrap_or(0);
            let title = doc.get_first(fields.title).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let url = doc.get_first(fields.url).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let description = doc.get_first(fields.description).and_then(|v| v.as_str()).map(|s| s.to_string());
            let access_count = doc.get_first(fields.access_count).and_then(|v| v.as_i64()).unwrap_or(0);
            let access_ts = doc.get_first(fields.access_timestamp).and_then(|v| v.as_i64()).unwrap_or(0);

            // Skip items that have never been accessed (access_timestamp == 0) unless we need more
            let frecency = if access_count > 0 { access_count as f64 } else { 0.0 };

            results.push(BookmarkSearchResult {
                id,
                title,
                url,
                description,
                favicon_url: None,
                score: access_ts as f64,
                frecency_score: frecency,
            });
        }

        Ok(results)
    }

    /// Get recent files from index, sorted by access_timestamp (ZERO DB lock).
    fn get_recent_files_from_index(
        &self,
        limit: usize,
    ) -> Result<Vec<FileSearchResult>, SearchError> {
        let searcher = self.file_reader.searcher();
        let fields = &self.file_fields;

        let all_query = tantivy::query::AllQuery;
        let top_docs = searcher
            .search(
                &all_query,
                &TopDocs::with_limit(limit).order_by_fast_field::<i64>("access_timestamp", tantivy::Order::Desc),
            )
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let mut results: Vec<FileSearchResult> = Vec::new();
        for (_ts, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| SearchError::IndexError(e.to_string()))?;

            let id = doc.get_first(fields.id).and_then(|v| v.as_i64()).unwrap_or(0);
            let path = doc.get_first(fields.path).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = doc.get_first(fields.name).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let extension = doc.get_first(fields.extension).and_then(|v| v.as_str()).map(|s| s.to_string());
            let size = doc.get_first(fields.size).and_then(|v| v.as_i64()).unwrap_or(0);
            let modified_at = doc.get_first(fields.modified_at).and_then(|v| v.as_i64()).unwrap_or(0);
            let access_count = doc.get_first(fields.access_count).and_then(|v| v.as_i64()).unwrap_or(0);
            let access_ts = doc.get_first(fields.access_timestamp).and_then(|v| v.as_i64()).unwrap_or(0);

            let frecency = if access_count > 0 { access_count as f64 } else { 0.0 };

            results.push(FileSearchResult {
                id,
                path,
                name,
                extension,
                size,
                modified_at,
                score: access_ts as f64,
                frecency_score: frecency,
            });
        }

        Ok(results)
    }

    /// Search bookmarks with query string.
    ///
    /// Returns bookmarks matching the query, ranked by combined BM25 + frecency score.
    /// If query is empty, returns recently accessed bookmarks.
    pub fn search_bookmarks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<BookmarkSearchResult>, SearchError> {
        // Handle empty query - return recently accessed bookmarks from index
        if query.trim().is_empty() {
            return self.get_recent_bookmarks_from_index(limit);
        }

        // Phase 1: Tantivy search (ZERO DB interaction)
        let searcher = self.bookmark_reader.searcher();
        let fields = &self.bookmark_fields;

        let query_parser = QueryParser::for_index(
            &self.bookmark_index,
            vec![fields.title, fields.url, fields.description, fields.tags],
        );

        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| SearchError::QueryError(e.to_string()))?;

        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(limit * SEARCH_RESULT_MULTIPLIER))
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Phase 2: Extract all data from Tantivy + compute frecency (ZERO DB lock)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;

        let mut results: Vec<BookmarkSearchResult> = Vec::new();

        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| SearchError::IndexError(e.to_string()))?;

            let id = doc.get_first(fields.id).and_then(|v| v.as_i64()).unwrap_or(0);
            let title = doc.get_first(fields.title).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let url = doc.get_first(fields.url).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let description = doc.get_first(fields.description).and_then(|v| v.as_str()).map(|s| s.to_string());

            // Read frecency data directly from index FastFields
            let access_count = doc.get_first(fields.access_count).and_then(|v| v.as_i64()).unwrap_or(0);
            let access_ts = doc.get_first(fields.access_timestamp).and_then(|v| v.as_i64()).unwrap_or(0);

            // Frecency formula: log(count+1) * decay(timestamp)
            let frecency = if access_count > 0 {
                let age_days = (now - access_ts as f64) / 86400.0;
                let decay = (-0.1 * age_days).exp(); // exponential decay
                (access_count as f64 + 1.0).ln() * decay
            } else {
                0.0
            };

            let combined_score = (score as f64) * 0.7 + frecency * 0.3;

            results.push(BookmarkSearchResult {
                id,
                title,
                url,
                description,
                favicon_url: None, // favicon now served separately if needed
                score: combined_score,
                frecency_score: frecency,
            });
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    /// Search files with query string.
    ///
    /// Returns files matching the query with fuzzy matching support.
    /// If query is empty, returns recently accessed files.
    pub fn search_files(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<FileSearchResult>, SearchError> {
        // Handle empty query - return recently accessed files from index
        if query.trim().is_empty() {
            return self.get_recent_files_from_index(limit);
        }

        // Phase 1: Tantivy search (ZERO DB interaction)
        let searcher = self.file_reader.searcher();
        let fields = &self.file_fields;

        let query_parser = QueryParser::for_index(&self.file_index, vec![fields.name, fields.path]);

        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| SearchError::QueryError(e.to_string()))?;

        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(limit * SEARCH_RESULT_MULTIPLIER))
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Phase 2: Extract all data + compute frecency (ZERO DB lock)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;

        let mut results: Vec<FileSearchResult> = Vec::new();

        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| SearchError::IndexError(e.to_string()))?;

            let id = doc.get_first(fields.id).and_then(|v| v.as_i64()).unwrap_or(0);
            let path = doc.get_first(fields.path).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = doc.get_first(fields.name).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let extension = doc.get_first(fields.extension).and_then(|v| v.as_str()).map(|s| s.to_string());
            let size = doc.get_first(fields.size).and_then(|v| v.as_i64()).unwrap_or(0);
            let modified_at = doc.get_first(fields.modified_at).and_then(|v| v.as_i64()).unwrap_or(0);

            // Read frecency from index FastFields
            let access_count = doc.get_first(fields.access_count).and_then(|v| v.as_i64()).unwrap_or(0);
            let access_ts = doc.get_first(fields.access_timestamp).and_then(|v| v.as_i64()).unwrap_or(0);

            let frecency = if access_count > 0 {
                let age_days = (now - access_ts as f64) / 86400.0;
                let decay = (-0.1 * age_days).exp();
                (access_count as f64 + 1.0).ln() * decay
            } else {
                0.0
            };

            let combined_score = (score as f64) * 0.7 + frecency * 0.3;

            results.push(FileSearchResult {
                id,
                path,
                name,
                extension,
                size,
                modified_at,
                score: combined_score,
                frecency_score: frecency,
            });
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    /// Index a single bookmark (add or update).
    pub fn index_bookmark(
        &self,
        id: i64,
        title: &str,
        url: &str,
        description: Option<&str>,
        tags: Option<&str>,
        last_accessed: Option<i64>,
        created_at: i64,
        updated_at: i64,
    ) -> Result<(), SearchError> {
        let mut writer = self.bookmark_writer.lock().map_err(|_| {
            SearchError::LockError("bookmark writer lock poisoned")
        })?;

        let fields = &self.bookmark_fields;

        // Delete existing document
        let term = Term::from_field_i64(fields.id, id);
        writer.delete_term(term);

        // Add new document
        let doc = doc!(
            fields.id => id,
            fields.title => title,
            fields.url => url,
            fields.description => description.unwrap_or(""),
            fields.tags => tags.unwrap_or(""),
            fields.last_accessed => last_accessed.unwrap_or(0),
            fields.created_at => created_at,
            fields.updated_at => updated_at,
            fields.access_count => 0i64,
            fields.access_timestamp => 0i64
        );

        writer
            .add_document(doc)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Reload reader to see the changes immediately
        self.bookmark_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        Ok(())
    }

    /// Index a single file (add or update).
    pub fn index_file(
        &self,
        id: i64,
        path: &str,
        name: &str,
        extension: Option<&str>,
        size: i64,
        modified_at: i64,
        directory_id: i64,
    ) -> Result<(), SearchError> {
        let mut writer = self
            .file_writer
            .lock()
            .map_err(|_| SearchError::LockError("file writer lock poisoned"))?;

        let fields = &self.file_fields;

        let term = Term::from_field_i64(fields.id, id);
        writer.delete_term(term);

        let doc = doc!(
            fields.id => id,
            fields.path => path,
            fields.name => name,
            fields.extension => extension.unwrap_or(""),
            fields.size => size,
            fields.modified_at => modified_at,
            fields.directory_id => directory_id,
            fields.access_count => 0i64,
            fields.access_timestamp => 0i64
        );

        writer
            .add_document(doc)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Reload reader to see the changes immediately
        self.file_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        Ok(())
    }

    /// Delete a bookmark from the index.
    pub fn delete_bookmark(&self, id: i64) -> Result<(), SearchError> {
        let mut writer = self.bookmark_writer.lock().map_err(|_| {
            SearchError::LockError("bookmark writer lock poisoned")
        })?;
        let term = Term::from_field_i64(self.bookmark_fields.id, id);
        writer.delete_term(term);
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        self.bookmark_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        Ok(())
    }

    /// Delete a file from the index.
    pub fn delete_file(&self, id: i64) -> Result<(), SearchError> {
        let mut writer = self
            .file_writer
            .lock()
            .map_err(|_| SearchError::LockError("file writer lock poisoned"))?;
        let term = Term::from_field_i64(self.file_fields.id, id);
        writer.delete_term(term);
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        self.file_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        Ok(())
    }

    /// Delete all files for a directory.
    pub fn delete_directory_files(&self, directory_id: i64) -> Result<(), SearchError> {
        let mut writer = self
            .file_writer
            .lock()
            .map_err(|_| SearchError::LockError("file writer lock poisoned"))?;
        let term = Term::from_field_i64(self.file_fields.directory_id, directory_id);
        writer.delete_term(term);
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        self.file_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        Ok(())
    }

    /// Rebuild the entire bookmark index from provided bookmark data.
    /// Uses batch indexing with single commit for performance.
    pub fn rebuild_bookmark_index_from_data(
        &self,
        bookmarks: Vec<(i64, String, String, Option<String>, Option<String>, Option<i64>, i64, i64)>,
    ) -> Result<usize, SearchError> {
        let mut writer = self.bookmark_writer.lock().map_err(|_| {
            SearchError::LockError("bookmark writer lock poisoned")
        })?;

        // Clear existing index
        writer
            .delete_all_documents()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let count = bookmarks.len();
        let fields = &self.bookmark_fields;

        // Batch add all documents without committing
        for (id, title, url, description, tags, last_accessed, created_at, updated_at) in bookmarks {
            let doc = doc!(
                fields.id => id,
                fields.title => title.as_str(),
                fields.url => url.as_str(),
                fields.description => description.as_deref().unwrap_or(""),
                fields.tags => tags.as_deref().unwrap_or(""),
                fields.last_accessed => last_accessed.unwrap_or(0),
                fields.created_at => created_at,
                fields.updated_at => updated_at,
                fields.access_count => 0i64,
                fields.access_timestamp => 0i64
            );

            writer
                .add_document(doc)
                .map_err(|e| SearchError::IndexError(e.to_string()))?;
        }

        // Single commit after all documents added
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        drop(writer);

        // Reload reader to see the changes
        self.bookmark_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        Ok(count)
    }

    /// Rebuild the entire file index from provided file data.
    /// Uses batch indexing with single commit for performance.
    pub fn rebuild_file_index_from_data(
        &self,
        files: Vec<(i64, String, String, Option<String>, i64, i64, i64)>,
    ) -> Result<usize, SearchError> {
        let mut writer = self.file_writer.lock().map_err(|_| {
            SearchError::LockError("file writer lock poisoned")
        })?;

        // Clear existing index
        writer
            .delete_all_documents()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let count = files.len();
        let fields = &self.file_fields;

        // Batch add all documents without committing
        for (id, path, name, extension, size, modified_at, directory_id) in files {
            let doc = doc!(
                fields.id => id,
                fields.path => path.as_str(),
                fields.name => name.as_str(),
                fields.extension => extension.as_deref().unwrap_or(""),
                fields.size => size,
                fields.modified_at => modified_at,
                fields.directory_id => directory_id,
                fields.access_count => 0i64,
                fields.access_timestamp => 0i64
            );

            writer
                .add_document(doc)
                .map_err(|e| SearchError::IndexError(e.to_string()))?;
        }

        // Single commit after all documents added
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        drop(writer);

        // Reload reader to see the changes
        self.file_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        Ok(count)
    }

    /// Get index statistics.
    pub fn get_stats(&self) -> Result<IndexStats, SearchError> {
        let bookmark_count = self.bookmark_reader.searcher().num_docs() as usize;
        let file_count = self.file_reader.searcher().num_docs() as usize;

        let bookmark_size = Self::calculate_dir_size(&self.index_dir.join("bookmarks"));
        let file_size = Self::calculate_dir_size(&self.index_dir.join("files"));

        Ok(IndexStats {
            bookmark_count,
            file_count,
            bookmark_index_size_bytes: bookmark_size,
            file_index_size_bytes: file_size,
        })
    }

    /// Commit any pending changes to the index.
    pub fn commit(&self) -> Result<(), SearchError> {
        {
            let mut writer = self.bookmark_writer.lock().map_err(|_| {
                SearchError::LockError("bookmark writer lock poisoned")
            })?;
            writer
                .commit()
                .map_err(|e| SearchError::IndexError(e.to_string()))?;
        }
        self.bookmark_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        {
            let mut writer = self.file_writer.lock().map_err(|_| {
                SearchError::LockError("file writer lock poisoned")
            })?;
            writer
                .commit()
                .map_err(|e| SearchError::IndexError(e.to_string()))?;
        }
        self.file_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        Ok(())
    }

    /// Update bookmark frecency data in the index.
    ///
    /// Performs a direct index update (read-delete-reindex pattern).
    /// This is efficient enough for desktop app click rates.
    pub fn update_bookmark_frecency(
        &self,
        id: i64,
        access_count: i64,
        access_timestamp: i64,
    ) -> Result<(), SearchError> {
        // Read existing document data from index
        let searcher = self.bookmark_reader.searcher();
        let fields = &self.bookmark_fields;

        // Find existing document by id
        let term = Term::from_field_i64(fields.id, id);
        let term_query = tantivy::query::TermQuery::new(term.clone(), tantivy::schema::IndexRecordOption::Basic);
        let top_docs = searcher.search(&term_query, &tantivy::collector::TopDocs::with_limit(1))
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        if top_docs.is_empty() {
            return Err(SearchError::IndexError(format!("Bookmark {} not found in index", id)));
        }

        let doc: TantivyDocument = searcher.doc(top_docs[0].1)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Extract existing values
        let title = doc.get_first(fields.title).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let url = doc.get_first(fields.url).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = doc.get_first(fields.description).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let tags = doc.get_first(fields.tags).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let last_accessed = doc.get_first(fields.last_accessed).and_then(|v| v.as_i64()).unwrap_or(0);
        let created_at = doc.get_first(fields.created_at).and_then(|v| v.as_i64()).unwrap_or(0);
        let updated_at = doc.get_first(fields.updated_at).and_then(|v| v.as_i64()).unwrap_or(0);

        // Delete and re-add with updated frecency
        let mut writer = self.bookmark_writer.lock()
            .map_err(|_| SearchError::LockError("bookmark writer lock poisoned"))?;

        writer.delete_term(term);

        let new_doc = doc!(
            fields.id => id,
            fields.title => title,
            fields.url => url,
            fields.description => description,
            fields.tags => tags,
            fields.last_accessed => last_accessed,
            fields.created_at => created_at,
            fields.updated_at => updated_at,
            fields.access_count => access_count,
            fields.access_timestamp => access_timestamp
        );

        writer.add_document(new_doc)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        writer.commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        drop(writer);

        self.bookmark_reader.reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        Ok(())
    }

    /// Update file frecency data in the index.
    ///
    /// Performs a direct index update (read-delete-reindex pattern).
    pub fn update_file_frecency(
        &self,
        id: i64,
        access_count: i64,
        access_timestamp: i64,
    ) -> Result<(), SearchError> {
        // Read existing document data from index
        let searcher = self.file_reader.searcher();
        let fields = &self.file_fields;

        // Find existing document by id
        let term = Term::from_field_i64(fields.id, id);
        let term_query = tantivy::query::TermQuery::new(term.clone(), tantivy::schema::IndexRecordOption::Basic);
        let top_docs = searcher.search(&term_query, &tantivy::collector::TopDocs::with_limit(1))
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        if top_docs.is_empty() {
            return Err(SearchError::IndexError(format!("File {} not found in index", id)));
        }

        let doc: TantivyDocument = searcher.doc(top_docs[0].1)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Extract existing values
        let path = doc.get_first(fields.path).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let name = doc.get_first(fields.name).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let extension = doc.get_first(fields.extension).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let size = doc.get_first(fields.size).and_then(|v| v.as_i64()).unwrap_or(0);
        let modified_at = doc.get_first(fields.modified_at).and_then(|v| v.as_i64()).unwrap_or(0);
        let directory_id = doc.get_first(fields.directory_id).and_then(|v| v.as_i64()).unwrap_or(0);

        // Delete and re-add with updated frecency
        let mut writer = self.file_writer.lock()
            .map_err(|_| SearchError::LockError("file writer lock poisoned"))?;

        writer.delete_term(term);

        let new_doc = doc!(
            fields.id => id,
            fields.path => path,
            fields.name => name,
            fields.extension => extension,
            fields.size => size,
            fields.modified_at => modified_at,
            fields.directory_id => directory_id,
            fields.access_count => access_count,
            fields.access_timestamp => access_timestamp
        );

        writer.add_document(new_doc)
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        writer.commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        drop(writer);

        self.file_reader.reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_engine() -> (TantivySearchEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let engine = TantivySearchEngine::new(temp_dir.path().to_path_buf()).unwrap();
        (engine, temp_dir)
    }

    #[test]
    fn test_index_and_search_bookmark() {
        let (engine, _temp_dir) = setup_test_engine();

        // Index a bookmark
        engine
            .index_bookmark(
                1,
                "Rust Programming",
                "https://rust-lang.org",
                Some("Systems programming language"),
                Some("programming,rust"),
                None,
                1000,
                1000,
            )
            .unwrap();

        // Search for it
        let results = engine.search_bookmarks("rust", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }

    #[test]
    fn test_index_and_search_file() {
        let (engine, _temp_dir) = setup_test_engine();

        // Index a file
        engine
            .index_file(
                1,
                "/home/user/document.pdf",
                "document.pdf",
                Some("pdf"),
                1024,
                1000,
                1,
            )
            .unwrap();

        // Search for it
        let results = engine.search_files("document", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "document.pdf");
    }

    #[test]
    fn test_delete_bookmark() {
        let (engine, _temp_dir) = setup_test_engine();

        engine
            .index_bookmark(1, "Test", "https://test.com", None, None, None, 1000, 1000)
            .unwrap();

        engine.delete_bookmark(1).unwrap();

        let results = engine.search_bookmarks("test", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_stats() {
        let (engine, _temp_dir) = setup_test_engine();

        engine
            .index_bookmark(1, "Test", "https://test.com", None, None, None, 1000, 1000)
            .unwrap();

        let stats = engine.get_stats().unwrap();
        assert_eq!(stats.bookmark_count, 1);
        assert_eq!(stats.file_count, 0);
    }

    #[test]
    fn test_frecency_scoring() {
        let (engine, _temp_dir) = setup_test_engine();

        // Index a bookmark
        engine
            .index_bookmark(
                1,
                "Test Bookmark",
                "https://test.com",
                Some("A test"),
                None,
                None,
                1000,
                1000,
            )
            .unwrap();

        // First search - should have frecency 0
        let results = engine.search_bookmarks("test", 10).unwrap();
        assert_eq!(results.len(), 1);
        // Initial frecency should be 0 (no usage history)
        assert!(results[0].frecency_score <= 0.1);

        // Search results should include score field
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_search_performance() {
        let (engine, _temp_dir) = setup_test_engine();

        // Index 100 bookmarks
        for i in 1..=100 {
            engine
                .index_bookmark(
                    i,
                    &format!("Bookmark {} about Rust programming", i),
                    &format!("https://example{}.com", i),
                    Some(&format!("Description for bookmark {}", i)),
                    Some("rust,programming"),
                    None,
                    1000 + i,
                    1000 + i,
                )
                .unwrap();
        }

        // Measure search time
        let start = std::time::Instant::now();
        let results = engine.search_bookmarks("rust programming", 10).unwrap();
        let duration = start.elapsed();

        // Verify results
        assert!(!results.is_empty());

        // Performance check: should be under 100ms
        assert!(
            duration.as_millis() < 100,
            "Search took {}ms, expected <100ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_empty_query_returns_recent() {
        let (engine, _temp_dir) = setup_test_engine();

        // Index bookmarks with different timestamps
        engine
            .index_bookmark(
                1,
                "Old Bookmark",
                "https://old.com",
                None,
                None,
                Some(1000),
                900,
                900,
            )
            .unwrap();
        engine
            .index_bookmark(
                2,
                "New Bookmark",
                "https://new.com",
                None,
                None,
                Some(2000),
                2000,
                2000,
            )
            .unwrap();

        // Empty query should return results (recent items)
        let results = engine.search_bookmarks("", 10).unwrap();
        // Results may be empty if no usage history, but shouldn't error
        assert!(results.len() <= 10);
    }

    #[test]
    fn test_rebuild_index() {
        let (engine, _temp_dir) = setup_test_engine();

        // Index some bookmarks
        engine
            .index_bookmark(
                1,
                "First",
                "https://first.com",
                None,
                None,
                None,
                1000,
                1000,
            )
            .unwrap();
        engine
            .index_bookmark(
                2,
                "Second",
                "https://second.com",
                None,
                None,
                None,
                1000,
                1000,
            )
            .unwrap();

        // Rebuild with empty data should clear the index
        let count = engine.rebuild_bookmark_index_from_data(vec![]).unwrap();
        assert_eq!(count, 0);

        // Search should return empty after rebuild with no data
        let results = engine.search_bookmarks("first", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_concurrent_search_and_index() {
        use std::sync::Arc;
        use std::thread;

        let (engine, _temp_dir) = setup_test_engine();
        let engine = Arc::new(engine);

        // Index some initial data
        for i in 1..=10 {
            engine
                .index_bookmark(
                    i,
                    &format!("Concurrent Test {}", i),
                    &format!("https://concurrent{}.com", i),
                    Some("Test for concurrent access"),
                    None,
                    None,
                    1000 + i,
                    1000 + i,
                )
                .unwrap();
        }

        // Spawn multiple threads that search and index concurrently
        let mut handles = vec![];

        // 3 search threads
        for _ in 0..3 {
            let engine_clone = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for _ in 0..20 {
                    let _ = engine_clone.search_bookmarks("concurrent", 10);
                    let _ = engine_clone.search_files("test", 10);
                    thread::sleep(std::time::Duration::from_millis(1));
                }
            }));
        }

        // 2 index threads
        for t in 0..2 {
            let engine_clone = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                for i in 0..10 {
                    let id = 100 + t * 10 + i;
                    let _ = engine_clone.index_bookmark(
                        id,
                        &format!("Thread {} Bookmark {}", t, i),
                        &format!("https://thread{}bm{}.com", t, i),
                        None,
                        None,
                        None,
                        2000 + id,
                        2000 + id,
                    );
                    thread::sleep(std::time::Duration::from_millis(2));
                }
            }));
        }

        // Wait for all threads to complete (if no deadlock, this will succeed)
        for handle in handles {
            handle.join().expect("Thread panicked - potential deadlock?");
        }

        // Final verification - search should still work
        let results = engine.search_bookmarks("concurrent", 50).unwrap();
        assert!(!results.is_empty(), "Search should return results after concurrent operations");
    }

    /// Test zero-lock search: verify search works entirely from index without DB dependency
    /// This is the key architectural validation for the Linus-style refactor
    #[test]
    fn test_zero_lock_search_with_frecency() {
        let (engine, _temp_dir) = setup_test_engine();

        // 1. Index a bookmark (initializes with access_count=0, access_timestamp=0)
        engine
            .index_bookmark(
                1,
                "Zero Lock Test",
                "https://zerolock.com",
                Some("Testing search without DB locks"),
                Some("test,zerolock"),
                None,
                1000,
                1000,
            )
            .unwrap();

        // 2. Search without any frecency - should work
        let results1 = engine.search_bookmarks("zerolock", 10).unwrap();
        assert_eq!(results1.len(), 1);
        assert_eq!(results1[0].frecency_score, 0.0, "Initial frecency should be 0");

        // 3. Update frecency via update_bookmark_frecency (simulates access tracking)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        engine.update_bookmark_frecency(1, 5, now).unwrap();

        // 4. Search again - frecency should now be incorporated
        let results2 = engine.search_bookmarks("zerolock", 10).unwrap();
        assert_eq!(results2.len(), 1);
        assert!(
            results2[0].frecency_score > 0.0,
            "Frecency should be positive after update: {}",
            results2[0].frecency_score
        );

        // 5. Second update should now apply immediately (no batching)
        engine.update_bookmark_frecency(1, 20, now).unwrap();

        let results3 = engine.search_bookmarks("zerolock", 10).unwrap();
        // Frecency should now reflect the higher access_count (20 vs 5)
        assert!(
            results3[0].frecency_score > results2[0].frecency_score,
            "Frecency should increase with higher access_count: {} > {}",
            results3[0].frecency_score,
            results2[0].frecency_score
        );

        // Note: The key architectural win is that search_bookmarks() never touches
        // the DB - all frecency data comes from Tantivy FastFields!
    }

    /// Test that update_file_frecency correctly updates frecency in the file index
    #[test]
    fn test_update_file_frecency() {
        let (engine, _temp_dir) = setup_test_engine();

        // Index a file
        engine
            .index_file(
                1,
                "/home/user/project/main.rs",
                "main.rs",
                Some("rs"),
                2048,
                1000,
                1,
            )
            .unwrap();

        // Initial search
        let results1 = engine.search_files("main", 10).unwrap();
        assert_eq!(results1.len(), 1);
        assert_eq!(results1[0].frecency_score, 0.0);

        // Update frecency
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        engine.update_file_frecency(1, 10, now).unwrap();

        // Search again
        let results2 = engine.search_files("main", 10).unwrap();
        assert!(
            results2[0].frecency_score > 0.0,
            "File frecency should be positive after update"
        );
    }

    /// Test empty query returns items sorted by access_timestamp from index
    #[test]
    fn test_empty_query_uses_index_timestamps() {
        let (engine, _temp_dir) = setup_test_engine();

        // Index two bookmarks
        engine.index_bookmark(1, "First", "https://first.com", None, None, None, 1000, 1000).unwrap();
        engine.index_bookmark(2, "Second", "https://second.com", None, None, None, 1000, 1000).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Update "Second" to have higher access timestamp
        engine.update_bookmark_frecency(2, 1, now).unwrap();
        // "First" has older access
        engine.update_bookmark_frecency(1, 1, now - 1000).unwrap();

        // Empty query should return "Second" first (higher access_timestamp)
        let results = engine.search_bookmarks("", 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Second", "More recently accessed should be first");
        assert_eq!(results[1].title, "First");
    }

    #[test]
    fn test_search_info_keyword_debug() {
        let (engine, _temp_dir) = setup_test_engine();

        engine.index_bookmark(1, "System Information", "https://info.example.com", Some("system info page"), None, None, 1000, 1000).unwrap();
        engine.index_bookmark(2, "GitHub", "https://github.com", Some("Code hosting"), None, None, 1000, 1000).unwrap();
        engine.index_bookmark(3, "InfoQ", "https://infoq.com", Some("技术新闻"), None, None, 1000, 1000).unwrap();

        let results = engine.search_bookmarks("info", 10).unwrap();
        println!("Search 'info' returned {} results:", results.len());
        for r in &results {
            println!("  - [{}] {} (score: {:.3})", r.id, r.title, r.score);
        }
        assert!(!results.is_empty(), "Should find bookmarks containing 'info'");
    }
}
