use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub hotkey: HotkeySettings,
    pub theme: ThemeSettings,
    pub search: SearchSettings,
    pub general: GeneralSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    pub global_shortcut: String,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let shortcut = "Cmd+Space".to_string();
        #[cfg(not(target_os = "macos"))]
        let shortcut = "Ctrl+Space".to_string();

        HotkeySettings {
            global_shortcut: shortcut,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub mode: ThemeMode,
    pub accent_color: String,
    pub font_size: u8,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        ThemeSettings {
            mode: ThemeMode::System,
            accent_color: "#ff6b6b".to_string(),
            font_size: 14,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::System
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSettings {
    pub max_results: usize,
    pub show_bookmarks: bool,
    pub show_files: bool,
    pub fuzzy_matching: bool,
}

impl Default for SearchSettings {
    fn default() -> Self {
        SearchSettings {
            max_results: 10,
            show_bookmarks: true,
            show_files: true,
            fuzzy_matching: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub launch_at_startup: bool,
    pub hide_dock_icon: bool,
    pub check_updates: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        GeneralSettings {
            launch_at_startup: false,
            hide_dock_icon: true,
            check_updates: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: i64,
    pub bookmarks: Vec<ExportedBookmark>,
    pub search_directories: Vec<String>,
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedBookmark {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub source: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub bookmarks_imported: usize,
    pub bookmarks_skipped: usize,
    pub directories_imported: usize,
    pub settings_imported: usize,
    pub errors: Vec<String>,
}
