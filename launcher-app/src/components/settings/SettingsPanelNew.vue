<script setup lang="ts">
/**
 * SettingsPanel - Refactored settings panel with tabbed navigation
 *
 * Tabs:
 * - General: Launch at startup, updates, dock icon
 * - Appearance: Theme, colors, layout
 * - Shortcuts: Global hotkey, UI shortcuts
 * - Data: Import, export, stats
 */
import { ref, computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../../types/settings";
import SettingToggle from "./SettingToggle.vue";
import SettingSelect from "./SettingSelect.vue";
import SettingNumber from "./SettingNumber.vue";
import SettingColor from "./SettingColor.vue";
import SettingSection from "./SettingSection.vue";
import ShortcutEditor from "../ShortcutEditor.vue";

// Types
interface DataStats {
  bookmarks_count: number;
  files_count: number;
  directories_count: number;
  settings_count: number;
}

type TabId = 'general' | 'appearance' | 'shortcuts' | 'data';

interface Tab {
  id: TabId;
  label: string;
  icon: string;
}

// Events
const emit = defineEmits<{ (e: "close"): void }>();

// State
const settings = ref<AppSettings | null>(null);
const stats = ref<DataStats | null>(null);
const saving = ref(false);
const importStatus = ref("");
const activeTab = ref<TabId>('general');
const searchQuery = ref("");

// Tabs configuration
const tabs: Tab[] = [
  { id: 'general', label: 'General', icon: '⚙️' },
  { id: 'appearance', label: 'Appearance', icon: '🎨' },
  { id: 'shortcuts', label: 'Shortcuts', icon: '⌨️' },
  { id: 'data', label: 'Data', icon: '📁' },
];

// Shortcuts configuration
const shortcuts = [
  { id: 'general.close', label: 'Close Window', description: 'Close the launcher or exit settings' },
  { id: 'general.settings', label: 'Open Settings', description: 'Toggle settings panel' },
  { id: 'search.next', label: 'Next Result', description: 'Select next search result' },
  { id: 'search.prev', label: 'Previous Result', description: 'Select previous search result' },
  { id: 'search.open', label: 'Open Result', description: 'Open the selected result' },
  { id: 'search.open_new_tab', label: 'Open in Background', description: '(Future) Open result in background tab' },
];

// Theme options
const themeOptions = [
  { value: 'system', label: 'System' },
  { value: 'dark', label: 'Dark' },
  { value: 'light', label: 'Light' },
];

// Search filtering
const filteredSettings = computed(() => {
  if (!searchQuery.value.trim()) return null;
  const query = searchQuery.value.toLowerCase();

  const matches: { tab: TabId; section: string; label: string }[] = [];

  // General tab items
  const generalItems = ['launch at startup', 'hide dock icon', 'check for updates'];
  generalItems.forEach(item => {
    if (item.includes(query)) {
      matches.push({ tab: 'general', section: 'General', label: item });
    }
  });

  // Appearance tab items
  const appearanceItems = ['theme mode', 'font size', 'border radius', 'window width', 'window height', 'accent color', 'background color', 'text color'];
  appearanceItems.forEach(item => {
    if (item.includes(query)) {
      matches.push({ tab: 'appearance', section: 'Appearance', label: item });
    }
  });

  // Shortcuts tab items
  shortcuts.forEach(s => {
    if (s.label.toLowerCase().includes(query) || s.description.toLowerCase().includes(query)) {
      matches.push({ tab: 'shortcuts', section: 'Shortcuts', label: s.label });
    }
  });

  return matches.length > 0 ? matches : null;
});

const hasSearchResults = computed(() => filteredSettings.value !== null);

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

function jumpToSetting(tab: TabId) {
  activeTab.value = tab;
  searchQuery.value = "";
}

// Keyboard navigation for tabs
function handleTabKeydown(e: KeyboardEvent, index: number) {
  let newIndex = index;

  if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
    e.preventDefault();
    newIndex = (index + 1) % tabs.length;
  } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
    e.preventDefault();
    newIndex = (index - 1 + tabs.length) % tabs.length;
  } else if (e.key === 'Home') {
    e.preventDefault();
    newIndex = 0;
  } else if (e.key === 'End') {
    e.preventDefault();
    newIndex = tabs.length - 1;
  }

  if (newIndex !== index) {
    activeTab.value = tabs[newIndex].id;
    // Focus the new tab button
    const tabButtons = document.querySelectorAll('[role="tab"]');
    (tabButtons[newIndex] as HTMLElement)?.focus();
  }
}

