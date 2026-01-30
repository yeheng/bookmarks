## MODIFIED Requirements

### Requirement: Bookmark Search with Ranking

The application SHALL provide real-time bookmark search with intelligent ranking using Tantivy search engine.

#### Scenario: Search by title
- **WHEN** user types a query matching bookmark titles
- **THEN** the application SHALL return matching bookmarks within 100ms
- **AND** rank results by relevance using Tantivy BM25 scoring
- **AND** boost frequently accessed bookmarks

#### Scenario: Search by URL
- **WHEN** user types a query matching bookmark URLs
- **THEN** the application SHALL return matching bookmarks
- **AND** highlight the matched URL portion

#### Scenario: Search by tags
- **WHEN** user types a query matching bookmark tags
- **THEN** the application SHALL return bookmarks with matching tags
- **AND** prioritize exact tag matches

#### Scenario: Frecency-based ranking
- **WHEN** multiple bookmarks match a query
- **THEN** results SHALL be ranked by combined score:
  - 70% Tantivy BM25 relevance score
  - 30% frecency score (frequency + recency of access)
- **AND** recently accessed bookmarks SHALL rank higher
- **AND** frequently accessed bookmarks SHALL rank higher

#### Scenario: Empty query
- **WHEN** search input is empty
- **THEN** the application SHALL display recently accessed bookmarks
- **AND** limit results to 10 items

#### Scenario: Prefix matching
- **WHEN** user types partial words
- **THEN** the application SHALL support prefix matching
- **AND** return bookmarks where indexed fields start with the query terms

### Requirement: Bookmark Data Storage

The application SHALL store bookmarks in a local SQLite database with Tantivy providing full-text search.

#### Scenario: Database schema
- **WHEN** application initializes
- **THEN** it SHALL create bookmarks table with fields: id, title, url, description, favicon_url, tags, source, created_at, updated_at, last_accessed
- **AND** create indexes for performance optimization
- **AND** NOT create FTS5 virtual tables (replaced by Tantivy)

#### Scenario: Data persistence
- **WHEN** bookmarks are added, edited, or deleted
- **THEN** changes SHALL be persisted to SQLite immediately
- **AND** Tantivy search index SHALL be updated in real-time
