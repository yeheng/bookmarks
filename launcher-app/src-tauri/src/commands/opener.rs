use crate::commands::bookmarks::AppState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenResult {
    pub success: bool,
    pub resource_type: String,
    pub resource_id: i64,
    pub error: Option<String>,
}

#[tauri::command]
pub fn open_bookmark(state: State<AppState>, bookmark_id: i64) -> Result<OpenResult, String> {
    let url = state
        .data_service
        .with_bookmark_store(|store| {
            let bookmark = store.find_by_id(bookmark_id).ok_or(AppError::BookmarkNotFound)?;
            Ok(bookmark.url.clone())
        })
        .map_err(|e| e.to_string())?;

    if url.is_empty() {
        return Ok(OpenResult {
            success: false,
            resource_type: "bookmark".to_string(),
            resource_id: bookmark_id,
            error: Some("Bookmark URL is empty".to_string()),
        });
    }

    if !is_valid_url(&url) {
        return Ok(OpenResult {
            success: false,
            resource_type: "bookmark".to_string(),
            resource_id: bookmark_id,
            error: Some("Invalid URL format".to_string()),
        });
    }

    match open::that(&url) {
        Ok(_) => {
            Ok(OpenResult {
                success: true,
                resource_type: "bookmark".to_string(),
                resource_id: bookmark_id,
                error: None,
            })
        }
        Err(e) => Ok(OpenResult {
            success: false,
            resource_type: "bookmark".to_string(),
            resource_id: bookmark_id,
            error: Some(format!("Failed to open URL: {}", e)),
        }),
    }
}

#[tauri::command]
pub fn open_file(state: State<AppState>, file_id: i64) -> Result<OpenResult, String> {
    // Look up file path from Tantivy search engine
    let file = state
        .data_service
        .search_engine()
        .search_files("", 1, None)
        .ok()
        .and_then(|results| results.into_iter().find(|f| f.id == file_id));

    let path = match file {
        Some(f) => f.path,
        None => {
            return Ok(OpenResult {
                success: false,
                resource_type: "file".to_string(),
                resource_id: file_id,
                error: Some("File not found in index".to_string()),
            });
        }
    };

    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Ok(OpenResult {
            success: false,
            resource_type: "file".to_string(),
            resource_id: file_id,
            error: Some("File no longer exists".to_string()),
        });
    }

    match open::that(&path) {
        Ok(_) => {
            // Record file access in Tantivy (best-effort)
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let _ = state
                .data_service
                .search_engine()
                .update_file_frecency(file_id, 1, now);

            Ok(OpenResult {
                success: true,
                resource_type: "file".to_string(),
                resource_id: file_id,
                error: None,
            })
        }
        Err(e) => Ok(OpenResult {
            success: false,
            resource_type: "file".to_string(),
            resource_id: file_id,
            error: Some(format!("Failed to open file: {}", e)),
        }),
    }
}

#[tauri::command]
pub fn open_file_by_path(path: String) -> Result<OpenResult, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Ok(OpenResult {
            success: false,
            resource_type: "file".to_string(),
            resource_id: 0,
            error: Some("File does not exist".to_string()),
        });
    }

    match open::that(&path) {
        Ok(_) => Ok(OpenResult {
            success: true,
            resource_type: "file".to_string(),
            resource_id: 0,
            error: None,
        }),
        Err(e) => Ok(OpenResult {
            success: false,
            resource_type: "file".to_string(),
            resource_id: 0,
            error: Some(format!("Failed to open file: {}", e)),
        }),
    }
}

#[tauri::command]
pub fn open_url(url: String) -> Result<OpenResult, String> {
    if !is_valid_url(&url) {
        return Ok(OpenResult {
            success: false,
            resource_type: "url".to_string(),
            resource_id: 0,
            error: Some("Invalid URL format".to_string()),
        });
    }

    match open::that(&url) {
        Ok(_) => Ok(OpenResult {
            success: true,
            resource_type: "url".to_string(),
            resource_id: 0,
            error: None,
        }),
        Err(e) => Ok(OpenResult {
            success: false,
            resource_type: "url".to_string(),
            resource_id: 0,
            error: Some(format!("Failed to open URL: {}", e)),
        }),
    }
}

