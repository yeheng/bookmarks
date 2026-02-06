/// Data Service Layer
///
/// This service provides atomic operations that coordinate writes between
/// SQLite (source of truth) and Tantivy (search index).
///
/// Design principles (Linus-style simplicity):
/// 1. SQLite is the authoritative source of truth
/// 2. Tantivy index is a cache - if it's corrupted, rebuild from SQLite
/// 3. No complex recovery mechanisms - just rebuild_if_needed on startup
/// 4. Direct index updates, no batching needed for desktop app
///
/// Locking Strategy:
/// - ALWAYS acquire DB lock first, then release BEFORE calling search_engine methods
/// - This prevents deadlocks and ensures DB operations are not blocked by slow index updates
/// - Pattern: acquire DB lock -> read/write DB -> release DB lock -> update index

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::search::TantivySearchEngine;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DataService {
    db: Mutex<Database>,
    search_engine: Arc<TantivySearchEngine>,
}

impl DataService {
    pub fn new(db: Database, search_engine: Arc<TantivySearchEngine>) -> Self {
        Self {
            db: Mutex::new(db),
            search_engine,
        }
    }

    /// Add a bookmark with atomic DB + Index update
    ///
    /// Strategy:
    /// 1. Write to SQLite (source of truth)
    /// 2. Update Tantivy index
    /// If index update fails, it's logged but not rolled back - index can be rebuilt later.
    pub fn add_bookmark(
        &self,
        title: String,
        url: String,
        description: Option<String>,
        tags: Option<String>,
    ) -> AppResult<i64> {
        let db = self
            .db
            .lock()
            .map_err(|_| AppError::DatabaseLock)?;
        let conn = db.get_connection();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;

        // Check for duplicates
        let existing: Result<i64, _> =
            conn.query_row("SELECT id FROM bookmarks WHERE url = ?1", [&url], |row| {
                row.get(0)
            });

        if existing.is_ok() {
            return Err(AppError::DuplicateBookmark(url));
        }

        // Write to SQLite (source of truth)
        conn.execute(
            "INSERT INTO bookmarks (title, url, description, tags, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![title, url, description, tags, "manual", now, now],
        )?;

        let id = conn.last_insert_rowid();

        // Release DB lock before indexing (Tantivy operations are slow)
        drop(db);

        // Update Tantivy index (best-effort)
        if let Err(e) = self.search_engine.index_bookmark(
            id,
            &title,
            &url,
            description.as_deref(),
            tags.as_deref(),
            None,
            now,
            now,
        ) {
            eprintln!(
                "[DataService] Warning: Failed to index bookmark {}: {}. Index may need rebuild.",
                id, e
            );
            // DO NOT rollback the database - SQLite is the source of truth
        }

        Ok(id)
    }

