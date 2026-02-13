# Tasks: Design Extensible Aggregated Search

## Phase 1: Core Registry & Dynamic Providers

- [ ] 1.1 Create `SearchRegistry` (SearchRouter) to manage `SearchProvider` lifecycle (add/remove/list).
- [ ] 1.2 Update `SearchAggregator` to use the dynamic `SearchRegistry` instead of a static list.
- [ ] 1.3 Expose `register_provider` as a public API for plugins (via FFI/IPC).

## Phase 2: Query Parser & Basic Syntax

- [ ] 2.1 Implement `QueryParser` struct to handle raw input strings.
- [ ] 2.2 Support `scope:` syntax (e.g., `gh:`, `file:`) for direct provider selection.
- [ ] 2.3 Update `SearchContext` to include parsed `StructuredQuery` fields.

## Phase 3: Ranking Engine & Context Awareness

- [ ] 3.1 Implement `RankingEngine` trait with `score(result, context)` method.
- [ ] 3.2 Create `ContextService` to track active app/window/time.
- [ ] 3.3 Update `SearchAggregator` to use `RankingEngine` for final sort.

## Phase 4: Advanced Filtering & Optimization

- [ ] 4.1 Define standardized filters (e.g., `type`, `date`, `size`).
- [ ] 4.2 Update `SearchProvider` trait to accept structured filters.
- [ ] 4.3 Optimize `QueryParser` for <10ms latency.
