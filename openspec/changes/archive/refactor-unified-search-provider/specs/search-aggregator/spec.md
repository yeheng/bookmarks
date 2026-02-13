## ADDED Requirements

### Requirement: Search Aggregator

The application SHALL provide a `SearchAggregator` that orchestrates parallel queries across all registered search providers and produces a unified ranked result list.

#### Scenario: Parallel provider execution
- **WHEN** a unified search query is received
- **THEN** the aggregator SHALL read current `SearchSettings` and construct a `SearchContext`
- **AND** filter providers based on `SearchSettings` enabled flags (`show_bookmarks`, `show_files`) and optional `sources` parameter
- **AND** query all remaining enabled providers in parallel
- **AND** collect results within the configured timeout
- **AND** return combined results even if some providers fail or timeout
- **AND** total query time SHALL NOT exceed 100ms for built-in providers (bookmarks + files)

#### Scenario: Score normalization
- **WHEN** results are collected from multiple providers
- **THEN** the aggregator SHALL normalize each provider's relevance scores to [0.0, 1.0] range using min-max normalization within that provider's result set
- **AND** normalize frecency scores to [0.0, 1.0] range using min-max normalization
- **AND** compute a global ranking score: `global_score = normalized_relevance * 0.7 + normalized_frecency * 0.3`
- **AND** sort all results by global_score descending

#### Scenario: Single provider results
- **WHEN** only one provider returns results
- **THEN** the aggregator SHALL skip normalization
- **AND** return results in the provider's original ranking order

#### Scenario: Empty query delegation
- **WHEN** an empty query is received
- **THEN** the aggregator SHALL delegate to each enabled provider
- **AND** providers SHALL return recently accessed items sorted by frecency
- **AND** the aggregator SHALL use fair-quota allocation: each enabled provider receives a per-provider limit of `ceil(max_results / enabled_provider_count)` to ensure balanced representation
- **AND** after collecting results, the aggregator SHALL interleave by normalized frecency score with a round-robin tiebreaker across providers to prevent any single high-frequency source from dominating
- **AND** the final result list SHALL contain at most `max_results` items

#### Scenario: Result limit enforcement
- **WHEN** `max_results` is specified in `SearchSettings`
- **THEN** the aggregator SHALL return at most `max_results` items after ranking
- **AND** each provider SHALL be queried with a per-provider limit of `max_results` to ensure sufficient candidates

### Requirement: Provider Registration

The application SHALL support dynamic registration of search providers at startup.

#### Scenario: Built-in provider registration
- **WHEN** the application initializes
- **THEN** the aggregator SHALL register `BookmarkSearchProvider` and `FileSearchProvider`
- **AND** both providers SHALL be enabled by default

#### Scenario: Plugin provider discovery
- **WHEN** the plugin registry discovers plugins with `mode = "search"`
- **THEN** it SHALL create a `PluginSearchProvider` for each
- **AND** register it with the aggregator

#### Scenario: Provider list query
- **WHEN** the frontend requests available search sources
- **THEN** the aggregator SHALL return a list of `(source_id, source_label, source_type, is_enabled)` tuples

### Requirement: Graceful Degradation

The application SHALL handle provider failures gracefully without affecting overall search availability.

#### Scenario: Provider throws error
- **WHEN** a provider returns an error during search
- **THEN** the aggregator SHALL log the error
- **AND** continue with results from other providers
- **AND** the frontend SHALL display available results without error indication for the failed provider

#### Scenario: Provider timeout
- **WHEN** a provider exceeds its configured timeout
- **THEN** the aggregator SHALL cancel the provider's query
- **AND** return results from providers that responded in time
- **AND** log a timeout warning
