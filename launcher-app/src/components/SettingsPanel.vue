<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types/settings";
import ShortcutEditor from "./ShortcutEditor.vue";

interface DataStats {
  bookmarks_count: number;
  files_count: number;
  directories_count: number;
  settings_count: number;
}

const emit = defineEmits<{ (e: "close"): void }>();

const settings = ref<AppSettings | null>(null);
const stats = ref<DataStats | null>(null);
const saving = ref(false);
const importStatus = ref("");

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
  try {
    await invoke("reset_settings");
    await loadSettings();
  } catch (err) {
    console.error("Failed to reset settings:", err);
  }
}

function handleShortcutUpdate(actionId: string, shortcut: string) {
  if (!settings.value) return;
  settings.value.hotkey.ui_shortcuts[actionId] = shortcut;
}

const shortcuts = [
  { id: 'general.close', label: 'Close Window', description: 'Close the launcher or exit settings' },
  { id: 'general.settings', label: 'Open Settings', description: 'Toggle settings panel' },
  { id: 'search.next', label: 'Next Result', description: 'Select next search result' },
  { id: 'search.prev', label: 'Previous Result', description: 'Select previous search result' },
  { id: 'search.open', label: 'Open Result', description: 'Open the selected result' },
  { id: 'search.open_new_tab', label: 'Open in Background', description: '(Future) Open result in background tab' },
];

onMounted(loadSettings);
</script>

