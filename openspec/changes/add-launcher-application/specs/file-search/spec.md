## ADDED Requirements

### Requirement: Search Directory Configuration

The application SHALL allow users to configure which directories to index for file search.

#### Scenario: Add search directory
- **WHEN** user adds a directory path to search configuration
- **THEN** the application SHALL validate the directory exists
- **AND** add it to the search directory list
- **AND** initiate indexing of that directory

#### Scenario: Remove search directory
- **WHEN** user removes a directory from search configuration
- **THEN** the application SHALL remove associated file entries from index
- **AND** update search results immediately

#### Scenario: Default directories
- **WHEN** application is first launched
- **THEN** it SHALL suggest common directories (Desktop, Documents, Downloads)
- **AND** allow user to accept or customize the list

#### Scenario: Invalid directory path
- **WHEN** user attempts to add a non-existent or inaccessible directory
- **THEN** the application SHALL display an error message
- **AND** SHALL NOT add the directory to configuration

### Requirement: File System Indexing

The application SHALL index files in configured directories for fast search.

#### Scenario: Initial directory scan
- **WHEN** user adds a new search directory
- **THEN** the application SHALL scan the directory recursively in background
- **AND** index file metadata (path, name, extension, size, modified_at)
- **AND** display indexing progress (files scanned / estimated total)
- **AND** make indexed files searchable immediately upon indexing

#### Scenario: Incremental index updates
- **WHEN** files are created, modified, or deleted in indexed directories
- **THEN** the application SHALL detect changes via file system watchers
- **AND** update the index within 1 second
- **AND** debounce rapid changes (100ms) to avoid thrashing

#### Scenario: Index size limits
- **WHEN** a directory contains more than 100,000 files
- **THEN** the application SHALL display a warning
- **AND** allow user to choose: continue, skip subdirectories, or cancel
- **AND** SHALL NOT index without user confirmation

#### Scenario: Skip hidden files
- **WHEN** indexing directories
- **THEN** the application SHALL skip hidden files (starting with '.') by default
- **AND** SHALL skip system directories (/System, /Windows, node_modules, etc.)
- **AND** allow users to configure inclusion rules in settings

#### Scenario: Background indexing
- **WHEN** indexing is in progress
- **THEN** the application SHALL NOT block the UI
- **AND** SHALL allow searching already-indexed files
- **AND** SHALL allow user to cancel indexing

### Requirement: File Search with Fuzzy Matching

The application SHALL provide real-time file search with fuzzy matching and ranking.

#### Scenario: Search by filename
- **WHEN** user types a query matching file names
- **THEN** the application SHALL return matching files within 100ms
- **AND** support fuzzy matching (e.g., "dcmt" matches "document.txt")
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
  - 70% FTS5 relevance score
  - 30% frecency score (frequency + recency of access)
- **AND** recently opened files SHALL rank higher
- **AND** frequently opened files SHALL rank higher

#### Scenario: Result limit
- **WHEN** search returns more than 10 results
- **THEN** the application SHALL display top 10 results
- **AND** allow scrolling to load more (up to 50 total)

### Requirement: File Metadata Display

The application SHALL display relevant file metadata in search results.

#### Scenario: File result display
- **WHEN** file appears in search results
- **THEN** it SHALL show file icon (based on type/extension)
- **AND** show file name as title
- **AND** show full path as subtitle
- **AND** show file size (if > 1MB)
- **AND** show last modified date (if within 7 days)

#### Scenario: File type icons
- **WHEN** displaying file results
- **THEN** the application SHALL show appropriate icons for common file types
- **AND** use system default icons when available
- **AND** fall back to generic file icon for unknown types

### Requirement: File Index Persistence

The application SHALL persist file index in SQLite database for fast startup.

#### Scenario: Database schema
- **WHEN** application initializes
- **THEN** it SHALL create files table with fields: id, path, name, extension, size, modified_at, created_at, indexed_at
- **AND** create FTS5 virtual table for full-text search on name and path
- **AND** create indexes for performance optimization

#### Scenario: Index freshness check
- **WHEN** application starts
- **THEN** it SHALL check if any indexed files have been modified since last scan
- **AND** update stale entries in background
- **AND** allow searching while refresh is in progress

#### Scenario: Periodic re-indexing
- **WHEN** 24 hours have passed since last full index scan
- **THEN** the application SHALL perform a background re-index
- **AND** remove entries for deleted files
- **AND** update metadata for modified files
- **AND** allow user to disable or configure re-index frequency

### Requirement: File System Watcher Reliability

The application SHALL maintain accurate file index through reliable file system monitoring.

#### Scenario: Watcher initialization
- **WHEN** search directory is added
- **THEN** the application SHALL start file system watcher for that directory
- **AND** watch for create, modify, delete, and rename events

#### Scenario: Watcher failure detection
- **WHEN** file system watcher fails or becomes unreliable
- **THEN** the application SHALL detect the failure
- **AND** fall back to periodic polling (every 60 seconds)
- **AND** notify user of degraded performance

#### Scenario: Watcher resource usage
- **WHEN** file system watchers are active
- **THEN** they SHALL use < 50MB of memory
- **AND** SHALL NOT cause significant CPU usage when idle
