//! Plugin manifest parsing and validation.
//!
//! Handles `plugin.toml` format — the single source of truth for plugin metadata,
//! commands, and preferences.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Current API version supported by the host.
pub const HOST_API_VERSION: &str = "0.1";

/// Plugin manifest — parsed from `plugin.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
    #[serde(default)]
    pub commands: Vec<PluginCommand>,
    #[serde(default)]
    pub preferences: Vec<PluginPreference>,
}

/// Core plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    /// Unique identifier (kebab-case, e.g. "github-search").
    pub name: String,
    /// Human-readable title shown in UI.
    pub title: String,
    /// What this plugin does.
    pub description: String,
    /// Semantic version (e.g. "1.0.0").
    pub version: String,
    /// Author name or handle.
    #[serde(default)]
    pub author: Option<String>,
    /// Minimum host API version required (e.g. "0.1").
    pub api_version: String,
    /// Icon filename relative to `assets/` directory.
    #[serde(default)]
    pub icon: Option<String>,
}

/// A single command exposed by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Command identifier (kebab-case).
    pub name: String,
    /// Human-readable title.
    pub title: String,
    /// Description of what this command does.
    pub description: String,
    /// Keyword prefix that triggers this command (e.g. "gh").
    pub keyword: String,
    /// Command mode: "search", "action", or "detail".
    #[serde(default = "default_mode")]
    pub mode: String,
    /// Script path relative to plugin directory.
    pub script: String,
    /// Runtime to use: "node", "python", "bash", or "binary".
    #[serde(default = "default_runtime")]
    pub runtime: String,
    /// Command-specific timeout in seconds (overrides plugin default).
    #[serde(default)]
    pub timeout: Option<u64>,
}

fn default_mode() -> String {
    "search".to_string()
}

fn default_runtime() -> String {
    "node".to_string()
}

/// A user-configurable preference declared by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPreference {
    /// Preference key name.
    pub name: String,
    /// Type: "text", "password", "number", "boolean", "select".
    #[serde(rename = "type")]
    pub pref_type: String,
    /// Whether this preference must be set before the plugin works.
    #[serde(default)]
    pub required: bool,
    /// Human-readable label.
    pub title: String,
    /// Extended description / help text.
    #[serde(default)]
    pub description: Option<String>,
    /// Default value as string.
    #[serde(default)]
    pub default: Option<String>,
    /// Options for "select" type preferences.
    #[serde(default)]
    pub options: Vec<SelectOption>,
}

/// An option for "select" type preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
}

