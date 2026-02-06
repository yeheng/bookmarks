use crate::commands::bookmarks::AppState;
use crate::error::AppError;
use crate::models::file::{DirectoryValidationResult, IndexingProgress, SearchDirectory};
use crate::services::file_scanner::FileScanner;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
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

    state.data_service.with_db(|conn| {
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM search_directories WHERE path = ?1",
                [&canonical_path],
                |row| row.get(0),
            )
            .ok();

        if existing.is_some() {
            return Err(AppError::Generic("Directory is already being indexed".to_string()));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO search_directories (path, enabled, include_hidden, created_at, file_count)
             VALUES (?1, 1, ?2, ?3, 0)",
            rusqlite::params![canonical_path, include_hidden, now],
        )
        .map_err(|e| AppError::Generic(format!("Failed to add directory: {}", e)))?;

        let id = conn.last_insert_rowid();

        Ok(SearchDirectory {
            id: Some(id),
            path: canonical_path,
            enabled: true,
            include_hidden,
            created_at: now,
            last_indexed_at: None,
            file_count: 0,
        })
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_search_directory(state: State<AppState>, directory_id: i64) -> Result<(), String> {
    state.data_service.with_db(|conn| {
        FileScanner::remove_directory_files(conn, directory_id)
            .map_err(|e| AppError::Generic(e))?;

        conn.execute(
            "DELETE FROM search_directories WHERE id = ?1",
            [directory_id],
        )
        .map_err(|e| AppError::Generic(format!("Failed to remove directory: {}", e)))?;

        Ok(())
    }).map_err(|e| e.to_string())?;

    // Also remove from Tantivy index
    state
        .search_engine
        .delete_directory_files(directory_id)
        .map_err(|e| format!("Failed to clean search index: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_search_directories(state: State<AppState>) -> Result<Vec<SearchDirectory>, String> {
    state.data_service.with_db(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id, path, enabled, include_hidden, created_at, last_indexed_at, file_count
                 FROM search_directories
                 ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::Generic(format!("Failed to prepare query: {}", e)))?;

        let results = stmt
            .query_map([], |row| {
                Ok(SearchDirectory {
                    id: Some(row.get(0)?),
                    path: row.get(1)?,
                    enabled: row.get::<_, i32>(2)? != 0,
                    include_hidden: row.get::<_, i32>(3)? != 0,
                    created_at: row.get(4)?,
                    last_indexed_at: row.get(5)?,
                    file_count: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Generic(format!("Failed to execute query: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Generic(format!("Failed to collect results: {}", e)))?;

        Ok(results)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_search_directory(
    state: State<AppState>,
    directory_id: i64,
    enabled: bool,
) -> Result<(), String> {
    state.data_service.with_db(|conn| {
        conn.execute(
            "UPDATE search_directories SET enabled = ?1 WHERE id = ?2",
            rusqlite::params![enabled, directory_id],
        )
        .map_err(|e| AppError::Generic(format!("Failed to update directory: {}", e)))?;

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn index_directory(state: State<AppState>, directory_id: i64) -> Result<IndexingProgress, String> {
    // Scan directory and rebuild file index in a single lock acquisition
    let (path, scan_result, files) = state.data_service.with_db(|conn| {
        let (path, include_hidden): (String, bool) = conn
            .query_row(
                "SELECT path, include_hidden FROM search_directories WHERE id = ?1",
                [directory_id],
                |row| Ok((row.get(0)?, row.get::<_, i32>(1)? != 0)),
            )
            .map_err(|e| AppError::Generic(format!("Directory not found: {}", e)))?;

        let dir_path = Path::new(&path);

        if !dir_path.exists() {
            return Err(AppError::Generic("Directory no longer exists".to_string()));
        }

        let scanner = FileScanner::new(include_hidden);
        let scan_result = scanner.scan_directory(conn, directory_id, dir_path)
            .map_err(|e| AppError::Generic(e))?;

        // Fetch all files for index rebuild
        let mut stmt = conn
            .prepare("SELECT id, path, name, extension, size, modified_at, directory_id FROM indexed_files")
            .map_err(|e| AppError::Generic(format!("Failed to prepare query: {}", e)))?;

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
            .map_err(|e| AppError::Generic(format!("Failed to query files: {}", e)))?
            .collect();

        let files = files.map_err(|e| AppError::Generic(format!("Failed to collect files: {}", e)))?;

        Ok((path, scan_result, files))
    }).map_err(|e| e.to_string())?;

    // Rebuild file index (lock already released)
    let _ = state
        .search_engine
        .rebuild_file_index_from_data(files)
        .map_err(|e| format!("Failed to rebuild file index: {}", e))?;

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
    state.data_service.with_db(|conn| {
        let path: String = conn
            .query_row(
                "SELECT path FROM search_directories WHERE id = ?1",
                [directory_id],
                |row| row.get(0),
            )
            .map_err(|e| AppError::Generic(format!("Directory not found: {}", e)))?;

        let dir_path = Path::new(&path);

        if !dir_path.exists() {
            return Err(AppError::Generic("Directory no longer exists".to_string()));
        }

        FileScanner::refresh_stale_files(conn, directory_id, dir_path)
            .map_err(|e| AppError::Generic(e))
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_default_search_directories() -> Result<Vec<String>, String> {
    let dirs = FileScanner::get_default_directories();
    Ok(dirs.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[tauri::command]
pub fn get_indexing_stats(state: State<AppState>) -> Result<IndexingStats, String> {
    state.data_service.with_db(|conn| {
        let total_directories: i64 = conn
            .query_row("SELECT COUNT(*) FROM search_directories", [], |row| {
                row.get(0)
            })
            .map_err(|e| AppError::Generic(format!("Failed to count directories: {}", e)))?;

        let enabled_directories: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM search_directories WHERE enabled = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| AppError::Generic(format!("Failed to count enabled directories: {}", e)))?;

        let total_files: i64 = conn
            .query_row("SELECT COUNT(*) FROM indexed_files", [], |row| row.get(0))
            .map_err(|e| AppError::Generic(format!("Failed to count files: {}", e)))?;

        let total_size: i64 = conn
            .query_row("SELECT COALESCE(SUM(size), 0) FROM indexed_files", [], |row| {
                row.get(0)
            })
            .map_err(|e| AppError::Generic(format!("Failed to sum file sizes: {}", e)))?;

        Ok(IndexingStats {
            total_directories,
            enabled_directories,
            total_files,
            total_size,
        })
    }).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct IndexingStats {
    pub total_directories: i64,
    pub enabled_directories: i64,
    pub total_files: i64,
    pub total_size: i64,
}
