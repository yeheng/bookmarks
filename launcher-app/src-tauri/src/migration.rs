//! One-time migration from SQLite to JSON files.
//!
//! On first launch after upgrade, this module detects an existing SQLite database
//! and exports all data to JSON files. The SQLite file is then renamed to `.bak`.

use std::path::Path;

/// Run migration if needed. Returns true if migration was performed.
///
/// Note: This module uses rusqlite only during migration. After migration,
/// the rusqlite dependency can be fully removed in a future release.
/// For now, we do a lightweight check — if the .db file exists and .json files don't,
/// we attempt migration.
pub fn migrate_if_needed(app_dir: &Path) -> bool {
    let db_path = app_dir.join("bookmarks.db");
    let bookmarks_json = app_dir.join("bookmarks.json");

    // Only migrate if SQLite file exists and JSON files don't
    if !db_path.exists() || bookmarks_json.exists() {
        return false;
    }

    println!("[Migration] SQLite database detected. Migrating to JSON files...");

    match run_migration(app_dir) {
        Ok(()) => {
            // Rename SQLite file to .bak
            let bak_path = app_dir.join("bookmarks.db.bak");
            if let Err(e) = std::fs::rename(&db_path, &bak_path) {
                eprintln!("[Migration] Warning: Failed to rename db to .bak: {}", e);
            }
            println!("[Migration] Migration complete.");
            true
        }
        Err(e) => {
            eprintln!("[Migration] Migration failed: {}. SQLite file preserved.", e);
            false
        }
    }
}

fn run_migration(app_dir: &Path) -> Result<(), String> {
    let db_path = app_dir.join("bookmarks.db");

    // Open SQLite database read-only using rusqlite
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("Failed to open SQLite database: {}", e))?;

    // 1. Export bookmarks
    export_bookmarks(&conn, app_dir)?;

    // 2. Export settings
    export_settings(&conn, app_dir)?;

    // 3. Export directories
    export_directories(&conn, app_dir)?;

    // 4. Export plugins
    export_plugins(&conn, app_dir)?;

    Ok(())
}

fn export_bookmarks(conn: &rusqlite::Connection, app_dir: &Path) -> Result<(), String> {
    use crate::models::bookmark::Bookmark;
    use crate::store::bookmark_store::BookmarkStoreData;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, url, description, favicon_url, tags, source, created_at, updated_at, last_accessed
             FROM bookmarks ORDER BY id",
        )
        .map_err(|e| format!("Failed to prepare bookmarks query: {}", e))?;

    let bookmarks: Vec<Bookmark> = stmt
        .query_map([], |row| {
            Ok(Bookmark {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                favicon_url: row.get(4)?,
                tags: row.get(5)?,
                source: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_accessed: row.get(9)?,
            })
        })
        .map_err(|e| format!("Failed to query bookmarks: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    let next_id = bookmarks
        .iter()
        .filter_map(|b| b.id)
        .max()
        .unwrap_or(0)
        + 1;

    let data = BookmarkStoreData {
        next_id,
        bookmarks,
    };

    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize bookmarks: {}", e))?;
    std::fs::write(app_dir.join("bookmarks.json"), json)
        .map_err(|e| format!("Failed to write bookmarks.json: {}", e))?;

    println!(
        "[Migration] Exported {} bookmarks",
        data.bookmarks.len()
    );
    Ok(())
}

fn export_settings(conn: &rusqlite::Connection, app_dir: &Path) -> Result<(), String> {
    use crate::models::settings::AppSettings;
    use crate::store::SettingsStore;

    let mut settings = AppSettings::default();

    // Read key-value settings from SQLite and apply to struct
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| format!("Failed to prepare settings query: {}", e))?;

    let kv_pairs: std::collections::HashMap<String, String> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to query settings: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    // Create a temporary store just for the flat-map conversion
    let temp_path = app_dir.join("settings.json");
    let mut temp_store = SettingsStore::new(temp_path);
    temp_store.apply_flat_map(&kv_pairs);
    temp_store
        .save()
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    // Also try reading structured settings like hotkey.global_shortcut
    if let Some(v) = kv_pairs.get("hotkey.global_shortcut") {
        settings.hotkey.global_shortcut = v.clone();
    }

    println!("[Migration] Exported {} settings", kv_pairs.len());
    Ok(())
}

fn export_directories(conn: &rusqlite::Connection, app_dir: &Path) -> Result<(), String> {
    use crate::models::file::SearchDirectory;
    use crate::store::directory_store::DirectoryStoreData;

    let mut stmt = conn
        .prepare(
            "SELECT id, path, enabled, include_hidden, created_at, last_indexed_at, file_count
             FROM search_directories ORDER BY id",
        )
        .map_err(|e| format!("Failed to prepare directories query: {}", e))?;

    let directories: Vec<SearchDirectory> = stmt
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
        .map_err(|e| format!("Failed to query directories: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    let next_id = directories
        .iter()
        .filter_map(|d| d.id)
        .max()
        .unwrap_or(0)
        + 1;

    let data = DirectoryStoreData {
        next_id,
        directories,
    };

    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize directories: {}", e))?;
    std::fs::write(app_dir.join("directories.json"), json)
        .map_err(|e| format!("Failed to write directories.json: {}", e))?;

    println!(
        "[Migration] Exported {} directories",
        data.directories.len()
    );
    Ok(())
}

fn export_plugins(conn: &rusqlite::Connection, app_dir: &Path) -> Result<(), String> {
    use crate::plugins::registry::PluginInfo;
    use crate::store::plugin_store::PluginStoreData;
    use std::collections::HashMap;

    // Check if plugins table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='plugins'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if !table_exists {
        // No plugins table, write empty store
        let data = PluginStoreData::default();
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize plugins: {}", e))?;
        std::fs::write(app_dir.join("plugins.json"), json)
            .map_err(|e| format!("Failed to write plugins.json: {}", e))?;
        return Ok(());
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, version, author, enabled, install_path, installed_at, updated_at, icon
             FROM plugins ORDER BY title",
        )
        .map_err(|e| format!("Failed to prepare plugins query: {}", e))?;

    let plugins: Vec<PluginInfo> = stmt
        .query_map([], |row| {
            Ok(PluginInfo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                version: row.get(3)?,
                author: row.get(4)?,
                enabled: row.get::<_, i32>(5)? == 1,
                install_path: row.get(6)?,
                installed_at: row.get(7)?,
                updated_at: row.get(8)?,
                icon: row.get(9)?,
                keywords: Vec::new(),
                command_count: 0,
            })
        })
        .map_err(|e| format!("Failed to query plugins: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    // Export preferences
    let prefs_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='plugin_preferences'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    let mut preferences: HashMap<String, HashMap<String, String>> = HashMap::new();
    if prefs_exists {
        if let Ok(mut pref_stmt) =
            conn.prepare("SELECT plugin_id, key, value FROM plugin_preferences")
        {
            if let Ok(rows) = pref_stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            }) {
                for row in rows.flatten() {
                    preferences
                        .entry(row.0)
                        .or_default()
                        .insert(row.1, row.2);
                }
            }
        }
    }

    let data = PluginStoreData {
        plugins,
        preferences,
    };

    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize plugins: {}", e))?;
    std::fs::write(app_dir.join("plugins.json"), json)
        .map_err(|e| format!("Failed to write plugins.json: {}", e))?;

    println!(
        "[Migration] Exported {} plugins",
        data.plugins.len()
    );
    Ok(())
}
