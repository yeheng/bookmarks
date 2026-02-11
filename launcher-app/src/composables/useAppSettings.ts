/**
 * Composable for managing application settings state and operations.
 * Handles loading, saving, and auto-save functionality.
 */
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types/settings";

// Types
export interface DataStats {
  bookmarks_count: number;
  files_count: number;
  directories_count: number;
  settings_count: number;
}

export type TabId = 'general' | 'appearance' | 'shortcuts' | 'data' | 'plugins';

export interface Tab {
  id: TabId;
  label: string;
  icon: string;
}

// Tabs configuration
export const tabs: Tab[] = [
  { id: 'general', label: 'General', icon: '⚙️' },
  { id: 'appearance', label: 'Appearance', icon: '🎨' },
  { id: 'shortcuts', label: 'Shortcuts', icon: '⌨️' },
  { id: 'plugins', label: 'Plugins', icon: '🧩' },
  { id: 'data', label: 'Data', icon: '📁' },
];

// Shortcuts configuration
export const shortcuts = [
  { id: 'general.close', label: 'Close Window', description: 'Close the launcher or exit settings' },
  { id: 'general.settings', label: 'Open Settings', description: 'Toggle settings panel' },
  { id: 'search.next', label: 'Next Result', description: 'Select next search result' },
  { id: 'search.prev', label: 'Previous Result', description: 'Select previous search result' },
  { id: 'search.open', label: 'Open Result', description: 'Open the selected result' },
  { id: 'search.open_new_tab', label: 'Open in Background', description: '(Future) Open result in background tab' },
];

// Theme options
export const themeOptions = [
  { value: 'system', label: 'System' },
  { value: 'dark', label: 'Dark' },
  { value: 'light', label: 'Light' },
];

export function useAppSettings() {
  // State
  const settings = ref<AppSettings | null>(null);
  const stats = ref<DataStats | null>(null);
  const saving = ref(false);
  const importStatus = ref("");

  // Data operations
  async function loadSettings() {
    try {
      settings.value = await invoke<AppSettings>("get_app_settings");
      stats.value = await invoke<DataStats>("get_data_stats");
    } catch (err) {
      console.error("Failed to load settings:", err);
    }
  }

  async function saveSettings() {
    if (!settings.value) return;
    saving.value = true;
    try {
      await invoke("save_app_settings", { settings: settings.value });
    } catch (err) {
      console.error("Failed to save settings:", err);
    } finally {
      saving.value = false;
    }
  }

  async function importBookmarks(browser: string) {
    importStatus.value = `Importing from ${browser}...`;
    try {
      const cmd =
        browser === "chrome"
          ? "import_chrome_bookmarks"
          : browser === "firefox"
            ? "import_firefox_bookmarks"
            : "import_safari_bookmarks";
      const result = await invoke<{ imported: number; skipped: number }>(cmd);
      importStatus.value = `Imported ${result.imported}, skipped ${result.skipped}`;
      stats.value = await invoke<DataStats>("get_data_stats");
    } catch (err) {
      importStatus.value = `Import failed: ${err}`;
    }
  }

  async function resetSettings() {
    if (!confirm('Are you sure you want to reset all settings to defaults?')) return;
    try {
      await invoke("reset_settings");
      await loadSettings();
      importStatus.value = "Settings reset to defaults";
    } catch (err) {
      console.error("Failed to reset settings:", err);
    }
  }

  async function exportSettings() {
    if (!settings.value) return;
    try {
      const data = JSON.stringify(settings.value, null, 2);
      const blob = new Blob([data], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'bookmark-launcher-settings.json';
      a.click();
      URL.revokeObjectURL(url);
      importStatus.value = "Settings exported successfully";
    } catch (err) {
      console.error("Failed to export settings:", err);
      importStatus.value = "Export failed";
    }
  }

  async function importSettingsFromFile() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      try {
        const text = await file.text();
        const imported = JSON.parse(text) as AppSettings;
        settings.value = imported;
        await saveSettings();
        importStatus.value = "Settings imported successfully";
      } catch (err) {
        console.error("Failed to import settings:", err);
        importStatus.value = "Import failed: Invalid file format";
      }
    };
    input.click();
  }

  function handleShortcutUpdate(actionId: string, shortcut: string) {
    if (!settings.value) return;
    settings.value.hotkey.ui_shortcuts[actionId] = shortcut;
  }

  // Auto-save when settings change (debounced)
  let saveTimeout: number | null = null;
  function setupAutoSave() {
    watch(
      settings,
      () => {
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = window.setTimeout(() => {
          saveSettings();
        }, 1000);
      },
      { deep: true }
    );
  }

  return {
    // State
    settings,
    stats,
    saving,
    importStatus,
    // Operations
    loadSettings,
    saveSettings,
    importBookmarks,
    resetSettings,
    exportSettings,
    importSettingsFromFile,
    handleShortcutUpdate,
    setupAutoSave,
  };
}
