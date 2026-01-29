use crate::commands::bookmarks::AppState;
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
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let url: String = conn
        .query_row(
            "SELECT url FROM bookmarks WHERE id = ?1",
            [bookmark_id],
            |row| row.get(0),
        )
        .map_err(|_| "Bookmark not found".to_string())?;

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
            record_resource_access(conn, "bookmark", bookmark_id)?;

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
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let path: String = conn
        .query_row(
            "SELECT path FROM indexed_files WHERE id = ?1",
            [file_id],
            |row| row.get(0),
        )
        .map_err(|_| "File not found in index".to_string())?;

    let file_path = Path::new(&path);

    if !file_path.exists() {
        conn.execute("DELETE FROM indexed_files WHERE id = ?1", [file_id])
            .ok();

        return Ok(OpenResult {
            success: false,
            resource_type: "file".to_string(),
            resource_id: file_id,
            error: Some("File no longer exists".to_string()),
        });
    }

    match open::that(&path) {
        Ok(_) => {
            record_resource_access(conn, "file", file_id)?;

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

fn record_resource_access(
    conn: &rusqlite::Connection,
    resource_type: &str,
    resource_id: i64,
) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    match resource_type {
        "bookmark" => {
            conn.execute(
                "UPDATE bookmarks SET last_accessed = ?1 WHERE id = ?2",
                rusqlite::params![now, resource_id],
            )
            .map_err(|e| format!("Failed to update bookmark access: {}", e))?;

            conn.execute(
                "INSERT INTO usage_history (bookmark_id, accessed_at) VALUES (?1, ?2)",
                rusqlite::params![resource_id, now],
            )
            .map_err(|e| format!("Failed to record bookmark usage: {}", e))?;
        }
        "file" => {
            conn.execute(
                "INSERT INTO file_usage_history (file_id, accessed_at) VALUES (?1, ?2)",
                rusqlite::params![resource_id, now],
            )
            .map_err(|e| format!("Failed to record file usage: {}", e))?;
        }
        _ => {}
    }

    Ok(())
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
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let url: String = conn
        .query_row(
            "SELECT url FROM bookmarks WHERE id = ?1",
            [bookmark_id],
            |row| row.get(0),
        )
        .map_err(|_| "Bookmark not found".to_string())?;

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
    let db = state.db.lock().unwrap();
    let conn = db.get_connection();

    let path: String = conn
        .query_row(
            "SELECT path FROM indexed_files WHERE id = ?1",
            [file_id],
            |row| row.get(0),
        )
        .map_err(|_| "File not found in index".to_string())?;

    let exists = Path::new(&path).exists();

    if !exists {
        conn.execute("DELETE FROM indexed_files WHERE id = ?1", [file_id])
            .ok();
    }

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