onMounted(loadSettings);

// Auto-save when settings change (debounced via watcher)
let saveTimeout: number | null = null;
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
</script>

<template>
  <div class="settings-panel" role="dialog" aria-labelledby="settings-title" aria-modal="true">
    <!-- Header -->
    <div class="settings-header">
      <h2 id="settings-title" class="settings-title">Settings</h2>
      <button class="close-btn" @click="emit('close')" aria-label="Close settings panel">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
          <path d="M15 5L5 15M5 5l10 10" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <!-- Search -->
    <div class="settings-search">
      <input
        v-model="searchQuery"
        type="search"
        class="search-input"
        placeholder="Search settings..."
        aria-label="Search settings"
      />
      <!-- Search Results -->
      <div v-if="hasSearchResults" class="search-results" role="listbox" aria-label="Search results">
        <button
          v-for="result in filteredSettings"
          :key="`${result.tab}-${result.label}`"
          class="search-result"
          role="option"
          @click="jumpToSetting(result.tab)"
        >
          <span class="search-result__section">{{ result.section }}</span>
          <span class="search-result__label">{{ result.label }}</span>
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="settings-tabs" role="tablist" aria-label="Settings categories">
      <button
        v-for="(tab, index) in tabs"
        :key="tab.id"
        :id="`tab-${tab.id}`"
        class="settings-tab"
        :class="{ 'settings-tab--active': activeTab === tab.id }"
        role="tab"
        :aria-selected="activeTab === tab.id"
        :aria-controls="`panel-${tab.id}`"
        :tabindex="activeTab === tab.id ? 0 : -1"
        @click="activeTab = tab.id"
        @keydown="handleTabKeydown($event, index)"
      >
        <span class="tab-icon" aria-hidden="true">{{ tab.icon }}</span>
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </div>

    <!-- Tab Panels -->
    <div v-if="settings" class="settings-body">
      <!-- General Tab -->
      <div
        v-show="activeTab === 'general'"
        id="panel-general"
        class="settings-panel-content"
        role="tabpanel"
        aria-labelledby="tab-general"
      >
        <SettingSection title="Startup" id="startup" icon="🚀">
          <SettingToggle
            v-model="settings.general.launch_at_startup"
            label="Launch at Startup"
            description="Automatically start the app when you log in"
          />
          <SettingToggle
            v-model="settings.general.hide_dock_icon"
            label="Hide Dock Icon"
            description="Hide the app icon from the dock (macOS only)"
          />
        </SettingSection>

        <SettingSection title="Updates" id="updates" icon="🔄">
          <SettingToggle
            v-model="settings.general.check_updates"
            label="Check for Updates"
            description="Automatically check for new versions"
          />
        </SettingSection>

        <SettingSection title="Search" id="search-settings" icon="🔍">
          <SettingNumber
            v-model="settings.search.max_results"
            label="Max Results"
            description="Maximum number of search results to display"
            :min="5"
            :max="50"
            width="small"
          />
          <SettingToggle
            v-model="settings.search.show_bookmarks"
            label="Show Bookmarks"
            description="Include bookmarks in search results"
          />
          <SettingToggle
            v-model="settings.search.show_files"
            label="Show Files"
            description="Include files in search results"
          />
          <SettingToggle
            v-model="settings.search.fuzzy_matching"
            label="Fuzzy Matching"
            description="Enable fuzzy search for more flexible matching"
          />
        </SettingSection>
      </div>

      <!-- Appearance Tab -->
      <div
        v-show="activeTab === 'appearance'"
        id="panel-appearance"
        class="settings-panel-content"
        role="tabpanel"
        aria-labelledby="tab-appearance"
      >
        <SettingSection title="Theme" id="theme" icon="🎨">
          <SettingSelect
            v-model="settings.theme.mode"
            label="Theme Mode"
            description="Choose your preferred color scheme"
            :options="themeOptions"
          />
        </SettingSection>

        <SettingSection title="Colors" id="colors" icon="🖌️">
          <div class="color-grid">
            <SettingColor
              v-model="settings.theme.accent_color"
              label="Accent"
              default-color="#ff6b6b"
            />
            <SettingColor
              v-model="settings.theme.bg_color"
              label="Background"
              :optional="true"
              default-color="#1a1a1a"
            />
            <SettingColor
              v-model="settings.theme.text_color"
              label="Text"
              :optional="true"
              default-color="#e0e0e0"
            />
            <SettingColor
              v-model="settings.theme.selection_bg_color"
              label="Selection"
              :optional="true"
              :default-color="settings.theme.accent_color"
            />
          </div>
        </SettingSection>

        <SettingSection title="Layout" id="layout" icon="📐">
          <div class="layout-grid">
            <SettingNumber
              v-model="settings.theme.font_size"
              label="Font Size"
              :min="10"
              :max="24"
              width="small"
            />
            <SettingNumber
              v-model="settings.theme.border_radius"
              label="Border Radius"
              :min="0"
              :max="24"
              width="small"
            />
          </div>
          <div class="layout-grid">
            <SettingNumber
              v-model="settings.theme.window_width"
              label="Window Width"
              :min="400"
              :max="1200"
              :step="10"
              width="medium"
            />
            <SettingNumber
              v-model="settings.theme.window_height"
              label="Window Height"
              :min="200"
              :max="900"
              :step="10"
              width="medium"
            />
          </div>
          <div class="layout-grid">
            <SettingNumber
              v-model="settings.theme.input_height"
              label="Input Height"
              :min="30"
              :max="80"
              width="small"
            />
            <SettingNumber
              v-model="settings.theme.item_height"
              label="Item Height"
              :min="20"
              :max="60"
              width="small"
            />
          </div>
        </SettingSection>
      </div>

      <!-- Shortcuts Tab -->
      <div
        v-show="activeTab === 'shortcuts'"
        id="panel-shortcuts"
        class="settings-panel-content"
        role="tabpanel"
        aria-labelledby="tab-shortcuts"
      >
        <SettingSection title="Global Hotkey" id="global-hotkey" icon="🌐">
          <div class="global-hotkey-field">
            <label for="global-shortcut" class="hotkey-label">Activation Shortcut</label>
            <p class="hotkey-description">Press this key combination to open the launcher from anywhere</p>
            <input
              id="global-shortcut"
              v-model="settings.hotkey.global_shortcut"
              class="hotkey-input"
              placeholder="Cmd+Space"
              aria-describedby="hotkey-help"
            />
            <p id="hotkey-help" class="hotkey-help">Examples: Cmd+Space, Alt+Space, Ctrl+Shift+P</p>
          </div>
        </SettingSection>

        <SettingSection title="UI Shortcuts" id="ui-shortcuts" icon="⌨️">
          <div class="shortcuts-list" role="list">
            <ShortcutEditor
              v-for="shortcut in shortcuts"
              :key="shortcut.id"
              :action-id="shortcut.id"
              :label="shortcut.label"
              :description="shortcut.description"
              :current-shortcut="settings.hotkey.ui_shortcuts[shortcut.id] || 'Not set'"
              @update="handleShortcutUpdate"
            />
          </div>
        </SettingSection>
      </div>

      <!-- Data Tab -->
      <div
        v-show="activeTab === 'data'"
        id="panel-data"
        class="settings-panel-content"
        role="tabpanel"
        aria-labelledby="tab-data"
      >
        <SettingSection title="Import Bookmarks" id="import" icon="📥">
          <p class="section-description">Import bookmarks from your browsers</p>
          <div class="import-buttons" role="group" aria-label="Import from browser">
            <button class="btn btn--secondary" @click="importBookmarks('chrome')" aria-label="Import from Chrome">
              <span aria-hidden="true">🌐</span> Chrome
            </button>
            <button class="btn btn--secondary" @click="importBookmarks('firefox')" aria-label="Import from Firefox">
              <span aria-hidden="true">🦊</span> Firefox
            </button>
            <button class="btn btn--secondary" @click="importBookmarks('safari')" aria-label="Import from Safari">
              <span aria-hidden="true">🧭</span> Safari
            </button>
          </div>
        </SettingSection>

        <SettingSection title="Settings Backup" id="backup" icon="💾">
          <p class="section-description">Export or import your settings configuration</p>
          <div class="backup-buttons">
            <button class="btn btn--secondary" @click="exportSettings">
              <span aria-hidden="true">📤</span> Export Settings
            </button>
            <button class="btn btn--secondary" @click="importSettingsFromFile">
              <span aria-hidden="true">📥</span> Import Settings
            </button>
          </div>
        </SettingSection>

        <SettingSection v-if="stats" title="Statistics" id="stats" icon="📊">
          <div class="stats-grid" role="list">
            <div class="stat" role="listitem">
              <span class="stat-value">{{ stats.bookmarks_count }}</span>
              <span class="stat-label">Bookmarks</span>
            </div>
            <div class="stat" role="listitem">
              <span class="stat-value">{{ stats.files_count }}</span>
              <span class="stat-label">Files</span>
            </div>
            <div class="stat" role="listitem">
              <span class="stat-value">{{ stats.directories_count }}</span>
              <span class="stat-label">Directories</span>
            </div>
          </div>
        </SettingSection>

        <SettingSection title="Danger Zone" id="danger" icon="⚠️">
          <p class="section-description danger-text">These actions cannot be undone</p>
          <button class="btn btn--danger" @click="resetSettings">
            Reset All Settings
          </button>
        </SettingSection>

        <!-- Status Message -->
        <p v-if="importStatus" class="status-message" role="status" aria-live="polite">
          {{ importStatus }}
        </p>
      </div>
    </div>

    <!-- Loading State -->
    <div v-else class="settings-loading" role="status" aria-live="polite">
      <div class="loading-spinner" aria-hidden="true"></div>
      <span>Loading settings...</span>
    </div>

    <!-- Footer -->
    <div class="settings-footer">
      <span v-if="saving" class="save-indicator" aria-live="polite">
        <span class="save-spinner" aria-hidden="true"></span>
        Saving...
      </span>
      <span v-else class="save-indicator save-indicator--saved">
        ✓ Auto-saved
      </span>
    </div>
  </div>
