use crate::models::bookmark::ImportResult;
use crate::store::BookmarkStore;
use std::fs;
use std::path::PathBuf;

pub struct FirefoxImporter;

impl FirefoxImporter {
    pub fn import(store: &mut BookmarkStore) -> Result<ImportResult, String> {
        let places_path = Self::get_firefox_places_path()?;

        let temp_copy = std::env::temp_dir().join("firefox_places_copy.sqlite");
        fs::copy(&places_path, &temp_copy)
            .map_err(|e| format!("Failed to copy Firefox database: {}", e))?;

        // Use rusqlite to read the Firefox places.sqlite (read-only, temporary copy)
        let firefox_conn = rusqlite::Connection::open(&temp_copy)
            .map_err(|e| format!("Failed to open Firefox database: {}", e))?;

        let mut result = ImportResult {
            imported: 0,
            skipped: 0,
            errors: Vec::new(),
        };

        let mut stmt = firefox_conn
            .prepare(
                "SELECT mp.url, mb.title
                 FROM moz_bookmarks mb
                 JOIN moz_places mp ON mb.fk = mp.id
                 WHERE mb.type = 1 AND mp.url IS NOT NULL",
            )
            .map_err(|e| format!("Failed to prepare Firefox query: {}", e))?;

        let bookmark_iter = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| format!("Failed to query Firefox bookmarks: {}", e))?;

        for bookmark in bookmark_iter {
            match bookmark {
                Ok((url, title)) => {
                    let title = title.unwrap_or_else(|| url.clone());
                    match store.import_bookmark(&title, &url, "firefox") {
                        Ok(true) => result.imported += 1,
                        Ok(false) => result.skipped += 1,
                        Err(e) => result.errors.push(e),
                    }
                }
                Err(e) => result
                    .errors
                    .push(format!("Failed to read bookmark: {}", e)),
            }
        }

        let _ = fs::remove_file(temp_copy);

        // Batch save after all imports
        store.save().map_err(|e| format!("Failed to save bookmarks: {}", e))?;

        Ok(result)
    }

    fn get_firefox_places_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("Failed to get home directory")?;

        #[cfg(target_os = "macos")]
        let profiles_dir = home.join("Library/Application Support/Firefox/Profiles");

        #[cfg(target_os = "windows")]
        let profiles_dir = home.join("AppData/Roaming/Mozilla/Firefox/Profiles");

        #[cfg(target_os = "linux")]
        let profiles_dir = home.join(".mozilla/firefox");

        if !profiles_dir.exists() {
            return Err("Firefox profiles directory not found".to_string());
        }

        for entry in fs::read_dir(&profiles_dir)
            .map_err(|e| format!("Failed to read profiles directory: {}", e))?
        {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let places_path = path.join("places.sqlite");
                    if places_path.exists() {
                        return Ok(places_path);
                    }
                }
            }
        }

        Err("Firefox places.sqlite not found".to_string())
    }
}
