export interface HotkeySettings {
  global_shortcut: string;
  ui_shortcuts: Record<string, string>;
}

export interface ThemeSettings {
  mode: 'light' | 'dark' | 'system';
  accent_color: string;
  bg_color?: string;
  text_color?: string;
  secondary_text_color?: string;
  border_color?: string;
  selection_bg_color?: string;
  selection_text_color?: string;
  font_size: number;
  window_width: number;
  window_height: number;
  input_height: number;
  item_height: number;
  border_radius: number;
}

export interface SearchSettings {
  max_results: number;
  show_bookmarks: boolean;
  show_files: boolean;
  fuzzy_matching: boolean;
}

export interface GeneralSettings {
  launch_at_startup: boolean;
  hide_dock_icon: boolean;
  check_updates: boolean;
}

export interface AppSettings {
  hotkey: HotkeySettings;
  theme: ThemeSettings;
  search: SearchSettings;
  general: GeneralSettings;
}
