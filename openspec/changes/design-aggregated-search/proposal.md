# Design Extensible Aggregated Search

## Summary

Upgrade the search system from hardcoded providers to a dynamic ecosystem with structured query syntax, while keeping ranking simple and proven.

## Motivation

The current search implementation has two key limitations:

1. **Hardcoded Registration**: Providers are registered in `lib.rs` startup code. Plugins cannot participate as first-class search sources.
2. **Unstructured Queries**: Input is treated as raw strings. Advanced filtering (e.g., `type:pdf`) requires per-provider hacks.

## Goals

1. **Structured Query Parser**: Implement `scope:` and `type:` syntax for precise search control.
2. **Dynamic Provider Registry**: Refactor `SearchAggregator` to use `HashMap` for runtime add/remove.
3. **Manifest-Based Plugin Registration**: Auto-register plugins as search providers when loading `plugin.toml`.

## Non-Goals

- **Runtime Plugin FFI Registration**: Plugins are subprocesses - they cannot directly register Rust trait objects. Registration must be Host-driven based on Manifest.
- **Context-Aware Ranking**: Tracking active apps/windows requires OS permissions and adds complexity with unclear benefit.
- **AI Re-ranking**: Adds latency (50-200ms), conflicts with fast search goal.
- **Separate RankingEngine**: Over-abstraction for a single weighted formula.

## Key Design Decisions

### Plugin Registration Architecture

**Wrong**: "Plugins register themselves via IPC/FFI at runtime"
**Correct**: "Host scans `plugin.toml`, creates `PluginSearchProvider` proxy, registers with Aggregator"

```toml
# plugin.toml
[plugin]
name = "github-search"
title = "GitHub Search"

[[commands]]
name = "search-repos"
mode = "search"  # Host detects this and auto-registers
keyword = "gh"
script = "dist/index.js"
runtime = "node"
```

### Aggregator Refactor

```rust
// Before: Vec-based, append-only
providers: Vec<Box<dyn SearchProvider>>

// After: HashMap with RwLock for dynamic access
providers: RwLock<HashMap<String, Box<dyn SearchProvider>>>
```

### Scope Routing

- Query `gh: react` → Only dispatch to provider with ID `"gh"` or `"plugin:gh"`
- Query `rust type:pdf` → Broadcast to all, `FileSearchProvider` applies filter
