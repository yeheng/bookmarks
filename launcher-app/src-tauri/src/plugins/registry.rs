//! Plugin registry — discovery, registration, and lifecycle management.
//!
//! Stores plugin metadata in JSON files and maintains an in-memory keyword index
//! for fast lookup during search.

use crate::error::AppError;
use crate::plugins::executor::PluginExecutor;
use crate::plugins::manifest::PluginManifest;
use crate::search::plugin_provider::PluginSearchProvider;
use crate::search::SearchAggregator;
use crate::services::data_service::DataService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Registered plugin info — stored in JSON + returned to frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub enabled: bool,
    pub install_path: String,
    pub installed_at: String,
    pub updated_at: Option<String>,
    pub icon: Option<String>,
    /// Keywords registered by this plugin's commands.
    pub keywords: Vec<String>,
    /// Number of commands in this plugin.
    pub command_count: usize,
}

/// A resolved keyword → plugin+command mapping.
#[derive(Debug, Clone)]
pub struct KeywordEntry {
    pub plugin_id: String,
    pub command_name: String,
    pub plugin_dir: PathBuf,
}

/// Plugin registry — manages plugin lifecycle and keyword routing.
pub struct PluginRegistry {
    /// Root directory where plugins are stored.
    plugins_dir: PathBuf,
    /// In-memory keyword → (plugin_id, command_name) index.
    keywords: Mutex<HashMap<String, KeywordEntry>>,
    /// Cached manifests for loaded plugins.
    manifests: Mutex<HashMap<String, PluginManifest>>,
    /// Search Aggregator for dynamic provider registration.
    aggregator: Arc<SearchAggregator>,
    /// Data Service for plugin store access.
    data_service: Arc<DataService>,
    /// Plugin Executor for running commands.
    executor: Arc<PluginExecutor>,
}

impl PluginRegistry {
    /// Create a new registry. Creates the plugins directory if it doesn't exist.
    pub fn new(
        plugins_dir: PathBuf,
        aggregator: Arc<SearchAggregator>,
        data_service: Arc<DataService>,
        executor: Arc<PluginExecutor>,
    ) -> Result<Self, AppError> {
        std::fs::create_dir_all(&plugins_dir)?;
        Ok(Self {
            plugins_dir,
            keywords: Mutex::new(HashMap::new()),
            manifests: Mutex::new(HashMap::new()),
            aggregator,
            data_service,
            executor,
        })
    }

    /// Get the plugins root directory path.
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    // ── Discovery ──────────────────────────────────────────────────

    /// Scan plugins directory and sync with plugin store.
    /// Returns list of newly discovered plugin IDs.
    pub fn discover(&self) -> Result<Vec<String>, AppError> {
        let mut discovered = Vec::new();
        let entries = std::fs::read_dir(&self.plugins_dir)?;

        let mut disk_plugins: HashMap<String, PathBuf> = HashMap::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("plugin.toml");
            if !manifest_path.exists() {
                continue;
            }

            match PluginManifest::from_file(&manifest_path) {
                Ok(manifest) => {
                    let plugin_id = manifest.id().to_string();
                    disk_plugins.insert(plugin_id.clone(), path.clone());

                    // Check if already registered
                    let existing_version = self.data_service.with_plugin_store(|store| {
                        Ok(store.find_by_id(&plugin_id).map(|p| p.version.clone()))
                    }).unwrap_or(None);

                    match existing_version {
                        None => {
                            // New plugin — register it
                            self.register_plugin(&manifest, &path)?;
                            discovered.push(plugin_id.clone());
                        }
                        Some(old_version) if old_version != manifest.plugin.version => {
                            // Version changed — update registry
                            self.update_plugin(&manifest, &path)?;
                        }
                        _ => {
                            // Already registered, same version
                        }
                    }

                    // Cache the manifest and register keywords
                    self.load_manifest_keywords(&manifest, &path)?;
                    // Register search providers
                    self.register_manifest_providers(&manifest, &path);
                }
                Err(e) => {
                    eprintln!(
                        "[PluginRegistry] Failed to parse {}: {}",
                        manifest_path.display(),
                        e
                    );
                }
            }
        }

        // Remove plugins whose directories no longer exist
        let registered_ids: Vec<String> = self
            .data_service
            .with_plugin_store(|store| {
                Ok(store.plugins().iter().map(|p| p.id.clone()).collect())
            })
            .unwrap_or_default();

        for id in registered_ids {
            if !disk_plugins.contains_key(&id) {
                self.remove_plugin(&id)?;
            }
        }

