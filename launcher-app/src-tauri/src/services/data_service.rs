/// Data Service Layer
///
/// This service provides atomic operations that coordinate writes between
/// SQLite (source of truth) and Tantivy (search index).
///
/// Design principles:
/// 1. SQLite is the authoritative source of truth
/// 2. Index updates should NOT block database commits
/// 3. Failed index updates are logged and recovered on next startup
/// 4. No defensive "rollback" logic - we trust our write path

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::search::{SearchEngine, TantivySearchEngine};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Tracks pending index operations that need to be applied
#[derive(Debug)]
pub struct PendingIndexOp {
    pub bookmark_id: i64,
    pub operation: IndexOperation,
}

#[derive(Debug)]
pub enum IndexOperation {
    Index,
    Delete,
}

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
    /// 2. Mark as "needs_indexing" in a recovery table
    /// 3. Update Tantivy index
    /// 4. Clear "needs_indexing" flag
    ///
    /// If step 3 fails, the recovery table will have the record,
    /// and we'll re-index on next startup.
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

        // Write to SQLite (source of truth) in a transaction
        let tx = conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO bookmarks (title, url, description, tags, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![title, url, description, tags, "manual", now, now],
        )?;

        let id = tx.last_insert_rowid();

        // Mark as needing indexing (for crash recovery)
        tx.execute(
            "INSERT OR REPLACE INTO pending_index_ops (bookmark_id, operation, created_at)
             VALUES (?1, 'index', ?2)",
            rusqlite::params![id, now],
        )?;

        tx.commit()?;

        // Release DB lock before indexing (Tantivy operations are slow)
        drop(db);

        // Update Tantivy index (non-blocking, best-effort)
        // If this fails, the pending_index_ops table will have the record
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
                "[DataService] Warning: Failed to index bookmark {}: {}. Will retry on next startup.",
                id, e
            );
            // DO NOT rollback the database - SQLite is the source of truth
            // The pending_index_ops table will ensure we re-index on next startup
        } else {
            // Indexing succeeded, clear the pending flag
            if let Ok(db) = self.db.lock() {
                let conn = db.get_connection();
                let _ = conn.execute(
                    "DELETE FROM pending_index_ops WHERE bookmark_id = ?1",
                    [id],
                );
            }
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

        let tx = conn.unchecked_transaction()?;

        let rows_affected = tx.execute(
            "UPDATE bookmarks SET title = ?1, url = ?2, description = ?3, tags = ?4, updated_at = ?5 WHERE id = ?6",
            rusqlite::params![title, url, description, tags, now, id],
        )?;

        if rows_affected == 0 {
            return Err(AppError::BookmarkNotFound);
        }

        // Mark as needing re-indexing
        tx.execute(
            "INSERT OR REPLACE INTO pending_index_ops (bookmark_id, operation, created_at)
             VALUES (?1, 'index', ?2)",
            rusqlite::params![id, now],
        )?;

        tx.commit()?;

        drop(db);

        // Re-index in Tantivy
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
        } else {
            if let Ok(db) = self.db.lock() {
                let conn = db.get_connection();
                let _ = conn.execute(
                    "DELETE FROM pending_index_ops WHERE bookmark_id = ?1",
                    [id],
                );
            }
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

        let tx = conn.unchecked_transaction()?;

        let rows_affected = tx.execute("DELETE FROM bookmarks WHERE id = ?1", [id])?;

        if rows_affected == 0 {
            return Err(AppError::BookmarkNotFound);
        }

        // Mark as needing deletion from index
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;

        tx.execute(
            "INSERT OR REPLACE INTO pending_index_ops (bookmark_id, operation, created_at)
             VALUES (?1, 'delete', ?2)",
            rusqlite::params![id, now],
        )?;

        tx.commit()?;

        drop(db);

        // Remove from Tantivy index
        if let Err(e) = self.search_engine.delete_bookmark(id) {
            eprintln!(
                "[DataService] Warning: Failed to remove bookmark {} from index: {}",
                id, e
            );
        } else {
            if let Ok(db) = self.db.lock() {
                let conn = db.get_connection();
                let _ = conn.execute(
                    "DELETE FROM pending_index_ops WHERE bookmark_id = ?1",
                    [id],
                );
            }
        }

        Ok(())
    }

    /// Recover pending index operations on startup
    /// This ensures that any failed index updates from previous runs are applied
    pub fn recover_pending_operations(&self) -> AppResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| AppError::DatabaseLock)?;
        let conn = db.get_connection();

        let pending: Vec<(i64, String)> = conn
            .prepare("SELECT bookmark_id, operation FROM pending_index_ops ORDER BY created_at")?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        if pending.is_empty() {
            return Ok(());
        }

        println!(
            "[DataService] Recovering {} pending index operations",
            pending.len()
        );

        drop(db);

        for (bookmark_id, operation) in pending {
            match operation.as_str() {
                "index" => {
                    // Re-fetch bookmark data and index it
                    if let Ok(db) = self.db.lock() {
                        let conn = db.get_connection();
                        if let Ok((title, url, description, tags, created_at, updated_at, last_accessed)) = conn.query_row(
                            "SELECT title, url, description, tags, created_at, updated_at, last_accessed FROM bookmarks WHERE id = ?1",
                            [bookmark_id],
                            |row| {
                                Ok((
                                    row.get::<_, String>(0)?,
                                    row.get::<_, String>(1)?,
                                    row.get::<_, Option<String>>(2)?,
                                    row.get::<_, Option<String>>(3)?,
                                    row.get::<_, i64>(4)?,
                                    row.get::<_, i64>(5)?,
                                    row.get::<_, Option<i64>>(6)?,
                                ))
                            },
                        ) {
                            if let Err(e) = self.search_engine.index_bookmark(
                                bookmark_id,
                                &title,
                                &url,
                                description.as_deref(),
                                tags.as_deref(),
                                last_accessed,
                                created_at,
                                updated_at,
                            ) {
                                eprintln!(
                                    "[DataService] Failed to recover index for bookmark {}: {}",
                                    bookmark_id, e
                                );
                                continue;
                            }
                        }
                    }
                }
                "delete" => {
                    if let Err(e) = self.search_engine.delete_bookmark(bookmark_id) {
                        eprintln!(
                            "[DataService] Failed to recover deletion for bookmark {}: {}",
                            bookmark_id, e
                        );
                        continue;
                    }
                }
                _ => {
                    eprintln!(
                        "[DataService] Unknown operation '{}' for bookmark {}",
                        operation, bookmark_id
                    );
                    continue;
                }
            }

            // Clear the pending op
            if let Ok(db) = self.db.lock() {
                let conn = db.get_connection();
                let _ = conn.execute(
                    "DELETE FROM pending_index_ops WHERE bookmark_id = ?1",
                    [bookmark_id],
                );
            }
        }

        println!("[DataService] Recovery completed");
        Ok(())
    }
}

/// Initialize the pending_index_ops table in the database
pub fn ensure_pending_ops_table(conn: &Connection) -> AppResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pending_index_ops (
            bookmark_id INTEGER PRIMARY KEY,
            operation TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(())
}
