## Context

The Bookmarks Launcher is a Spotlight/Alfred-style desktop app providing unified search across browser bookmarks, local files, and plugin-provided results. The current search layer uses `TantivySearchEngine` directly with separate methods per data source, and the frontend hard-codes parallel calls to `search_bookmarks` + `search_files`. Plugin search is a completely separate code path triggered by keyword prefix detection, making it impossible to mix plugin results with built-in results.

**Current pain points:**
1. Adding a new data source (e.g., notes, contacts, clipboard history) requires touching 4-5 files across frontend and backend
2. No cross-source score normalization — BM25 scores from different Tantivy indexes are incomparable
3. Plugin results and built-in results are mutually exclusive (either/or routing)
4. `SearchSettings` fields (`show_bookmarks`, `show_files`, `max_results`, `fuzzy_matching`) exist in models but are dead code
5. Frontend concatenates results without global ranking: `[...bookmarks, ...files]`

**Stakeholders:** End users (want relevant results regardless of source), plugin developers (want results to appear alongside built-in results), maintainers (want O(1) cost to add new sources).

**Constraints (from project.md):**
- Search MUST respond in < 100ms
- Idle memory < 100MB
- Offline-first: all functionality without internet
- Privacy: no telemetry

## Goals / Non-Goals

### Goals
- Introduce `SearchProvider` trait as the common interface for all searchable data sources
- Create `SearchAggregator` to orchestrate parallel provider queries with unified ranking
- Provide a single `unified_search` Tauri command as the primary frontend search endpoint
- Wire `SearchSettings` into the actual search pipeline
- Allow plugin results to appear alongside built-in results (not mutually exclusive)
- Maintain backward compatibility: existing per-source commands still work

### Non-Goals
- Implementing new data sources (notes, contacts, etc.) — this proposal only creates the extensibility foundation
- Full-text content indexing (indexing file contents) — out of scope
- Remote/network data sources — the trait supports async but this proposal does not implement any
- Changes to the plugin subprocess protocol — plugins continue to use stdin/stdout JSON
- UI redesign — the existing grouped results display with collapsible headers remains

## Decisions

### Decision 1: `SearchProvider` as an async trait

```rust
#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn source_id(&self) -> &str;
    fn source_label(&self) -> &str;
    fn source_type(&self) -> SourceType; // Bookmark, File, Plugin, Custom
    async fn search(&self, ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError>;
}
```

The `is_enabled()` method is removed from the trait. Enable/disable logic is handled by the aggregator based on `SearchSettings` and the `SearchContext.sources` filter — providers themselves are stateless query executors.

**Why async:** Plugin providers need to spawn subprocesses; future providers may call HTTP APIs. Using `async_trait` keeps the interface uniform.

**Alternatives considered:**
- Synchronous trait with `spawn_blocking` for async providers — rejected because it adds complexity at every call site
- Enum-based dispatch instead of trait — rejected because it closes the system to external extension
- Generic trait with associated types — rejected as over-engineered for current needs

### Decision 2: Score normalization via min-max per provider

Each provider returns raw scores. The aggregator normalizes each provider's scores to [0.0, 1.0] using min-max normalization within that provider's result set, then applies a global ranking formula:

```
global_score = normalized_relevance * 0.7 + normalized_frecency * 0.3
```

**Why min-max:** Simple, deterministic, no learning required. BM25 scores have different ranges across indexes (bookmark titles vs file paths), so raw comparison is meaningless.

**Alternatives considered:**
- Z-score normalization — rejected because it requires sufficient sample size
- Pre-calibrated score ranges — rejected because ranges shift as index grows
- Interleaving (round-robin from each source) — rejected because it ignores relevance

### Decision 3: Aggregator holds Vec<Box<dyn SearchProvider>>

```rust
pub struct SearchAggregator {
    providers: Vec<Box<dyn SearchProvider>>,
}
```

**Why dynamic dispatch:** Provider set is small (3-5 providers), dispatched once per search. Virtual call overhead is negligible vs. Tantivy query time. Enables runtime registration of new providers (e.g., plugin providers discovered at startup).

