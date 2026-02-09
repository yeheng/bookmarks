# Tasks: Replace SQLite FTS with Tantivy

## 1. Setup and Dependencies

- [x] 1.1 Add tantivy dependency to Cargo.toml (`tantivy = "0.22"`, `async-trait = "0.1"`)
- [x] 1.2 Create search module directory structure (`src/search/`)
- [x] 1.3 Create module exports in `src/search/mod.rs`

## 2. Core Search Engine Implementation

- [x] 2.1 Define `SearchEngine` trait in `src/search/engine.rs`
- [x] 2.2 Define search result types (`BookmarkSearchResult`, `FileSearchResult`)
- [x] 2.3 Define `IndexStats` struct for statistics
- [x] 2.4 Implement bookmark schema builder in `src/search/schema.rs`
- [x] 2.5 Implement file schema builder with ngram tokenizer in `src/search/schema.rs`
- [x] 2.6 Write unit tests for schema builders

## 3. Tantivy Engine Implementation

- [x] 3.1 Implement `TantivySearchEngine::new()` - initialize indexes and writers
- [x] 3.2 Implement bookmark indexing (`index_bookmark`)
- [x] 3.3 Implement bookmark deletion (`delete_bookmark`)
- [x] 3.4 Implement bookmark search with BM25 (`search_bookmarks`)
- [x] 3.5 Implement file indexing (`index_file`)
- [x] 3.6 Implement file deletion (`delete_file`, `delete_directory_files`)
- [x] 3.7 Implement file search with fuzzy matching (`search_files`)
- [x] 3.8 Implement frecency score calculation (query SQLite usage_history)
- [x] 3.9 Implement combined scoring (BM25 70% + frecency 30%)
- [x] 3.10 Implement `rebuild_bookmark_index()` - full rebuild from SQLite
- [x] 3.11 Implement `rebuild_file_index()` - full rebuild from SQLite
- [x] 3.12 Implement `get_stats()` - index statistics
- [x] 3.13 Implement empty query handling (return recent items)
- [x] 3.14 Write integration tests for TantivySearchEngine

## 4. Application State Integration

- [x] 4.1 Add `SearchEngine` to `AppState` struct
- [x] 4.2 Initialize `TantivySearchEngine` in `setup()` function
- [x] 4.3 Configure index directory path (app_data/tantivy_indexes)
- [x] 4.4 Add background index initialization on startup
- [x] 4.5 Add error handling for search engine initialization

## 5. Command Layer Updates

- [x] 5.1 Update `search_bookmarks` command to use SearchEngine
- [x] 5.2 Update `search_files` command to use SearchEngine
- [x] 5.3 Add `rebuild_search_index` command
- [x] 5.4 Add `get_search_stats` command
- [x] 5.5 Update `create_bookmark` to index in Tantivy
- [x] 5.6 Update `update_bookmark` to re-index in Tantivy
- [x] 5.7 Update `delete_bookmark` to remove from Tantivy
- [x] 5.8 Update file scanner to index files in Tantivy
- [x] 5.9 Update directory removal to clean Tantivy index

## 6. Database Schema Cleanup

- [x] 6.1 Remove FTS5 virtual table creation for bookmarks_fts
- [x] 6.2 Remove FTS5 virtual table creation for files_fts
- [x] 6.3 Remove FTS5 triggers (bookmarks_ai, bookmarks_au, bookmarks_ad)
- [x] 6.4 Remove FTS5 triggers (files_ai, files_au, files_ad)
- [x] 6.5 Update database tests to remove FTS5 assertions

## 7. Testing and Validation

- [x] 7.1 Write unit tests for search engine trait
- [x] 7.2 Write integration tests for bookmark search
- [x] 7.3 Write integration tests for file search
- [x] 7.4 Write integration tests for index rebuild
- [x] 7.5 Test empty query behavior
- [x] 7.6 Test frecency scoring accuracy
- [x] 7.7 Performance test: verify <100ms search latency
- [x] 7.8 Test index persistence across app restarts

## 8. Documentation

- [x] 8.1 Update project.md tech stack (SQLite FTS5 -> Tantivy)
- [x] 8.2 Document search engine architecture in code comments
- [x] 8.3 Add inline documentation for SearchEngine trait
