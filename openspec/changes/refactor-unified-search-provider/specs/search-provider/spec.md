## ADDED Requirements

### Requirement: Search Provider Trait

The application SHALL define a `SearchProvider` trait as the common interface for all searchable data sources.

#### Scenario: Trait definition
- **WHEN** implementing a new searchable data source
- **THEN** the implementor SHALL implement the `SearchProvider` trait with methods:
  - `source_id()` — returns a unique string identifier (e.g., "bookmarks", "files", "plugin:calc")
  - `source_label()` — returns a human-readable label for display (e.g., "Bookmarks", "Files")
  - `source_type()` — returns one of: `Bookmark`, `File`, `Plugin`, `Custom`
  - `search(ctx: &SearchContext)` — async method receiving a `SearchContext` and returning `Vec<ProviderResult>`
- **AND** the trait SHALL be `Send + Sync` for safe concurrent access
- **AND** providers SHALL NOT hold references to `SearchSettings`; all query-scoped parameters are passed via `SearchContext`

#### Scenario: SearchContext structure
- **WHEN** the aggregator dispatches a query to providers
- **THEN** it SHALL construct a `SearchContext` containing:
  - `query` — the search string
  - `limit` — maximum results to return per provider
  - `fuzzy` — whether to use fuzzy/ngram matching (from `SearchSettings.fuzzy_matching`)
  - `sources` — optional filter for specific source IDs
- **AND** the context SHALL be constructed fresh per query by reading current `SearchSettings`
- **AND** this ensures settings changes take effect immediately without reloading providers

#### Scenario: Provider result format
- **WHEN** a search provider returns results
- **THEN** each result SHALL include:
  - `id` — unique resource identifier within the provider
  - `source_id` — matches the provider's `source_id()`
  - `source_type` — matches the provider's `source_type()`
  - `title` — primary display text
  - `subtitle` — secondary display text
  - `icon` — optional icon reference (URL, path, or emoji)
  - `relevance_score` — raw relevance score from the provider's ranking algorithm
  - `frecency_score` — raw frecency score (0.0 if not applicable)
  - `resource_ref` — typed reference for opening the resource (URL, file path, or action)
- **AND** all score fields SHALL be non-negative floats

### Requirement: Bookmark Search Provider

The application SHALL provide a `BookmarkSearchProvider` implementing the `SearchProvider` trait.

#### Scenario: Bookmark provider delegates to Tantivy
- **WHEN** the bookmark provider receives a `SearchContext`
- **THEN** it SHALL delegate to `TantivySearchEngine::search_bookmarks()`
- **AND** map results to `ProviderResult` format
- **AND** preserve BM25 + frecency scores
- **AND** if `ctx.fuzzy` is `true`, use the ngram-tokenized query parser (default behavior)
- **AND** if `ctx.fuzzy` is `false`, use exact/prefix matching against the raw untokenized field

#### Scenario: Bookmark provider skipped when disabled
- **WHEN** `SearchSettings.show_bookmarks` is `false`
- **THEN** the aggregator SHALL exclude `BookmarkSearchProvider` from the query dispatch
- **AND** the provider itself is NOT responsible for checking enabled state

### Requirement: File Search Provider

The application SHALL provide a `FileSearchProvider` implementing the `SearchProvider` trait.

#### Scenario: File provider delegates to Tantivy
- **WHEN** the file provider receives a `SearchContext`
- **THEN** it SHALL delegate to `TantivySearchEngine::search_files()`
- **AND** map results to `ProviderResult` format
- **AND** preserve BM25 + frecency scores
- **AND** if `ctx.fuzzy` is `true`, use the ngram-tokenized query parser (default behavior)
- **AND** if `ctx.fuzzy` is `false`, use exact/prefix matching against the raw untokenized field

#### Scenario: File provider skipped when disabled
- **WHEN** `SearchSettings.show_files` is `false`
- **THEN** the aggregator SHALL exclude `FileSearchProvider` from the query dispatch
- **AND** the provider itself is NOT responsible for checking enabled state

### Requirement: Plugin Search Provider

The application SHALL provide a `PluginSearchProvider` implementing the `SearchProvider` trait for plugins declaring `mode = "search"`.

#### Scenario: Plugin provider participates in generic search
- **WHEN** a plugin declares `mode = "search"` in its manifest
- **AND** a non-keyword-prefixed query is executed
- **THEN** the plugin SHALL participate in unified search
- **AND** results SHALL be mapped to `ProviderResult` format

#### Scenario: Plugin provider excluded for non-search plugins
- **WHEN** a plugin declares `mode = "action"` or `mode = "detail"`
- **THEN** it SHALL NOT participate in unified search
- **AND** it SHALL only be triggered by explicit keyword prefix

#### Scenario: Plugin provider timeout in unified mode
- **WHEN** a plugin provider is queried during unified search
- **THEN** the timeout SHALL be configurable (default 2 seconds)
- **AND** if the plugin times out, the aggregator SHALL return results from other providers without waiting
