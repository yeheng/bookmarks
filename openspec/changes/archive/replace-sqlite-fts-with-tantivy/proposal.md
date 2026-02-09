# Change: Replace SQLite FTS5 with Tantivy Search Engine

## Why

SQLite FTS5 is adequate for basic full-text search, but Tantivy offers significant advantages:

1. **Better Chinese/CJK support** - Tantivy supports custom tokenizers including jieba for Chinese text segmentation
2. **Advanced search features** - Phrase queries, fuzzy matching, ngram-based search, faceted search
3. **Better performance at scale** - Optimized for search-heavy workloads with better memory efficiency
4. **Richer ranking options** - More control over BM25 parameters and custom scoring
5. **Index management** - Better support for incremental updates without rebuild

## What Changes

### Backend (Rust)

- **BREAKING**: Remove FTS5 virtual tables from SQLite schema
- Add new `search` module with Tantivy-based search engine
- Create `SearchEngine` trait for abstraction
- Implement `TantivySearchEngine` for bookmarks and files indexing
- Modify CRUD operations to update Tantivy index (replace SQL triggers)
- Add index rebuild and maintenance commands

### Database Schema

- Remove `bookmarks_fts` virtual table and triggers
- Remove `files_fts` virtual table and triggers
- SQLite remains as source of truth for data storage

### Index Storage

- Create new index directory: `APP_DATA/tantivy_indexes/`
- Separate indexes for bookmarks and files
- Persistent index with mmap-based access

### API Changes

- Keep existing Tauri command signatures (frontend unchanged)
- Add new commands for index management:
  - `rebuild_search_index(index_type: String)`
  - `get_search_stats()`

## Impact

- **Affected specs**: bookmark-search, file-search
- **New spec**: search-engine (abstraction layer)
- **Affected code**:
  - `src-tauri/src/db/mod.rs` - Remove FTS5 schema
  - `src-tauri/src/commands/search.rs` - Use SearchEngine
  - `src-tauri/src/commands/file_search.rs` - Use SearchEngine
  - `src-tauri/src/commands/bookmarks.rs` - Index updates
  - `src-tauri/src/services/file_scanner.rs` - Index updates
  - `src-tauri/src/lib.rs` - Initialize search engine
- **New files**:
  - `src-tauri/src/search/mod.rs`
  - `src-tauri/src/search/engine.rs`
  - `src-tauri/src/search/schema.rs`
  - `src-tauri/src/search/tantivy_engine.rs`
- **Dependencies**: Add `tantivy = "0.22"`, `async-trait = "0.1"`