**Alternatives considered:**
- Static dispatch with generics — rejected because provider set is determined at runtime
- HashMap by source_id — unnecessary; Vec iteration over 3-5 items is faster

### Decision 4: Single `unified_search` command, keep legacy commands

The new `unified_search` command becomes the primary search endpoint. Existing `search_bookmarks` and `search_files` commands remain functional but are marked deprecated.

**Why keep legacy:** Avoids a breaking change. Settings UI and diagnostic tools may use per-source queries.

### Decision 5: Settings passed via SearchContext (not held by providers)

Providers SHALL NOT hold references to `SearchSettings`. Instead, the aggregator constructs a `SearchContext` per query and passes it to each provider:

```rust
pub struct SearchContext {
    pub query: String,
    pub limit: usize,
    pub fuzzy: bool,
    pub sources: Option<Vec<String>>,  // None = all enabled
}
```

The aggregator reads `SearchSettings` once at query time and builds a `SearchContext`. Each provider receives the context as a parameter to `search()`.

**Why context-per-query:**
- Providers stay stateless and pure — no need to observe settings changes
- Settings changes take effect immediately on the next query without reloading providers
- Easier to test — inject any context without mocking a settings store
- Enables per-call overrides (e.g., frontend passes explicit `sources` filter)

**Alternatives considered:**
- Inject `Arc<RwLock<SearchSettings>>` into each provider — rejected because it couples providers to the settings store and requires synchronization
- Read settings inside each provider — rejected because it scatters settings access across N providers instead of centralizing it in the aggregator

### Decision 6: Plugin integration via PluginSearchProvider

Create a `PluginSearchProvider` that wraps `PluginExecutor` and participates in the unified search pipeline. This requires a mode switch:

- **Keyword-prefixed queries** (e.g., "calc 2+3"): Route only to the matching plugin (existing behavior)
- **Generic queries** (no keyword prefix): Optionally include plugins that declare `mode = "search"` in their manifest

**Why conditional inclusion:** Not all plugins make sense for generic search (e.g., a calculator plugin). Only plugins explicitly declaring `mode = "search"` participate in unified results.

### Decision 7: fuzzy_matching toggle via query strategy (not index rebuild)

The current `TantivySearchEngine` uses ngram tokenizer at index time for CJK and prefix matching support. The `fuzzy_matching` setting controls **query-time behavior only**, not index structure:

- **`fuzzy_matching = true` (default):** Use the ngram-tokenized query parser (current behavior). Queries like "goo" match "google" via ngram overlap.
- **`fuzzy_matching = false`:** Use a `TermQuery` with exact term matching against the raw (untokenized) field. Only exact or prefix matches are returned.

Implementation approach:
```rust
// Inside BookmarkSearchProvider / FileSearchProvider
fn build_query(&self, query: &str, fuzzy: bool) -> Box<dyn Query> {
    if fuzzy {
        // Use QueryParser with ngram-tokenized field (existing behavior)
        let parser = QueryParser::for_index(&self.index, vec![self.title_field]);
        parser.parse_query(query).unwrap_or_else(|_| Box::new(AllQuery))
    } else {
        // Use TermQuery against the raw (untokenized) stored field
        let term = Term::from_field_text(self.title_raw_field, query);
        Box::new(PhrasePrefixQuery::new(vec![term]))
    }
}
```

**Why query-time only:** Rebuilding the Tantivy index to toggle tokenizer is prohibitively expensive (seconds) and violates the <100ms constraint. Query-time switching is instant and the ngram index still supports exact queries.

**Prerequisite:** Ensure each indexed field has both a `TEXT` (ngram-tokenized) variant and a `STRING` (raw/untokenized) variant in the Tantivy schema. The raw field is already present for stored values; we add it as a searchable field.

### Decision 8: SearchAggregator as a peer of DataService in AppState

The `SearchAggregator` sits alongside `DataService` in `AppState`, not inside it:

```rust
pub struct AppState {
    pub data_service: Arc<DataService>,        // CRUD + index maintenance
    pub search_aggregator: Arc<SearchAggregator>, // Search orchestration
    pub http_client: reqwest::Client,
    pub plugin_registry: Option<Arc<PluginRegistry>>,
    pub plugin_executor: Option<Arc<PluginExecutor>>,
}
```

**Responsibility boundaries:**

- `DataService` — data persistence (SQLite CRUD), index maintenance (rebuild, add/remove documents), source of truth
- `SearchAggregator` — search orchestration only: dispatch queries to providers, normalize scores, rank results

The aggregator holds references to providers, which in turn hold `Arc<TantivySearchEngine>` (shared with `DataService`) for read-only search access. This avoids circular dependencies while sharing the underlying index.

```
                        AppState
                       /        \
               DataService    SearchAggregator
                   |              |
            TantivySearchEngine   Vec<Box<dyn SearchProvider>>
            (write: add/remove)      /         |          \
                              Bookmark    File     Plugin
                              Provider   Provider  Provider
                                 |          |          |
                            TantivySearchEngine   PluginExecutor
                            (read: search only)
```

**Why peer-level:** Placing the aggregator inside `DataService` would conflate data management with query orchestration. Peer-level keeps each concern independently testable and replaceable.

## Risks / Trade-offs

| Risk | Severity | Mitigation |
|------|----------|------------|
| Performance regression from async trait overhead | Low | Provider count is small (3-5); benchmark before/after |
| Score normalization produces unintuitive rankings | Medium | Default weights (0.7/0.3) configurable; fallback to source-grouped display |
| Plugin subprocess latency slows unified results | Medium | Plugin providers have independent timeout; aggregator returns available results without waiting for slow providers |
| Breaking change if legacy commands removed too early | Low | Keep legacy commands, deprecate in docs only |
| Over-abstraction for current 2-source system | Low | Trait is minimal (~5 methods); complexity is justified by upcoming data sources |

## Migration Plan

### Phase 1: Foundation (non-breaking)

1. Add `search/provider.rs` with `SearchProvider` trait, `SearchContext`, and `ProviderResult` type
2. Add `search/aggregator.rs` with `SearchAggregator`
3. Record performance benchmark baseline for current search
4. All existing tests continue to pass

### Phase 2: Built-in Provider Implementations

1. Implement `BookmarkSearchProvider` and `FileSearchProvider` wrapping existing `TantivySearchEngine` methods
2. Add raw (untokenized) `STRING` fields to Tantivy schema for fuzzy_matching toggle (Decision 7)
3. Providers receive `SearchContext` — no direct settings access

### Phase 3: Unified Search Command

1. Add `unified_search` Tauri command alongside existing commands
2. Add `Arc<SearchAggregator>` as peer of `DataService` in `AppState` (Decision 8)
3. Aggregator reads `SearchSettings`, builds `SearchContext`, dispatches to providers
4. Benchmark `unified_search` latency vs baseline

### Phase 4: Frontend migration

1. Update `App.vue` to use `unified_search` instead of dual `invoke()` calls
2. Wire `SearchSettings` into the `unified_search` call
3. Update `SearchResult` type to support unified result format
4. Keep legacy code paths behind feature flag for rollback

### Phase 5: Plugin integration

1. Add `PluginSearchProvider` wrapping `PluginExecutor`
2. Update plugin manifest to support `mode = "search"` declaration
3. Move `detectPluginKeyword()` to server-side routing
4. Allow non-keyword plugin results in unified search

### Rollback

- Phase 1-2 are additive-only; rollback = delete new files
- Phase 3: command is additive; rollback = remove command registration
- Phase 4: frontend can revert to dual `invoke()` calls
- Phase 5: plugin integration is opt-in per plugin

## Open Questions

1. **Should the aggregator support streaming results?** (show bookmark results immediately while files are still loading) — deferred to future proposal
2. **Should plugin timeout be shorter in unified search mode?** (e.g., 2s instead of 10s to avoid blocking) — recommended yes, configurable per-provider
3. **Should we expose provider-level search stats in the UI?** (e.g., "3 bookmarks, 2 files, 1 plugin") — already supported by `useGroupedResults`, no changes needed