#[tauri::command]
pub fn reveal_in_finder(path: String) -> Result<OpenResult, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Ok(OpenResult {
            success: false,
            resource_type: "file".to_string(),
            resource_id: 0,
            error: Some("Path does not exist".to_string()),
        });
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        match Command::new("open").args(["-R", &path]).spawn() {
            Ok(_) => Ok(OpenResult {
                success: true,
                resource_type: "file".to_string(),
                resource_id: 0,
                error: None,
            }),
            Err(e) => Ok(OpenResult {
                success: false,
                resource_type: "file".to_string(),
                resource_id: 0,
                error: Some(format!("Failed to reveal in Finder: {}", e)),
            }),
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        match Command::new("explorer").args(["/select,", &path]).spawn() {
            Ok(_) => Ok(OpenResult {
                success: true,
                resource_type: "file".to_string(),
                resource_id: 0,
                error: None,
            }),
            Err(e) => Ok(OpenResult {
                success: false,
                resource_type: "file".to_string(),
                resource_id: 0,
                error: Some(format!("Failed to reveal in Explorer: {}", e)),
            }),
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let parent = file_path.parent().unwrap_or(file_path);
        match Command::new("xdg-open")
            .arg(parent.to_string_lossy().to_string())
            .spawn()
        {
            Ok(_) => Ok(OpenResult {
                success: true,
                resource_type: "file".to_string(),
                resource_id: 0,
                error: None,
            }),
            Err(e) => Ok(OpenResult {
                success: false,
                resource_type: "file".to_string(),
                resource_id: 0,
                error: Some(format!("Failed to open file manager: {}", e)),
            }),
        }
    }
}

#[tauri::command]
pub fn open_resource(
    state: State<AppState>,
    resource_type: String,
    resource_id: i64,
) -> Result<OpenResult, String> {
    match resource_type.as_str() {
        "bookmark" => open_bookmark(state, resource_id),
        "file" => open_file(state, resource_id),
        _ => Ok(OpenResult {
            success: false,
            resource_type,
            resource_id,
            error: Some("Unknown resource type".to_string()),
        }),
    }
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("file://")
        || url.starts_with("mailto:")
        || url.starts_with("tel:")
}

#[tauri::command]
pub fn check_bookmark_url(state: State<AppState>, bookmark_id: i64) -> Result<UrlCheckResult, String> {
    let url = state
        .data_service
        .with_bookmark_store(|store| {
            let bookmark = store.find_by_id(bookmark_id).ok_or(AppError::BookmarkNotFound)?;
            Ok(bookmark.url.clone())
        })
        .map_err(|e| e.to_string())?;

    let valid = is_valid_url(&url);

    Ok(UrlCheckResult {
        bookmark_id,
        url,
        valid,
        error: if valid {
            None
        } else {
            Some("Invalid URL format".to_string())
        },
    })
}

#[tauri::command]
pub fn check_file_exists(state: State<AppState>, file_id: i64) -> Result<FileCheckResult, String> {
    // Look up file path from Tantivy
    let file = state
        .data_service
        .search_engine()
        .search_files("", 1, None)
        .ok()
        .and_then(|results| results.into_iter().find(|f| f.id == file_id));

    let path = match file {
        Some(f) => f.path,
        None => {
            return Ok(FileCheckResult {
                file_id,
                path: String::new(),
                exists: false,
            });
        }
    };

    let exists = Path::new(&path).exists();

    Ok(FileCheckResult {
        file_id,
        path,
        exists,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlCheckResult {
    pub bookmark_id: i64,
    pub url: String,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileCheckResult {
    pub file_id: i64,
    pub path: String,
    pub exists: bool,
}
