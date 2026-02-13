## ADDED Requirements

### Requirement: Plugin Registry Management
Launcher MUST 维护一个插件注册表，支持发现、注册、启用/禁用、卸载操作。

注册表数据存储在 SQLite 中:
```sql
CREATE TABLE plugins (
    id TEXT PRIMARY KEY,          -- plugin.toml [plugin].name
    title TEXT NOT NULL,
    description TEXT,
    version TEXT NOT NULL,
    author TEXT,
    enabled INTEGER DEFAULT 1,    -- 0=disabled, 1=enabled
    install_path TEXT NOT NULL,   -- absolute path to plugin directory
    installed_at TEXT NOT NULL,
    updated_at TEXT
);
```

#### Scenario: Plugin installed from local directory
- **WHEN** user provides a path to a plugin directory or archive (.zip/.tar.gz)
- **THEN** the system SHALL validate the manifest
- **THEN** the system SHALL copy/extract the plugin to the plugins directory
- **THEN** the system SHALL register the plugin in the database
- **THEN** the plugin SHALL be immediately available for use

#### Scenario: Plugin uninstalled
- **WHEN** user requests to uninstall a plugin
- **THEN** the system SHALL remove the plugin directory
- **THEN** the system SHALL remove the registry entry
- **THEN** the system SHALL clean up plugin data (with user confirmation)

#### Scenario: Plugin enabled/disabled
- **WHEN** user toggles a plugin's enabled state
- **THEN** the system SHALL update the `enabled` field in the database
- **THEN** disabled plugins SHALL NOT appear in keyword matching
- **THEN** disabled plugins SHALL still appear in the plugin management UI

#### Scenario: List installed plugins
- **WHEN** user opens the plugin management interface
- **THEN** the system SHALL return all registered plugins with their status (enabled/disabled/error)

### Requirement: Plugin Discovery on Startup
Launcher MUST 在启动时扫描插件目录，发现新插件并同步注册表状态。

#### Scenario: New plugin found on startup
- **WHEN** a new plugin directory (with valid manifest) exists but is not in the registry
- **THEN** the system SHALL automatically register it as enabled

#### Scenario: Removed plugin detected on startup
- **WHEN** a registered plugin's directory no longer exists on disk
- **THEN** the system SHALL remove it from the registry
- **THEN** the system SHALL log the removal

#### Scenario: Plugin manifest changed
- **WHEN** a registered plugin's `plugin.toml` has changed since last registration
- **THEN** the system SHALL update the registry with the new metadata

### Requirement: Plugin Preferences Storage
Launcher MUST 为每个插件存储用户配置（preferences），与插件包分离。

```sql
CREATE TABLE plugin_preferences (
    plugin_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT,
    PRIMARY KEY (plugin_id, key),
    FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
);
```

#### Scenario: Preference saved
- **WHEN** user configures a plugin preference
- **THEN** the system SHALL store the value in `plugin_preferences` table
- **THEN** sensitive values (type=password) SHALL be encrypted before storage

#### Scenario: Preferences passed to plugin on execution
- **WHEN** a plugin command is executed
- **THEN** the system SHALL load all preferences for that plugin
- **THEN** the system SHALL include them in the stdin JSON request
- **THEN** the system SHALL inject them as environment variables

### Requirement: Keyword Registry
系统 MUST 维护一个全局 keyword 注册表，确保 keyword 唯一性并支持快速查找。

#### Scenario: Keyword uniqueness enforced
- **WHEN** a new plugin registers a keyword that conflicts with an existing plugin
- **THEN** the system SHALL warn the user and require resolution (rename or disable one)

#### Scenario: Keyword lookup for search input
- **WHEN** user types a query matching `{keyword}:{query}` pattern
- **THEN** the system SHALL resolve the keyword to the corresponding plugin command
- **THEN** the system SHALL execute that plugin command with the query portion
