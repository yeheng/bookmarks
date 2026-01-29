use crate::commands::bookmarks::AppState;
use crate::models::bookmark::BookmarkSearchResult;
use tauri::State;

#[tauri::command]
pub fn search_bookmarks(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<BookmarkSearchResult>, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let limit = limit.unwrap_or(10);

    if query.trim().is_empty() {
        let mut stmt = conn
            .prepare(
                "SELECT b.id, b.title, b.url, b.description, b.favicon_url, 0.0 as score
                 FROM bookmarks b
                 LEFT JOIN usage_history uh ON b.id = uh.bookmark_id
                 GROUP BY b.id
                 ORDER BY MAX(uh.accessed_at) DESC NULLS LAST
                 LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let results = stmt
            .query_map([limit], |row| {
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
            .map_err(|e| format!("Failed to execute query: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(results)
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT b.id, b.title, b.url, b.description, b.favicon_url,
                        bm25(bookmarks_fts) as fts_score,
                        COALESCE(
                            (SELECT COUNT(*) * 0.3 + 
                             (julianday('now') - julianday(MAX(accessed_at), 'unixepoch')) * -0.1
                             FROM usage_history WHERE bookmark_id = b.id),
                            0
                        ) as frecency_score
                 FROM bookmarks_fts
                 JOIN bookmarks b ON bookmarks_fts.rowid = b.id
                 WHERE bookmarks_fts MATCH ?1
                 ORDER BY (fts_score * 0.7 + frecency_score * 0.3) DESC
                 LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare search query: {}", e))?;

        let results = stmt
            .query_map(rusqlite::params![&query, limit], |row| {
                let fts_score: f64 = row.get(5)?;
                let frecency: f64 = row.get(6)?;
                Ok(BookmarkSearchResult {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    description: row.get(3)?,
                    favicon_url: row.get(4)?,
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
pub fn record_bookmark_access(state: State<AppState>, bookmark_id: i64) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
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