        Ok(discovered)
    }

    /// Load a manifest into memory and register its keywords.
    fn load_manifest_keywords(
        &self,
        manifest: &PluginManifest,
        plugin_dir: &Path,
    ) -> Result<(), AppError> {
        let plugin_id = manifest.id().to_string();

        let mut keywords = self
            .keywords
            .lock()
            .map_err(|_| AppError::Generic("keyword lock poisoned".to_string()))?;
        let mut manifests = self
            .manifests
            .lock()
            .map_err(|_| AppError::Generic("manifest lock poisoned".to_string()))?;

        for cmd in &manifest.commands {
            keywords.insert(
                cmd.keyword.clone(),
                KeywordEntry {
                    plugin_id: plugin_id.clone(),
                    command_name: cmd.name.clone(),
                    plugin_dir: plugin_dir.to_path_buf(),
                },
            );
        }

        manifests.insert(plugin_id, manifest.clone());
        Ok(())
    }

    // ── CRUD ───────────────────────────────────────────────────────

    fn register_plugin(
        &self,
        manifest: &PluginManifest,
        plugin_dir: &Path,
    ) -> Result<(), AppError> {
        let now = now_iso();
        let info = PluginInfo {
            id: manifest.plugin.name.clone(),
            title: manifest.plugin.title.clone(),
            description: manifest.plugin.description.clone(),
            version: manifest.plugin.version.clone(),
            author: manifest.plugin.author.clone(),
            enabled: true,
            install_path: plugin_dir.to_string_lossy().to_string(),
            installed_at: now,
            updated_at: None,
            icon: manifest.plugin.icon.clone(),
            keywords: manifest.commands.iter().map(|c| c.keyword.clone()).collect(),
            command_count: manifest.commands.len(),
        };

        self.data_service.with_plugin_store_mut(|store| {
            store.register(info).map_err(|e| AppError::Generic(e))
        })
    }

    fn update_plugin(
        &self,
        manifest: &PluginManifest,
        plugin_dir: &Path,
    ) -> Result<(), AppError> {
        let now = now_iso();
        let info = PluginInfo {
            id: manifest.plugin.name.clone(),
            title: manifest.plugin.title.clone(),
            description: manifest.plugin.description.clone(),
            version: manifest.plugin.version.clone(),
            author: manifest.plugin.author.clone(),
            enabled: true,
            install_path: plugin_dir.to_string_lossy().to_string(),
            installed_at: String::new(), // keep existing
            updated_at: Some(now),
            icon: manifest.plugin.icon.clone(),
            keywords: manifest.commands.iter().map(|c| c.keyword.clone()).collect(),
            command_count: manifest.commands.len(),
        };

        self.data_service.with_plugin_store_mut(|store| {
            store.update(info).map_err(|e| AppError::Generic(e))
        })
    }

    fn remove_plugin(&self, id: &str) -> Result<(), AppError> {
        // Remove keywords from memory
        if let Ok(mut keywords) = self.keywords.lock() {
            keywords.retain(|_, v| v.plugin_id != id);
        }
        if let Ok(mut manifests) = self.manifests.lock() {
            manifests.remove(id);
        }

        self.data_service.with_plugin_store_mut(|store| {
            store.remove(id).map_err(|e| AppError::Generic(e))
        })
    }

    /// Install a plugin from a directory path (copy into plugins dir).
    pub fn install_from_dir(&self, source_dir: &Path) -> Result<String, AppError> {
        let manifest_path = source_dir.join("plugin.toml");
        if !manifest_path.exists() {
            return Err(AppError::Generic(
                "No plugin.toml found in source directory".to_string(),
            ));
        }

        let manifest = PluginManifest::from_file(&manifest_path)
            .map_err(|e| AppError::Generic(format!("Invalid manifest: {}", e)))?;

        let plugin_id = manifest.id().to_string();
        let dest_dir = self.plugins_dir.join(&plugin_id);

        // Check keyword conflicts
        self.check_keyword_conflicts(&manifest)?;

        // Copy plugin files
        if source_dir != dest_dir {
            if dest_dir.exists() {
                std::fs::remove_dir_all(&dest_dir)?;
            }
            copy_dir_recursive(source_dir, &dest_dir)?;
        }

        // Register
        self.register_plugin(&manifest, &dest_dir)?;
        self.load_manifest_keywords(&manifest, &dest_dir)?;
        self.register_manifest_providers(&manifest, &dest_dir);

        Ok(plugin_id)
    }

    /// Uninstall a plugin by ID.
    pub fn uninstall(&self, plugin_id: &str) -> Result<(), AppError> {
        // Unregister providers first
        if let Some(manifest) = self.get_manifest(plugin_id) {
            self.unregister_manifest_providers(&manifest);
        }

        let install_path = self
            .data_service
            .with_plugin_store(|store| {
                store
                    .find_by_id(plugin_id)
                    .map(|p| p.install_path.clone())
                    .ok_or_else(|| AppError::Generic(format!("Plugin '{}' not found", plugin_id)))
            })?;

        let dir = PathBuf::from(&install_path);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }

        self.remove_plugin(plugin_id)
    }

    /// Enable a plugin.
    pub fn enable(&self, plugin_id: &str) -> Result<(), AppError> {
        self.data_service.with_plugin_store_mut(|store| {
            store.enable(plugin_id).map_err(|e| AppError::Generic(e))
        })?;

        // Re-register keywords from the manifest
        let manifests = self
            .manifests
            .lock()
            .map_err(|_| AppError::Generic("lock".to_string()))?;
        if let Some(manifest) = manifests.get(plugin_id) {
            let install_path = self.data_service.with_plugin_store(|store| {
                store
                    .find_by_id(plugin_id)
                    .map(|p| p.install_path.clone())
                    .ok_or_else(|| AppError::Generic(format!("Plugin '{}' not found", plugin_id)))
            })?;

            let mut keywords = self
                .keywords
                .lock()
                .map_err(|_| AppError::Generic("lock".to_string()))?;
            for cmd in &manifest.commands {
                keywords.insert(
                    cmd.keyword.clone(),
                    KeywordEntry {
                        plugin_id: plugin_id.to_string(),
                        command_name: cmd.name.clone(),
                        plugin_dir: PathBuf::from(&install_path),
                    },
                );
            }
            // Register providers
            self.register_manifest_providers(manifest, &PathBuf::from(&install_path));
        }

        Ok(())
    }

    /// Disable a plugin.
    pub fn disable(&self, plugin_id: &str) -> Result<(), AppError> {
        self.data_service.with_plugin_store_mut(|store| {
            store.disable(plugin_id).map_err(|e| AppError::Generic(e))
        })?;

        // Unregister search providers
        if let Some(manifest) = self.get_manifest(plugin_id) {
            self.unregister_manifest_providers(&manifest);
        }

        // Remove keywords from the index
        let mut keywords = self
            .keywords
            .lock()
            .map_err(|_| AppError::Generic("lock".to_string()))?;
        keywords.retain(|_, v| v.plugin_id != plugin_id);

        Ok(())
    }

    // ── Query ──────────────────────────────────────────────────────

    /// List all plugins.
    pub fn list(&self) -> Result<Vec<PluginInfo>, AppError> {
        let plugins = self
            .data_service
            .with_plugin_store(|store| Ok(store.list()))?;

        // Enrich with keyword + command data from cached manifests
        let manifests = self
            .manifests
            .lock()
            .map_err(|_| AppError::Generic("lock".to_string()))?;
        let enriched = plugins
            .into_iter()
            .map(|mut p| {
                if let Some(m) = manifests.get(&p.id) {
                    p.keywords = m.commands.iter().map(|c| c.keyword.clone()).collect();
                    p.command_count = m.commands.len();
                }
                p
            })
            .collect();

        Ok(enriched)
    }

    /// Get a single plugin's info.
    pub fn get(&self, plugin_id: &str) -> Result<PluginInfo, AppError> {
        let all = self.list()?;
        all.into_iter()
            .find(|p| p.id == plugin_id)
            .ok_or_else(|| AppError::Generic(format!("Plugin '{}' not found", plugin_id)))
    }

    /// Look up a keyword → get the plugin command entry.
    pub fn resolve_keyword(&self, keyword: &str) -> Option<KeywordEntry> {
        self.keywords.lock().ok()?.get(keyword).cloned()
    }

    /// Get all registered keywords (for frontend to detect).
    pub fn get_keywords(&self) -> Vec<String> {
        self.keywords
            .lock()
            .map(|k| k.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get manifest for a plugin.
    pub fn get_manifest(&self, plugin_id: &str) -> Option<PluginManifest> {
        self.manifests.lock().ok()?.get(plugin_id).cloned()
    }

    // ── Preferences ────────────────────────────────────────────────

    /// Get all preferences for a plugin.
    pub fn get_preferences(&self, plugin_id: &str) -> Result<HashMap<String, String>, AppError> {
        self.data_service.with_plugin_store(|store| {
            Ok(store.get_preferences(plugin_id))
        })
    }

    /// Set a preference value.
    pub fn set_preference(
        &self,
        plugin_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), AppError> {
        self.data_service.with_plugin_store_mut(|store| {
            store
                .set_preference(plugin_id, key, value)
                .map_err(|e| AppError::Generic(e))
        })
    }

    // ── Helpers ────────────────────────────────────────────────────

    fn check_keyword_conflicts(&self, manifest: &PluginManifest) -> Result<(), AppError> {
        let keywords = self
            .keywords
            .lock()
            .map_err(|_| AppError::Generic("lock".to_string()))?;
        for cmd in &manifest.commands {
            if let Some(existing) = keywords.get(&cmd.keyword) {
                if existing.plugin_id != manifest.id() {
                    return Err(AppError::Generic(format!(
                        "Keyword '{}' already registered by plugin '{}'",
                        cmd.keyword, existing.plugin_id
                    )));
                }
            }
        }
        Ok(())
    }

    fn register_manifest_providers(&self, manifest: &PluginManifest, plugin_dir: &Path) {
        for cmd in &manifest.commands {
            if cmd.mode == "search" {
                let provider = PluginSearchProvider::new(
                    manifest.plugin.name.clone(),
                    plugin_dir.to_path_buf(),
                    cmd.clone(),
                    self.executor.clone(),
                    self.data_service.clone(),
                );
                self.aggregator
                    .register(format!("plugin:{}", cmd.keyword), Box::new(provider));
            }
        }
    }

    fn unregister_manifest_providers(&self, manifest: &PluginManifest) {
        for cmd in &manifest.commands {
            if cmd.mode == "search" {
                self.aggregator
                    .unregister(&format!("plugin:{}", cmd.keyword));
            }
        }
    }
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
