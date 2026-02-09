use crate::commands::bookmarks::AppState;
use crate::error::AppError;
use crate::models::settings::{
    AppSettings, ExportData, ExportedBookmark, GeneralSettings, HotkeySettings, ImportResult,
    SearchSettings, ThemeMode, ThemeSettings,
};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

fn get_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Save a batch of settings within a transaction.
///
/// Handles both required settings (always written) and optional settings
/// (written if Some, deleted if None). Wraps everything in BEGIN/COMMIT
/// for atomicity and performance (single fsync).
fn save_settings_batch(
    conn: &rusqlite::Connection,
    required: &[(&str, String)],
    optional: &[(&str, &Option<String>)],
    now: i64,
) -> Result<(), AppError> {
    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| AppError::Generic(format!("Failed to begin transaction: {}", e)))?;

    let result = (|| -> Result<(), AppError> {
        for (key, value) in required {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value, now],
            )
            .map_err(|e| AppError::Generic(format!("Failed to save setting {}: {}", key, e)))?;
        }

        for (key, value) in optional {
            if let Some(v) = value {
                conn.execute(
                    "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![key, v, now],
                )
                .map_err(|e| AppError::Generic(format!("Failed to save setting {}: {}", key, e)))?;
            } else {
                conn.execute("DELETE FROM settings WHERE key = ?1", rusqlite::params![key])
                    .map_err(|e| AppError::Generic(format!("Failed to delete setting {}: {}", key, e)))?;
            }
        }

        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")
                .map_err(|e| AppError::Generic(format!("Failed to commit: {}", e)))?;
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    state.data_service.with_db(|conn| {
        let result: Option<String> = conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [&key], |row| {
                row.get(0)
            })
            .ok();

        Ok(result)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    let now = get_now();

    state.data_service.with_db(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![key, value, now],
        )
        .map_err(|e| AppError::Generic(format!("Failed to save setting: {}", e)))?;

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_setting(state: State<AppState>, key: String) -> Result<(), String> {
    state.data_service.with_db(|conn| {
        conn.execute("DELETE FROM settings WHERE key = ?1", [&key])
            .map_err(|e| AppError::Generic(format!("Failed to delete setting: {}", e)))?;

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_settings(state: State<AppState>) -> Result<HashMap<String, String>, String> {
    state.data_service.with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| AppError::Generic(format!("Failed to prepare query: {}", e)))?;

        let settings: HashMap<String, String> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| AppError::Generic(format!("Failed to query settings: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Generic(format!("Failed to collect settings: {}", e)))?
            .into_iter()
            .collect();

        Ok(settings)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let settings = get_all_settings(state)?;

    // Parse ui_shortcuts from JSON in database, or use defaults
    let ui_shortcuts = settings
        .get("hotkey.ui_shortcuts")
        .and_then(|s| serde_json::from_str::<HashMap<String, String>>(s).ok())
        .unwrap_or_else(crate::models::settings::default_ui_shortcuts);

    let hotkey = HotkeySettings {
        global_shortcut: settings
            .get("hotkey.global_shortcut")
            .cloned()
            .unwrap_or_else(|| {
                #[cfg(target_os = "macos")]
                return "Cmd+F1".to_string();
                #[cfg(not(target_os = "macos"))]
                return "Ctrl+F1".to_string();
            }),
        ui_shortcuts,
    };

    let theme = ThemeSettings {
        mode: settings
            .get("theme.mode")
            .and_then(|s| match s.as_str() {
                "light" => Some(ThemeMode::Light),
                "dark" => Some(ThemeMode::Dark),
                _ => Some(ThemeMode::System),
            })
            .unwrap_or_default(),
        accent_color: settings
            .get("theme.accent_color")
            .cloned()
            .unwrap_or_else(|| "#ff6b6b".to_string()),
        bg_color: settings.get("theme.bg_color").cloned(),
        text_color: settings.get("theme.text_color").cloned(),
        secondary_text_color: settings.get("theme.secondary_text_color").cloned(),
        border_color: settings.get("theme.border_color").cloned(),
        selection_bg_color: settings.get("theme.selection_bg_color").cloned(),
        selection_text_color: settings.get("theme.selection_text_color").cloned(),
        font_size: settings
            .get("theme.font_size")
            .and_then(|s| s.parse().ok())
            .unwrap_or(14),
        window_width: settings
            .get("theme.window_width")
            .and_then(|s| s.parse().ok())
            .unwrap_or(750),
        window_height: settings
            .get("theme.window_height")
            .and_then(|s| s.parse().ok())
            .unwrap_or(480),
        input_height: settings
            .get("theme.input_height")
            .and_then(|s| s.parse().ok())
            .unwrap_or(60),
        item_height: settings
            .get("theme.item_height")
            .and_then(|s| s.parse().ok())
            .unwrap_or(44),
        border_radius: settings
            .get("theme.border_radius")
            .and_then(|s| s.parse().ok())
            .unwrap_or(12),
    };

    let search = SearchSettings {
        max_results: settings
            .get("search.max_results")
            .and_then(|s| s.parse().ok())
            .unwrap_or(10),
        show_bookmarks: settings
            .get("search.show_bookmarks")
            .map(|s| s == "true")
            .unwrap_or(true),
        show_files: settings
            .get("search.show_files")
            .map(|s| s == "true")
            .unwrap_or(true),
        fuzzy_matching: settings
            .get("search.fuzzy_matching")
            .map(|s| s == "true")
            .unwrap_or(true),
    };

    let general = GeneralSettings {
        launch_at_startup: settings
            .get("general.launch_at_startup")
            .map(|s| s == "true")
            .unwrap_or(false),
        hide_dock_icon: settings
            .get("general.hide_dock_icon")
            .map(|s| s == "true")
            .unwrap_or(true),
        check_updates: settings
            .get("general.check_updates")
            .map(|s| s == "true")
            .unwrap_or(true),
    };

    Ok(AppSettings {
        hotkey,
        theme,
        search,
        general,
    })
}

#[tauri::command]
pub fn save_app_settings(
    app: tauri::AppHandle,
    state: State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    let now = get_now();

    state.data_service.with_db(|conn| {
        #[cfg(target_os = "macos")]
        {
            let old_hide_dock: bool = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'general.hide_dock_icon'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .map(|v| v == "true")
                .unwrap_or(true);

            if old_hide_dock != settings.general.hide_dock_icon {
                if settings.general.hide_dock_icon {
                    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                } else {
                    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                }
            }
        }

        // Serialize ui_shortcuts to JSON for storage
        let ui_shortcuts_json = serde_json::to_string(&settings.hotkey.ui_shortcuts)
            .map_err(|e| format!("Failed to serialize ui_shortcuts: {}", e))?;

        let settings_map: Vec<(&str, String)> = vec![
            ("hotkey.global_shortcut", settings.hotkey.global_shortcut.clone()),
            ("hotkey.ui_shortcuts", ui_shortcuts_json),
            (
                "theme.mode",
                match settings.theme.mode {
                    ThemeMode::Light => "light",
                    ThemeMode::Dark => "dark",
                    ThemeMode::System => "system",
                }
                .to_string(),
            ),
            ("theme.accent_color", settings.theme.accent_color.clone()),
            ("theme.font_size", settings.theme.font_size.to_string()),
            (
                "theme.window_width",
                settings.theme.window_width.to_string(),
            ),
            (
                "theme.window_height",
                settings.theme.window_height.to_string(),
            ),
            (
                "theme.input_height",
                settings.theme.input_height.to_string(),
            ),
            ("theme.item_height", settings.theme.item_height.to_string()),
            (
                "theme.border_radius",
                settings.theme.border_radius.to_string(),
            ),
            (
                "search.max_results",
                settings.search.max_results.to_string(),
            ),
            (
                "search.show_bookmarks",
                settings.search.show_bookmarks.to_string(),
            ),
            ("search.show_files", settings.search.show_files.to_string()),
            (
                "search.fuzzy_matching",
                settings.search.fuzzy_matching.to_string(),
            ),
            (
                "general.launch_at_startup",
                settings.general.launch_at_startup.to_string(),
            ),
            (
                "general.hide_dock_icon",
                settings.general.hide_dock_icon.to_string(),
            ),
            (
                "general.check_updates",
                settings.general.check_updates.to_string(),
            ),
        ];

        // Save optional theme color settings
        let optional_settings: Vec<(&str, &Option<String>)> = vec![
            ("theme.bg_color", &settings.theme.bg_color),
            ("theme.text_color", &settings.theme.text_color),
            ("theme.secondary_text_color", &settings.theme.secondary_text_color),
            ("theme.border_color", &settings.theme.border_color),
            ("theme.selection_bg_color", &settings.theme.selection_bg_color),
            ("theme.selection_text_color", &settings.theme.selection_text_color),
        ];

        save_settings_batch(conn, &settings_map, &optional_settings, now)?;

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_hotkey_settings(state: State<AppState>) -> Result<HotkeySettings, String> {
    let settings = get_app_settings(state)?;
    Ok(settings.hotkey)
}

#[tauri::command]
pub fn save_hotkey_settings(state: State<AppState>, hotkey: HotkeySettings) -> Result<(), String> {
    let now = get_now();

    state.data_service.with_db(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params!["hotkey.global_shortcut", hotkey.global_shortcut, now],
        )
        .map_err(|e| AppError::Generic(format!("Failed to save hotkey: {}", e)))?;

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_theme_settings(state: State<AppState>) -> Result<ThemeSettings, String> {
    let settings = get_app_settings(state)?;
    Ok(settings.theme)
}

#[tauri::command]
pub fn save_theme_settings(state: State<AppState>, theme: ThemeSettings) -> Result<(), String> {
    let now = get_now();

    state.data_service.with_db(|conn| {
        let mode_str = match theme.mode {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::System => "system",
        };

        let settings: Vec<(&str, String)> = vec![
            ("theme.mode", mode_str.to_string()),
            ("theme.accent_color", theme.accent_color.clone()),
            ("theme.font_size", theme.font_size.to_string()),
            ("theme.window_width", theme.window_width.to_string()),
            ("theme.window_height", theme.window_height.to_string()),
            ("theme.input_height", theme.input_height.to_string()),
            ("theme.item_height", theme.item_height.to_string()),
            ("theme.border_radius", theme.border_radius.to_string()),
        ];

        // Save optional theme color settings
        let optional_settings: Vec<(&str, &Option<String>)> = vec![
            ("theme.bg_color", &theme.bg_color),
            ("theme.text_color", &theme.text_color),
            ("theme.secondary_text_color", &theme.secondary_text_color),
            ("theme.border_color", &theme.border_color),
            ("theme.selection_bg_color", &theme.selection_bg_color),
            ("theme.selection_text_color", &theme.selection_text_color),
        ];

        save_settings_batch(conn, &settings, &optional_settings, now)?;

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_search_settings(state: State<AppState>) -> Result<SearchSettings, String> {
    let settings = get_app_settings(state)?;
    Ok(settings.search)
}

#[tauri::command]
pub fn save_search_settings(state: State<AppState>, search: SearchSettings) -> Result<(), String> {
    let now = get_now();

    state.data_service.with_db(|conn| {
        let settings: Vec<(&str, String)> = vec![
            ("search.max_results", search.max_results.to_string()),
            ("search.show_bookmarks", search.show_bookmarks.to_string()),
            ("search.show_files", search.show_files.to_string()),
            ("search.fuzzy_matching", search.fuzzy_matching.to_string()),
        ];

        for (key, value) in settings {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value, now],
            )
            .map_err(|e| AppError::Generic(format!("Failed to save search setting: {}", e)))?;
        }

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_data(state: State<AppState>, file_path: String) -> Result<(), String> {
    let (bookmarks, directories, settings) = state.data_service.with_db(|conn| {
        let mut bookmark_stmt = conn
            .prepare("SELECT title, url, description, tags, source, created_at FROM bookmarks")
            .map_err(|e| AppError::Generic(format!("Failed to prepare bookmark query: {}", e)))?;

        let bookmarks: Vec<ExportedBookmark> = bookmark_stmt
            .query_map([], |row| {
                Ok(ExportedBookmark {
                    title: row.get(0)?,
                    url: row.get(1)?,
                    description: row.get(2)?,
                    tags: row.get(3)?,
                    source: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Generic(format!("Failed to query bookmarks: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Generic(format!("Failed to collect bookmarks: {}", e)))?;

        let mut dir_stmt = conn
            .prepare("SELECT path FROM search_directories WHERE enabled = 1")
            .map_err(|e| AppError::Generic(format!("Failed to prepare directory query: {}", e)))?;

        let directories: Vec<String> = dir_stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| AppError::Generic(format!("Failed to query directories: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Generic(format!("Failed to collect directories: {}", e)))?;

        let mut settings_stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| AppError::Generic(format!("Failed to prepare settings query: {}", e)))?;

        let settings: HashMap<String, String> = settings_stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| AppError::Generic(format!("Failed to query settings: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Generic(format!("Failed to collect settings: {}", e)))?
            .into_iter()
            .collect();

        Ok((bookmarks, directories, settings))
    }).map_err(|e| e.to_string())?;

    let export = ExportData {
        version: "1.0".to_string(),
        exported_at: get_now(),
        bookmarks,
        search_directories: directories,
        settings,
    };

    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| format!("Failed to serialize data: {}", e))?;

    fs::write(&file_path, json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn import_data(state: State<AppState>, file_path: String) -> Result<ImportResult, String> {
    let path = Path::new(&file_path);

    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let import: ExportData =
        serde_json::from_str(&content).map_err(|e| format!("Invalid file format: {}", e))?;

    let now = get_now();

    state.data_service.with_db(|conn| {
        let mut result = ImportResult {
            bookmarks_imported: 0,
            bookmarks_skipped: 0,
            directories_imported: 0,
            settings_imported: 0,
            errors: Vec::new(),
        };

        for bookmark in &import.bookmarks {
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM bookmarks WHERE url = ?1",
                    [&bookmark.url],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if exists {
                result.bookmarks_skipped += 1;
                continue;
            }

            match conn.execute(
                "INSERT INTO bookmarks (title, url, description, tags, source, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    bookmark.title,
                    bookmark.url,
                    bookmark.description,
                    bookmark.tags,
                    bookmark.source,
                    bookmark.created_at,
                    now
                ],
            ) {
                Ok(_) => result.bookmarks_imported += 1,
                Err(e) => result
                    .errors
                    .push(format!("Failed to import bookmark {}: {}", bookmark.url, e)),
            }
        }

        for dir_path in &import.search_directories {
            let path = Path::new(&dir_path);
            if !path.exists() {
                result
                    .errors
                    .push(format!("Directory does not exist: {}", dir_path));
                continue;
            }

            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM search_directories WHERE path = ?1",
                    [&dir_path],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if exists {
                continue;
            }

            match conn.execute(
                "INSERT INTO search_directories (path, enabled, include_hidden, created_at, file_count)
                 VALUES (?1, 1, 0, ?2, 0)",
                rusqlite::params![dir_path, now],
            ) {
                Ok(_) => result.directories_imported += 1,
                Err(e) => result
                    .errors
                    .push(format!("Failed to import directory {}: {}", dir_path, e)),
            }
        }

        for (key, value) in &import.settings {
            match conn.execute(
                "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value, now],
            ) {
                Ok(_) => result.settings_imported += 1,
                Err(e) => result
                    .errors
                    .push(format!("Failed to import setting: {}", e)),
            }
        }

        Ok(result)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_settings(state: State<AppState>) -> Result<(), String> {
    state.data_service.with_db(|conn| {
        conn.execute("DELETE FROM settings", [])
            .map_err(|e| AppError::Generic(format!("Failed to reset settings: {}", e)))?;

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_data_stats(state: State<AppState>) -> Result<DataStats, String> {
    state.data_service.with_db(|conn| {
        conn.query_row(
            "SELECT
                (SELECT COUNT(*) FROM bookmarks) AS bookmarks_count,
                (SELECT COUNT(*) FROM indexed_files) AS files_count,
                (SELECT COUNT(*) FROM search_directories) AS directories_count,
                (SELECT COUNT(*) FROM settings) AS settings_count",
            [],
            |row| {
                Ok(DataStats {
                    bookmarks_count: row.get(0)?,
                    files_count: row.get(1)?,
                    directories_count: row.get(2)?,
                    settings_count: row.get(3)?,
                })
            },
        )
        .map_err(|e| AppError::Generic(format!("Failed to get data stats: {}", e)))
    }).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct DataStats {
    pub bookmarks_count: i64,
    pub files_count: i64,
    pub directories_count: i64,
    pub settings_count: i64,
}
