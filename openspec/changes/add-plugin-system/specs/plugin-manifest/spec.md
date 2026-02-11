## ADDED Requirements

### Requirement: Plugin Manifest Format
插件 MUST 使用 `plugin.toml` 文件声明元数据、命令和配置项。Launcher MUST 能够解析并校验 manifest 格式。

Manifest 最小结构:
```toml
[plugin]
name = "plugin-id"           # kebab-case, unique identifier
title = "Plugin Display Name"
description = "What this plugin does"
version = "1.0.0"
author = "author-name"
api_version = "0.1"
icon = "icon.png"            # optional, relative to assets/

[[commands]]
name = "command-name"
title = "Command Display Title"
description = "What this command does"
keyword = "kw"               # trigger prefix
mode = "search"              # "search" | "action" | "detail"
script = "dist/index.js"     # relative path to executable
runtime = "node"             # "node" | "python" | "bash" | "binary"

[[preferences]]
name = "api_key"
type = "password"            # "text" | "password" | "number" | "boolean" | "select"
required = true
title = "API Key"
description = "Your API key"
default = ""
```

#### Scenario: Valid manifest parsed successfully
- **WHEN** a plugin directory contains a valid `plugin.toml`
- **THEN** the system SHALL parse all fields and create a `PluginManifest` struct
- **THEN** the plugin SHALL be available for registration

#### Scenario: Invalid manifest rejected with clear error
- **WHEN** a plugin directory contains an invalid or incomplete `plugin.toml`
- **THEN** the system SHALL return a descriptive error (missing field, invalid type, etc.)
- **THEN** the plugin SHALL NOT be registered

#### Scenario: Manifest version compatibility check
- **WHEN** a plugin manifest declares `api_version` higher than the host supports
- **THEN** the system SHALL reject the plugin with a version incompatibility error

### Requirement: Plugin Directory Structure
每个插件 MUST 遵循标准目录结构以便被 Launcher 发现和加载。

```
plugins/
  github-search/
    plugin.toml          # Manifest (required)
    assets/              # Icons and resources (optional)
      icon.png
    dist/                # Executable scripts (required)
      index.js
    data/                # Plugin runtime data (auto-created)
    logs/                # Plugin log files (auto-created)
```

#### Scenario: Plugin discovered from standard directory
- **WHEN** a directory under the plugins root contains `plugin.toml`
- **THEN** the system SHALL recognize it as a plugin directory
- **THEN** the system SHALL attempt to parse the manifest

#### Scenario: Non-plugin directory ignored
- **WHEN** a directory under the plugins root does NOT contain `plugin.toml`
- **THEN** the system SHALL silently skip it without errors
