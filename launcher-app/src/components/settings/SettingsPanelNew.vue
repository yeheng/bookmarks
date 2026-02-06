<script setup lang="ts">
/**
 * SettingsPanel - Main settings panel with tabbed navigation
 *
 * This component serves as a layout container, delegating content to:
 * - SettingsTabGeneral, SettingsTabAppearance, SettingsTabShortcuts, SettingsTabData
 *
 * State management is handled by useAppSettings composable.
 */
import { ref, computed, onMounted } from "vue";
import { useAppSettings, tabs, type TabId } from "../../composables/useAppSettings";
import SettingsTabGeneral from "./SettingsTabGeneral.vue";
import SettingsTabAppearance from "./SettingsTabAppearance.vue";
import SettingsTabShortcuts from "./SettingsTabShortcuts.vue";
import SettingsTabData from "./SettingsTabData.vue";

// Events
const emit = defineEmits<{ (e: "close"): void }>();

// State from composable
const {
  settings,
  stats,
  saving,
  importStatus,
  loadSettings,
  importBookmarks,
  resetSettings,
  exportSettings,
  importSettingsFromFile,
  handleShortcutUpdate,
  setupAutoSave,
} = useAppSettings();

const activeTab = ref<TabId>('general');
const searchQuery = ref("");

// Search filtering
const filteredSettings = computed(() => {
  if (!searchQuery.value.trim()) return null;
  const query = searchQuery.value.toLowerCase();
  const matches: { tab: TabId; section: string; label: string }[] = [];

  const searchItems: { tab: TabId; section: string; items: string[] }[] = [
    { tab: 'general', section: 'General', items: ['launch at startup', 'hide dock icon', 'check for updates', 'max results', 'fuzzy matching'] },
    { tab: 'appearance', section: 'Appearance', items: ['theme mode', 'font size', 'border radius', 'window width', 'window height', 'accent color'] },
    { tab: 'shortcuts', section: 'Shortcuts', items: ['global hotkey', 'close window', 'next result', 'previous result', 'open result'] },
    { tab: 'data', section: 'Data', items: ['import bookmarks', 'export settings', 'statistics', 'reset settings'] },
  ];

  searchItems.forEach(({ tab, section, items }) => {
    items.forEach(item => {
      if (item.includes(query)) {
        matches.push({ tab, section, label: item });
      }
    });
  });

  return matches.length > 0 ? matches : null;
});

const hasSearchResults = computed(() => filteredSettings.value !== null);

function jumpToSetting(tab: TabId) {
  activeTab.value = tab;
  searchQuery.value = "";
}

function handleTabKeydown(e: KeyboardEvent, index: number) {
  let newIndex = index;
  if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
    e.preventDefault();
    newIndex = (index + 1) % tabs.length;
  } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
    e.preventDefault();
    newIndex = (index - 1 + tabs.length) % tabs.length;
  }
  if (newIndex !== index) {
    activeTab.value = tabs[newIndex].id;
    const tabButtons = document.querySelectorAll('[role="tab"]');
    (tabButtons[newIndex] as HTMLElement)?.focus();
  }
}

onMounted(() => {
  loadSettings();
  setupAutoSave();
});
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
      <input v-model="searchQuery" type="search" class="search-input" placeholder="Search settings..." aria-label="Search settings" />
      <div v-if="hasSearchResults" class="search-results" role="listbox" aria-label="Search results">
        <button v-for="result in filteredSettings" :key="`${result.tab}-${result.label}`" class="search-result" role="option" @click="jumpToSetting(result.tab)">
          <span class="search-result__section">{{ result.section }}</span>
          <span class="search-result__label">{{ result.label }}</span>
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="settings-tabs" role="tablist" aria-label="Settings categories">
      <button v-for="(tab, index) in tabs" :key="tab.id" :id="`tab-${tab.id}`" class="settings-tab" :class="{ 'settings-tab--active': activeTab === tab.id }" role="tab" :aria-selected="activeTab === tab.id" :aria-controls="`panel-${tab.id}`" :tabindex="activeTab === tab.id ? 0 : -1" @click="activeTab = tab.id" @keydown="handleTabKeydown($event, index)">
        <span class="tab-icon" aria-hidden="true">{{ tab.icon }}</span>
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </div>

    <!-- Tab Panels -->
    <div v-if="settings" class="settings-body">
      <div v-show="activeTab === 'general'" id="panel-general" role="tabpanel" aria-labelledby="tab-general">
        <SettingsTabGeneral :settings="settings" />
      </div>
      <div v-show="activeTab === 'appearance'" id="panel-appearance" role="tabpanel" aria-labelledby="tab-appearance">
        <SettingsTabAppearance :settings="settings" />
      </div>
      <div v-show="activeTab === 'shortcuts'" id="panel-shortcuts" role="tabpanel" aria-labelledby="tab-shortcuts">
        <SettingsTabShortcuts :settings="settings" @shortcut-update="handleShortcutUpdate" />
      </div>
      <div v-show="activeTab === 'data'" id="panel-data" role="tabpanel" aria-labelledby="tab-data">
        <SettingsTabData :stats="stats" :import-status="importStatus" @import-bookmarks="importBookmarks" @export-settings="exportSettings" @import-settings="importSettingsFromFile" @reset-settings="resetSettings" />
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
        <span class="save-spinner" aria-hidden="true"></span> Saving...
      </span>
      <span v-else class="save-indicator save-indicator--saved">Auto-saved</span>
    </div>
  </div>
