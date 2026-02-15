## 1. Data Models and JSON Store Infrastructure

- [x] 1.1 Create `src-tauri/src/store/mod.rs` with `JsonStore<T>` generic struct: load from file, save atomically (tmp + rename), default on missing file
- [x] 1.2 Create `src-tauri/src/store/bookmark_store.rs` with `BookmarkStore` (Vec<Bookmark> + next_id counter), CRUD methods, serde serialization
- [x] 1.3 Create `src-tauri/src/store/directory_store.rs` with `DirectoryStore` (Vec<SearchDirectory>), CRUD methods
- [x] 1.4 Create `src-tauri/src/store/settings_store.rs` with `SettingsStore` wrapping `AppSettings`, load/save/reset methods
- [x] 1.5 Add unit tests for `JsonStore` atomic write, load, corrupt file recovery, and default creation

## 2. Refactor DataService to Use JSON Stores

- [x] 2.1 Rewrite `DataService` to hold `Arc<RwLock<BookmarkStore>>`, `Arc<RwLock<DirectoryStore>>`, and `Arc<TantivySearchEngine>` instead of `Mutex<Database>`
- [x] 2.2 Rewrite `add_bookmark()`: write to BookmarkStore → persist to disk → index in Tantivy
- [x] 2.3 Rewrite `update_bookmark()`: update BookmarkStore → persist → re-index
- [x] 2.4 Rewrite `delete_bookmark()`: remove from BookmarkStore → persist → remove from index
- [x] 2.5 Rewrite `rebuild_index_if_needed()`: compare BookmarkStore count vs Tantivy count
- [x] 2.6 Rewrite `get_all_index_data()`: read from BookmarkStore instead of SQLite
- [x] 2.7 Add unit tests for DataService CRUD + index coordination

## 3. Refactor Settings Commands

- [x] 3.1 Rewrite `get_setting`, `set_setting`, `delete_setting` to use SettingsStore
- [x] 3.2 Rewrite `get_all_settings`, `get_app_settings`, `save_app_settings` to use SettingsStore
- [x] 3.3 Rewrite `save_theme_settings`, `save_hotkey_settings`, `save_search_settings` to use SettingsStore
- [x] 3.4 Rewrite `reset_settings` to reset SettingsStore to defaults and persist
- [x] 3.5 Rewrite `get_data_stats` to read counts from stores instead of SQLite
- [x] 3.6 Verify settings_cache (`RwLock<HashMap>`) still works or simplify (SettingsStore may replace it)

## 4. Refactor Bookmark and Directory Commands

- [x] 4.1 Rewrite `commands/bookmarks.rs` to use DataService with JSON stores
- [x] 4.2 Rewrite `commands/directories.rs` to use DirectoryStore for CRUD
- [x] 4.3 Rewrite `services/file_scanner.rs` to write indexed files directly to Tantivy (no SQLite)
- [x] 4.4 Rewrite browser importers (chrome, firefox, safari) to write to BookmarkStore instead of SQLite

## 5. Export/Import

- [x] 5.1 Rewrite `export_data` to read from BookmarkStore, DirectoryStore, and SettingsStore
- [x] 5.2 Rewrite `import_data` to write to stores and persist, then trigger index rebuild

## 6. SQLite Migration

- [x] 6.1 Create `src-tauri/src/migration.rs` with `migrate_from_sqlite()`: detect SQLite file, read all tables, write JSON files, rename to `.bak`
- [x] 6.2 Call migration at app startup (in `lib.rs`) before initializing stores
- [x] 6.3 Test migration with a sample SQLite database

## 7. Remove SQLite Dependency

- [x] 7.1 Remove `rusqlite` from `Cargo.toml` — kept for migration.rs and firefox_importer.rs only (reads external SQLite DBs, not ours)
- [x] 7.2 Delete `src-tauri/src/db/mod.rs`
- [x] 7.3 Remove `rusqlite::Error` variant from `error.rs`
- [x] 7.4 Remove all `use rusqlite::*` imports across the codebase (except migration.rs and firefox_importer.rs)
- [x] 7.5 Update `AppState` in `commands/bookmarks.rs` to remove Database reference

## 8. App Initialization

- [x] 8.1 Rewrite `lib.rs` app setup: create stores, run migration, initialize Tantivy, wire into AppState
- [x] 8.2 Update `AppState` struct to hold new store types

## 9. Update Project Configuration

- [x] 9.1 Update `openspec/project.md` to reflect new tech stack (remove SQLite, add JSON file storage)
- [x] 9.2 Verify `cargo check` passes with no SQLite references
- [x] 9.3 Verify `cargo test` passes
- [x] 9.4 Verify `npm run build` (frontend) still works (no API changes)
