use crate::models::bookmark::ImportResult;
use plist::Value;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SafariImporter;

impl SafariImporter {
    pub fn import(conn: &Connection) -> Result<ImportResult, String> {
        #[cfg(not(target_os = "macos"))]
        return Err("Safari is only available on macOS".to_string());

        #[cfg(target_os = "macos")]
        {
            let bookmarks_path = Self::get_safari_bookmarks_path()?;
            let content = fs::read(&bookmarks_path)
                .map_err(|e| format!("Failed to read Safari bookmarks: {}", e))?;

            let cursor = std::io::Cursor::new(content);
            let plist = Value::from_reader(cursor)
                .map_err(|e| format!("Failed to parse Safari bookmarks: {}", e))?;

            let mut result = ImportResult {
                imported: 0,
                skipped: 0,
                errors: Vec::new(),
            };

            if let Some(dict) = plist.as_dictionary() {
                if let Some(children) = dict.get("Children").and_then(|v| v.as_array()) {
                    for child in children {
                        Self::process_bookmark_node(conn, child, &mut result);
                    }
                }
            }

            Ok(result)
        }
    }

    #[cfg(target_os = "macos")]
    fn process_bookmark_node(conn: &Connection, node: &Value, result: &mut ImportResult) {
        if let Some(dict) = node.as_dictionary() {
            if let Some(bookmark_type) = dict.get("WebBookmarkType").and_then(|v| v.as_string()) {
                if bookmark_type == "WebBookmarkTypeLeaf" {
                    if let (Some(title), Some(url)) = (
                        dict.get("URIDictionary")
                            .and_then(|v| v.as_dictionary())
                            .and_then(|d| d.get("title"))
                            .and_then(|v| v.as_string()),
                        dict.get("URLString").and_then(|v| v.as_string()),
                    ) {
                        match Self::insert_bookmark(conn, title, url) {
                            Ok(true) => result.imported += 1,
                            Ok(false) => result.skipped += 1,
                            Err(e) => result.errors.push(e),
                        }
                    }
                } else if bookmark_type == "WebBookmarkTypeList" {
                    if let Some(children) = dict.get("Children").and_then(|v| v.as_array()) {
                        for child in children {
                            Self::process_bookmark_node(conn, child, result);
                        }
                    }
                }
            }
        }
    }

    fn insert_bookmark(conn: &Connection, title: &str, url: &str) -> Result<bool, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let existing: Result<i64, _> =
            conn.query_row("SELECT id FROM bookmarks WHERE url = ?1", [url], |row| {
                row.get(0)
            });

        if existing.is_ok() {
            return Ok(false);
        }

        conn.execute(
            "INSERT INTO bookmarks (title, url, source, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![title, url, "safari", now, now],
        )
        .map_err(|e| format!("Failed to insert bookmark: {}", e))?;

        Ok(true)
    }

    fn get_safari_bookmarks_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("Failed to get home directory")?;
        let path = home.join("Library/Safari/Bookmarks.plist");

        if !path.exists() {
            return Err("Safari bookmarks file not found".to_string());
        }

        Ok(path)
    }
}
