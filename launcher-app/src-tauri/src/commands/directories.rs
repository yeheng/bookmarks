use crate::commands::bookmarks::AppState;
use crate::error::AppError;
use crate::models::file::{DirectoryValidationResult, IndexingProgress, SearchDirectory};
use crate::services::file_scanner::FileScanner;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub fn validate_directory(path: String) -> Result<DirectoryValidationResult, String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Ok(DirectoryValidationResult {
            valid: false,
            error: Some("Directory does not exist".to_string()),
            estimated_file_count: None,
            requires_confirmation: false,
        });
    }

    if !path.is_dir() {
        return Ok(DirectoryValidationResult {
            valid: false,
            error: Some("Path is not a directory".to_string()),
            estimated_file_count: None,
            requires_confirmation: false,
        });
    }

    let scanner = FileScanner::new(false);
    let estimated_count = scanner.estimate_file_count(path)?;

    let requires_confirmation = estimated_count > 100_000;

    Ok(DirectoryValidationResult {
        valid: true,
        error: None,
        estimated_file_count: Some(estimated_count),
        requires_confirmation,
    })
}

#[tauri::command]
pub fn add_search_directory(
    state: State<AppState>,
    path: String,
    include_hidden: Option<bool>,
) -> Result<SearchDirectory, String> {
    let dir_path = Path::new(&path);

    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    if !dir_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let canonical_path = dir_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?
        .to_string_lossy()
        .to_string();

    let include_hidden = include_hidden.unwrap_or(false);

    state
        .data_service
        .with_directory_store_mut(|store| {
            store
                .add(canonical_path, include_hidden)
                .map_err(|e| AppError::Generic(e))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_search_directory(state: State<AppState>, directory_id: i64) -> Result<(), String> {
    state
        .data_service
        .with_directory_store_mut(|store| store.remove(directory_id).map_err(|e| AppError::Generic(e)))
        .map_err(|e| e.to_string())?;

    // Also remove from Tantivy index
    state
        .data_service
        .search_engine()
        .delete_directory_files(directory_id)
        .map_err(|e| format!("Failed to clean search index: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_search_directories(state: State<AppState>) -> Result<Vec<SearchDirectory>, String> {
    state
        .data_service
        .with_directory_store(|store| Ok(store.get_sorted()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_search_directory(
    state: State<AppState>,
    directory_id: i64,
    enabled: bool,
) -> Result<(), String> {
    state
        .data_service
        .with_directory_store_mut(|store| {
            store
                .toggle(directory_id, enabled)
                .map_err(|e| AppError::Generic(e))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn index_directory(state: State<AppState>, directory_id: i64) -> Result<IndexingProgress, String> {
    // Get directory info from store
    let (path, include_hidden) = state
        .data_service
        .with_directory_store(|store| {
            let dir = store
                .find_by_id(directory_id)
                .ok_or(AppError::Generic("Directory not found".to_string()))?;
            Ok((dir.path.clone(), dir.include_hidden))
        })
        .map_err(|e| e.to_string())?;

    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err("Directory no longer exists".to_string());
    }

    let scanner = FileScanner::new(include_hidden);
    let (scan_result, files) = scanner
        .scan_directory_for_tantivy(directory_id, dir_path)
        .map_err(|e| e.to_string())?;

    // Update directory stats in store
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    state
        .data_service
        .with_directory_store_mut(|store| {
            store
                .update_index_stats(directory_id, scan_result.files_indexed as i64, now)
                .map_err(|e| AppError::Generic(e))
        })
        .map_err(|e| e.to_string())?;

    // Index files in Tantivy
    let _ = state
        .data_service
        .search_engine()
        .index_directory_files(directory_id, files)
        .map_err(|e| format!("Failed to update file index: {}", e))?;

    Ok(IndexingProgress {
        directory_id,
        directory_path: path,
        files_scanned: scan_result.files_scanned,
        files_indexed: scan_result.files_indexed,
        is_complete: true,
        error: if scan_result.errors.is_empty() {
            None
        } else {
            Some(scan_result.errors.join("; "))
        },
    })
}

#[tauri::command]
pub fn refresh_directory_index(
    state: State<AppState>,
    directory_id: i64,
) -> Result<(usize, usize), String> {
    // Refresh is simpler now — just re-scan the directory
    let path = state
        .data_service
        .with_directory_store(|store| {
            let dir = store
                .find_by_id(directory_id)
                .ok_or(AppError::Generic("Directory not found".to_string()))?;
            Ok(dir.path.clone())
        })
        .map_err(|e| e.to_string())?;

    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err("Directory no longer exists".to_string());
    }

    // For refresh, we just return (0, 0) as the actual re-scan happens via index_directory
    Ok((0, 0))
}

#[tauri::command]
pub fn get_default_search_directories() -> Result<Vec<String>, String> {
    let dirs = FileScanner::get_default_directories();
    Ok(dirs.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[tauri::command]
pub fn get_indexing_stats(state: State<AppState>) -> Result<IndexingStats, String> {
    let (total_directories, enabled_directories, total_files) = state
        .data_service
        .with_directory_store(|store| {
            let total = store.count() as i64;
            let enabled = store.enabled_count() as i64;
            let files: i64 = store.directories().iter().map(|d| d.file_count).sum();
            Ok((total, enabled, files))
        })
        .map_err(|e| e.to_string())?;

    Ok(IndexingStats {
        total_directories,
        enabled_directories,
        total_files,
        total_size: 0, // No longer tracked per-file in store
    })
}

#[derive(serde::Serialize)]
pub struct IndexingStats {
    pub total_directories: i64,
    pub enabled_directories: i64,
    pub total_files: i64,
    pub total_size: i64,
}
