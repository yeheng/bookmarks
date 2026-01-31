use crate::commands::bookmarks::AppState;
use crate::models::bookmark::BookmarkSearchResult;
use crate::search::{SearchEngine, IndexStats};
use tauri::State;

#[tauri::command]
pub fn search_bookmarks(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<BookmarkSearchResult>, String> {
    let limit = limit.unwrap_or(10);

    state
        .search_engine
        .search_bookmarks(&query, limit)
        .map_err(|e| format!("Search failed: {}", e))
}

#[tauri::command]
pub fn record_bookmark_access(state: State<AppState>, bookmark_id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("Failed to acquire database lock: {}", e))?;
    let conn = db.get_connection();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {}", e))?
        .as_secs() as i64;

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

    Ok(())
}

#[tauri::command]
pub fn rebuild_search_index(state: State<AppState>) -> Result<(usize, usize), String> {
    let bookmark_count = state
        .search_engine
        .rebuild_bookmark_index()
        .map_err(|e| format!("Failed to rebuild bookmark index: {}", e))?;

    let file_count = state
        .search_engine
        .rebuild_file_index()
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
