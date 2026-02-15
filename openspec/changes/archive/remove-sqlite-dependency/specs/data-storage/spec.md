## ADDED Requirements

### Requirement: JSON File-Based Data Storage
The system SHALL store bookmarks in a JSON file (`bookmarks.json`) located in the application data directory.

The system SHALL store search directory configurations in a JSON file (`directories.json`) located in the application data directory.

The system SHALL load data files into memory on startup and persist changes to disk using atomic file writes (write to temporary file, then rename).

The system SHALL use `Arc<RwLock<T>>` for thread-safe in-memory data access across Tauri commands.

#### Scenario: Bookmark persistence across restarts
- **WHEN** a user adds a bookmark and restarts the application
- **THEN** the bookmark SHALL be present in the loaded data

#### Scenario: Atomic write prevents corruption
- **WHEN** the application writes `bookmarks.json`
- **THEN** the system SHALL write to a temporary file first and then atomically rename it to the target path

#### Scenario: Missing data file on fresh install
- **WHEN** the application starts and no `bookmarks.json` exists
- **THEN** the system SHALL create an empty data file with default structure

### Requirement: SQLite to JSON Migration
The system SHALL detect an existing SQLite database (`bookmarks.db`) on startup when no JSON data files exist.

The system SHALL migrate all bookmarks, search directories, and settings from SQLite to their respective JSON files.

The system SHALL rename the SQLite file to `bookmarks.db.bak` after successful migration.

#### Scenario: Automatic migration on upgrade
- **WHEN** the application starts with `bookmarks.db` present and `bookmarks.json` absent
- **THEN** the system SHALL read all data from SQLite, write JSON files, and rename the database to `.bak`

#### Scenario: Migration failure handling
- **WHEN** migration fails during data transfer
- **THEN** the original SQLite file SHALL remain intact and the system SHALL log the error

### Requirement: Indexed File Metadata in Tantivy
The system SHALL store indexed file metadata (path, name, extension, size, modified_at) exclusively in Tantivy index fields.

The system SHALL rebuild file index data by re-scanning configured directories rather than from a separate data store.

#### Scenario: File index rebuild from filesystem
- **WHEN** the Tantivy file index is corrupted or missing
- **THEN** the system SHALL re-scan all enabled directories from `directories.json` to rebuild the index

#### Scenario: File search without external data store
- **WHEN** a user searches for files
- **THEN** the system SHALL return results entirely from the Tantivy file index without querying any external data store

## REMOVED Requirements

### Requirement: SQLite Database Module
**Reason**: SQLite is replaced by JSON files for structured data and Tantivy for search/file indexing. The `db/mod.rs` module, all `rusqlite` imports, and the `rusqlite` crate dependency are removed.
**Migration**: Data automatically migrated to JSON files on first launch after upgrade.
