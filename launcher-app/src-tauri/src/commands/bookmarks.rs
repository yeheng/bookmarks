use crate::db::Database;
use crate::models::bookmark::ImportResult;
use crate::services::{
    chrome_importer::ChromeImporter, firefox_importer::FirefoxImporter,
    safari_importer::SafariImporter,
};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

pub struct AppState {
    pub db: Mutex<Database>,
}

#[tauri::command]
pub fn add_bookmark(
    state: State<AppState>,
    title: String,
    url: String,
    description: Option<String>,
    tags: Option<String>,
) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let existing: Result<i64, _> =
        conn.query_row("SELECT id FROM bookmarks WHERE url = ?1", [&url], |row| {
            row.get(0)
        });

    if existing.is_ok() {
        return Err("Bookmark with this URL already exists".to_string());
    }

    conn.execute(
        "INSERT INTO bookmarks (title, url, description, tags, source, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![title, url, description, tags, "manual", now, now],
    )
    .map_err(|e| format!("Failed to insert bookmark: {}", e))?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_bookmark(
    state: State<AppState>,
    id: i64,
    title: String,
    url: String,
    description: Option<String>,
    tags: Option<String>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let rows_affected = conn
        .execute(
            "UPDATE bookmarks SET title = ?1, url = ?2, description = ?3, tags = ?4, updated_at = ?5 WHERE id = ?6",
            rusqlite::params![title, url, description, tags, now, id],
        )
        .map_err(|e| format!("Failed to update bookmark: {}", e))?;

    if rows_affected == 0 {
        return Err("Bookmark not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn delete_bookmark(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let rows_affected = conn
        .execute("DELETE FROM bookmarks WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to delete bookmark: {}", e))?;

    if rows_affected == 0 {
        return Err("Bookmark not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn import_chrome_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();
    ChromeImporter::import(conn)
}

#[tauri::command]
pub fn import_firefox_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();
    FirefoxImporter::import(conn)
}

#[tauri::command]
pub fn import_safari_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();
    SafariImporter::import(conn)
}
