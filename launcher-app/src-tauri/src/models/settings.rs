use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
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
    #[serde(default = "default_ui_shortcuts")]
    pub ui_shortcuts: HashMap<String, String>,
}

pub fn default_ui_shortcuts() -> HashMap<String, String> {
    let mut shortcuts = HashMap::new();

    shortcuts.insert("general.close".to_string(), "Escape".to_string());

    #[cfg(target_os = "macos")]
    {
        shortcuts.insert("general.settings".to_string(), "Meta+,".to_string());
        shortcuts.insert("search.open_new_tab".to_string(), "Meta+Enter".to_string());
    }
    #[cfg(not(target_os = "macos"))]
    {
        shortcuts.insert("general.settings".to_string(), "Ctrl+,".to_string());
        shortcuts.insert("search.open_new_tab".to_string(), "Ctrl+Enter".to_string());
    }

    shortcuts.insert("search.next".to_string(), "ArrowDown".to_string());
    shortcuts.insert("search.prev".to_string(), "ArrowUp".to_string());
    shortcuts.insert("search.open".to_string(), "Enter".to_string());

    shortcuts
}

impl Default for HotkeySettings {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let shortcut = "Cmd+F1".to_string();
        #[cfg(not(target_os = "macos"))]
        let shortcut = "Ctrl+F1".to_string();

        HotkeySettings {
            global_shortcut: shortcut,
            ui_shortcuts: default_ui_shortcuts(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub mode: ThemeMode,
    pub accent_color: String,
    pub bg_color: Option<String>,
    pub text_color: Option<String>,
    pub secondary_text_color: Option<String>,
    pub border_color: Option<String>,
    pub selection_bg_color: Option<String>,
    pub selection_text_color: Option<String>,
    pub font_size: u8,
    pub window_width: u32,
    pub window_height: u32,
    pub input_height: u32,
    pub item_height: u32,
    pub border_radius: u32,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        ThemeSettings {
            mode: ThemeMode::System,
            accent_color: "#ff6b6b".to_string(),
            bg_color: None,
            text_color: None,
            secondary_text_color: None,
            border_color: None,
            selection_bg_color: None,
            selection_text_color: None,
            font_size: 14,
            window_width: 750,
            window_height: 480,
            input_height: 60,
            item_height: 44,
            border_radius: 12,
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
