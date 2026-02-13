use crate::commands::bookmarks::AppState;
use crate::models::bookmark::BookmarkSearchResult;
use crate::search::provider::SourceType;
use crate::search::{IndexStats, SearchContext};
use crate::error::AppError;
use serde::Serialize;
use tauri::State;

/// Unified search result returned to the frontend.
///
/// Contains all fields needed to render any result type (bookmark, file, plugin).
/// The frontend maps this to its existing `SearchResult` type.
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedSearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub source_type: SourceType,
    pub source_id: String,
    pub score: f64,
    pub frecency_score: f64,
    pub icon: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
    pub favicon_url: Option<String>,
    pub description: Option<String>,
    pub extension: Option<String>,
    pub size: Option<i64>,
    pub modified_at: Option<i64>,
    pub plugin_actions: Option<Vec<serde_json::Value>>,
    pub plugin_badge: Option<String>,
    pub plugin_keyword: Option<String>,
}

/// Unified search across all registered providers.
///
/// Reads `SearchSettings` from DB to determine enabled sources and limits,
/// then delegates to `SearchAggregator` for parallel multi-source search.
#[tauri::command]
pub async fn unified_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    sources: Option<Vec<String>>,
) -> Result<Vec<UnifiedSearchResult>, String> {
    // Read search settings from DB
    let search_settings = state.data_service.with_db(|conn| {
        let max_results: usize = conn
            .query_row("SELECT value FROM settings WHERE key = 'search.max_results'", [], |row| row.get::<_, String>(0))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let show_bookmarks: bool = conn
            .query_row("SELECT value FROM settings WHERE key = 'search.show_bookmarks'", [], |row| row.get::<_, String>(0))
            .ok()
            .map(|s| s == "true")
            .unwrap_or(true);
        let show_files: bool = conn
            .query_row("SELECT value FROM settings WHERE key = 'search.show_files'", [], |row| row.get::<_, String>(0))
            .ok()
            .map(|s| s == "true")
            .unwrap_or(true);
        let fuzzy_matching: bool = conn
            .query_row("SELECT value FROM settings WHERE key = 'search.fuzzy_matching'", [], |row| row.get::<_, String>(0))
            .ok()
            .map(|s| s == "true")
            .unwrap_or(true);

        Ok((max_results, show_bookmarks, show_files, fuzzy_matching))
    }).map_err(|e: AppError| e.to_string())?;

    let (max_results, show_bookmarks, show_files, fuzzy_matching) = search_settings;
    let effective_limit = limit.unwrap_or(max_results);

    // Build source filter from settings + explicit sources argument
    let effective_sources = if let Some(explicit) = sources {
        Some(explicit)
    } else {
        let mut enabled = Vec::new();
        if show_bookmarks { enabled.push("bookmarks".to_string()); }
        if show_files { enabled.push("files".to_string()); }
        // Always include plugin sources when available
        enabled.push("plugins".to_string());
        Some(enabled)
    };

    let ctx = SearchContext {
        query,
        limit: effective_limit,
        fuzzy: fuzzy_matching,
        sources: effective_sources,
    };

    let results = state.search_aggregator.search(&ctx).await
        .map_err(|e| format!("Unified search failed: {}", e))?;

    Ok(results
        .into_iter()
        .map(|r| UnifiedSearchResult {
            id: r.id,
            title: r.title,
            subtitle: r.subtitle,
            source_type: r.source_type,
            source_id: r.source_id,
            score: r.score,
            frecency_score: r.frecency_score,
            icon: r.icon,
            url: r.url,
            path: r.path,
            favicon_url: r.favicon_url,
            description: r.description,
            extension: r.extension,
            size: r.size,
            modified_at: r.modified_at,
            plugin_actions: r.plugin_actions,
            plugin_badge: r.plugin_badge,
            plugin_keyword: r.plugin_keyword,
        })
        .collect())
}

