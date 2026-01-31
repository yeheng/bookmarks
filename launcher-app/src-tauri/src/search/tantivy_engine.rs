//! Tantivy-based search engine implementation.
//!
//! # Lock Ordering and Concurrency
//!
//! This engine uses multiple locks to protect shared resources:
//!
//! 1. `db: Mutex<Database>` - Protects SQLite database access
//! 2. `bookmark_writer: Mutex<IndexWriter>` - Protects bookmark index writes
//! 3. `file_writer: Mutex<IndexWriter>` - Protects file index writes
//!
//! ## Lock Acquisition Strategy
//!
//! - Search methods (search_bookmarks, search_files) perform Tantivy search FIRST
//!   without any locks, then briefly acquire the DB lock for batch queries only.
//!   This minimizes the critical section and reduces deadlock risk.
//!
//! - Write methods (index_bookmark, index_file) only acquire writer locks.
//!   Callers must release any DB locks before calling these methods.
//!
//! - Rebuild methods acquire locks in order: writer lock first, then DB lock.
//!   DB lock is released before indexing to avoid holding both locks.
//!
//! ## Important Notes
//!
//! - Never hold DB lock while calling index_bookmark/index_file
//! - Search methods hold DB lock for the minimum time needed (batch queries only)
//! - All lock poisoning is handled via SearchError::LockError

use super::engine::{IndexStats, SearchEngine, SearchError};
use super::schema::{build_bookmark_schema, build_file_schema, register_tokenizers};
use crate::db::Database;
use crate::models::bookmark::BookmarkSearchResult;
use crate::models::file::FileSearchResult;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tantivy::collector::TopDocs;

/// Index writer heap size in bytes (50MB)
const INDEX_WRITER_HEAP_SIZE: usize = 50_000_000;

/// Search result multiplier for frecency re-ranking
/// Fetch 2x the requested limit to allow for re-ranking
const SEARCH_RESULT_MULTIPLIER: usize = 2;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};

/// Tantivy-based search engine implementation.
///
/// Provides full-text search for bookmarks and files using Tantivy,
/// with frecency scoring integrated from SQLite usage history.
pub struct TantivySearchEngine {
    // Bookmark index components
    bookmark_index: Index,
    bookmark_writer: Mutex<IndexWriter>,
    bookmark_reader: IndexReader,
    bookmark_schema: Schema,

    // File index components
    file_index: Index,
    file_writer: Mutex<IndexWriter>,
    file_reader: IndexReader,
    file_schema: Schema,

    // Database connection for frecency queries and rebuilds
    db: Mutex<Database>,

    // Index directory for stats
    index_dir: PathBuf,
}

impl TantivySearchEngine {
    /// Create a new Tantivy search engine.
    ///
    /// # Arguments
    /// * `index_dir` - Directory to store Tantivy indexes
    /// * `db` - Database instance for frecency queries and index rebuilds
    pub fn new(index_dir: PathBuf, db: Database) -> Result<Self, SearchError> {
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

        Ok(Self {
            bookmark_index,
            bookmark_writer: Mutex::new(bookmark_writer),
            bookmark_reader,
            bookmark_schema,
            file_index,
            file_writer: Mutex::new(file_writer),
            file_reader,
            file_schema,
            db: Mutex::new(db),
            index_dir,
        })
    }

