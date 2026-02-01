mod commands;
mod db;
mod models;
mod search;
mod services;

use commands::bookmarks::AppState;
use db::Database;
use search::{SearchEngine, TantivySearchEngine};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::window::toggle_window,
            commands::window::show_window,
            commands::window::hide_window,
            commands::bookmarks::add_bookmark,
            commands::bookmarks::update_bookmark,
            commands::bookmarks::delete_bookmark,
            commands::bookmarks::import_chrome_bookmarks,
            commands::bookmarks::import_firefox_bookmarks,
            commands::bookmarks::import_safari_bookmarks,
            commands::search::search_bookmarks,
            commands::search::record_bookmark_access,
            commands::search::rebuild_search_index,
            commands::search::get_search_stats,
            commands::favicon::fetch_favicon,
            // Phase 5: File Search Commands
            commands::file_search::search_files,
            commands::file_search::search_files_by_extension,
            commands::file_search::record_file_access,
            commands::file_search::get_file_by_id,
            commands::directories::validate_directory,
            commands::directories::add_search_directory,
            commands::directories::remove_search_directory,
            commands::directories::get_search_directories,
            commands::directories::toggle_search_directory,
            commands::directories::index_directory,
            commands::directories::refresh_directory_index,
            commands::directories::get_default_search_directories,
            commands::directories::get_indexing_stats,
            // Phase 6: Resource Opening Commands
            commands::opener::open_bookmark,
            commands::opener::open_file,
            commands::opener::open_file_by_path,
            commands::opener::open_url,
            commands::opener::reveal_in_finder,
            commands::opener::open_resource,
            commands::opener::check_bookmark_url,
            commands::opener::check_file_exists,
            // Phase 7: Settings Commands
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::delete_setting,
            commands::settings::get_all_settings,
            commands::settings::get_app_settings,
            commands::settings::save_app_settings,
            commands::settings::get_hotkey_settings,
            commands::settings::save_hotkey_settings,
            commands::settings::get_theme_settings,
            commands::settings::save_theme_settings,
            commands::settings::get_search_settings,
            commands::settings::save_search_settings,
            commands::settings::export_data,
            commands::settings::import_data,
            commands::settings::reset_settings,
            commands::settings::get_data_stats,
        ])
        .setup(|app| {
            let app_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;

            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create app data dir: {}", e))?;

            let db_path = app_dir.join("bookmarks.db");
            let db = Database::new(db_path)
                .map_err(|e| format!("Failed to create database: {}", e))?;

            db.initialize()
                .map_err(|e| format!("Failed to initialize database: {}", e))?;

            // Initialize Tantivy search engine
            let index_dir = app_dir.join("tantivy_indexes");
            let search_db_path = app_dir.join("bookmarks.db");
            let search_db = Database::new(search_db_path)
                .map_err(|e| format!("Failed to create search database: {}", e))?;

            let search_engine = TantivySearchEngine::new(index_dir, search_db)
                .map_err(|e| format!("Failed to create search engine: {}", e))?;

            let search_engine = Arc::new(search_engine);

            // Rebuild indexes in background on first run
            let engine_clone = search_engine.clone();
            std::thread::spawn(move || {
                if let Err(e) = engine_clone.rebuild_bookmark_index() {
                    eprintln!("Failed to rebuild bookmark index: {}", e);
                }
                if let Err(e) = engine_clone.rebuild_file_index() {
                    eprintln!("Failed to rebuild file index: {}", e);
                }
            });

            app.manage(AppState {
                db: Mutex::new(db),
                search_engine,
            });
            
            let handle = app.handle().clone();
            
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, _shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            if let Some(window) = handle.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    })
                    .build(),
            )?;
            
            #[cfg(target_os = "macos")]
            let shortcut = "Cmd+Space";
            #[cfg(not(target_os = "macos"))]
            let shortcut = "Ctrl+Space";
            
            app.global_shortcut().register(shortcut)?;
            
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();

            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = window_clone.hide();
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

