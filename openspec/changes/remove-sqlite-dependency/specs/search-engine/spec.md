## MODIFIED Requirements

### Requirement: Tantivy as Primary Data Source for Search
The system SHALL use Tantivy as the sole data source during search operations, with no dependency on an external database.

The system SHALL store frecency data (access_count, access_timestamp) exclusively in Tantivy FastFields.

The system SHALL rebuild indexes from JSON data files (bookmarks) and filesystem scans (files) when index integrity checks fail.

#### Scenario: Bookmark index rebuild from JSON
- **WHEN** the Tantivy bookmark index is missing or corrupt
- **THEN** the system SHALL read `bookmarks.json` and rebuild the bookmark index from its contents

#### Scenario: File index rebuild from filesystem
- **WHEN** the Tantivy file index is missing or corrupt
- **THEN** the system SHALL read directory configurations from `directories.json` and re-scan the filesystem to rebuild the file index

#### Scenario: Frecency data integrity
- **WHEN** frecency data is updated after a user accesses a resource
- **THEN** the system SHALL update Tantivy FastFields directly without writing to any external data store

#### Scenario: Index consistency check on startup
- **WHEN** the application starts
- **THEN** the system SHALL compare bookmark count in `bookmarks.json` with the Tantivy bookmark index count and trigger a rebuild if they differ significantly