/// Errors that can occur during manifest parsing.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Failed to read plugin.toml: {0}")]
    ReadFailed(#[from] std::io::Error),

    #[error("Failed to parse plugin.toml: {0}")]
    ParseFailed(#[from] toml::de::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for '{field}': {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("API version incompatible: plugin requires {required}, host supports {supported}")]
    ApiVersionIncompatible { required: String, supported: String },

    #[error("Plugin has no commands defined")]
    NoCommands,

    #[error("Duplicate keyword '{0}' in commands")]
    DuplicateKeyword(String),

    #[error("Script file not found: {0}")]
    ScriptNotFound(String),
}

impl PluginManifest {
    /// Parse a `plugin.toml` file from disk.
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parse manifest from a TOML string.
    pub fn from_str(content: &str) -> Result<Self, ManifestError> {
        let manifest: PluginManifest = toml::from_str(content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate manifest after parsing.
    fn validate(&self) -> Result<(), ManifestError> {
        // Required fields
        if self.plugin.name.is_empty() {
            return Err(ManifestError::MissingField("plugin.name".to_string()));
        }
        if self.plugin.title.is_empty() {
            return Err(ManifestError::MissingField("plugin.title".to_string()));
        }
        if self.plugin.version.is_empty() {
            return Err(ManifestError::MissingField("plugin.version".to_string()));
        }
        if self.plugin.api_version.is_empty() {
            return Err(ManifestError::MissingField("plugin.api_version".to_string()));
        }

        // Name format: kebab-case only
        if !self.plugin.name.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit()) {
            return Err(ManifestError::InvalidValue {
                field: "plugin.name".to_string(),
                reason: "must be kebab-case (lowercase letters, digits, hyphens only)".to_string(),
            });
        }

        // API version compatibility
        if !is_api_compatible(&self.plugin.api_version, HOST_API_VERSION) {
            return Err(ManifestError::ApiVersionIncompatible {
                required: self.plugin.api_version.clone(),
                supported: HOST_API_VERSION.to_string(),
            });
        }

        // Must have at least one command
        if self.commands.is_empty() {
            return Err(ManifestError::NoCommands);
        }

        // Validate commands
        let mut keywords = std::collections::HashSet::new();
        for cmd in &self.commands {
            if cmd.name.is_empty() {
                return Err(ManifestError::MissingField("commands[].name".to_string()));
            }
            if cmd.keyword.is_empty() {
                return Err(ManifestError::MissingField(format!("commands[{}].keyword", cmd.name)));
            }
            if cmd.script.is_empty() {
                return Err(ManifestError::MissingField(format!("commands[{}].script", cmd.name)));
            }

            // Validate mode
            if !["search", "action", "detail"].contains(&cmd.mode.as_str()) {
                return Err(ManifestError::InvalidValue {
                    field: format!("commands[{}].mode", cmd.name),
                    reason: format!("must be one of: search, action, detail (got: {})", cmd.mode),
                });
            }

            // Validate runtime
            if !["node", "python", "bash", "binary"].contains(&cmd.runtime.as_str()) {
                return Err(ManifestError::InvalidValue {
                    field: format!("commands[{}].runtime", cmd.name),
                    reason: format!("must be one of: node, python, bash, binary (got: {})", cmd.runtime),
                });
            }

            // Keyword uniqueness within plugin
            if !keywords.insert(cmd.keyword.clone()) {
                return Err(ManifestError::DuplicateKeyword(cmd.keyword.clone()));
            }
        }

        // Validate preferences
        for pref in &self.preferences {
            if pref.name.is_empty() {
                return Err(ManifestError::MissingField("preferences[].name".to_string()));
            }
            if !["text", "password", "number", "boolean", "select"].contains(&pref.pref_type.as_str()) {
                return Err(ManifestError::InvalidValue {
                    field: format!("preferences[{}].type", pref.name),
                    reason: format!("must be one of: text, password, number, boolean, select (got: {})", pref.pref_type),
                });
            }
            if pref.pref_type == "select" && pref.options.is_empty() {
                return Err(ManifestError::InvalidValue {
                    field: format!("preferences[{}].options", pref.name),
                    reason: "select type preferences must have at least one option".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate that script files exist on disk (call after installation).
    pub fn validate_scripts(&self, plugin_dir: &Path) -> Result<(), ManifestError> {
        for cmd in &self.commands {
            let script_path = plugin_dir.join(&cmd.script);
            if !script_path.exists() {
                return Err(ManifestError::ScriptNotFound(cmd.script.clone()));
            }
        }
        Ok(())
    }

    /// Get the plugin's unique name/id.
    pub fn id(&self) -> &str {
        &self.plugin.name
    }

    /// Find a command by keyword.
    pub fn find_command_by_keyword(&self, keyword: &str) -> Option<&PluginCommand> {
        self.commands.iter().find(|c| c.keyword == keyword)
    }

    /// Get the absolute icon path for this plugin.
    pub fn icon_path(&self, plugin_dir: &Path) -> Option<PathBuf> {
        self.plugin.icon.as_ref().map(|icon| plugin_dir.join("assets").join(icon))
    }
}

/// Simple API version compatibility check.
/// Currently: exact major match (0.x matches 0.y where x <= y).
fn is_api_compatible(required: &str, supported: &str) -> bool {
    let req_parts: Vec<&str> = required.split('.').collect();
    let sup_parts: Vec<&str> = supported.split('.').collect();

    if req_parts.is_empty() || sup_parts.is_empty() {
        return false;
    }

    // Major version must match
    if req_parts[0] != sup_parts[0] {
        return false;
    }

    // For 0.x versions, minor must be <= supported
    let req_minor: u32 = req_parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
    let sup_minor: u32 = sup_parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);

    req_minor <= sup_minor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_toml() -> &'static str {
        r#"
[plugin]
name = "hello-world"
title = "Hello World"
description = "A test plugin"
version = "1.0.0"
api_version = "0.1"

[[commands]]
name = "greet"
title = "Say Hello"
description = "Returns a greeting"
keyword = "hello"
script = "dist/index.js"
runtime = "node"

[[preferences]]
name = "greeting"
type = "text"
required = false
title = "Custom Greeting"
default = "Hello"
"#
    }

    #[test]
    fn test_parse_valid_manifest() {
        let manifest = PluginManifest::from_str(valid_toml()).unwrap();
        assert_eq!(manifest.plugin.name, "hello-world");
        assert_eq!(manifest.plugin.title, "Hello World");
        assert_eq!(manifest.commands.len(), 1);
        assert_eq!(manifest.commands[0].keyword, "hello");
        assert_eq!(manifest.preferences.len(), 1);
        assert_eq!(manifest.preferences[0].pref_type, "text");
    }

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[plugin]
name = "minimal"
title = "Minimal Plugin"
description = "Bare minimum"
version = "0.1.0"
api_version = "0.1"

[[commands]]
name = "run"
title = "Run"
description = "Do something"
keyword = "min"
script = "run.sh"
runtime = "bash"
"#;
        let manifest = PluginManifest::from_str(toml).unwrap();
        assert_eq!(manifest.id(), "minimal");
        assert_eq!(manifest.commands[0].mode, "search"); // default
    }

    #[test]
    fn test_missing_required_field() {
        let toml = r#"
[plugin]
name = ""
title = "No Name"
description = "Missing name"
version = "1.0.0"
api_version = "0.1"

[[commands]]
name = "run"
title = "Run"
description = "Do something"
keyword = "test"
script = "run.sh"
"#;
        let result = PluginManifest::from_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("plugin.name"), "Expected name error, got: {}", err);
    }

    #[test]
    fn test_invalid_name_format() {
        let toml = r#"
[plugin]
name = "Hello_World"
title = "Bad Name"
description = "Invalid name"
version = "1.0.0"
api_version = "0.1"

[[commands]]
name = "run"
title = "Run"
description = "Do something"
keyword = "test"
script = "run.sh"
"#;
        let result = PluginManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("kebab-case"));
    }

    #[test]
    fn test_api_version_incompatible() {
        let toml = r#"
[plugin]
name = "future"
title = "Future Plugin"
description = "Needs future API"
version = "1.0.0"
api_version = "1.0"

[[commands]]
name = "run"
title = "Run"
description = "Do something"
keyword = "test"
script = "run.sh"
"#;
        let result = PluginManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("incompatible"));
    }

    #[test]
    fn test_no_commands() {
        let toml = r#"
[plugin]
name = "empty"
title = "Empty Plugin"
description = "No commands"
version = "1.0.0"
api_version = "0.1"
"#;
        let result = PluginManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no commands"));
    }

    #[test]
    fn test_duplicate_keyword() {
        let toml = r#"
[plugin]
name = "dup"
title = "Duplicate Keywords"
description = "Has duplicate keywords"
version = "1.0.0"
api_version = "0.1"

[[commands]]
name = "cmd1"
title = "Command 1"
description = "First"
keyword = "test"
script = "a.js"

[[commands]]
name = "cmd2"
title = "Command 2"
description = "Second"
keyword = "test"
script = "b.js"
"#;
        let result = PluginManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate keyword"));
    }

    #[test]
    fn test_invalid_mode() {
        let toml = r#"
[plugin]
name = "bad-mode"
title = "Bad Mode"
description = "Invalid mode"
version = "1.0.0"
api_version = "0.1"

[[commands]]
name = "run"
title = "Run"
description = "Do something"
keyword = "test"
script = "run.sh"
mode = "invalid"
"#;
        let result = PluginManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mode"));
    }

    #[test]
    fn test_invalid_runtime() {
        let toml = r#"
[plugin]
name = "bad-runtime"
title = "Bad Runtime"
description = "Invalid runtime"
version = "1.0.0"
api_version = "0.1"

[[commands]]
name = "run"
title = "Run"
description = "Do something"
keyword = "test"
script = "run.sh"
runtime = "ruby"
"#;
        let result = PluginManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("runtime"));
    }

    #[test]
    fn test_select_without_options() {
        let toml = r#"
[plugin]
name = "bad-select"
title = "Bad Select"
description = "Select without options"
version = "1.0.0"
api_version = "0.1"

[[commands]]
name = "run"
title = "Run"
description = "Do something"
keyword = "test"
script = "run.sh"

[[preferences]]
name = "choice"
type = "select"
title = "Pick one"
"#;
        let result = PluginManifest::from_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("option"));
    }

    #[test]
    fn test_find_command_by_keyword() {
        let manifest = PluginManifest::from_str(valid_toml()).unwrap();
        assert!(manifest.find_command_by_keyword("hello").is_some());
        assert!(manifest.find_command_by_keyword("nonexistent").is_none());
    }

    #[test]
    fn test_api_version_compatibility() {
        assert!(is_api_compatible("0.1", "0.1")); // exact match
        assert!(is_api_compatible("0.1", "0.2")); // minor upgrade ok
        assert!(!is_api_compatible("0.2", "0.1")); // too new
        assert!(!is_api_compatible("1.0", "0.1")); // major mismatch
    }

    #[test]
    fn test_invalid_toml_syntax() {
        let result = PluginManifest::from_str("this is not valid toml [[[");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse"));
    }

    #[test]
    fn test_multiple_commands_and_preferences() {
        let toml = r#"
[plugin]
name = "multi"
title = "Multi Command"
description = "Multiple commands"
version = "1.0.0"
author = "test-author"
api_version = "0.1"
icon = "icon.png"

[[commands]]
name = "search"
title = "Search"
description = "Search things"
keyword = "ms"
script = "dist/search.js"
mode = "search"

[[commands]]
name = "detail"
title = "Detail"
description = "Show detail"
keyword = "md"
script = "dist/detail.js"
mode = "detail"
timeout = 30

[[preferences]]
name = "api_key"
type = "password"
required = true
title = "API Key"
description = "Your API key"

[[preferences]]
name = "region"
type = "select"
title = "Region"
options = [
    { label = "US", value = "us" },
    { label = "EU", value = "eu" },
]
"#;
        let manifest = PluginManifest::from_str(toml).unwrap();
        assert_eq!(manifest.commands.len(), 2);
        assert_eq!(manifest.commands[1].timeout, Some(30));
        assert_eq!(manifest.preferences.len(), 2);
        assert_eq!(manifest.preferences[1].options.len(), 2);
        assert_eq!(manifest.plugin.author, Some("test-author".to_string()));
        assert_eq!(manifest.plugin.icon, Some("icon.png".to_string()));
    }
}
