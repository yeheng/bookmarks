## ADDED Requirements

### Requirement: JSON File-Based Settings Storage
The system SHALL store all application settings in a single JSON file (`settings.json`) located in the application data directory.

The system SHALL serialize settings using `serde_json` with the existing `AppSettings` struct as the schema.

The system SHALL load settings from the JSON file into memory on startup and cache them for fast access.

The system SHALL persist settings changes to disk atomically (write to temporary file, then rename).

#### Scenario: Settings saved to JSON file
- **WHEN** a user changes a setting via the UI
- **THEN** the system SHALL update the in-memory cache and write the complete settings to `settings.json` atomically

#### Scenario: Settings loaded on startup
- **WHEN** the application starts
- **THEN** the system SHALL read `settings.json` and populate the in-memory settings cache

#### Scenario: Default settings on fresh install
- **WHEN** the application starts and no `settings.json` exists
- **THEN** the system SHALL use default values for all settings and create `settings.json` with defaults

#### Scenario: Corrupt settings file recovery
- **WHEN** `settings.json` exists but contains invalid JSON
- **THEN** the system SHALL log a warning, rename the corrupt file to `settings.json.corrupt`, and start with default settings

### Requirement: Settings Export and Import Compatibility
The system SHALL maintain the existing export/import JSON format for data portability.

The system SHALL read settings directly from `settings.json` during export (no database query needed).

#### Scenario: Export includes settings from JSON file
- **WHEN** a user exports data
- **THEN** the export file SHALL include all settings read from the in-memory settings cache

#### Scenario: Import overwrites settings file
- **WHEN** a user imports data containing settings
- **THEN** the system SHALL merge imported settings into the current settings and persist to `settings.json`

## REMOVED Requirements

### Requirement: SQLite Settings Table
**Reason**: The `settings` table in SQLite is replaced by `settings.json` file. All key-value operations (`INSERT OR REPLACE`, `SELECT`, `DELETE`) are replaced by JSON read/write operations on the `AppSettings` struct.
**Migration**: Settings automatically migrated from SQLite to `settings.json` on first launch after upgrade.
