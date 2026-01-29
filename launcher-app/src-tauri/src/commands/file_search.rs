use crate::commands::bookmarks::AppState;
use crate::models::file::FileSearchResult;
use tauri::State;

#[tauri::command]
pub fn search_files(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<FileSearchResult>, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let limit = limit.unwrap_or(10);

    if query.trim().is_empty() {
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.path, f.name, f.extension, f.size, f.modified_at, 0.0 as score
                 FROM indexed_files f
                 LEFT JOIN file_usage_history fuh ON f.id = fuh.file_id
                 GROUP BY f.id
                 ORDER BY MAX(fuh.accessed_at) DESC NULLS LAST, f.modified_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let results = stmt
            .query_map([limit], |row| {
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
            .map_err(|e| format!("Failed to execute query: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(results)
    } else {
        let search_query = prepare_fuzzy_query(&query);

        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.path, f.name, f.extension, f.size, f.modified_at,
                        bm25(files_fts) as fts_score,
                        COALESCE(
                            (SELECT COUNT(*) * 0.3 +
                             (julianday('now') - julianday(MAX(accessed_at), 'unixepoch')) * -0.1
                             FROM file_usage_history WHERE file_id = f.id),
                            0
                        ) as frecency_score
                 FROM files_fts
                 JOIN indexed_files f ON files_fts.rowid = f.id
                 WHERE files_fts MATCH ?1
                 ORDER BY (fts_score * 0.7 + frecency_score * 0.3) DESC
                 LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare search query: {}", e))?;

        let results = stmt
            .query_map(rusqlite::params![&search_query, limit], |row| {
                let fts_score: f64 = row.get(6)?;
                let frecency: f64 = row.get(7)?;
                Ok(FileSearchResult {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    name: row.get(2)?,
                    extension: row.get(3)?,
                    size: row.get(4)?,
                    modified_at: row.get(5)?,
                    score: fts_score * 0.7 + frecency * 0.3,
                    frecency_score: frecency,
                })
            })
            .map_err(|e| format!("Failed to execute search: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect search results: {}", e))?;

        Ok(results)
    }
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
        .query_map(rusqlite::params![ext, limit], |row| {
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

fn prepare_fuzzy_query(query: &str) -> String {
    let terms: Vec<&str> = query.split_whitespace().collect();

    if terms.is_empty() {
        return String::new();
    }

    let fuzzy_terms: Vec<String> = terms
        .iter()
        .map(|term| {
            let escaped = term.replace('"', "");
            if escaped.len() >= 2 {
                format!("{}*", escaped)
            } else {
                escaped
            }
        })
        .collect();

    fuzzy_terms.join(" ")
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