/// Search bookmarks with query string.
///
/// **DEPRECATED**: Use `unified_search` instead. This command is kept for
/// backward compatibility and will be removed in a future version.

#[tauri::command]
pub fn search_bookmarks(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<BookmarkSearchResult>, String> {
    let limit = limit.unwrap_or(10);

    let results = state
        .data_service
        .search_engine()
        .search_bookmarks(&query, limit)
        .map_err(|e| format!("Search failed: {}", e))?;

    Ok(results)
}

#[tauri::command]
pub fn record_bookmark_access(state: State<AppState>, bookmark_id: i64) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {}", e))?
        .as_secs() as i64;

    // Update SQLite and get the new access count
    let access_count = state.data_service.with_db(|conn| {
        conn.execute(
            "UPDATE bookmarks SET last_accessed = ?1 WHERE id = ?2",
            rusqlite::params![now, bookmark_id],
        )
        .map_err(|e| AppError::Generic(format!("Failed to update last_accessed: {}", e)))?;

        conn.execute(
            "INSERT INTO usage_history (bookmark_id, accessed_at) VALUES (?1, ?2)",
            rusqlite::params![bookmark_id, now],
        )
        .map_err(|e| AppError::Generic(format!("Failed to insert usage history: {}", e)))?;

        // Get the total access count for this bookmark
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM usage_history WHERE bookmark_id = ?1",
                [bookmark_id],
                |row| row.get(0),
            )
            .unwrap_or(1);

        Ok(count)
    }).map_err(|e| e.to_string())?;

    // Update Tantivy index with new frecency data (fire and forget for UI responsiveness)
    let _ = state.data_service.search_engine().update_bookmark_frecency(bookmark_id, access_count, now);

    Ok(())
}

#[tauri::command]
pub fn rebuild_search_index(state: State<AppState>) -> Result<(usize, usize), String> {
    // Fetch both bookmark and file data in a single lock acquisition
    let (bookmarks, files) = state.data_service.with_db(|conn| {
        // Fetch all bookmarks
        let mut stmt = conn
            .prepare("SELECT id, title, url, description, tags, last_accessed, created_at, updated_at FROM bookmarks")
            .map_err(|e| AppError::Generic(format!("Failed to prepare query: {}", e)))?;

        let bookmarks: Result<Vec<_>, _> = stmt
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
            .map_err(|e| AppError::Generic(format!("Failed to query bookmarks: {}", e)))?
            .collect();

        let bookmarks = bookmarks.map_err(|e| AppError::Generic(format!("Failed to collect bookmarks: {}", e)))?;

        // Fetch all files
        let mut stmt = conn
            .prepare("SELECT id, path, name, extension, size, modified_at, directory_id FROM indexed_files")
            .map_err(|e| AppError::Generic(format!("Failed to prepare query: {}", e)))?;

        let files: Result<Vec<_>, _> = stmt
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
            .map_err(|e| AppError::Generic(format!("Failed to query files: {}", e)))?
            .collect();

        let files = files.map_err(|e| AppError::Generic(format!("Failed to collect files: {}", e)))?;

        Ok((bookmarks, files))
    }).map_err(|e| e.to_string())?;

    // Rebuild indexes with data (lock already released)
    let bookmark_count = state
        .data_service
        .search_engine()
        .rebuild_bookmark_index_from_data(bookmarks)
        .map_err(|e| format!("Failed to rebuild bookmark index: {}", e))?;

    let file_count = state
        .data_service
        .search_engine()
        .rebuild_file_index_from_data(files)
        .map_err(|e| format!("Failed to rebuild file index: {}", e))?;

    Ok((bookmark_count, file_count))
}

#[tauri::command]
pub fn get_search_stats(state: State<AppState>) -> Result<IndexStats, String> {
    state
        .data_service
        .search_engine()
        .get_stats()
        .map_err(|e| format!("Failed to get search stats: {}", e))
}