</template>

<style scoped>
.settings-panel { width: 100%; height: 100%; display: flex; flex-direction: column; overflow: hidden; background: var(--color-bg-primary, rgba(26, 26, 26, 0.95)); }
.settings-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; border-bottom: 1px solid var(--color-border-default, #3a3a3a); flex-shrink: 0; }
.settings-title { font-size: 18px; font-weight: 600; color: var(--color-text-primary, #e0e0e0); }
.close-btn { display: flex; align-items: center; justify-content: center; width: 32px; height: 32px; background: none; border: none; color: var(--color-text-tertiary, #a3a3a3); cursor: pointer; border-radius: 6px; transition: all 0.15s ease; }
.close-btn:hover { background: rgba(128, 128, 128, 0.1); color: var(--color-text-primary, #e0e0e0); }
.close-btn:focus-visible { outline: 2px solid var(--color-accent, #ff6b6b); outline-offset: 2px; }
.settings-search { position: relative; padding: 12px 20px; border-bottom: 1px solid var(--color-border-default, #3a3a3a); flex-shrink: 0; }
.search-input { width: 100%; padding: 10px 14px; font-size: 14px; background: var(--color-bg-secondary, #262626); border: 1px solid var(--color-border-default, #3a3a3a); border-radius: 8px; color: var(--color-text-primary, #e0e0e0); outline: none; transition: border-color 0.15s ease; }
.search-input:focus { border-color: var(--color-accent, #ff6b6b); }
.search-input::placeholder { color: var(--color-text-tertiary, #a3a3a3); }
.search-results { position: absolute; top: 100%; left: 20px; right: 20px; background: var(--color-bg-elevated, #262626); border: 1px solid var(--color-border-default, #3a3a3a); border-radius: 8px; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2); z-index: 10; max-height: 200px; overflow-y: auto; }
.search-result { display: flex; flex-direction: column; width: 100%; padding: 10px 14px; background: none; border: none; text-align: left; cursor: pointer; transition: background-color 0.1s ease; }
.search-result:hover { background: rgba(128, 128, 128, 0.1); }
.search-result__section { font-size: 10px; font-weight: 600; text-transform: uppercase; color: var(--color-accent, #ff6b6b); margin-bottom: 2px; }
.search-result__label { font-size: 13px; color: var(--color-text-primary, #e0e0e0); text-transform: capitalize; }
.settings-tabs { display: flex; gap: 4px; padding: 12px 20px; border-bottom: 1px solid var(--color-border-default, #3a3a3a); flex-shrink: 0; overflow-x: auto; }
.settings-tab { display: flex; align-items: center; gap: 6px; padding: 8px 14px; font-size: 13px; font-weight: 500; color: var(--color-text-secondary, #b3b3b3); background: none; border: none; border-radius: 6px; cursor: pointer; transition: all 0.15s ease; white-space: nowrap; }
.settings-tab:hover { background: rgba(128, 128, 128, 0.1); color: var(--color-text-primary, #e0e0e0); }
.settings-tab--active { background: var(--color-accent, #ff6b6b); color: white; }
.settings-tab--active:hover { background: var(--color-accent, #ff6b6b); color: white; }
.settings-tab:focus-visible { outline: 2px solid var(--color-accent, #ff6b6b); outline-offset: 2px; }
.tab-icon { font-size: 14px; }
.settings-body { flex: 1; overflow-y: auto; padding: 20px; }
.settings-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 12px; color: var(--color-text-secondary, #b3b3b3); }
.loading-spinner { width: 24px; height: 24px; border: 2px solid rgba(255, 107, 107, 0.2); border-top-color: var(--color-accent, #ff6b6b); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.settings-footer { display: flex; justify-content: flex-end; padding: 12px 20px; border-top: 1px solid var(--color-border-default, #3a3a3a); flex-shrink: 0; }
.save-indicator { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--color-text-secondary, #b3b3b3); }
.save-indicator--saved { color: #10b981; }
.save-spinner { width: 12px; height: 12px; border: 2px solid rgba(255, 107, 107, 0.2); border-top-color: var(--color-accent, #ff6b6b); border-radius: 50%; animation: spin 0.8s linear infinite; }
@media (prefers-color-scheme: light) {
  .settings-panel { background: var(--color-bg-primary, rgba(255, 255, 255, 0.95)); }
  .settings-header, .settings-search, .settings-tabs, .settings-footer { border-color: var(--color-border-default, #e0e0e0); }
  .settings-title { color: var(--color-text-primary, #1a1a1a); }
  .search-input { background: var(--color-bg-secondary, #f5f5f5); border-color: var(--color-border-default, #e0e0e0); color: var(--color-text-primary, #1a1a1a); }
  .search-results { background: var(--color-bg-elevated, #ffffff); border-color: var(--color-border-default, #e0e0e0); }
  .search-result__label { color: var(--color-text-primary, #1a1a1a); }
  .settings-tab { color: var(--color-text-secondary, #525252); }
}
@media (prefers-reduced-motion: reduce) {
  .loading-spinner, .save-spinner { animation: none; }
  .settings-tab, .close-btn, .search-input { transition: none; }
}
</style>
