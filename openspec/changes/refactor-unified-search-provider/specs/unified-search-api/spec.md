## ADDED Requirements

### Requirement: Unified Search Command

The application SHALL provide a single `unified_search` Tauri command as the primary search endpoint for the frontend.

#### Scenario: Basic unified search
- **WHEN** the frontend invokes `unified_search` with a query string and optional limit
- **THEN** the backend SHALL delegate to the `SearchAggregator`
- **AND** return a list of `UnifiedSearchResult` items sorted by global ranking score
- **AND** each result SHALL include `source_id`, `source_type`, `title`, `subtitle`, `icon`, `relevance_score`, `frecency_score`, and `resource_ref`

#### Scenario: Source filtering
- **WHEN** the frontend invokes `unified_search` with an optional `sources` parameter (e.g., `["bookmarks", "files"]`)
- **THEN** the backend SHALL only query the specified providers
- **AND** skip providers not in the `sources` list
- **AND** still apply score normalization and ranking among the filtered providers

#### Scenario: Keyword-prefixed query routing
- **WHEN** the frontend invokes `unified_search` with a query that matches a plugin keyword prefix
- **THEN** the backend SHALL route the query to only the matching plugin provider
- **AND** SHALL NOT query built-in providers (bookmarks, files)
- **AND** return the plugin results directly without cross-source normalization

#### Scenario: Settings-aware search
- **WHEN** the frontend invokes `unified_search` without explicit `sources` parameter
- **THEN** the backend SHALL read `SearchSettings` to determine enabled sources:
  - If `show_bookmarks` is `false`, skip bookmark provider
  - If `show_files` is `false`, skip file provider
- **AND** use `max_results` from settings as the result limit (default: 10)

### Requirement: Unified Search Result Type

The application SHALL define a `UnifiedSearchResult` type for cross-source results.

#### Scenario: Result type fields
- **WHEN** a unified search result is serialized
- **THEN** it SHALL contain:
  - `id` — string, unique within source (e.g., "bookmark:42", "file:108")
  - `source_id` — string, provider identifier (e.g., "bookmarks", "files", "plugin:calc")
  - `source_type` — enum: "bookmark", "file", "plugin", "custom"
  - `title` — string, primary display text
  - `subtitle` — string, secondary display text
  - `icon` — optional string (URL, path, or emoji)
  - `relevance_score` — float, normalized [0.0, 1.0]
  - `frecency_score` — float, normalized [0.0, 1.0]
  - `global_score` — float, combined ranking score
  - `resource_ref` — object describing how to open the result:
    - For bookmarks: `{ "type": "url", "value": "https://..." }`
    - For files: `{ "type": "file", "value": "/path/to/file" }`
    - For plugins: `{ "type": "action", "actions": [...] }`

#### Scenario: Frontend type mapping
- **WHEN** the frontend receives `UnifiedSearchResult` items
- **THEN** it SHALL map them to the existing `SearchResult` interface
- **AND** preserve `source_type` for grouping via `useGroupedResults`
- **AND** preserve `resource_ref` for the `handleSelect` action dispatch

### Requirement: Legacy Command Compatibility

The existing `search_bookmarks` and `search_files` commands SHALL remain functional.

#### Scenario: Legacy command still works
- **WHEN** the frontend invokes `search_bookmarks` or `search_files` directly
- **THEN** the backend SHALL return results in the existing format
- **AND** behavior SHALL be identical to pre-refactor

#### Scenario: Deprecation notice
- **WHEN** documentation references `search_bookmarks` or `search_files`
- **THEN** it SHALL note these commands are deprecated in favor of `unified_search`
- **AND** recommend migration to `unified_search`
