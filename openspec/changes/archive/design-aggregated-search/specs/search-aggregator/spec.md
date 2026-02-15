# Capability: Dynamic Search Aggregator

## Status

Proposed (Refactored from `SearchAggregator`)

## Summary

Refactor `SearchAggregator` to use `HashMap` storage with `std::sync::RwLock` for thread-safe dynamic registration. Plugins are registered by Host based on Manifest, not via runtime API.

## Motivation

Current `Vec`-based storage doesn't support:
1. O(1) lookup for scope routing
2. Dynamic unregister for plugin lifecycle

## Critical Architecture Note

**Plugins are subprocesses.** They CANNOT directly register Rust trait objects via FFI/IPC.

**Correct Flow**:
1. Plugin declares `mode = "search"` on a command in `plugin.toml`
2. Host (`PluginRegistry::discover`) detects this
3. Host creates a **per-command** `PluginSearchProvider` proxy
4. Host calls `aggregator.register("plugin:<keyword>", proxy)`

## Requirements

### ADDED HashMap Storage

The aggregator MUST use `HashMap<String, Box<dyn SearchProvider>>` for O(1) provider lookup.

#### Scenario: Scope Routing

Given query with `scope = "gh"`
When aggregator dispatches search
Then only provider with ID `"gh"` or `"plugin:gh"` is invoked

### ADDED Thread-Safe Registration

The aggregator MUST support concurrent read (search) and write (register/unregister).

**Implementation**: Use `std::sync::RwLock` (not `tokio::sync::RwLock`) with a clone-and-release pattern. The lock is acquired briefly to collect provider references, then released before any `.await` calls in the search path.

#### Scenario: Concurrent Access

Given aggregator with providers registered
When search is executing while `register()` is called
Then no data race or panic occurs
(`std::sync::RwLock` with clone-and-release provides this guarantee without holding the lock across async boundaries)

### ADDED Manifest-Based Plugin Registration

The system MUST auto-register plugins when their manifest declares `mode = "search"` on a command.

#### Scenario: Plugin Discovery at Startup

Given a plugin with `plugin.toml` containing a command with `mode = "search"` and `keyword = "gh"`
When `PluginRegistry::discover()` processes the manifest
Then a per-command `PluginSearchProvider` proxy is created
And `aggregator.register("plugin:gh", proxy)` is called automatically

#### Scenario: Plugin Install at Runtime

Given a new plugin is installed via `PluginRegistry::install_from_dir()`
When the manifest contains a search-mode command
Then the aggregator registers the new provider immediately (no restart needed)

#### Scenario: Plugin Uninstall

Given an installed plugin with keyword "gh"
When the plugin is uninstalled
Then `aggregator.unregister("plugin:gh")` is called
And subsequent queries with `scope = "gh"` return `ProviderNotFound`

### Wiring Requirement

`PluginRegistry` MUST hold `Arc<SearchAggregator>` to call `register()`/`unregister()` during plugin lifecycle events. This is injected at startup in `lib.rs`.

## Removed (Over-Engineered)

- ~~Plugin-initiated FFI registration~~ - Architecturally impossible with subprocess plugins
- ~~Auto-cleanup on disconnect~~ - Requires heartbeat monitoring, premature optimization
- ~~Provider Capabilities~~ - Not needed with current provider count
