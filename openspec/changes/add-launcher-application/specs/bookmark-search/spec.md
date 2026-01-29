## ADDED Requirements

### Requirement: Browser Bookmark Import

The application SHALL import bookmarks from Chrome, Firefox, and Safari browsers.

#### Scenario: Chrome bookmark import
- **WHEN** user initiates Chrome bookmark import
- **THEN** the application SHALL locate Chrome's bookmarks file
- **AND** parse the JSON bookmarks structure
- **AND** import all bookmark folders and items
- **AND** store them with source attribution ('chrome')
- **AND** display import success with count of imported bookmarks

#### Scenario: Firefox bookmark import
- **WHEN** user initiates Firefox bookmark import
- **THEN** the application SHALL locate Firefox profile directory
- **AND** read the places.sqlite database
- **AND** extract bookmarks from moz_bookmarks and moz_places tables
- **AND** store them with source attribution ('firefox')
- **AND** display import success with count of imported bookmarks

#### Scenario: Safari bookmark import
- **WHEN** user initiates Safari bookmark import (macOS only)
- **THEN** the application SHALL locate Safari's Bookmarks.plist file
- **AND** parse the plist bookmark structure
- **AND** import all bookmark folders and items
- **AND** store them with source attribution ('safari')
- **AND** display import success with count of imported bookmarks

#### Scenario: Duplicate bookmark handling
- **WHEN** importing a bookmark with URL that already exists
- **THEN** the application SHALL skip the duplicate
- **AND** increment the skipped count
- **AND** display skipped count in import summary

#### Scenario: Browser not found
- **WHEN** user attempts to import from a browser not installed
- **THEN** the application SHALL display an error message
- **AND** SHALL NOT create empty bookmark entries

### Requirement: Manual Bookmark Management

The application SHALL allow users to manually add, edit, and delete bookmarks.

#### Scenario: Add bookmark manually
- **WHEN** user adds a bookmark with title and URL
- **THEN** the application SHALL validate the URL format
- **AND** store the bookmark with source 'manual'
- **AND** automatically fetch the favicon
- **AND** make it searchable immediately

#### Scenario: Edit existing bookmark
- **WHEN** user edits a bookmark's title, URL, or description
- **THEN** the application SHALL update the bookmark
- **AND** update the last_updated timestamp
- **AND** refresh the search index

#### Scenario: Delete bookmark
- **WHEN** user deletes a bookmark
- **THEN** the application SHALL remove it from storage
- **AND** remove it from search index
- **AND** remove associated usage history

#### Scenario: Add tags to bookmark
- **WHEN** user adds tags to a bookmark
- **THEN** tags SHALL be stored as searchable metadata
- **AND** SHALL appear in search results
- **AND** SHALL improve search ranking when matched

### Requirement: Bookmark Search with Ranking

The application SHALL provide real-time bookmark search with intelligent ranking.

#### Scenario: Search by title
- **WHEN** user types a query matching bookmark titles
- **THEN** the application SHALL return matching bookmarks within 100ms
- **AND** rank results by relevance (FTS5 BM25 score)
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
  - 70% FTS5 relevance score
  - 30% frecency score (frequency + recency of access)
- **AND** recently accessed bookmarks SHALL rank higher
- **AND** frequently accessed bookmarks SHALL rank higher

#### Scenario: Empty query
- **WHEN** search input is empty
- **THEN** the application SHALL display recently accessed bookmarks
- **AND** limit results to 10 items

### Requirement: Bookmark Icon Display

The application SHALL display favicons for bookmarks to improve visual recognition.

#### Scenario: Fetch favicon on bookmark creation
- **WHEN** a new bookmark is added
- **THEN** the application SHALL attempt to fetch the favicon
- **AND** store it locally for offline access
- **AND** display the favicon in search results

#### Scenario: Favicon fallback
- **WHEN** favicon fetch fails or no favicon exists
- **THEN** the application SHALL display a default bookmark icon
- **AND** SHALL NOT block bookmark creation or search

### Requirement: Bookmark Data Storage

The application SHALL store bookmarks in a local SQLite database with full-text search support.

#### Scenario: Database schema
- **WHEN** application initializes
- **THEN** it SHALL create bookmarks table with fields: id, title, url, description, favicon_url, tags, source, created_at, updated_at, last_accessed
- **AND** create FTS5 virtual table for full-text search on title, url, description, tags
- **AND** create indexes for performance optimization

#### Scenario: Data persistence
- **WHEN** bookmarks are added, edited, or deleted
- **THEN** changes SHALL be persisted to SQLite immediately
- **AND** search index SHALL be updated in real-time
