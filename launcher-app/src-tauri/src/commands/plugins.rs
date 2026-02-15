//! Tauri commands for the plugin system.

use crate::commands::bookmarks::AppState;
use crate::plugins::executor::PluginResponse;
use crate::plugins::manifest::HOST_API_VERSION;
use crate::plugins::registry::PluginInfo;
use std::collections::HashMap;
use tauri::State;

/// Execute a plugin command by keyword.
#[tauri::command]
pub fn execute_plugin_command(
    state: State<AppState>,
    keyword: String,
    query: String,
) -> Result<PluginResponse, String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;
    let executor = state
        .plugin_executor
        .as_ref()
        .ok_or("Plugin executor not initialized")?;

    // Resolve keyword to plugin command
    let entry = registry
        .resolve_keyword(&keyword)
        .ok_or(format!("No plugin found for keyword '{}'", keyword))?;

    // Get the manifest to find the command details
    let manifest = registry.get_manifest(&entry.plugin_id).ok_or(format!(
        "Manifest not found for plugin '{}'",
        entry.plugin_id
    ))?;

    let command = manifest
        .commands
        .iter()
        .find(|c| c.name == entry.command_name)
        .ok_or(format!(
            "Command '{}' not found in plugin '{}'",
            entry.command_name, entry.plugin_id
        ))?;

    // Get plugin preferences
    let preferences = registry
        .get_preferences(&entry.plugin_id)
        .unwrap_or_default();

    // Execute the plugin command
    executor
        .execute(
            &entry.plugin_dir,
            command,
            &query,
            HashMap::new(),
            preferences,
            &HOST_API_VERSION,
        )
        .map_err(|e| e.to_string())
}

/// List all installed plugins.
#[tauri::command]
pub fn list_plugins(state: State<AppState>) -> Result<Vec<PluginInfo>, String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    registry.list().map_err(|e| e.to_string())
}

/// Install a plugin from a local directory path.
#[tauri::command]
pub fn install_plugin(state: State<AppState>, source_path: String) -> Result<String, String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    let path = std::path::PathBuf::from(&source_path);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", source_path));
    }

    registry
        .install_from_dir(&path)
        .map_err(|e| e.to_string())
}

/// Uninstall a plugin by ID.
#[tauri::command]
pub fn uninstall_plugin(state: State<AppState>, plugin_id: String) -> Result<(), String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    registry.uninstall(&plugin_id).map_err(|e| e.to_string())
}

/// Enable a plugin.
#[tauri::command]
pub fn enable_plugin(state: State<AppState>, plugin_id: String) -> Result<(), String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    registry.enable(&plugin_id).map_err(|e| e.to_string())
}

/// Disable a plugin.
#[tauri::command]
pub fn disable_plugin(state: State<AppState>, plugin_id: String) -> Result<(), String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    registry.disable(&plugin_id).map_err(|e| e.to_string())
}

/// Get plugin preferences.
#[tauri::command]
pub fn get_plugin_preferences(
    state: State<AppState>,
    plugin_id: String,
) -> Result<HashMap<String, String>, String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    registry
        .get_preferences(&plugin_id)
        .map_err(|e| e.to_string())
}

/// Set a plugin preference.
#[tauri::command]
pub fn set_plugin_preference(
    state: State<AppState>,
    plugin_id: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    registry
        .set_preference(&plugin_id, &key, &value)
        .map_err(|e| e.to_string())
}

/// Get plugin log contents.
#[tauri::command]
pub fn get_plugin_log(state: State<AppState>, plugin_id: String) -> Result<String, String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;
    let executor = state
        .plugin_executor
        .as_ref()
        .ok_or("Plugin executor not initialized")?;

    let info = registry.get(&plugin_id).map_err(|e| e.to_string())?;

    let plugin_dir = std::path::PathBuf::from(&info.install_path);
    executor.get_log(&plugin_dir).map_err(|e| e.to_string())
}

/// Get all registered keywords (for frontend detection).
#[tauri::command]
pub fn get_plugin_keywords(state: State<AppState>) -> Result<Vec<String>, String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    Ok(registry.get_keywords())
}

/// Get the manifest preferences schema for a plugin.
#[tauri::command]
pub fn get_plugin_manifest_preferences(
    state: State<AppState>,
    plugin_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let registry = state
        .plugin_registry
        .as_ref()
        .ok_or("Plugin system not initialized")?;

    let manifest = registry
        .get_manifest(&plugin_id)
        .ok_or(format!("Plugin '{}' not found", plugin_id))?;

    let prefs_json: Vec<serde_json::Value> = manifest
        .preferences
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();

    Ok(prefs_json)
}
