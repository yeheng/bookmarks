mod commands;
mod db;
mod error;
mod models;
mod plugins;
mod search;
mod services;

use commands::bookmarks::AppState;
use db::Database;
use plugins::executor::PluginExecutor;
use plugins::registry::PluginRegistry;
use search::bookmark_provider::BookmarkSearchProvider;
use search::file_provider::FileSearchProvider;
use search::plugin_provider::PluginSearchProvider;
use search::SearchAggregator;
use search::TantivySearchEngine;
use services::data_service::DataService;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager};
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
            commands::search::unified_search,
            commands::search::record_bookmark_access,
            commands::search::rebuild_search_index,
            commands::search::get_search_stats,
            commands::search::refresh_settings_cache,
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
            // Plugin System Commands
            commands::plugins::execute_plugin_command,
            commands::plugins::list_plugins,
            commands::plugins::install_plugin,
            commands::plugins::uninstall_plugin,
            commands::plugins::enable_plugin,
            commands::plugins::disable_plugin,
            commands::plugins::get_plugin_preferences,
            commands::plugins::set_plugin_preference,
            commands::plugins::get_plugin_log,
            commands::plugins::get_plugin_keywords,
            commands::plugins::get_plugin_manifest_preferences,
        ])
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;

            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create app data dir: {}", e))?;

            let db_path = app_dir.join("bookmarks.db");
            let db =
                Database::new(db_path).map_err(|e| format!("Failed to create database: {}", e))?;

            db.initialize()
                .map_err(|e| format!("Failed to initialize database: {}", e))?;

            // Initialize Tantivy search engine (decoupled from database)
            let index_dir = app_dir.join("tantivy_indexes");
            let search_engine = TantivySearchEngine::new(index_dir)
                .map_err(|e| format!("Failed to create search engine: {}", e))?;

            let search_engine = Arc::new(search_engine);

            // Initialize DataService for atomic DB + Index operations
            let data_service = Arc::new(DataService::new(db, search_engine));

            // Check index integrity and rebuild if needed (simple, Linus-style)
            let data_service_clone = data_service.clone();
            std::thread::spawn(move || match data_service_clone.rebuild_index_if_needed() {
                Ok(true) => println!("[Startup] Index rebuilt successfully"),
                Ok(false) => println!("[Startup] Index integrity check passed"),
                Err(e) => eprintln!("[Startup] Index check failed: {}", e),
            });

            // Initialize Plugin System
            let plugins_dir = app_dir.join("plugins");
            let (plugin_registry, plugin_executor) = match PluginRegistry::new(plugins_dir) {
                Ok(registry) => {
                    // Discover plugins on startup
                    let registry = Arc::new(registry);
                    let executor = Arc::new(PluginExecutor::new());

                    let reg_clone = registry.clone();
                    let ds_clone = data_service.clone();
                    std::thread::spawn(move || {
                        match ds_clone.with_db(|conn| {
                            reg_clone
                                .discover(conn)
                                .map_err(|e| crate::error::AppError::Generic(e.to_string()))
                        }) {
                            Ok(discovered) => {
                                if !discovered.is_empty() {
                                    println!(
                                        "[Startup] Discovered {} new plugin(s): {:?}",
                                        discovered.len(),
                                        discovered
                                    );
                                } else {
                                    println!("[Startup] Plugin system ready (no new plugins)");
                                }
                            }
                            Err(e) => eprintln!("[Startup] Plugin discovery failed: {}", e),
                        }
                    });

                    (Some(registry), Some(executor))
                }
                Err(e) => {
                    eprintln!("[Startup] Plugin system initialization failed: {}", e);
                    (None, None)
                }
            };

            // AppState now uses DataService for all DB access (no duplicate connections)
            let http_client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

            // Initialize SearchAggregator with built-in providers
            let search_engine_arc = data_service.search_engine().clone();
            let mut aggregator = SearchAggregator::new();
            aggregator.register(Box::new(BookmarkSearchProvider::new(
                search_engine_arc.clone(),
            )));
            aggregator.register(Box::new(FileSearchProvider::new(search_engine_arc)));

            // Register plugin provider if plugin system initialized successfully
            if let (Some(ref reg), Some(ref exec)) = (&plugin_registry, &plugin_executor) {
                aggregator.register(Box::new(PluginSearchProvider::new(
                    reg.clone(),
                    exec.clone(),
                    data_service.clone(),
                )));
            }

            let search_aggregator = Arc::new(aggregator);

            app.manage(AppState {
                data_service: data_service.clone(),
                http_client,
                plugin_registry,
                plugin_executor,
                search_aggregator,
                settings_cache: std::sync::RwLock::new(HashMap::new()),
            });

            let handle = app.handle().clone();

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, _shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            if let Some(window) = handle.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.emit("tauri://window-hidden", ());
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    let _ = window.emit("tauri://window-shown", ());
                                }
                            }
                        }
                    })
                    .build(),
            )?;

            #[cfg(target_os = "macos")]
            let shortcut = "Cmd+F1";
            #[cfg(not(target_os = "macos"))]
            let shortcut = "Ctrl+F1";

            app.global_shortcut().register(shortcut)?;

            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();

                #[cfg(target_os = "macos")]
                {
                    // Read dock icon setting using data_service
                    let hide_dock = data_service
                        .with_db(|conn| {
                            let hide: bool = conn
                        .query_row(
                            "SELECT value FROM settings WHERE key = 'general.hide_dock_icon'",
                            [],
                            |row| row.get::<_, String>(0),
                        )
                        .map(|v| v == "true")
                        .unwrap_or(true);
                            Ok(hide)
                        })
                        .unwrap_or(true);

                    if hide_dock {
                        app.handle()
                            .set_activation_policy(tauri::ActivationPolicy::Accessory);
                    } else {
                        app.handle()
                            .set_activation_policy(tauri::ActivationPolicy::Regular);
                    }
                }

                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