</template>

<style scoped>
.settings-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--color-bg-primary, rgba(26, 26, 26, 0.95));
}

/* Header */
.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border-default, #3a3a3a);
  flex-shrink: 0;
}

.settings-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary, #e0e0e0);
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  color: var(--color-text-tertiary, #a3a3a3);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.close-btn:hover {
  background: rgba(128, 128, 128, 0.1);
  color: var(--color-text-primary, #e0e0e0);
}

.close-btn:focus-visible {
  outline: 2px solid var(--color-accent, #ff6b6b);
  outline-offset: 2px;
}

/* Search */
.settings-search {
  position: relative;
  padding: 12px 20px;
  border-bottom: 1px solid var(--color-border-default, #3a3a3a);
  flex-shrink: 0;
}

.search-input {
  width: 100%;
  padding: 10px 14px;
  font-size: 14px;
  background: var(--color-bg-secondary, #262626);
  border: 1px solid var(--color-border-default, #3a3a3a);
  border-radius: 8px;
  color: var(--color-text-primary, #e0e0e0);
  outline: none;
  transition: border-color 0.15s ease;
}

.search-input:focus {
  border-color: var(--color-accent, #ff6b6b);
}

.search-input::placeholder {
  color: var(--color-text-tertiary, #a3a3a3);
}

.search-results {
  position: absolute;
  top: 100%;
  left: 20px;
  right: 20px;
  background: var(--color-bg-elevated, #262626);
  border: 1px solid var(--color-border-default, #3a3a3a);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  z-index: 10;
  max-height: 200px;
  overflow-y: auto;
}

.search-result {
  display: flex;
  flex-direction: column;
  width: 100%;
  padding: 10px 14px;
  background: none;
  border: none;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.1s ease;
}

.search-result:hover {
  background: rgba(128, 128, 128, 0.1);
}

.search-result__section {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--color-accent, #ff6b6b);
  margin-bottom: 2px;
}

.search-result__label {
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
  text-transform: capitalize;
}

/* Tabs */
.settings-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--color-border-default, #3a3a3a);
  flex-shrink: 0;
  overflow-x: auto;
}

.settings-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary, #b3b3b3);
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.settings-tab:hover {
  background: rgba(128, 128, 128, 0.1);
  color: var(--color-text-primary, #e0e0e0);
}

.settings-tab--active {
  background: var(--color-accent, #ff6b6b);
  color: white;
}

.settings-tab--active:hover {
  background: var(--color-accent, #ff6b6b);
  color: white;
}

.settings-tab:focus-visible {
  outline: 2px solid var(--color-accent, #ff6b6b);
  outline-offset: 2px;
}

.tab-icon {
  font-size: 14px;
}

/* Body */
.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.settings-panel-content {
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* Layout helpers */
.color-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.layout-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 8px;
}

/* Global hotkey field */
.global-hotkey-field {
  padding: 12px 0;
}

.hotkey-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary, #e0e0e0);
  margin-bottom: 4px;
}

.hotkey-description {
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
  margin-bottom: 12px;
}

.hotkey-input {
  width: 100%;
  max-width: 240px;
  padding: 10px 14px;
  font-size: 14px;
  font-family: monospace;
  background: var(--color-bg-secondary, #262626);
  border: 1px solid var(--color-border-default, #3a3a3a);
  border-radius: 6px;
  color: var(--color-text-primary, #e0e0e0);
  outline: none;
}

.hotkey-input:focus {
  border-color: var(--color-accent, #ff6b6b);
}

.hotkey-help {
  font-size: 11px;
  color: var(--color-text-tertiary, #a3a3a3);
  margin-top: 8px;
}

/* Shortcuts list */
.shortcuts-list {
  display: flex;
  flex-direction: column;
}

/* Section description */
.section-description {
  font-size: 13px;
  color: var(--color-text-secondary, #b3b3b3);
  margin-bottom: 12px;
}

.danger-text {
  color: var(--color-accent, #ff6b6b);
}

/* Buttons */
.import-buttons,
.backup-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 500;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn--secondary {
  background: var(--color-bg-secondary, #3a3a3a);
  color: var(--color-text-primary, #e0e0e0);
}

.btn--secondary:hover {
  background: #4a4a4a;
}

.btn--danger {
  background: transparent;
  color: var(--color-accent, #ff6b6b);
  border: 1px solid var(--color-accent, #ff6b6b);
}

.btn--danger:hover {
  background: rgba(255, 107, 107, 0.1);
}

.btn:focus-visible {
  outline: 2px solid var(--color-accent, #ff6b6b);
  outline-offset: 2px;
}

/* Stats */
.stats-grid {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 20px;
  background: var(--color-bg-secondary, #262626);
  border-radius: 8px;
  min-width: 80px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-accent, #ff6b6b);
}

.stat-label {
  font-size: 11px;
  color: var(--color-text-secondary, #b3b3b3);
  margin-top: 4px;
}

/* Status message */
.status-message {
  margin-top: 16px;
  padding: 10px 14px;
  font-size: 13px;
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  border-radius: 6px;
  color: #10b981;
}

/* Loading */
.settings-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  color: var(--color-text-secondary, #b3b3b3);
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 107, 107, 0.2);
  border-top-color: var(--color-accent, #ff6b6b);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Footer */
.settings-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--color-border-default, #3a3a3a);
  flex-shrink: 0;
}

.save-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
}

.save-indicator--saved {
  color: #10b981;
}

.save-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid rgba(255, 107, 107, 0.2);
  border-top-color: var(--color-accent, #ff6b6b);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

/* Light mode */
@media (prefers-color-scheme: light) {
  .settings-panel {
    background: var(--color-bg-primary, rgba(255, 255, 255, 0.95));
  }
  .settings-header,
  .settings-search,
  .settings-tabs,
  .settings-footer {
    border-color: var(--color-border-default, #e0e0e0);
  }
  .settings-title {
    color: var(--color-text-primary, #1a1a1a);
  }
  .search-input {
    background: var(--color-bg-secondary, #f5f5f5);
    border-color: var(--color-border-default, #e0e0e0);
    color: var(--color-text-primary, #1a1a1a);
  }
  .search-results {
    background: var(--color-bg-elevated, #ffffff);
    border-color: var(--color-border-default, #e0e0e0);
  }
  .search-result__label {
    color: var(--color-text-primary, #1a1a1a);
  }
  .settings-tab {
    color: var(--color-text-secondary, #525252);
  }
  .btn--secondary {
    background: var(--color-bg-secondary, #e0e0e0);
    color: var(--color-text-primary, #1a1a1a);
  }
  .stat {
    background: var(--color-bg-secondary, #f5f5f5);
  }
  .hotkey-input {
    background: var(--color-bg-secondary, #f5f5f5);
    border-color: var(--color-border-default, #e0e0e0);
    color: var(--color-text-primary, #1a1a1a);
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .settings-panel-content {
    animation: none;
  }
  .loading-spinner,
  .save-spinner {
    animation: none;
  }
  .settings-tab,
  .close-btn,
  .btn,
  .search-input {
    transition: none;
  }
}
</style>
