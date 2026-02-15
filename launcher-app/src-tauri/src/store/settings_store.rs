use crate::models::settings::AppSettings;
use crate::store::json_store::JsonStore;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct SettingsStore {
    store: JsonStore,
    data: AppSettings,
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        let store = JsonStore::new(path);
        let data: AppSettings = store.load();
        Self { store, data }
    }

    pub fn save(&self) -> Result<(), String> {
        self.store.save(&self.data)
    }

    pub fn settings(&self) -> &AppSettings {
        &self.data
    }

    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.data
    }

    pub fn set(&mut self, settings: AppSettings) -> Result<(), String> {
        self.data = settings;
        self.save()
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.data = AppSettings::default();
        self.save()
    }

    /// Convert settings to a flat key-value HashMap (for compatibility with settings cache
    /// and export).
    pub fn to_flat_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let s = &self.data;

        // Hotkey settings
        map.insert(
            "hotkey.global_shortcut".to_string(),
            s.hotkey.global_shortcut.clone(),
        );
        if let Ok(json) = serde_json::to_string(&s.hotkey.ui_shortcuts) {
            map.insert("hotkey.ui_shortcuts".to_string(), json);
        }

        // Theme settings
        map.insert(
            "theme.mode".to_string(),
            match s.theme.mode {
                crate::models::settings::ThemeMode::Light => "light",
                crate::models::settings::ThemeMode::Dark => "dark",
                crate::models::settings::ThemeMode::System => "system",
            }
            .to_string(),
        );
        map.insert("theme.accent_color".to_string(), s.theme.accent_color.clone());
        if let Some(ref v) = s.theme.bg_color {
            map.insert("theme.bg_color".to_string(), v.clone());
        }
        if let Some(ref v) = s.theme.text_color {
            map.insert("theme.text_color".to_string(), v.clone());
        }
        if let Some(ref v) = s.theme.secondary_text_color {
            map.insert("theme.secondary_text_color".to_string(), v.clone());
        }
        if let Some(ref v) = s.theme.border_color {
            map.insert("theme.border_color".to_string(), v.clone());
        }
        if let Some(ref v) = s.theme.selection_bg_color {
            map.insert("theme.selection_bg_color".to_string(), v.clone());
        }
        if let Some(ref v) = s.theme.selection_text_color {
            map.insert("theme.selection_text_color".to_string(), v.clone());
        }
        map.insert("theme.font_size".to_string(), s.theme.font_size.to_string());
        map.insert("theme.window_width".to_string(), s.theme.window_width.to_string());
        map.insert("theme.window_height".to_string(), s.theme.window_height.to_string());
        map.insert("theme.input_height".to_string(), s.theme.input_height.to_string());
        map.insert("theme.item_height".to_string(), s.theme.item_height.to_string());
        map.insert("theme.border_radius".to_string(), s.theme.border_radius.to_string());

        // Search settings
        map.insert("search.max_results".to_string(), s.search.max_results.to_string());
        map.insert("search.show_bookmarks".to_string(), s.search.show_bookmarks.to_string());
        map.insert("search.show_files".to_string(), s.search.show_files.to_string());
        map.insert("search.fuzzy_matching".to_string(), s.search.fuzzy_matching.to_string());

        // General settings
        map.insert(
            "general.launch_at_startup".to_string(),
            s.general.launch_at_startup.to_string(),
        );
        map.insert(
            "general.hide_dock_icon".to_string(),
            s.general.hide_dock_icon.to_string(),
        );
        map.insert(
            "general.check_updates".to_string(),
            s.general.check_updates.to_string(),
        );

        map
    }

    /// Update settings from a flat key-value HashMap (for import compatibility).
    pub fn apply_flat_map(&mut self, map: &HashMap<String, String>) {
        use crate::models::settings::ThemeMode;

        if let Some(v) = map.get("hotkey.global_shortcut") {
            self.data.hotkey.global_shortcut = v.clone();
        }
        if let Some(v) = map.get("hotkey.ui_shortcuts") {
            if let Ok(shortcuts) = serde_json::from_str(v) {
                self.data.hotkey.ui_shortcuts = shortcuts;
            }
        }

        if let Some(v) = map.get("theme.mode") {
            self.data.theme.mode = match v.as_str() {
                "light" => ThemeMode::Light,
                "dark" => ThemeMode::Dark,
                _ => ThemeMode::System,
            };
        }
        if let Some(v) = map.get("theme.accent_color") {
            self.data.theme.accent_color = v.clone();
        }
        self.data.theme.bg_color = map.get("theme.bg_color").cloned();
        self.data.theme.text_color = map.get("theme.text_color").cloned();
        self.data.theme.secondary_text_color = map.get("theme.secondary_text_color").cloned();
        self.data.theme.border_color = map.get("theme.border_color").cloned();
        self.data.theme.selection_bg_color = map.get("theme.selection_bg_color").cloned();
        self.data.theme.selection_text_color = map.get("theme.selection_text_color").cloned();

        if let Some(v) = map.get("theme.font_size").and_then(|s| s.parse().ok()) {
            self.data.theme.font_size = v;
        }
        if let Some(v) = map.get("theme.window_width").and_then(|s| s.parse().ok()) {
            self.data.theme.window_width = v;
        }
        if let Some(v) = map.get("theme.window_height").and_then(|s| s.parse().ok()) {
            self.data.theme.window_height = v;
        }
        if let Some(v) = map.get("theme.input_height").and_then(|s| s.parse().ok()) {
            self.data.theme.input_height = v;
        }
        if let Some(v) = map.get("theme.item_height").and_then(|s| s.parse().ok()) {
            self.data.theme.item_height = v;
        }
        if let Some(v) = map.get("theme.border_radius").and_then(|s| s.parse().ok()) {
            self.data.theme.border_radius = v;
        }

        if let Some(v) = map.get("search.max_results").and_then(|s| s.parse().ok()) {
            self.data.search.max_results = v;
        }
        if let Some(v) = map.get("search.show_bookmarks") {
            self.data.search.show_bookmarks = v == "true";
        }
        if let Some(v) = map.get("search.show_files") {
            self.data.search.show_files = v == "true";
        }
        if let Some(v) = map.get("search.fuzzy_matching") {
            self.data.search.fuzzy_matching = v == "true";
        }

        if let Some(v) = map.get("general.launch_at_startup") {
            self.data.general.launch_at_startup = v == "true";
        }
        if let Some(v) = map.get("general.hide_dock_icon") {
            self.data.general.hide_dock_icon = v == "true";
        }
        if let Some(v) = map.get("general.check_updates") {
            self.data.general.check_updates = v == "true";
        }
    }
}
