use crate::commands::bookmarks::AppState;
use crate::models::file::FileSearchResult;
use crate::search::SearchEngine;
use tauri::State;

#[tauri::command]
pub fn search_files(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<FileSearchResult>, String> {
    let limit = limit.unwrap_or(10);

    state
        .search_engine
        .search_files(&query, limit)
        .map_err(|e| format!("Search failed: {}", e))
}

#[tauri::command]
pub fn search_files_by_extension(
    state: State<AppState>,
    extension: String,
    limit: Option<usize>,
) -> Result<Vec<FileSearchResult>, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let limit = limit.unwrap_or(10);
    let ext = extension.trim_start_matches('.');

    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.path, f.name, f.extension, f.size, f.modified_at,
                    COALESCE(
                        (SELECT COUNT(*) * 0.3 +
                         (julianday('now') - julianday(MAX(accessed_at), 'unixepoch')) * -0.1
                         FROM file_usage_history WHERE file_id = f.id),
                        0
                    ) as frecency_score
             FROM indexed_files f
             WHERE f.extension = ?1
             ORDER BY frecency_score DESC, f.modified_at DESC
             LIMIT ?2",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let results = stmt
        .query_map(rusqlite::params![ext, limit as i64], |row| {
            let frecency: f64 = row.get(6)?;
            Ok(FileSearchResult {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                extension: row.get(3)?,
                size: row.get(4)?,
                modified_at: row.get(5)?,
                score: frecency,
                frecency_score: frecency,
            })
        })
        .map_err(|e| format!("Failed to execute query: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

    Ok(results)
}

#[tauri::command]
pub fn record_file_access(state: State<AppState>, file_id: i64) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO file_usage_history (file_id, accessed_at) VALUES (?1, ?2)",
        rusqlite::params![file_id, now],
    )
    .map_err(|e| format!("Failed to insert file usage history: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_file_by_id(
    state: State<AppState>,
    file_id: i64,
) -> Result<Option<FileSearchResult>, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let result = conn
        .query_row(
            "SELECT id, path, name, extension, size, modified_at
             FROM indexed_files WHERE id = ?1",
            [file_id],
            |row| {
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
            },
        )
        .ok();

    Ok(result)
}
