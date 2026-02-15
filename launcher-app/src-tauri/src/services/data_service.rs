/// Data Service Layer
///
/// This service provides atomic operations that coordinate writes between
/// JSON stores (source of truth) and Tantivy (search index).
///
/// Design principles:
/// 1. JSON files are the authoritative source of truth for bookmarks/directories
/// 2. Tantivy index is a cache - if it's corrupted, rebuild from JSON stores
/// 3. No complex recovery mechanisms - just rebuild_if_needed on startup
/// 4. Direct index updates, no batching needed for desktop app

use crate::error::{AppError, AppResult};
use crate::search::TantivySearchEngine;
use crate::store::{BookmarkStore, DirectoryStore, SettingsStore, PluginStore};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Type alias for bookmark data tuple used for Tantivy indexing.
pub type BookmarkData = (
    i64,              // id
    String,           // title
    String,           // url
    Option<String>,   // description
    Option<String>,   // tags
    Option<i64>,      // last_accessed
    i64,              // created_at
    i64,              // updated_at
);

/// Type alias for file data tuple used for Tantivy indexing.
pub type FileData = (
    i64,              // id
    String,           // path
    String,           // name
    Option<String>,   // extension
    i64,              // size
    i64,              // modified_at
    i64,              // directory_id
);

pub struct DataService {
    bookmark_store: RwLock<BookmarkStore>,
    directory_store: RwLock<DirectoryStore>,
    settings_store: RwLock<SettingsStore>,
    plugin_store: RwLock<PluginStore>,
    search_engine: Arc<TantivySearchEngine>,
}

impl DataService {
    pub fn new(
        bookmark_store: BookmarkStore,
        directory_store: DirectoryStore,
        settings_store: SettingsStore,
        plugin_store: PluginStore,
        search_engine: Arc<TantivySearchEngine>,
    ) -> Self {
        Self {
            bookmark_store: RwLock::new(bookmark_store),
            directory_store: RwLock::new(directory_store),
            settings_store: RwLock::new(settings_store),
            plugin_store: RwLock::new(plugin_store),
            search_engine,
        }
    }

    /// Get access to the search engine.
    pub fn search_engine(&self) -> &Arc<TantivySearchEngine> {
        &self.search_engine
    }

    // ── Bookmark operations ──────────────────────────────────────────