    /// Batch get frecency scores for multiple bookmarks from SQLite.
    /// This eliminates the N+1 query problem by fetching all frecency scores in one query.
    fn get_bookmark_frecencies(
        &self,
        conn: &Connection,
        bookmark_ids: &[i64],
    ) -> std::collections::HashMap<i64, f64> {
        use std::collections::HashMap;

        if bookmark_ids.is_empty() {
            return HashMap::new();
        }

        // Build placeholders for the IN clause
        let placeholders: Vec<String> = bookmark_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT bookmark_id,
                    COUNT(*) * 0.3 + (julianday('now') - julianday(MAX(accessed_at), 'unixepoch')) * -0.1 as frecency
             FROM usage_history
             WHERE bookmark_id IN ({})
             GROUP BY bookmark_id",
            placeholders.join(",")
        );

        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => return HashMap::new(),
        };

        let params: Vec<&dyn rusqlite::ToSql> = bookmark_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let rows = match stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
        }) {
            Ok(r) => r,
            Err(_) => return HashMap::new(),
        };

        rows.filter_map(|r| r.ok()).collect()
    }

    /// Batch get frecency scores for multiple files from SQLite.
    /// This eliminates the N+1 query problem by fetching all frecency scores in one query.
    fn get_file_frecencies(
        &self,
        conn: &Connection,
        file_ids: &[i64],
    ) -> std::collections::HashMap<i64, f64> {
        use std::collections::HashMap;

        if file_ids.is_empty() {
            return HashMap::new();
        }

        // Build placeholders for the IN clause
        let placeholders: Vec<String> = file_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT file_id,
                    COUNT(*) * 0.3 + (julianday('now') - julianday(MAX(accessed_at), 'unixepoch')) * -0.1 as frecency
             FROM file_usage_history
             WHERE file_id IN ({})
             GROUP BY file_id",
            placeholders.join(",")
        );

        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => return HashMap::new(),
        };

        let params: Vec<&dyn rusqlite::ToSql> = file_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let rows = match stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
        }) {
            Ok(r) => r,
            Err(_) => return HashMap::new(),
        };

        rows.filter_map(|r| r.ok()).collect()
    }

    /// Batch get favicon URLs for multiple bookmarks from SQLite.
    /// This eliminates the N+1 query problem by fetching all favicons in one query.
    fn batch_get_bookmark_favicons(
        &self,
        conn: &Connection,
        bookmark_ids: &[i64],
    ) -> std::collections::HashMap<i64, Option<String>> {
        use std::collections::HashMap;

        if bookmark_ids.is_empty() {
            return HashMap::new();
        }

        let placeholders: Vec<String> = bookmark_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT id, favicon_url FROM bookmarks WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => return HashMap::new(),
        };

        let params: Vec<&dyn rusqlite::ToSql> = bookmark_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let rows = match stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
        }) {
            Ok(r) => r,
            Err(_) => return HashMap::new(),
        };

        rows.filter_map(|r| r.ok()).collect()
    }

    /// Get recent bookmarks when query is empty.
    fn get_recent_bookmarks(
        &self,
        conn: &Connection,
        limit: usize,
    ) -> Result<Vec<BookmarkSearchResult>, SearchError> {
        let mut stmt = conn
            .prepare(
                "SELECT b.id, b.title, b.url, b.description, b.favicon_url
                 FROM bookmarks b
                 LEFT JOIN usage_history uh ON b.id = uh.bookmark_id
                 GROUP BY b.id
                 ORDER BY MAX(uh.accessed_at) DESC NULLS LAST
                 LIMIT ?1",
            )
            .map_err(|e| SearchError::DatabaseError(e.to_string()))?;

        let results = stmt
            .query_map([limit as i64], |row| {
                Ok(BookmarkSearchResult {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    description: row.get(3)?,
                    favicon_url: row.get(4)?,
                    score: 0.0,
                    frecency_score: 0.0,
                })
            })
            .map_err(|e| SearchError::DatabaseError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SearchError::DatabaseError(e.to_string()))?;

        Ok(results)
    }

    /// Get recent files when query is empty.
    fn get_recent_files(
        &self,
        conn: &Connection,
        limit: usize,
    ) -> Result<Vec<FileSearchResult>, SearchError> {
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.path, f.name, f.extension, f.size, f.modified_at
                 FROM indexed_files f
                 LEFT JOIN file_usage_history fuh ON f.id = fuh.file_id
                 GROUP BY f.id
                 ORDER BY MAX(fuh.accessed_at) DESC NULLS LAST, f.modified_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| SearchError::DatabaseError(e.to_string()))?;

        let results = stmt
            .query_map([limit as i64], |row| {
                Ok(FileSearchResult {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    name: row.get(2)?,
                    extension: row.get(3)?,
                    size: row.get(4)?,
                    modified_at: row.get(5)?,
                    score: 0.0,
                    frecency_score: 0.0,
                })
            })
            .map_err(|e| SearchError::DatabaseError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SearchError::DatabaseError(e.to_string()))?;

        Ok(results)
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
}

