use crate::db::Database;
use crate::models::bookmark::ImportResult;
use crate::search::TantivySearchEngine;
use crate::services::{
    chrome_importer::ChromeImporter, firefox_importer::FirefoxImporter,
    safari_importer::SafariImporter, data_service::DataService,
};
use std::sync::{Arc, Mutex};
use tauri::State;

pub struct AppState {
    pub db: Mutex<Database>,
    pub search_engine: Arc<TantivySearchEngine>,
    pub data_service: Arc<DataService>,
}

/// Helper function to rebuild bookmark index after import
fn rebuild_bookmark_index_after_import(state: &State<AppState>) -> Result<(), String> {
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
        // db lock released here automatically
    };

    state.search_engine
        .rebuild_bookmark_index_from_data(bookmarks)
        .map_err(|e| format!("Failed to rebuild index: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn add_bookmark(
    state: State<AppState>,
    title: String,
    url: String,
    description: Option<String>,
    tags: Option<String>,
) -> Result<i64, String> {
    // Use DataService for atomic DB + Index operations
    state.data_service.add_bookmark(title, url, description, tags)
        .map_err(|e| e.to_string())
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
    // Use DataService for atomic DB + Index operations
    state.data_service.update_bookmark(id, title, url, description, tags)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_bookmark(state: State<AppState>, id: i64) -> Result<(), String> {
    // Use DataService for atomic DB + Index operations
    state.data_service.delete_bookmark(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_chrome_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to acquire database lock: {}", e))?;
    let conn = db.get_connection();
    let result = ChromeImporter::import(conn)?;
    drop(db);

    if result.imported > 0 {
        if let Err(e) = rebuild_bookmark_index_after_import(&state) {
            eprintln!(
                "Warning: Failed to rebuild bookmark index after Chrome import: {}",
                e
            );
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn import_firefox_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to acquire database lock: {}", e))?;
    let conn = db.get_connection();
    let result = FirefoxImporter::import(conn)?;
    drop(db);

    if result.imported > 0 {
        if let Err(e) = rebuild_bookmark_index_after_import(&state) {
            eprintln!(
                "Warning: Failed to rebuild bookmark index after Firefox import: {}",
                e
            );
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn import_safari_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Failed to acquire database lock: {}", e))?;
    let conn = db.get_connection();
    let result = SafariImporter::import(conn)?;
    drop(db);

    if result.imported > 0 {
        if let Err(e) = rebuild_bookmark_index_after_import(&state) {
            eprintln!(
                "Warning: Failed to rebuild bookmark index after Safari import: {}",
                e
            );
        }
    }

    Ok(result)
}