    pub fn with_bookmark_store<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&BookmarkStore) -> AppResult<T>,
    {
        let store = self.bookmark_store.read().map_err(|_| AppError::StoreLock)?;
        f(&store)
    }

    pub fn with_bookmark_store_mut<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut BookmarkStore) -> AppResult<T>,
    {
        let mut store = self.bookmark_store.write().map_err(|_| AppError::StoreLock)?;
        f(&mut store)
    }

    /// Add a bookmark with atomic store + index update.
    pub fn add_bookmark(
        &self,
        title: String,
        url: String,
        description: Option<String>,
        tags: Option<String>,
    ) -> AppResult<i64> {
        let id = {
            let mut store = self.bookmark_store.write().map_err(|_| AppError::StoreLock)?;
            store
                .add(title.clone(), url.clone(), description.clone(), tags.clone(), "manual".to_string())
                .map_err(|e| AppError::DuplicateBookmark(e))?
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;

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
        }

        Ok(id)
    }

    /// Update a bookmark atomically.
    pub fn update_bookmark(
        &self,
        id: i64,
        title: String,
        url: String,
        description: Option<String>,
        tags: Option<String>,
    ) -> AppResult<()> {
        let (created_at, last_accessed) = {
            let store = self.bookmark_store.read().map_err(|_| AppError::StoreLock)?;
            let bookmark = store.find_by_id(id).ok_or(AppError::BookmarkNotFound)?;
            (bookmark.created_at, bookmark.last_accessed)
        };

        {
            let mut store = self.bookmark_store.write().map_err(|_| AppError::StoreLock)?;
            store.update(id, title.clone(), url.clone(), description.clone(), tags.clone())
                .map_err(|e| AppError::Generic(e))?;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;

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

    /// Delete a bookmark atomically.
    pub fn delete_bookmark(&self, id: i64) -> AppResult<()> {
        {
            let mut store = self.bookmark_store.write().map_err(|_| AppError::StoreLock)?;
            store.delete(id).map_err(|e| AppError::Generic(e))?;
        }

        // Remove from Tantivy index (best-effort)
        if let Err(e) = self.search_engine.delete_bookmark(id) {
            eprintln!(
                "[DataService] Warning: Failed to remove bookmark {} from index: {}",
                id, e
            );
        }

        Ok(())
    }

    // ── Directory operations ─────────────────────────────────────────

    pub fn with_directory_store<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&DirectoryStore) -> AppResult<T>,
    {
        let store = self.directory_store.read().map_err(|_| AppError::StoreLock)?;
        f(&store)
    }

    pub fn with_directory_store_mut<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut DirectoryStore) -> AppResult<T>,
    {
        let mut store = self.directory_store.write().map_err(|_| AppError::StoreLock)?;
        f(&mut store)
    }

    // ── Settings operations ──────────────────────────────────────────

    pub fn with_settings_store<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&SettingsStore) -> AppResult<T>,
    {
        let store = self.settings_store.read().map_err(|_| AppError::StoreLock)?;
        f(&store)
    }

    pub fn with_settings_store_mut<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut SettingsStore) -> AppResult<T>,
    {
        let mut store = self.settings_store.write().map_err(|_| AppError::StoreLock)?;
        f(&mut store)
    }

    // ── Plugin operations ────────────────────────────────────────────

    pub fn with_plugin_store<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&PluginStore) -> AppResult<T>,
    {
        let store = self.plugin_store.read().map_err(|_| AppError::StoreLock)?;
        f(&store)
    }

    pub fn with_plugin_store_mut<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut PluginStore) -> AppResult<T>,
    {
        let mut store = self.plugin_store.write().map_err(|_| AppError::StoreLock)?;
        f(&mut store)
    }

    // ── Index operations ─────────────────────────────────────────────

    /// Get all bookmark data for index rebuilding.
    pub fn get_all_index_data(&self) -> AppResult<(Vec<BookmarkData>, Vec<FileData>)> {
        let bookmarks = {
            let store = self.bookmark_store.read().map_err(|_| AppError::StoreLock)?;
            store.get_index_data()
        };

        // Files are stored in Tantivy only; for rebuild, we re-scan from filesystem.
        // Return empty file list — caller should re-scan directories.
        Ok((bookmarks, Vec::new()))
    }

    /// Check index integrity and rebuild if needed.
    pub fn rebuild_index_if_needed(&self) -> AppResult<bool> {
        let db_bookmark_count = {
            let store = self.bookmark_store.read().map_err(|_| AppError::StoreLock)?;
            store.count() as i64
        };

        let stats = self.search_engine.get_stats()
            .map_err(|e| AppError::Search(e.to_string()))?;

        let index_bookmark_count = stats.bookmark_count as i64;
        let bookmark_diff = (db_bookmark_count - index_bookmark_count).abs();

        let needs_rebuild =
            (db_bookmark_count > 0 && bookmark_diff > db_bookmark_count / 10) ||
            bookmark_diff > 10;

        if needs_rebuild {
            println!(
                "[DataService] Index mismatch detected. Store: {} bookmarks. Index: {} bookmarks. Rebuilding...",
                db_bookmark_count, index_bookmark_count
            );
            self.rebuild_bookmark_index()?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Rebuild bookmark index from JSON store.
    fn rebuild_bookmark_index(&self) -> AppResult<()> {
        let bookmarks = {
            let store = self.bookmark_store.read().map_err(|_| AppError::StoreLock)?;
            store.get_index_data()
        };

        let count = self.search_engine.rebuild_bookmark_index_from_data(bookmarks)
            .map_err(|e| AppError::Search(e.to_string()))?;
        println!("[DataService] Rebuilt bookmark index with {} entries", count);

        Ok(())
    }
}