<template>
  <div class="settings-panel" role="dialog" aria-labelledby="settings-title" aria-modal="true">
    <div class="settings-header">
      <h2 id="settings-title" class="settings-title">Settings</h2>
      <button class="close-btn" @click="emit('close')" aria-label="Close settings panel">
        &times;
      </button>
    </div>

    <div v-if="settings" class="settings-body">
      <!-- Hotkey -->
      <section class="settings-section" role="group" aria-labelledby="hotkey-title">
        <h3 id="hotkey-title" class="section-title">Hotkey</h3>
        <label class="field">
          <span class="field-label">Global Shortcut</span>
          <input
            v-model="settings.hotkey.global_shortcut"
            class="field-input"
            placeholder="Cmd+Space"
            aria-label="Global shortcut key combination"
            aria-describedby="hotkey-desc"
          />
        </label>
        <p id="hotkey-desc" class="visually-hidden">Press keys to set global shortcut for opening the launcher</p>
      </section>

      <!-- Shortcuts -->
      <section class="settings-section" role="group" aria-labelledby="shortcuts-title">
        <h3 id="shortcuts-title" class="section-title">Keyboard Shortcuts</h3>
        <div v-if="settings" class="shortcuts-list" role="list">
          <ShortcutEditor
            v-for="shortcut in shortcuts"
            :key="shortcut.id"
            :action-id="shortcut.id"
            :label="shortcut.label"
            :description="shortcut.description"
            :current-shortcut="settings.hotkey.ui_shortcuts[shortcut.id] || 'Not set'"
            @update="handleShortcutUpdate"
            role="listitem"
          />
        </div>
      </section>

      <!-- Theme -->
      <section class="settings-section" role="group" aria-labelledby="theme-title">
        <h3 id="theme-title" class="section-title">Theme & Layout</h3>
          <label class="field">
            <span class="field-label">Mode</span>
            <select v-model="settings.theme.mode" class="field-input" aria-label="Theme mode">
              <option value="system">System</option>
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </label>
        
        <!-- Advanced Colors -->
        <div class="advanced-colors" role="group" aria-labelledby="colors-title">
            <h4 id="colors-title" class="subsection-title">Colors</h4>
            <div class="color-grid">
                <label class="color-field">
                    <span class="color-label">Accent</span>
                    <input v-model="settings.theme.accent_color" type="color" class="field-color" aria-label="Accent color" />
                </label>
                <label class="color-field">
                    <span class="color-label">Background</span>
                    <div class="color-input-wrapper">
                        <input type="checkbox" :checked="!!settings.theme.bg_color" @change="(e) => settings!.theme.bg_color = (e.target as HTMLInputElement).checked ? '#1a1a1a' : undefined" aria-label="Enable custom background color">
                        <input v-if="settings.theme.bg_color" v-model="settings.theme.bg_color" type="color" class="field-color" aria-label="Background color" />
                    </div>
                </label>
                <label class="color-field">
                    <span class="color-label">Text</span>
                    <div class="color-input-wrapper">
                        <input type="checkbox" :checked="!!settings.theme.text_color" @change="(e) => settings!.theme.text_color = (e.target as HTMLInputElement).checked ? '#e0e0e0' : undefined" aria-label="Enable custom text color">
                        <input v-if="settings.theme.text_color" v-model="settings.theme.text_color" type="color" class="field-color" aria-label="Text color" />
                    </div>
                </label>
                <label class="color-field">
                    <span class="color-label">Selection</span>
                    <div class="color-input-wrapper">
                         <input type="checkbox" :checked="!!settings.theme.selection_bg_color" @change="(e) => settings!.theme.selection_bg_color = (e.target as HTMLInputElement).checked ? settings!.theme.accent_color : undefined" aria-label="Enable custom selection color">
                         <input v-if="settings.theme.selection_bg_color" v-model="settings.theme.selection_bg_color" type="color" class="field-color" aria-label="Selection background color" />
                    </div>
                </label>
            </div>
        </div>
        
        <div class="field-group">
            <label class="field">
            <span class="field-label">Font Size</span>
            <input
                v-model.number="settings.theme.font_size"
                type="number"
                min="10"
                max="24"
                class="field-input field-input--small"
                aria-label="Font size in pixels"
                aria-valuemin="10"
                aria-valuemax="24"
            />
            </label>
            <label class="field">
            <span class="field-label">Border Radius</span>
            <input
                v-model.number="settings.theme.border_radius"
                type="number"
                min="0"
                max="24"
                class="field-input field-input--small"
                aria-label="Border radius in pixels"
                aria-valuemin="0"
                aria-valuemax="24"
            />
            </label>
        </div>

        <div class="field-group">
            <label class="field">
            <span class="field-label">Window Width</span>
            <input
                v-model.number="settings.theme.window_width"
                type="number"
                min="400"
                max="1200"
                step="10"
                class="field-input field-input--small"
                aria-label="Window width in pixels"
                aria-valuemin="400"
                aria-valuemax="1200"
            />
            </label>
            <label class="field">
            <span class="field-label">Window Height</span>
            <input
                v-model.number="settings.theme.window_height"
                type="number"
                min="200"
                max="900"
                step="10"
                class="field-input field-input--small"
                aria-label="Window height in pixels"
                aria-valuemin="200"
                aria-valuemax="900"
            />
            </label>
        </div>

        <div class="field-group">
            <label class="field">
            <span class="field-label">Input Height</span>
            <input
                v-model.number="settings.theme.input_height"
                type="number"
                min="30"
                max="80"
                class="field-input field-input--small"
                aria-label="Search input height in pixels"
                aria-valuemin="30"
                aria-valuemax="80"
            />
            </label>
            <label class="field">
            <span class="field-label">Item Height</span>
            <input
                v-model.number="settings.theme.item_height"
                type="number"
                min="20"
                max="60"
                class="field-input field-input--small"
                aria-label="Result item height in pixels"
                aria-valuemin="20"
                aria-valuemax="60"
            />
            </label>
        </div>
      </section>

      <!-- Search -->
      <section class="settings-section" role="group" aria-labelledby="search-title">
        <h3 id="search-title" class="section-title">Search</h3>
        <label class="field">
          <span class="field-label">Max Results</span>
          <input
            v-model.number="settings.search.max_results"
            type="number"
            min="5"
            max="50"
            class="field-input field-input--small"
            aria-label="Maximum number of search results"
            aria-valuemin="5"
            aria-valuemax="50"
          />
        </label>
        <label class="field field--toggle">
          <span class="field-label">Show Bookmarks</span>
          <input v-model="settings.search.show_bookmarks" type="checkbox" aria-label="Include bookmarks in search results" />
        </label>
        <label class="field field--toggle">
          <span class="field-label">Show Files</span>
          <input v-model="settings.search.show_files" type="checkbox" aria-label="Include files in search results" />
        </label>
        <label class="field field--toggle">
          <span class="field-label">Fuzzy Matching</span>
          <input v-model="settings.search.fuzzy_matching" type="checkbox" aria-label="Enable fuzzy search matching" />
        </label>
      </section>

      <!-- General -->
      <section class="settings-section" role="group" aria-labelledby="general-title">
        <h3 id="general-title" class="section-title">General</h3>
        <label class="field field--toggle">
          <span class="field-label">Launch at Startup</span>
          <input v-model="settings.general.launch_at_startup" type="checkbox" aria-label="Launch application at system startup" />
        </label>
        <label class="field field--toggle">
          <span class="field-label">Hide Dock Icon</span>
          <input v-model="settings.general.hide_dock_icon" type="checkbox" aria-label="Hide dock icon (macOS only)" />
        </label>
        <label class="field field--toggle">
          <span class="field-label">Check for Updates</span>
          <input v-model="settings.general.check_updates" type="checkbox" aria-label="Automatically check for updates" />
        </label>
      </section>

      <!-- Import -->
      <section class="settings-section" role="group" aria-labelledby="import-title">
        <h3 id="import-title" class="section-title">Import Bookmarks</h3>
        <div class="import-buttons" role="group" aria-label="Import from browser">
          <button class="btn btn--secondary" @click="importBookmarks('chrome')" aria-label="Import bookmarks from Chrome">Chrome</button>
          <button class="btn btn--secondary" @click="importBookmarks('firefox')" aria-label="Import bookmarks from Firefox">Firefox</button>
          <button class="btn btn--secondary" @click="importBookmarks('safari')" aria-label="Import bookmarks from Safari">Safari</button>
        </div>
        <p v-if="importStatus" class="import-status" role="status" aria-live="polite">{{ importStatus }}</p>
      </section>

      <!-- Stats -->
      <section v-if="stats" class="settings-section" role="group" aria-labelledby="data-title">
        <h3 id="data-title" class="section-title">Data</h3>
        <div class="stats-grid" role="list">
          <div class="stat" role="listitem">
            <span class="stat-value" aria-label="Number of bookmarks">{{ stats.bookmarks_count }}</span>
            <span class="stat-label">Bookmarks</span>
          </div>
          <div class="stat" role="listitem">
            <span class="stat-value" aria-label="Number of files">{{ stats.files_count }}</span>
            <span class="stat-label">Files</span>
          </div>
          <div class="stat" role="listitem">
            <span class="stat-value" aria-label="Number of directories">{{ stats.directories_count }}</span>
            <span class="stat-label">Directories</span>
          </div>
        </div>
      </section>

      <!-- Actions -->
      <section class="settings-actions">
        <button
          class="btn btn--primary"
          :disabled="saving"
          @click="saveSettings"
          :aria-label="saving ? 'Saving settings' : 'Save settings'"
          :aria-busy="saving"
        >
          {{ saving ? "Saving..." : "Save Settings" }}
        </button>
        <button class="btn btn--danger" @click="resetSettings" aria-label="Reset all settings to default values">Reset to Defaults</button>
      </section>
    </div>

    <div v-else class="settings-loading" role="status" aria-live="polite">Loading settings...</div>
  </div>
