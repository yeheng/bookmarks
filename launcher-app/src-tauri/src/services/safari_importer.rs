use crate::models::bookmark::ImportResult;
use crate::store::BookmarkStore;
use plist::Value;
use std::fs;
use std::path::PathBuf;

pub struct SafariImporter;

impl SafariImporter {
    pub fn import(store: &mut BookmarkStore) -> Result<ImportResult, String> {
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
                        Self::process_bookmark_node(store, child, &mut result);
                    }
                }
            }

            // Batch save after all imports
            store.save().map_err(|e| format!("Failed to save bookmarks: {}", e))?;

            Ok(result)
        }
    }

    #[cfg(target_os = "macos")]
    fn process_bookmark_node(store: &mut BookmarkStore, node: &Value, result: &mut ImportResult) {
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
                        match store.import_bookmark(title, url, "safari") {
                            Ok(true) => result.imported += 1,
                            Ok(false) => result.skipped += 1,
                            Err(e) => result.errors.push(e),
                        }
                    }
                } else if bookmark_type == "WebBookmarkTypeList" {
                    if let Some(children) = dict.get("Children").and_then(|v| v.as_array()) {
                        for child in children {
                            Self::process_bookmark_node(store, child, result);
                        }
                    }
                }
            }
        }
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
