use crate::models::bookmark::ImportResult;
use crate::store::BookmarkStore;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub struct ChromeImporter;

impl ChromeImporter {
    pub fn import(store: &mut BookmarkStore) -> Result<ImportResult, String> {
        let bookmarks_path = Self::get_chrome_bookmarks_path()?;
        let content = fs::read_to_string(&bookmarks_path)
            .map_err(|e| format!("Failed to read Chrome bookmarks: {}", e))?;

        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse Chrome bookmarks: {}", e))?;

        let mut result = ImportResult {
            imported: 0,
            skipped: 0,
            errors: Vec::new(),
        };

        if let Some(roots) = json.get("roots").and_then(|r| r.as_object()) {
            for (_, root_value) in roots.iter() {
                Self::process_bookmark_node(store, root_value, &mut result);
            }
        }

        // Batch save after all imports
        store.save().map_err(|e| format!("Failed to save bookmarks: {}", e))?;

        Ok(result)
    }

    fn process_bookmark_node(store: &mut BookmarkStore, node: &Value, result: &mut ImportResult) {
        if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
            if node_type == "url" {
                if let (Some(title), Some(url)) = (
                    node.get("name").and_then(|n| n.as_str()),
                    node.get("url").and_then(|u| u.as_str()),
                ) {
                    match store.import_bookmark(title, url, "chrome") {
                        Ok(true) => result.imported += 1,
                        Ok(false) => result.skipped += 1,
                        Err(e) => result.errors.push(e),
                    }
                }
            } else if node_type == "folder" {
                if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
                    for child in children {
                        Self::process_bookmark_node(store, child, result);
                    }
                }
            }
        }
    }

    fn get_chrome_bookmarks_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("Failed to get home directory")?;

        #[cfg(target_os = "macos")]
        let path = home.join("Library/Application Support/Google/Chrome/Default/Bookmarks");

        #[cfg(target_os = "windows")]
        let path = home.join("AppData/Local/Google/Chrome/User Data/Default/Bookmarks");

        #[cfg(target_os = "linux")]
        let path = home.join(".config/google-chrome/Default/Bookmarks");

        if !path.exists() {
            return Err("Chrome bookmarks file not found".to_string());
        }

        Ok(path)
    }
}