</template>

<style scoped>
.settings-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid #3a3a3a;
}

.settings-title {
  font-size: 18px;
  font-weight: 600;
  color: #e0e0e0;
}

.close-btn {
  background: none;
  border: none;
  color: var(--color-text-tertiary, #a3a3a3);
  font-size: 24px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}
.close-btn:hover {
  color: var(--color-text-primary, #e0e0e0);
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.settings-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary, #b3b3b3);
  margin-bottom: 12px;
}

.field-group {
  display: flex;
  gap: 16px;
}

.field-group .field {
  flex: 1;
}

.field {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.field-label {
  font-size: 14px;
  color: #e0e0e0;
}

.field-input {
  width: 200px;
  padding: 6px 10px;
  font-size: 13px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  color: #e0e0e0;
  outline: none;
}
.field-input:focus {
  border-color: #ff6b6b;
}

.field-input--small {
  width: 80px;
}

.field-color {
  width: 40px;
  height: 28px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  background: none;
  padding: 0;
}

.advanced-colors {
    margin: 16px 0;
    padding: 12px;
    background: rgba(0,0,0,0.1);
    border-radius: 8px;
}

.subsection-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-secondary, #b3b3b3);
    margin-bottom: 8px;
}

.color-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
}

.color-field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    color: #e0e0e0;
}

