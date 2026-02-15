## Context

The application currently uses a dual-storage architecture: SQLite as "source of truth" for all structured data (bookmarks, files, directories, settings, usage history) and Tantivy as a search index cache. For a desktop launcher app with typical data volumes (hundreds of bookmarks, tens of thousands of indexed files), SQLite's relational capabilities are underutilized. The data is essentially document-oriented (bookmarks are independent records, settings are key-value pairs) and doesn't benefit from joins or complex queries.

### Stakeholders

- End users: affected by migration on upgrade; benefit from smaller binary
- Developers: simplified build toolchain (no C compiler for SQLite), fewer dependencies

### Current SQLite Usage Breakdown

| Table | Record Count | Access Pattern | Relational Features Used |
|-------|-------------|----------------|-------------------------|
| bookmarks | ~100-1000 | CRUD, full scan for export | UNIQUE on url, auto-increment ID |
| indexed_files | ~1K-100K | Bulk insert/delete per directory | FK to search_directories |
| search_directories | ~1-20 | CRUD | None significant |
| usage_history | ~100-10K | Append-only, aggregate queries | FK to bookmarks |
| file_usage_history | ~100-10K | Append-only, aggregate queries | FK to indexed_files |
| settings | ~20-30 | Key-value read/write | None |

## Goals / Non-Goals

### Goals

- Remove `rusqlite` crate and all SQLite-related code
- Settings stored as a single JSON file (`settings.json`) in `{app_data_dir}/`
- Bookmarks stored as a JSON file (`bookmarks.json`) in `{app_data_dir}/`
- Directory configuration stored as a JSON file (`directories.json`) in `{app_data_dir}/`
- Tantivy remains the search engine; indexed files metadata stored in Tantivy index fields
- Frecency data remains in Tantivy FastFields (no change from current search behavior)
- Automatic one-time migration from SQLite to JSON on first launch after upgrade
- Maintain all existing Tauri command APIs (frontend unchanged)

### Non-Goals

- Changing the frontend code or Tauri command signatures
- Changing the Tantivy search schema or scoring algorithms
- Adding a new database engine (e.g., sled, redb)
- Implementing real-time file watching (separate concern)

## Decisions

### Decision 1: JSON files for structured data

**What**: Replace SQLite tables with JSON files serialized via `serde_json`.

**Why**: Bookmarks and settings are small, document-oriented data. A JSON file can be read entirely into memory on startup and written atomically (write to temp file, then rename). This matches the existing in-memory cache pattern already used for settings.

**Alternatives considered**:
- **sled/redb** (embedded KV store): Adds a new dependency without clear benefit. JSON is human-readable and debuggable.
- **TOML/YAML for settings**: JSON is already used for export/import and matches serde ecosystem. Consistency wins.
- **Keep SQLite for bookmarks only**: Half-measure that doesn't achieve the goal of removing the dependency.

### Decision 2: Tantivy stores indexed file metadata

**What**: The `indexed_files` and `file_usage_history` tables are replaced by storing file metadata directly in Tantivy's file index. File path, name, extension, size, and modified_at are already stored as STORED/FAST fields in the Tantivy schema.

**Why**: Tantivy already indexes all file metadata. The SQLite copy exists only for rebuild capability. Instead, we can rebuild from the filesystem (re-scan directories) which is the actual source of truth for files.

**Alternatives considered**:
- **Separate files.json**: Could grow very large (100K files × ~200 bytes = 20MB). Tantivy handles this scale better.
- **Keep SQLite for files only**: Defeats the purpose.

### Decision 3: Atomic file writes with temp + rename

**What**: All JSON file writes use a "write to .tmp, then atomic rename" pattern.

**Why**: Prevents data corruption if the app crashes during a write. This is the standard pattern for file-based persistence. On macOS/Linux, rename is atomic within the same filesystem.

### Decision 4: In-memory data model with file persistence

**What**: On startup, load all JSON files into memory (`Arc<RwLock<BookmarkStore>>`, `Arc<RwLock<Settings>>`). Mutations go to memory first, then persist to disk asynchronously.

**Why**: This matches the current architecture where SQLite is read into memory (settings_cache) and Tantivy searches happen in-memory. Reads are instant; writes are fast with async file I/O.

### Decision 5: Migration strategy

**What**: On startup, if `bookmarks.db` (SQLite file) exists but `bookmarks.json` does not, run one-time migration. Read all data from SQLite, write to JSON files, and optionally rename the SQLite file to `.bak`.

**Why**: Seamless upgrade experience. Users don't lose data.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| JSON files corrupted on crash | Data loss | Atomic write (temp + rename); keep `.bak` after migration |
| Large bookmark collections (>10K) slow to serialize | Slight startup delay | Unlikely for bookmarks; files stay in Tantivy |
| No transactional guarantees across files | Inconsistent state if app crashes between writes | Each file is independent; Tantivy index can be rebuilt from JSON |
| Concurrent access from multiple processes | Data corruption | Desktop app is single-process; use file locks if needed |
| Migration fails mid-way | Partial migration | Keep SQLite file intact; retry on next launch |

## Migration Plan

1. On app startup, check for `{app_data_dir}/bookmarks.db` (SQLite) existence
2. If SQLite exists AND `bookmarks.json` does not exist:
   a. Open SQLite database read-only
   b. Export bookmarks → `bookmarks.json`
   c. Export settings → `settings.json`
   d. Export search_directories → `directories.json`
   e. Rename `bookmarks.db` → `bookmarks.db.bak`
3. If migration fails, log error and continue with empty state (SQLite file preserved)
4. Tantivy index rebuild happens naturally via existing `rebuild_index_if_needed` logic

### Rollback

- Keep `.bak` file for at least one release cycle
- Users can manually rename `.bak` back to `.db` and downgrade

## Open Questions

- Should `usage_history` tracking beyond Tantivy's FastFields be preserved? Currently usage_history stores individual access events while Tantivy only stores aggregate count + last timestamp. Recommendation: drop individual event history, keep only aggregate frecency data in Tantivy (sufficient for ranking).
- Should the `directories.json` include cached file counts and last_indexed_at? Recommendation: yes, useful for UI display without re-scanning.
