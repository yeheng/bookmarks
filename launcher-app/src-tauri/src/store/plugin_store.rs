use crate::plugins::registry::PluginInfo;
use crate::store::json_store::JsonStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginStoreData {
    pub plugins: Vec<PluginInfo>,
    pub preferences: HashMap<String, HashMap<String, String>>,
}

pub struct PluginStore {
    store: JsonStore,
    data: PluginStoreData,
}

impl PluginStore {
    pub fn new(path: PathBuf) -> Self {
        let store = JsonStore::new(path);
        let data: PluginStoreData = store.load();
        Self { store, data }
    }

    pub fn save(&self) -> Result<(), String> {
        self.store.save(&self.data)
    }

    pub fn plugins(&self) -> &[PluginInfo] {
        &self.data.plugins
    }

    pub fn find_by_id(&self, id: &str) -> Option<&PluginInfo> {
        self.data.plugins.iter().find(|p| p.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut PluginInfo> {
        self.data.plugins.iter_mut().find(|p| p.id == id)
    }

    pub fn register(&mut self, plugin: PluginInfo) -> Result<(), String> {
        // Remove existing if present (re-register)
        self.data.plugins.retain(|p| p.id != plugin.id);
        self.data.plugins.push(plugin);
        self.save()
    }

    pub fn update(&mut self, plugin: PluginInfo) -> Result<(), String> {
        if let Some(existing) = self.data.plugins.iter_mut().find(|p| p.id == plugin.id) {
            existing.title = plugin.title;
            existing.description = plugin.description;
            existing.version = plugin.version;
            existing.author = plugin.author;
            existing.install_path = plugin.install_path;
            existing.updated_at = plugin.updated_at;
            existing.icon = plugin.icon;
        }
        self.save()
    }

    pub fn remove(&mut self, id: &str) -> Result<(), String> {
        self.data.plugins.retain(|p| p.id != id);
        self.data.preferences.remove(id);
        self.save()
    }

    pub fn enable(&mut self, id: &str) -> Result<(), String> {
        if let Some(p) = self.find_by_id_mut(id) {
            p.enabled = true;
        }
        self.save()
    }

    pub fn disable(&mut self, id: &str) -> Result<(), String> {
        if let Some(p) = self.find_by_id_mut(id) {
            p.enabled = false;
        }
        self.save()
    }

    pub fn get_preferences(&self, plugin_id: &str) -> HashMap<String, String> {
        self.data
            .preferences
            .get(plugin_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_preference(
        &mut self,
        plugin_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        self.data
            .preferences
            .entry(plugin_id.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
        self.save()
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        let mut plugins = self.data.plugins.clone();
        plugins.sort_by(|a, b| a.title.cmp(&b.title));
        plugins
    }
}