.color-input-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
}

.field--toggle {
  cursor: pointer;
}

.import-buttons {
  display: flex;
  gap: 8px;
}

.import-status {
  margin-top: 8px;
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
}

.stats-grid {
  display: flex;
  gap: 16px;
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 16px;
  background: #2a2a2a;
  border-radius: 6px;
  flex: 1;
}

.stat-value {
  font-size: 20px;
  font-weight: 600;
  color: #ff6b6b;
}

.stat-label {
  font-size: 11px;
  color: var(--color-text-secondary, #b3b3b3);
  margin-top: 4px;
}

.settings-actions {
  display: flex;
  gap: 8px;
  padding-top: 16px;
  border-top: 1px solid #3a3a3a;
}

.btn {
  padding: 8px 16px;
  font-size: 13px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.btn--primary {
  background: #ff6b6b;
  color: #fff;
}
.btn--primary:hover {
  background: #ff5252;
}
.btn--primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn--secondary {
  background: #3a3a3a;
  color: #e0e0e0;
}
.btn--secondary:hover {
  background: #4a4a4a;
}

.btn--danger {
  background: transparent;
  color: #ff6b6b;
  border: 1px solid #ff6b6b;
}
.btn--danger:hover {
  background: rgba(255, 107, 107, 0.1);
}

.settings-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-secondary, #b3b3b3);
}

/* Visually hidden class for screen readers */
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}

@media (prefers-color-scheme: light) {
  .settings-header {
    border-bottom-color: var(--color-border-default, #e0e0e0);
  }
  .settings-title {
    color: var(--color-text-primary, #1a1a1a);
  }
  .close-btn {
    color: var(--color-text-tertiary, #6b6b6b);
  }
  .close-btn:hover {
    color: var(--color-text-primary, #1a1a1a);
  }
  .field-label {
    color: var(--color-text-primary, #1a1a1a);
  }
  .field-input {
    background: var(--color-bg-secondary, #f5f5f5);
    border-color: var(--color-border-default, #e0e0e0);
    color: var(--color-text-primary, #1a1a1a);
  }
  .stat {
    background: var(--color-bg-secondary, #f5f5f5);
  }
  .section-title {
    color: var(--color-text-secondary, #525252);
  }
  .subsection-title {
    color: var(--color-text-secondary, #525252);
  }
  .btn--secondary {
    background: var(--color-bg-secondary, #e0e0e0);
    color: var(--color-text-primary, #1a1a1a);
  }
  .settings-actions {
    border-top-color: var(--color-border-default, #e0e0e0);
  }
  .import-status,
  .stat-label,
  .settings-loading {
    color: var(--color-text-secondary, #525252);
  }
}
</style>
