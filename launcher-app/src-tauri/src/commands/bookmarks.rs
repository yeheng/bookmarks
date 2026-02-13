use crate::models::bookmark::ImportResult;
use crate::plugins::executor::PluginExecutor;
use crate::plugins::registry::PluginRegistry;
use crate::search::SearchAggregator;
use crate::services::{
    chrome_importer::ChromeImporter, data_service::DataService, firefox_importer::FirefoxImporter,
    safari_importer::SafariImporter,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

pub struct AppState {
    pub data_service: Arc<DataService>,
    pub http_client: reqwest::Client,
    pub plugin_registry: Option<Arc<PluginRegistry>>,
    pub plugin_executor: Option<Arc<PluginExecutor>>,
    pub search_aggregator: Arc<SearchAggregator>,
    /// Cached search settings for performance optimization.
    /// Key: setting key, Value: setting value
    pub settings_cache: std::sync::RwLock<HashMap<String, String>>,
}

/// Validate URL format.
/// Returns Ok(()) if valid, Err(message) if invalid.
fn validate_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }
    if url.len() > 4096 {
        return Err("URL is too long (max 4096 characters)".to_string());
    }
    // Basic URL scheme validation
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("file://") {
        return Err("URL must start with http://, https://, or file://".to_string());
    }
    Ok(())
}

/// Validate bookmark title.
/// Returns Ok(()) if valid, Err(message) if invalid.
fn validate_title(title: &str) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("Title cannot be empty or whitespace only".to_string());
    }
    if title.len() > 512 {
        return Err("Title is too long (max 512 characters)".to_string());
    }
    Ok(())
}

/// Validate limit parameter.
/// Returns Ok(limit) if valid, Err(message) if invalid.
pub fn validate_limit(limit: Option<usize>, max_limit: usize) -> Result<usize, String> {
    match limit {
        Some(l) if l == 0 => Err("Limit cannot be zero".to_string()),
        Some(l) if l > max_limit => Err(format!("Limit cannot exceed {}", max_limit)),
        Some(l) => Ok(l),
        None => Ok(10), // Default limit
    }
}

/// Incrementally index only newly imported bookmarks.
///
/// Filters by `source` and `created_at >= import_start_time` to avoid
/// re-indexing the entire bookmark table after each import.
fn index_newly_imported_bookmarks(
    state: &State<AppState>,
    source: &str,
    import_start_time: i64,
) -> Result<(), String> {
    let bookmarks = state.data_service.with_db(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, url, description, tags, last_accessed, created_at, updated_at \
                 FROM bookmarks WHERE source = ?1 AND created_at >= ?2"
            )
            .map_err(|e| crate::error::AppError::Generic(format!("Failed to prepare query: {}", e)))?;

        let bookmarks: Result<Vec<_>, _> = stmt
            .query_map(rusqlite::params![source, import_start_time], |row| {
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
            .map_err(|e| crate::error::AppError::Generic(format!("Failed to query bookmarks: {}", e)))?
            .collect();

        bookmarks.map_err(|e| crate::error::AppError::Generic(format!("Failed to collect bookmarks: {}", e)))
    }).map_err(|e| e.to_string())?;

    state
        .data_service
        .search_engine()
        .batch_index_bookmarks(bookmarks)
        .map_err(|e| format!("Failed to index bookmarks: {}", e))?;

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
    // Input validation
    validate_title(&title)?;
    validate_url(&url)?;

    // Use DataService for atomic DB + Index operations
    state
        .data_service
        .add_bookmark(title, url, description, tags)
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
    // Input validation
    if id <= 0 {
        return Err("Invalid bookmark ID".to_string());
    }
    validate_title(&title)?;
    validate_url(&url)?;

    // Use DataService for atomic DB + Index operations
    state
        .data_service
        .update_bookmark(id, title, url, description, tags)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_bookmark(state: State<AppState>, id: i64) -> Result<(), String> {
    // Input validation
    if id <= 0 {
        return Err("Invalid bookmark ID".to_string());
    }

    // Use DataService for atomic DB + Index operations
    state
        .data_service
        .delete_bookmark(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_chrome_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let import_start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let result = state
        .data_service
        .with_db(|conn| {
            ChromeImporter::import(conn).map_err(|e| crate::error::AppError::ChromeImport(e))
        })
        .map_err(|e| e.to_string())?;

    if result.imported > 0 {
        if let Err(e) = index_newly_imported_bookmarks(&state, "chrome", import_start_time) {
            eprintln!(
                "Warning: Failed to index bookmarks after Chrome import: {}",
                e
            );
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn import_firefox_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let import_start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let result = state
        .data_service
        .with_db(|conn| {
            FirefoxImporter::import(conn).map_err(|e| crate::error::AppError::FirefoxImport(e))
        })
        .map_err(|e| e.to_string())?;

    if result.imported > 0 {
        if let Err(e) = index_newly_imported_bookmarks(&state, "firefox", import_start_time) {
            eprintln!(
                "Warning: Failed to index bookmarks after Firefox import: {}",
                e
            );
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn import_safari_bookmarks(state: State<AppState>) -> Result<ImportResult, String> {
    let import_start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let result = state
        .data_service
        .with_db(|conn| {
            SafariImporter::import(conn).map_err(|e| crate::error::AppError::SafariImport(e))
        })
        .map_err(|e| e.to_string())?;

    if result.imported > 0 {
        if let Err(e) = index_newly_imported_bookmarks(&state, "safari", import_start_time) {
            eprintln!(
                "Warning: Failed to index bookmarks after Safari import: {}",
                e
            );
        }
    }

    Ok(result)
}
