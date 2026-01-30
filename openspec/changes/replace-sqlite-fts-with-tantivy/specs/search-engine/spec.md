## ADDED Requirements

### Requirement: Search Engine Abstraction

The application SHALL provide a search engine abstraction layer that separates search indexing from data storage.

#### Scenario: Search engine trait definition
- **WHEN** implementing search functionality
- **THEN** the system SHALL use a `SearchEngine` trait that defines:
  - `search_bookmarks(query, limit)` - Search indexed bookmarks
  - `search_files(query, limit)` - Search indexed files
  - `index_bookmark(...)` - Add or update bookmark in index
  - `index_file(...)` - Add or update file in index
  - `delete_bookmark(id)` - Remove bookmark from index
  - `delete_file(id)` - Remove file from index
  - `rebuild_bookmark_index()` - Full rebuild from database
  - `rebuild_file_index()` - Full rebuild from database
  - `get_stats()` - Return index statistics
- **AND** the trait SHALL be async-compatible

#### Scenario: Tantivy implementation
- **WHEN** the application initializes
- **THEN** it SHALL create a `TantivySearchEngine` implementation
- **AND** store indexes in `APP_DATA/tantivy_indexes/` directory
- **AND** create separate indexes for bookmarks and files
- **AND** use mmap-based directory for efficient access

### Requirement: Index Initialization

The application SHALL initialize search indexes on startup.

#### Scenario: First launch index build
- **WHEN** application starts and no index exists
- **THEN** it SHALL build indexes from existing SQLite data in background
- **AND** report progress to the user
- **AND** allow searching already-indexed items during build

#### Scenario: Subsequent startup
- **WHEN** application starts with existing indexes
- **THEN** it SHALL verify index health
- **AND** make search available immediately
- **AND** NOT rebuild unless explicitly requested

### Requirement: Index Management Commands

The application SHALL provide commands for managing search indexes.

#### Scenario: Rebuild index command
- **WHEN** user or system requests index rebuild
- **THEN** the application SHALL delete existing index
- **AND** rebuild from SQLite source data
- **AND** report rebuild progress and completion

#### Scenario: Get index statistics
- **WHEN** user requests search statistics
- **THEN** the application SHALL return:
  - Number of indexed bookmarks
  - Number of indexed files
  - Index size on disk (bytes)

### Requirement: Index Synchronization

The application SHALL keep search indexes synchronized with SQLite data.

#### Scenario: Bookmark created
- **WHEN** a new bookmark is created in SQLite
- **THEN** it SHALL be indexed in Tantivy immediately
- **AND** be searchable within 1 second

#### Scenario: Bookmark updated
- **WHEN** a bookmark is updated in SQLite
- **THEN** the Tantivy index SHALL be updated immediately
- **AND** search results SHALL reflect the changes

#### Scenario: Bookmark deleted
- **WHEN** a bookmark is deleted from SQLite
- **THEN** it SHALL be removed from Tantivy index immediately
- **AND** SHALL NOT appear in search results

#### Scenario: File indexed
- **WHEN** a file is indexed during directory scan
- **THEN** it SHALL be added to Tantivy index
- **AND** be searchable immediately

#### Scenario: File removed
- **WHEN** a file is removed from index (deleted or directory removed)
- **THEN** it SHALL be removed from Tantivy index
- **AND** SHALL NOT appear in search results
