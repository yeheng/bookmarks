use crate::commands::bookmarks::AppState;
use crate::models::file::FileSearchResult;
use tauri::State;

/// Search files with query string.
#[tauri::command]
pub fn search_files(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<FileSearchResult>, String> {
    let limit = limit.unwrap_or(10);

    state
        .data_service
        .search_engine()
        .search_files(&query, limit, None)
        .map_err(|e| format!("Search failed: {}", e))
}

#[tauri::command]
pub fn search_files_by_extension(
    state: State<AppState>,
    extension: String,
    limit: Option<usize>,
) -> Result<Vec<FileSearchResult>, String> {
    let limit = limit.unwrap_or(10);
    let ext = extension.trim_start_matches('.');

    state
        .data_service
        .search_engine()
        .search_files("", limit, Some(ext))
        .map_err(|e| format!("Search failed: {}", e))
}

#[tauri::command]
pub fn record_file_access(state: State<AppState>, file_id: i64) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let _ = state
        .data_service
        .search_engine()
        .update_file_frecency(file_id, 1, now);

    Ok(())
}

#[tauri::command]
pub fn get_file_by_id(
    state: State<AppState>,
    file_id: i64,
) -> Result<Option<FileSearchResult>, String> {
    // Search all files and filter by ID — not ideal but sufficient for single lookups.
    // A large limit is used since we're looking for a specific file.
    let results = state
        .data_service
        .search_engine()
        .search_files("", 10000, None)
        .map_err(|e| format!("Failed to search files: {}", e))?;

    Ok(results.into_iter().find(|f| f.id == file_id))
}
