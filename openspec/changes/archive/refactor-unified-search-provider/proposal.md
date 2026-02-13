# Change: Refactor Search Architecture to Support Multi-Resource Unified Search

## Why

The current search architecture uses hardcoded, per-source search calls (`search_bookmarks` + `search_files`) with simple concatenation of results. This design makes it expensive to add new data sources — each new source requires changes in 4-5 files (Tantivy engine, Tauri commands, frontend `App.vue`, type definitions, grouping logic). Additionally, results from different sources cannot be cross-ranked, and the existing `SearchSettings` (show_bookmarks, show_files, max_results, fuzzy_matching) are defined but never wired into actual search calls.

This refactoring introduces a `SearchProvider` trait abstraction and a `SearchAggregator` coordinator, enabling O(1) effort to add new data sources while providing unified cross-source ranking and proper settings integration.

## What Changes

- **ADDED** `SearchProvider` trait — a common interface for all searchable data sources (bookmarks, files, plugins, future sources)
- **ADDED** `SearchAggregator` — orchestrates parallel queries across all registered providers, normalizes scores, and produces a single ranked result list
- **ADDED** `unified_search` Tauri command — single IPC entry point replacing per-source frontend calls
- **MODIFIED** Search Settings integration — wire `SearchSettings` (max_results, show_bookmarks, show_files, fuzzy_matching) into actual search execution
- **MODIFIED** Frontend search flow — replace multiple `invoke()` calls with single `unified_search` call
- Existing `TantivySearchEngine` split into `BookmarkSearchProvider` and `FileSearchProvider` implementations
- Plugin results integrated into the unified search pipeline (no longer mutually exclusive with built-in results)

## Impact

- Affected specs: `search-engine` (archived), `bookmark-search`, `file-search`, `plugin-runtime`
- Affected code:
  - `src-tauri/src/search/` — new `provider.rs`, `aggregator.rs` modules; refactored `tantivy_engine.rs`
  - `src-tauri/src/commands/search.rs` — new `unified_search` command
  - `src-tauri/src/lib.rs` — register new command, update `AppState`
  - `src-tauri/src/services/data_service.rs` — delegate to aggregator
  - `src/App.vue` — replace dual `invoke()` with single `unified_search`
  - `src/types/search.ts` — add `UnifiedSearchResult` type
  - `src/composables/useGroupedResults.ts` — adapt to unified results
- **NOT breaking**: Existing per-source commands (`search_bookmarks`, `search_files`) remain available for backward compatibility; deprecated but not removed
- Performance constraint: unified search MUST respond within 100ms (per project.md)
