use crate::commands::bookmarks::AppState;
use crate::models::settings::{
    AppSettings, ExportData, ExportedBookmark, HotkeySettings, ImportResult,
    SearchSettings, ThemeSettings,
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

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    let flat = state
        .data_service
        .with_settings_store(|store| Ok(store.to_flat_map()))
        .map_err(|e| e.to_string())?;

    Ok(flat.get(&key).cloned())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    state
        .data_service
        .with_settings_store_mut(|store| {
            let mut map = store.to_flat_map();
            map.insert(key.clone(), value.clone());
            store.apply_flat_map(&map);
            store.save().map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_setting(state: State<AppState>, key: String) -> Result<(), String> {
    state
        .data_service
        .with_settings_store_mut(|store| {
            let mut map = store.to_flat_map();
            map.remove(&key);
            store.apply_flat_map(&map);
            store.save().map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_settings(state: State<AppState>) -> Result<HashMap<String, String>, String> {
    state
        .data_service
        .with_settings_store(|store| Ok(store.to_flat_map()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state
        .data_service
        .with_settings_store(|store| Ok(store.settings().clone()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_app_settings(
    app: tauri::AppHandle,
    state: State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let old_hide_dock = state
            .data_service
            .with_settings_store(|store| Ok(store.settings().general.hide_dock_icon))
            .unwrap_or(true);

        if old_hide_dock != settings.general.hide_dock_icon {
            if settings.general.hide_dock_icon {
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            } else {
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
            }
        }
    }

    state
        .data_service
        .with_settings_store_mut(|store| {
            store.set(settings).map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_hotkey_settings(state: State<AppState>) -> Result<HotkeySettings, String> {
    state
        .data_service
        .with_settings_store(|store| Ok(store.settings().hotkey.clone()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_hotkey_settings(state: State<AppState>, hotkey: HotkeySettings) -> Result<(), String> {
    state
        .data_service
        .with_settings_store_mut(|store| {
            store.settings_mut().hotkey = hotkey;
            store.save().map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_theme_settings(state: State<AppState>) -> Result<ThemeSettings, String> {
    state
        .data_service
        .with_settings_store(|store| Ok(store.settings().theme.clone()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_theme_settings(state: State<AppState>, theme: ThemeSettings) -> Result<(), String> {
    state
        .data_service
        .with_settings_store_mut(|store| {
            store.settings_mut().theme = theme;
            store.save().map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_search_settings(state: State<AppState>) -> Result<SearchSettings, String> {
    state
        .data_service
        .with_settings_store(|store| Ok(store.settings().search.clone()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_search_settings(state: State<AppState>, search: SearchSettings) -> Result<(), String> {
    state
        .data_service
        .with_settings_store_mut(|store| {
            store.settings_mut().search = search.clone();
            store.save().map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())?;

    // Refresh settings cache after saving
    if let Ok(mut cache) = state.settings_cache.write() {
        cache.insert("search.max_results".to_string(), search.max_results.to_string());
        cache.insert("search.show_bookmarks".to_string(), search.show_bookmarks.to_string());
        cache.insert("search.show_files".to_string(), search.show_files.to_string());
        cache.insert("search.fuzzy_matching".to_string(), search.fuzzy_matching.to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn export_data(state: State<AppState>, file_path: String) -> Result<(), String> {
    let bookmarks = state
        .data_service
        .with_bookmark_store(|store| {
            Ok(store
                .bookmarks()
                .iter()
                .map(|b| ExportedBookmark {
                    title: b.title.clone(),
                    url: b.url.clone(),
                    description: b.description.clone(),
                    tags: b.tags.clone(),
                    source: b.source.clone(),
                    created_at: b.created_at,
                })
                .collect::<Vec<_>>())
        })
        .map_err(|e| e.to_string())?;

    let directories = state
        .data_service
        .with_directory_store(|store| Ok(store.get_enabled_paths()))
        .map_err(|e| e.to_string())?;

    let settings = state
        .data_service
        .with_settings_store(|store| Ok(store.to_flat_map()))
        .map_err(|e| e.to_string())?;

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

    let mut result = ImportResult {
        bookmarks_imported: 0,
        bookmarks_skipped: 0,
        directories_imported: 0,
        settings_imported: 0,
        errors: Vec::new(),
    };

    // Import bookmarks
    state
        .data_service
        .with_bookmark_store_mut(|store| {
            for bookmark in &import.bookmarks {
                match store.import_bookmark(&bookmark.title, &bookmark.url, &bookmark.source) {
                    Ok(true) => result.bookmarks_imported += 1,
                    Ok(false) => result.bookmarks_skipped += 1,
                    Err(e) => result
                        .errors
                        .push(format!("Failed to import bookmark {}: {}", bookmark.url, e)),
                }
            }
            store
                .save()
                .map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())?;

    // Import directories
    state
        .data_service
        .with_directory_store_mut(|store| {
            for dir_path in &import.search_directories {
                let p = Path::new(&dir_path);
                if !p.exists() {
                    result
                        .errors
                        .push(format!("Directory does not exist: {}", dir_path));
                    continue;
                }

                match store.import_directory(dir_path) {
                    Ok(true) => result.directories_imported += 1,
                    Ok(false) => {} // already exists
                    Err(e) => result
                        .errors
                        .push(format!("Failed to import directory {}: {}", dir_path, e)),
                }
            }
            store
                .save()
                .map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())?;

    // Import settings
    if !import.settings.is_empty() {
        state
            .data_service
            .with_settings_store_mut(|store| {
                store.apply_flat_map(&import.settings);
                result.settings_imported = import.settings.len();
                store.save().map_err(|e| crate::error::AppError::Generic(e))
            })
            .map_err(|e| e.to_string())?;
    }

    Ok(result)
}

#[tauri::command]
pub fn reset_settings(state: State<AppState>) -> Result<(), String> {
    state
        .data_service
        .with_settings_store_mut(|store| {
            store.reset().map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_data_stats(state: State<AppState>) -> Result<DataStats, String> {
    let bookmarks_count = state
        .data_service
        .with_bookmark_store(|store| Ok(store.count() as i64))
        .map_err(|e| e.to_string())?;

    let directories_count = state
        .data_service
        .with_directory_store(|store| Ok(store.count() as i64))
        .map_err(|e| e.to_string())?;

    let settings_count = state
        .data_service
        .with_settings_store(|store| Ok(store.to_flat_map().len() as i64))
        .map_err(|e| e.to_string())?;

    // File count from search engine stats
    let files_count = state
        .data_service
        .search_engine()
        .get_stats()
        .map(|s| s.file_count as i64)
        .unwrap_or(0);

    Ok(DataStats {
        bookmarks_count,
        files_count,
        directories_count,
        settings_count,
    })
}

#[derive(Debug, Serialize)]
pub struct DataStats {
    pub bookmarks_count: i64,
    pub files_count: i64,
    pub directories_count: i64,
    pub settings_count: i64,
}
