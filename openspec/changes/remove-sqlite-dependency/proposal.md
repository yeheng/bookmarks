# Change: Remove SQLite dependency, use JSON files + Tantivy

## Why

SQLite is currently used as the "source of truth" for bookmarks, files, directories, usage history, and settings. However, Tantivy already handles all full-text search and frecency scoring. SQLite adds significant complexity (bundled C library, cross-compilation overhead, WAL mode configuration, lock management) without proportional benefit. For a desktop app with modest data volumes (<100K records), JSON files for structured data + Tantivy for search indexing is simpler, lighter, and sufficient.

## What Changes

- **BREAKING**: Remove `rusqlite` crate dependency entirely
- **BREAKING**: Replace SQLite-based settings storage with a JSON file in the application data directory (`{app_data_dir}/settings.json`)
- **BREAKING**: Replace SQLite-based bookmark/directory/file storage with JSON files in the application data directory (`{app_data_dir}/bookmarks.json`, `{app_data_dir}/directories.json`)
- Tantivy becomes the primary search index (already is) and the JSON files become the source of truth for CRUD operations
- Usage history (frecency data) stays in Tantivy FastFields (already stored there for search scoring)
- Data migration: on first launch after upgrade, detect existing SQLite database and migrate data to new JSON format
- Remove `db/mod.rs` module entirely
- Refactor `DataService` to coordinate JSON file I/O + Tantivy index updates
- Simplify `AppState` (no more `Mutex<Database>`)

## Impact

- Affected specs: data-storage, settings-storage, search-engine
- Affected code:
  - `src-tauri/src/db/mod.rs` (removed)
  - `src-tauri/src/services/data_service.rs` (major rewrite)
  - `src-tauri/src/commands/settings.rs` (rewrite to use JSON file)
  - `src-tauri/src/commands/bookmarks.rs` (rewrite to use JSON + Tantivy)
  - `src-tauri/src/commands/directories.rs` (rewrite to use JSON)
  - `src-tauri/src/services/file_scanner.rs` (remove SQLite writes)
  - `src-tauri/src/services/chrome_importer.rs` (write to JSON instead of SQLite)
  - `src-tauri/src/services/firefox_importer.rs` (write to JSON instead of SQLite)
  - `src-tauri/src/services/safari_importer.rs` (write to JSON instead of SQLite)
  - `src-tauri/src/lib.rs` (app initialization)
  - `src-tauri/src/error.rs` (remove rusqlite error variant)
  - `src-tauri/Cargo.toml` (remove rusqlite dependency)
- Binary size reduction: ~2-3MB (bundled SQLite library removed)
- Compile time improvement: no more C compilation for SQLite
- Simpler cross-platform builds (no C toolchain needed for SQLite)
