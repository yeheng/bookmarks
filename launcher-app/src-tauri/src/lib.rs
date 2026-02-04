mod commands;
mod db;
mod models;
mod search;
mod services;

use commands::bookmarks::AppState;
use db::Database;
use search::{SearchEngine, TantivySearchEngine};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Maximum allowed discrepancy between SQLite and Tantivy counts (as a ratio).
/// If discrepancy exceeds 5%, index rebuild is triggered.
const INDEX_HEALTH_THRESHOLD: f64 = 0.05;

/// Check if Tantivy indexes are in sync with SQLite database.
/// Returns Ok(true) if healthy, Ok(false) if rebuild is needed.
fn check_index_health(
    search_engine: &Arc<TantivySearchEngine>,
    db_path: &Path,
) -> Result<bool, String> {
    // Get Tantivy document counts
    let stats = search_engine
        .get_stats()
        .map_err(|e| format!("Failed to get index stats: {}", e))?;

    let tantivy_bookmark_count = stats.bookmark_count;
    let tantivy_file_count = stats.file_count;

    // Get SQLite counts
    let db = Database::new(db_path.to_path_buf())
        .map_err(|e| format!("Failed to open health check database: {}", e))?;

    let conn = db.get_connection();

    let sqlite_bookmark_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))
        .unwrap_or(0);

    let sqlite_file_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM indexed_files", [], |row| row.get(0))
        .unwrap_or(0);

    // Calculate discrepancy
    let bookmark_diff = (tantivy_bookmark_count as i64 - sqlite_bookmark_count).abs() as f64;
    let file_diff = (tantivy_file_count as i64 - sqlite_file_count).abs() as f64;

    let bookmark_threshold = (sqlite_bookmark_count as f64 * INDEX_HEALTH_THRESHOLD).max(1.0);
    let file_threshold = (sqlite_file_count as f64 * INDEX_HEALTH_THRESHOLD).max(1.0);

    let bookmark_healthy = sqlite_bookmark_count == 0 || bookmark_diff <= bookmark_threshold;
    let file_healthy = sqlite_file_count == 0 || file_diff <= file_threshold;

    println!(
        "[HealthCheck] Bookmarks: SQLite={}, Tantivy={}, healthy={}",
        sqlite_bookmark_count, tantivy_bookmark_count, bookmark_healthy
    );
    println!(
        "[HealthCheck] Files: SQLite={}, Tantivy={}, healthy={}",
        sqlite_file_count, tantivy_file_count, file_healthy
    );

    Ok(bookmark_healthy && file_healthy)
}

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

            // Health check: verify SQLite and Tantivy are in sync
            // If discrepancy > 5%, trigger background rebuild
            let engine_clone = search_engine.clone();
            let db_path_clone = app_dir.join("bookmarks.db");
            std::thread::spawn(move || {
                // Check index health at startup
                let needs_rebuild = match check_index_health(&engine_clone, &db_path_clone) {
                    Ok(healthy) => !healthy,
                    Err(e) => {
                        eprintln!("[HealthCheck] Error checking index health: {}", e);
                        true // Rebuild on error
                    }
                };

                if needs_rebuild {
                    println!("[HealthCheck] Index inconsistency detected, rebuilding...");
                    if let Err(e) = engine_clone.rebuild_bookmark_index() {
                        eprintln!("[HealthCheck] Failed to rebuild bookmark index: {}", e);
                    }
                    if let Err(e) = engine_clone.rebuild_file_index() {
                        eprintln!("[HealthCheck] Failed to rebuild file index: {}", e);
                    }
                    println!("[HealthCheck] Index rebuild completed");
                } else {
                    println!("[HealthCheck] Indexes are healthy");
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

            #[cfg(target_os = "macos")]
            {
                let db_path = app_dir.join("bookmarks.db");
                if let Ok(db) = Database::new(db_path) {
                    if let Ok(conn) = db.initialize().and_then(|_| Ok(db.get_connection())) {
                         let hide_dock: bool = conn
                            .query_row(
                                "SELECT value FROM settings WHERE key = 'general.hide_dock_icon'",
                                [],
                                |row| row.get::<_, String>(0),
                            )
                            .map(|v| v == "true")
                            .unwrap_or(true);

                        if hide_dock {
                            app.handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
                        } else {
                            app.handle().set_activation_policy(tauri::ActivationPolicy::Regular);
                        }
                    }
                }
            }

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

