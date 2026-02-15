# Tasks: Design Extensible Aggregated Search

## Phase 1: Structured Query Engine (The Foundation)

**What**: Implement a parser for `[scope:] [terms] [key:value]` syntax.
**Why**: Providers need structured intent, not just raw strings.

- [ ] 1.1 Create `src-tauri/src/search/query_parser.rs` with `StructuredQuery` struct.
- [ ] 1.2 Implement parsing logic for Scopes (`gh:`), Filters (`type:pdf`), and Quoted Terms (`"hello world"`).
- [ ] 1.3 Extend `SearchContext` with `structured_query: StructuredQuery` field. Keep all existing fields (`query`, `limit`, `fuzzy`, `sources`) unchanged.
- [ ] 1.4 Update `unified_search` command to parse raw query via `QueryParser` and populate `SearchContext.structured_query`.
- [ ] 1.5 Test Parser with edge cases (empty, only filters, multiple scopes, colons in URLs).

## Phase 2: Dynamic Aggregator (The Registry)

**What**: Refactor `SearchAggregator` to support dynamic add/remove of providers via HashMap.
**Why**: To support toggling plugins without restarting app code.

- [ ] 2.1 Refactor `SearchAggregator` internal storage from `Vec` to `RwLock<HashMap<String, Box<dyn SearchProvider>>>`.
- [ ] 2.2 Add public methods `register_provider(id, provider)` and `unregister_provider(id)`.
- [ ] 2.3 Update `BookmarkSearchProvider` and `FileSearchProvider` to read `ctx.structured_query.filters` where applicable (e.g., `type:` filter in file search).
- [ ] 2.4 Update existing aggregator unit tests to use the new `RwLock<HashMap>` API and `SearchContext.structured_query`.

## Phase 3: Plugin Integration (The Wiring)

**What**: Auto-register plugins as search providers based on Manifest.
**Why**: Plugins should just "work" when installed.

- [ ] 3.1 Refactor `PluginSearchProvider` from single monolith to per-command proxy: new constructor takes `(plugin_name, PluginCommand, Arc<PluginExecutor>, Arc<DataService>)`. Remove `detect_keyword()` logic.
- [ ] 3.2 Update `PluginRegistry::discover()`: when a command with `mode="search"` is found, create a `PluginSearchProvider` proxy and call `aggregator.register("plugin:<keyword>", proxy)`. This requires `PluginRegistry` to hold a reference to `Arc<SearchAggregator>` (see wiring note below).
- [ ] 3.3 Update startup wiring in `lib.rs`: inject `Arc<SearchAggregator>` into `PluginRegistry` so it can register/unregister providers during plugin lifecycle events (discover, install, uninstall).
- [ ] 3.4 Update `PluginSearchProvider.search()`: pass structured filters to the plugin process (update JSON protocol to include `filters` object from `ctx.structured_query`).
- [ ] 3.5 Verify scope routing end-to-end: query `gh: react` → `QueryParser` extracts scope → Aggregator dispatches only to `"plugin:gh"` provider.

---

## Future Work (Not In Scope for v1)

The following may be explored after Phases 1-3 are stable and validated:

- **Context Boosting**: Add `context: HashMap<String, String>` to search parameters for basic boosting (e.g., if context is "IDE", boost code files). Deferred because the ranking-engine spec marks this as "not v1 requirement".
- **Provider Capabilities**: Dynamic capability negotiation between aggregator and providers. Not needed until provider count grows significantly.

---

## Removed (Over-Engineered)

The following were considered but rejected:

| Rejected | Reason |
|----------|--------|
| Runtime FFI registration | Plugins are subprocesses, cannot register Rust objects |
| ContextAware Service | Requires OS permissions, complex, unclear benefit |
| AI Re-ranking | Adds 50-200ms latency, conflicts with fast search goal |
| Separate RankingEngine | Over-abstraction for a single weighted formula |
| Provider Capabilities | Not needed with 3 providers |
