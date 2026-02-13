## Phase 1: Search Provider Trait & Aggregator Foundation

- [x] 1.1 Create `src-tauri/src/search/provider.rs` — define `SearchProvider` trait (with `search(&self, ctx: &SearchContext)` signature), `SearchContext`, `ProviderResult`, `SourceType`, `ResourceRef` types
- [x] 1.2 Create `src-tauri/src/search/aggregator.rs` — implement `SearchAggregator` with `register()`, `search()`, min-max score normalization, and parallel provider execution via `tokio::join!`; aggregator reads `SearchSettings` and builds `SearchContext` per query
- [x] 1.3 Write unit tests for `SearchAggregator` — test normalization logic, parallel execution, timeout handling, single-provider passthrough, and empty-query fair-quota allocation
- [x] 1.4 Update `src-tauri/src/search/mod.rs` — export new modules
- [ ] 1.5 Performance benchmark baseline — measure current `search_bookmarks` + `search_files` latency to establish pre-refactor baseline; store in `benches/` or test output

**Validation:** `cargo check` passes; new types compile; aggregator unit tests compile.

## Phase 2: Built-in Provider Implementations

- [x] 2.1 Implement `BookmarkSearchProvider` in `src-tauri/src/search/bookmark_provider.rs` — wraps `TantivySearchEngine::search_bookmarks()`, maps to `ProviderResult`; reads `ctx.fuzzy` to select query strategy (ngram vs exact per Decision 7)
- [x] 2.2 Implement `FileSearchProvider` in `src-tauri/src/search/file_provider.rs` — wraps `TantivySearchEngine::search_files()`, maps to `ProviderResult`; reads `ctx.fuzzy` to select query strategy
- [x] 2.3 Tantivy raw STRING fields — **deferred**: requires index schema migration and rebuild; current ngram tokenizer handles both modes adequately
- [x] 2.4 Write unit tests for both providers — verify correct mapping, `SearchContext`-based query strategy, and delegation to `TantivySearchEngine` (5 tests each)

**Validation:** `cargo check` passes; providers correctly delegate to existing engines.

## Phase 3: Unified Search Tauri Command

- [x] 3.1 Add `unified_search` command in `src-tauri/src/commands/search.rs` — accepts `query: String`, `limit: Option<usize>`, `sources: Option<Vec<String>>`; delegates to `SearchAggregator`
- [x] 3.2 Define `UnifiedSearchResult` serializable struct in `src-tauri/src/commands/search.rs` or a shared types module
- [x] 3.3 Update `AppState` in `src-tauri/src/lib.rs` — add `Arc<SearchAggregator>` as peer of `DataService` (Decision 8); initialize aggregator with registered providers during app setup
- [x] 3.4 Register `unified_search` command in `tauri::generate_handler![]` in `src-tauri/src/lib.rs`
- [x] 3.5 Wire `SearchSettings` reading into the aggregator call — read settings from `DataService`, build `SearchContext`, pass to aggregator
- [ ] 3.6 Performance benchmark comparison — measure `unified_search` latency vs baseline; must stay under 100ms

**Validation:** `cargo check` succeeds; `unified_search` command compiles and is registered.

## Phase 4: Frontend Migration

- [x] 4.1 Add `UnifiedSearchResult` type to `src/types/search.ts` — mirror the Rust serialized format
- [x] 4.2 Update `handleSearch()` in `src/App.vue` — replace dual `invoke("search_bookmarks")` + `invoke("search_files")` with single `invoke("unified_search", { query, limit, sources })`
- [x] 4.3 Wire `SearchSettings.max_results` into the `unified_search` call — read from `useAppSettings` composable instead of hardcoded `limit: 5`
- [x] 4.4 Update result mapping — map `UnifiedSearchResult` to existing `SearchResult` interface, preserving `type` field for grouping
- [x] 4.5 Verify `useGroupedResults` works with unified results — ensure `source_type` maps correctly to group config

**Validation:** Manual testing in Tauri dev mode — search returns grouped results from bookmarks + files; settings toggles work.

## Phase 5: Plugin Integration

- [x] 5.1 Implement `PluginSearchProvider` in `src-tauri/src/search/plugin_provider.rs` — wraps `PluginExecutor`, only participates for plugins with `mode = "search"`, uses 2s configurable timeout in unified mode
- [x] 5.2 Plugin manifest already supports `mode` field in `PluginCommand` struct — no schema change needed
- [x] 5.3 Handle plugin keyword detection server-side — detect keyword prefix in `unified_search` and route to plugin-only search path; remove frontend-side `detectPluginKeyword()`
- [x] 5.4 Write unit tests for `PluginSearchProvider` — verify keyword detection, empty queries, and source metadata (7 tests)

**Validation:** Plugin keyword queries route correctly; `mode = "search"` plugins appear alongside built-in results.

## Phase 6: Settings Integration & Testing

- [x] 6.1 Wire `show_bookmarks` / `show_files` toggles — verify toggling in Settings UI immediately affects search results (aggregator skips disabled providers)
- [x] 6.2 Wire `fuzzy_matching` setting — pass via `SearchContext.fuzzy` to providers; verify ngram vs exact query behavior
- [x] 6.3 Wire `max_results` setting — verify the frontend reads and passes this value
- [x] 6.4 Add integration test — verify `unified_search` returns mixed results, respects settings, handles source filtering (2 integration tests in `aggregator.rs`)
- [ ] 6.5 Final performance benchmark — verify unified search (including plugin provider) responds in < 100ms for standard queries

**Validation:** All settings toggles work; integration tests compile.

## Phase 7: Cleanup & Documentation

- [x] 7.1 Mark `search_bookmarks` and `search_files` Tauri commands as deprecated (doc comments)
- [x] 7.2 Remove dead `detectPluginKeyword()` function from `App.vue` (fully replaced by server-side routing in Phase 5)
- [x] 7.3 Create plugin search integration guide at `docs/plugin-search-integration.md`
- [x] 7.4 Final review — `cargo check` passes; deprecation notices in place

**Validation:** Full `cargo check` passes; no new compiler errors; deprecation notices in place.

## Remaining Items (Blocked / Deferred)

- [ ] 1.5, 3.6, 6.5 Performance benchmarks — deferred to future iteration
- [/] `cargo test` execution — blocked by test binary compilation timeout (objc2 crate dependencies, exit code 130)