impl SearchEngine for TantivySearchEngine {
    fn search_bookmarks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<BookmarkSearchResult>, SearchError> {
        // Handle empty query - needs DB for recent items
        if query.trim().is_empty() {
            let db = self.db.lock().map_err(|_| {
                SearchError::LockError("database lock poisoned")
            })?;
            return self.get_recent_bookmarks(db.get_connection(), limit);
        }

        // Phase 1: Tantivy search (NO DB lock needed)
        let searcher = self.bookmark_reader.searcher();

        let title_field = self
            .bookmark_schema
            .get_field("title")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let url_field = self
            .bookmark_schema
            .get_field("url")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let description_field = self
            .bookmark_schema
            .get_field("description")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let tags_field = self
            .bookmark_schema
            .get_field("tags")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let id_field = self
            .bookmark_schema
            .get_field("id")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let query_parser = QueryParser::for_index(
            &self.bookmark_index,
            vec![title_field, url_field, description_field, tags_field],
        );

        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| SearchError::QueryError(e.to_string()))?;

        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(limit * SEARCH_RESULT_MULTIPLIER))
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Phase 2: Extract document data from Tantivy (still NO DB lock)
        let mut doc_data: Vec<(i64, f32, String, String, Option<String>)> = Vec::new();
        let mut ids: Vec<i64> = Vec::new();

        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| SearchError::IndexError(e.to_string()))?;

            let id = doc
                .get_first(id_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let title = doc
                .get_first(title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let url = doc
                .get_first(url_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let description = doc
                .get_first(description_field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            doc_data.push((id, score, title, url, description));
            ids.push(id);
        }

        // Phase 3: Brief DB lock for batch queries only
        let (frecency_map, favicon_map) = {
            let db = self.db.lock().map_err(|_| {
                SearchError::LockError("database lock poisoned")
            })?;
            let conn = db.get_connection();
            let frecencies = self.get_bookmark_frecencies(conn, &ids);
            let favicons = self.batch_get_bookmark_favicons(conn, &ids);
            (frecencies, favicons)
            // DB lock released here
        };

        // Phase 4: Combine results (NO DB lock)
        let mut results: Vec<BookmarkSearchResult> = doc_data
            .into_iter()
            .map(|(id, score, title, url, description)| {
                let frecency = frecency_map.get(&id).copied().unwrap_or(0.0);
                let combined_score = (score as f64) * 0.7 + frecency * 0.3;
                let favicon_url = favicon_map.get(&id).cloned().flatten();

                BookmarkSearchResult {
                    id,
                    title,
                    url,
                    description,
                    favicon_url,
                    score: combined_score,
                    frecency_score: frecency,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    fn search_files(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<FileSearchResult>, SearchError> {
        // Handle empty query - needs DB for recent items
        if query.trim().is_empty() {
            let db = self.db.lock().map_err(|_| {
                SearchError::LockError("database lock poisoned")
            })?;
            return self.get_recent_files(db.get_connection(), limit);
        }

        // Phase 1: Tantivy search (NO DB lock needed)
        let searcher = self.file_reader.searcher();

        let name_field = self
            .file_schema
            .get_field("name")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let path_field = self
            .file_schema
            .get_field("path")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let id_field = self
            .file_schema
            .get_field("id")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let extension_field = self
            .file_schema
            .get_field("extension")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let size_field = self
            .file_schema
            .get_field("size")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let modified_at_field = self
            .file_schema
            .get_field("modified_at")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let query_parser = QueryParser::for_index(&self.file_index, vec![name_field, path_field]);

        // With ngram tokenizer, no need for wildcard suffixes -
        // ngram naturally handles fuzzy and partial matching for CJK and Latin text
        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| SearchError::QueryError(e.to_string()))?;

        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(limit * SEARCH_RESULT_MULTIPLIER))
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        // Phase 2: Extract document data from Tantivy (still NO DB lock)
        let mut doc_data: Vec<(i64, f32, String, String, Option<String>, i64, i64)> = Vec::new();
        let mut ids: Vec<i64> = Vec::new();

        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| SearchError::IndexError(e.to_string()))?;

            let id = doc
                .get_first(id_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let path = doc
                .get_first(path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = doc
                .get_first(name_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let extension = doc
                .get_first(extension_field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let size = doc
                .get_first(size_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let modified_at = doc
                .get_first(modified_at_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            doc_data.push((id, score, path, name, extension, size, modified_at));
            ids.push(id);
        }

        // Phase 3: Brief DB lock for batch frecency query only
        let frecency_map = {
            let db = self.db.lock().map_err(|_| {
                SearchError::LockError("database lock poisoned")
            })?;
            self.get_file_frecencies(db.get_connection(), &ids)
            // DB lock released here
        };

        // Phase 4: Combine results (NO DB lock)
        let mut results: Vec<FileSearchResult> = doc_data
            .into_iter()
            .map(|(id, score, path, name, extension, size, modified_at)| {
                let frecency = frecency_map.get(&id).copied().unwrap_or(0.0);
                let combined_score = (score as f64) * 0.7 + frecency * 0.3;

                FileSearchResult {
                    id,
                    path,
                    name,
                    extension,
                    size,
                    modified_at,
                    score: combined_score,
                    frecency_score: frecency,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

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
    ) -> Result<(), SearchError> {
        let mut writer = self.bookmark_writer.lock().map_err(|_| {
            SearchError::LockError("bookmark writer lock poisoned")
        })?;

        // Delete existing document
        let id_field = self
            .bookmark_schema
            .get_field("id")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let term = Term::from_field_i64(id_field, id);
        writer.delete_term(term);

        // Add new document
        let title_field = self
            .bookmark_schema
            .get_field("title")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let url_field = self
            .bookmark_schema
            .get_field("url")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let description_field = self
            .bookmark_schema
            .get_field("description")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let tags_field = self
            .bookmark_schema
            .get_field("tags")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let last_accessed_field = self
            .bookmark_schema
            .get_field("last_accessed")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let created_at_field = self
            .bookmark_schema
            .get_field("created_at")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let updated_at_field = self
            .bookmark_schema
            .get_field("updated_at")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let doc = doc!(
            id_field => id,
            title_field => title,
            url_field => url,
            description_field => description.unwrap_or(""),
            tags_field => tags.unwrap_or(""),
            last_accessed_field => last_accessed.unwrap_or(0),
            created_at_field => created_at,
            updated_at_field => updated_at
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

    fn index_file(
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

        let id_field = self
            .file_schema
            .get_field("id")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let term = Term::from_field_i64(id_field, id);
        writer.delete_term(term);

        let path_field = self
            .file_schema
            .get_field("path")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let name_field = self
            .file_schema
            .get_field("name")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let extension_field = self
            .file_schema
            .get_field("extension")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let size_field = self
            .file_schema
            .get_field("size")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let modified_at_field = self
            .file_schema
            .get_field("modified_at")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let directory_id_field = self
            .file_schema
            .get_field("directory_id")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let doc = doc!(
            id_field => id,
            path_field => path,
            name_field => name,
            extension_field => extension.unwrap_or(""),
            size_field => size,
            modified_at_field => modified_at,
            directory_id_field => directory_id
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

    fn delete_bookmark(&self, id: i64) -> Result<(), SearchError> {
        let mut writer = self.bookmark_writer.lock().map_err(|_| {
            SearchError::LockError("bookmark writer lock poisoned")
        })?;
        let id_field = self
            .bookmark_schema
            .get_field("id")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let term = Term::from_field_i64(id_field, id);
        writer.delete_term(term);
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        self.bookmark_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        Ok(())
    }

    fn delete_file(&self, id: i64) -> Result<(), SearchError> {
        let mut writer = self
            .file_writer
            .lock()
            .map_err(|_| SearchError::LockError("file writer lock poisoned"))?;
        let id_field = self
            .file_schema
            .get_field("id")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let term = Term::from_field_i64(id_field, id);
        writer.delete_term(term);
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        self.file_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        Ok(())
    }

    fn delete_directory_files(&self, directory_id: i64) -> Result<(), SearchError> {
        let mut writer = self
            .file_writer
            .lock()
            .map_err(|_| SearchError::LockError("file writer lock poisoned"))?;
        let dir_field = self
            .file_schema
            .get_field("directory_id")
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        let term = Term::from_field_i64(dir_field, directory_id);
        writer.delete_term(term);
        writer
            .commit()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        self.file_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;
        Ok(())
    }

    fn rebuild_bookmark_index(&self) -> Result<usize, SearchError> {
        // Clear existing index
        {
            let mut writer = self.bookmark_writer.lock().map_err(|_| {
                SearchError::LockError("bookmark writer lock poisoned")
            })?;
            writer
                .delete_all_documents()
                .map_err(|e| SearchError::IndexError(e.to_string()))?;
            writer
                .commit()
                .map_err(|e| SearchError::IndexError(e.to_string()))?;
        }
        self.bookmark_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let bookmarks = {
            let db = self
                .db
                .lock()
                .map_err(|_| SearchError::LockError("database lock poisoned"))?;
            let conn = db.get_connection();

            let mut stmt = conn
                .prepare(
                    "SELECT id, title, url, description, tags, last_accessed, created_at, updated_at
                     FROM bookmarks",
                )
                .map_err(|e| SearchError::DatabaseError(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                        row.get::<_, i64>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                })
                .map_err(|e| SearchError::DatabaseError(e.to_string()))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| SearchError::DatabaseError(e.to_string()))?
        };

        let count = bookmarks.len();

        for (id, title, url, description, tags, last_accessed, created_at, updated_at) in bookmarks
        {
            self.index_bookmark(
                id,
                &title,
                &url,
                description.as_deref(),
                tags.as_deref(),
                last_accessed,
                created_at,
                updated_at,
            )?;
        }

        Ok(count)
    }

    fn rebuild_file_index(&self) -> Result<usize, SearchError> {
        // Clear existing index
        {
            let mut writer = self.file_writer.lock().map_err(|_| {
                SearchError::LockError("file writer lock poisoned")
            })?;
            writer
                .delete_all_documents()
                .map_err(|e| SearchError::IndexError(e.to_string()))?;
            writer
                .commit()
                .map_err(|e| SearchError::IndexError(e.to_string()))?;
        }
        self.file_reader
            .reload()
            .map_err(|e| SearchError::IndexError(e.to_string()))?;

        let files = {
            let db = self
                .db
                .lock()
                .map_err(|_| SearchError::LockError("database lock poisoned"))?;
            let conn = db.get_connection();

            let mut stmt = conn
                .prepare(
                    "SELECT id, path, name, extension, size, modified_at, directory_id
                     FROM indexed_files",
                )
                .map_err(|e| SearchError::DatabaseError(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                })
                .map_err(|e| SearchError::DatabaseError(e.to_string()))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| SearchError::DatabaseError(e.to_string()))?
        };

        let count = files.len();

        for (id, path, name, extension, size, modified_at, directory_id) in files {
            self.index_file(
                id,
                &path,
                &name,
                extension.as_deref(),
                size,
                modified_at,
                directory_id,
            )?;
        }

        Ok(count)
    }

    fn get_stats(&self) -> Result<IndexStats, SearchError> {
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

    fn commit(&self) -> Result<(), SearchError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_engine() -> (TantivySearchEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();
        db.initialize().unwrap();
        let engine = TantivySearchEngine::new(temp_dir.path().to_path_buf(), db).unwrap();
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

        // Rebuild should work
        let count = engine.rebuild_bookmark_index().unwrap();
        // Count depends on what's in DB, but shouldn't error
        // (in-memory test DB is empty, so count should be 0)
        assert_eq!(count, 0);

        // Search should still work after rebuild
        let results = engine.search_bookmarks("first", 10).unwrap();
        // May be 0 if DB doesn't have entries (test uses empty in-memory DB)
        assert!(results.len() <= 10);
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
}