    /// Update a bookmark atomically
    pub fn update_bookmark(
        &self,
        id: i64,
        title: String,
        url: String,
        description: Option<String>,
        tags: Option<String>,
    ) -> AppResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| AppError::DatabaseLock)?;
        let conn = db.get_connection();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;

        // Get metadata for re-indexing
        let (created_at, last_accessed): (i64, Option<i64>) = conn
            .query_row(
                "SELECT created_at, last_accessed FROM bookmarks WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| AppError::BookmarkNotFound)?;

        let rows_affected = conn.execute(
            "UPDATE bookmarks SET title = ?1, url = ?2, description = ?3, tags = ?4, updated_at = ?5 WHERE id = ?6",
            rusqlite::params![title, url, description, tags, now, id],
        )?;

        if rows_affected == 0 {
            return Err(AppError::BookmarkNotFound);
        }

        drop(db);

        // Re-index in Tantivy (best-effort)
        if let Err(e) = self.search_engine.index_bookmark(
            id,
            &title,
            &url,
            description.as_deref(),
            tags.as_deref(),
            last_accessed,
            created_at,
            now,
        ) {
            eprintln!(
                "[DataService] Warning: Failed to re-index bookmark {}: {}",
                id, e
            );
        }

        Ok(())
    }

    /// Delete a bookmark atomically
    pub fn delete_bookmark(&self, id: i64) -> AppResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| AppError::DatabaseLock)?;
        let conn = db.get_connection();

        let rows_affected = conn.execute("DELETE FROM bookmarks WHERE id = ?1", [id])?;

        if rows_affected == 0 {
            return Err(AppError::BookmarkNotFound);
        }

        drop(db);

        // Remove from Tantivy index (best-effort)
        if let Err(e) = self.search_engine.delete_bookmark(id) {
            eprintln!(
                "[DataService] Warning: Failed to remove bookmark {} from index: {}",
                id, e
            );
        }

        Ok(())
    }

    /// Check index integrity and rebuild if needed.
    ///
    /// Simple strategy: compare document counts. If they differ significantly,
    /// trigger a full rebuild.
    pub fn rebuild_index_if_needed(&self) -> AppResult<bool> {
        let db = self
            .db
            .lock()
            .map_err(|_| AppError::DatabaseLock)?;
        let conn = db.get_connection();

        // Get bookmark count from SQLite
        let db_bookmark_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))
            .unwrap_or(0);

        // Get file count from SQLite
        let db_file_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM indexed_files", [], |row| row.get(0))
            .unwrap_or(0);

        drop(db);

        // Get counts from Tantivy index
        let stats = self.search_engine.get_stats()
            .map_err(|e| AppError::Search(e.to_string()))?;

        let index_bookmark_count = stats.bookmark_count as i64;
        let index_file_count = stats.file_count as i64;

        // Check if counts differ significantly (allow some tolerance)
        let bookmark_diff = (db_bookmark_count - index_bookmark_count).abs();
        let file_diff = (db_file_count - index_file_count).abs();

        // If difference is more than 10% or absolute difference > 10, rebuild
        let needs_rebuild =
            (db_bookmark_count > 0 && bookmark_diff > db_bookmark_count / 10) ||
            bookmark_diff > 10 ||
            (db_file_count > 0 && file_diff > db_file_count / 10) ||
            file_diff > 10;

        if needs_rebuild {
            println!(
                "[DataService] Index mismatch detected. DB: {} bookmarks, {} files. Index: {} bookmarks, {} files. Rebuilding...",
                db_bookmark_count, db_file_count, index_bookmark_count, index_file_count
            );
            self.rebuild_all_indexes()?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Rebuild all indexes from database
    fn rebuild_all_indexes(&self) -> AppResult<()> {
        // Fetch all data from database in a scoped block
        let (bookmarks, files) = {
            let db = self
                .db
                .lock()
                .map_err(|_| AppError::DatabaseLock)?;
            let conn = db.get_connection();

            // Fetch all bookmarks
            let mut stmt = conn.prepare(
                "SELECT id, title, url, description, tags, last_accessed, created_at, updated_at
                 FROM bookmarks"
            )?;
            let bookmarks: Vec<_> = stmt
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
                })?
                .collect::<Result<Vec<_>, _>>()?;

            // Fetch all files
            let mut stmt = conn.prepare(
                "SELECT id, path, name, extension, size, modified_at, directory_id
                 FROM indexed_files"
            )?;
            let files: Vec<_> = stmt
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
                })?
                .collect::<Result<Vec<_>, _>>()?;

            (bookmarks, files)
            // db lock released here at end of scope
        };

        // Rebuild bookmark index (no DB lock held)
        let bookmark_count = self.search_engine.rebuild_bookmark_index_from_data(bookmarks)
            .map_err(|e| AppError::Search(e.to_string()))?;
        println!("[DataService] Rebuilt bookmark index with {} entries", bookmark_count);

        // Rebuild file index
        let file_count = self.search_engine.rebuild_file_index_from_data(files)
            .map_err(|e| AppError::Search(e.to_string()))?;
        println!("[DataService] Rebuilt file index with {} entries", file_count);

        Ok(())
    }
}
