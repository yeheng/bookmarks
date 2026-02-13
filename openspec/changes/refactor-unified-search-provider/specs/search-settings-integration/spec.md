## MODIFIED Requirements

### Requirement: Search Settings Integration

The application SHALL wire `SearchSettings` into the actual search pipeline, replacing hardcoded values.

#### Scenario: max_results respected
- **WHEN** `SearchSettings.max_results` is set to N (default: 10)
- **THEN** the unified search SHALL return at most N results
- **AND** each provider SHALL query with limit = N to ensure sufficient candidates for cross-source ranking
- **AND** the frontend SHALL NOT hardcode `limit: 5`

#### Scenario: show_bookmarks toggle
- **WHEN** `SearchSettings.show_bookmarks` is set to `false`
- **THEN** the aggregator SHALL exclude `BookmarkSearchProvider` from query dispatch when building `SearchContext`
- **AND** no bookmark results SHALL appear in the result list
- **AND** setting it back to `true` SHALL immediately include bookmarks again (effective on next query)

#### Scenario: show_files toggle
- **WHEN** `SearchSettings.show_files` is set to `false`
- **THEN** the aggregator SHALL exclude `FileSearchProvider` from query dispatch when building `SearchContext`
- **AND** no file results SHALL appear in the result list
- **AND** setting it back to `true` SHALL immediately include files again (effective on next query)

#### Scenario: fuzzy_matching configuration
- **WHEN** `SearchSettings.fuzzy_matching` is set to `true`
- **THEN** the aggregator SHALL set `SearchContext.fuzzy = true`
- **AND** providers SHALL use the ngram-tokenized query parser for matching (current default behavior; queries like "goo" match "google")
- **WHEN** `SearchSettings.fuzzy_matching` is set to `false`
- **THEN** the aggregator SHALL set `SearchContext.fuzzy = false`
- **AND** providers SHALL use exact/prefix matching against the raw (untokenized) `STRING` field in the Tantivy schema
- **AND** this is a query-time-only switch — no index rebuild is required
- **AND** the Tantivy schema SHALL include both a `TEXT` (ngram-tokenized) variant and a `STRING` (raw/untokenized) variant for each searchable field to support this toggle

#### Scenario: Settings persisted and loaded
- **WHEN** the user changes search settings via the Settings panel
- **THEN** changes SHALL be persisted to SQLite immediately
- **AND** the next search query SHALL use the updated settings
- **AND** no application restart SHALL be required

### Requirement: Frontend Search Flow

The application frontend SHALL use the unified search endpoint with settings-aware configuration.

#### Scenario: Single unified search call
- **WHEN** user types a query in the search input
- **THEN** the frontend SHALL invoke `unified_search` with the query
- **AND** pass `max_results` from current `SearchSettings`
- **AND** NOT invoke `search_bookmarks` and `search_files` separately

#### Scenario: Plugin keyword detection preserved
- **WHEN** user types a query with a plugin keyword prefix (e.g., "calc 2+3")
- **THEN** the frontend SHALL detect the keyword prefix
- **AND** pass the query to `unified_search` which handles keyword routing
- **AND** display only the matching plugin's results

#### Scenario: Result grouping preserved
- **WHEN** unified search returns mixed-source results
- **THEN** the frontend SHALL group results by `source_type` using `useGroupedResults`
- **AND** display group headers (Bookmarks, Files, Plugins)
- **AND** allow collapsing/expanding groups
- **AND** maintain the existing sort order within groups (by global_score descending)

#### Scenario: Error handling
- **WHEN** the unified search call fails
- **THEN** the frontend SHALL display an error message
- **AND** provide a retry option
- **AND** NOT crash or show an empty state without explanation
