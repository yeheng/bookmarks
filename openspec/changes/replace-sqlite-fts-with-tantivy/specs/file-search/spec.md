## MODIFIED Requirements

### Requirement: File Search with Fuzzy Matching

The application SHALL provide real-time file search with fuzzy matching and ranking using Tantivy search engine.

#### Scenario: Search by filename
- **WHEN** user types a query matching file names
- **THEN** the application SHALL return matching files within 100ms
- **AND** support fuzzy matching using ngram tokenization (e.g., "dcmt" matches "document.txt")
- **AND** rank results by relevance and frecency

#### Scenario: Search by file extension
- **WHEN** user types a file extension (e.g., ".pdf")
- **THEN** the application SHALL return all files with that extension
- **AND** rank by recency and frequency of access

#### Scenario: Search by partial path
- **WHEN** user types a query matching part of the file path
- **THEN** the application SHALL return files with matching paths
- **AND** highlight matched path segments

#### Scenario: Frecency-based ranking for files
- **WHEN** multiple files match a query
- **THEN** results SHALL be ranked by:
  - 70% Tantivy BM25 relevance score
  - 30% frecency score (frequency + recency of access)
- **AND** recently opened files SHALL rank higher
- **AND** frequently opened files SHALL rank higher

#### Scenario: Result limit
- **WHEN** search returns more than 10 results
- **THEN** the application SHALL display top 10 results
- **AND** allow scrolling to load more (up to 50 total)

#### Scenario: Empty query
- **WHEN** search input is empty
- **THEN** the application SHALL display recently accessed files
- **AND** limit results to 10 items

### Requirement: File Index Persistence

The application SHALL persist file index using Tantivy for fast search with SQLite as source of truth.

#### Scenario: Database schema
- **WHEN** application initializes
- **THEN** it SHALL create indexed_files table with fields: id, path, name, extension, size, modified_at, created_at, indexed_at, directory_id
- **AND** create indexes for performance optimization
- **AND** NOT create FTS5 virtual tables (replaced by Tantivy)

#### Scenario: Tantivy file index schema
- **WHEN** indexing files in Tantivy
- **THEN** it SHALL use ngram tokenizer (3-gram) for file names
- **AND** enable fuzzy substring matching
- **AND** store file metadata for result display

#### Scenario: Index freshness check
- **WHEN** application starts
- **THEN** it SHALL check if any indexed files have been modified since last scan
- **AND** update stale entries in background
- **AND** allow searching while refresh is in progress
