use crate::commands::bookmarks::AppState;
use crate::models::bookmark::BookmarkSearchResult;
use crate::search::IndexStats;
use tauri::State;

#[tauri::command]
pub fn search_bookmarks(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<BookmarkSearchResult>, String> {
    let limit = limit.unwrap_or(10);

    let results = state
        .search_engine
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
    let access_count = {
        let db = state.db.lock().map_err(|e| format!("Failed to acquire database lock: {}", e))?;
        let conn = db.get_connection();

        conn.execute(
            "UPDATE bookmarks SET last_accessed = ?1 WHERE id = ?2",
            rusqlite::params![now, bookmark_id],
        )
        .map_err(|e| format!("Failed to update last_accessed: {}", e))?;

        conn.execute(
            "INSERT INTO usage_history (bookmark_id, accessed_at) VALUES (?1, ?2)",
            rusqlite::params![bookmark_id, now],
        )
        .map_err(|e| format!("Failed to insert usage history: {}", e))?;

        // Get the total access count for this bookmark
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM usage_history WHERE bookmark_id = ?1",
                [bookmark_id],
                |row| row.get(0),
            )
            .unwrap_or(1);

        count
        // DB lock released here
    };

    // Update Tantivy index with new frecency data (fire and forget for UI responsiveness)
    let _ = state.search_engine.update_bookmark_frecency(bookmark_id, access_count, now);

    Ok(())
}

#[tauri::command]
pub fn rebuild_search_index(state: State<AppState>) -> Result<(usize, usize), String> {
    // Fetch bookmark data from database
    let bookmarks = {
        let db = state.db.lock().map_err(|e| format!("Failed to acquire database lock: {}", e))?;
        let conn = db.get_connection();

        let mut stmt = conn
            .prepare("SELECT id, title, url, description, tags, last_accessed, created_at, updated_at FROM bookmarks")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

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
            .map_err(|e| format!("Failed to query bookmarks: {}", e))?
            .collect();

        bookmarks.map_err(|e| format!("Failed to collect bookmarks: {}", e))?
    };

    // Fetch file data from database
    let files = {
        let db = state.db.lock().map_err(|e| format!("Failed to acquire database lock: {}", e))?;
        let conn = db.get_connection();

        let mut stmt = conn
            .prepare("SELECT id, path, name, extension, size, modified_at, directory_id FROM files")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

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
            .map_err(|e| format!("Failed to query files: {}", e))?
            .collect();

        files.map_err(|e| format!("Failed to collect files: {}", e))?
    };

    // Rebuild indexes with data
    let bookmark_count = state
        .search_engine
        .rebuild_bookmark_index_from_data(bookmarks)
        .map_err(|e| format!("Failed to rebuild bookmark index: {}", e))?;

    let file_count = state
        .search_engine
        .rebuild_file_index_from_data(files)
        .map_err(|e| format!("Failed to rebuild file index: {}", e))?;

    Ok((bookmark_count, file_count))
}

#[tauri::command]
pub fn get_search_stats(state: State<AppState>) -> Result<IndexStats, String> {
    state
        .search_engine
        .get_stats()
        .map_err(|e| format!("Failed to get search stats: {}", e))
}
