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
import { TabGroup, TabList, Tab, TabPanels, TabPanel } from '@headlessui/vue';
import { useAppSettings, tabs, type TabId } from "../../composables/useAppSettings";
import SettingsTabGeneral from "./SettingsTabGeneral.vue";
import SettingsTabAppearance from "./SettingsTabAppearance.vue";
import SettingsTabShortcuts from "./SettingsTabShortcuts.vue";
import SettingsTabPlugins from "./SettingsTabPlugins.vue";
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

const activeTab = ref<number>(0);
const searchQuery = ref("");

// Search filtering
const filteredSettings = computed(() => {
  if (!searchQuery.value.trim()) return null;
  const query = searchQuery.value.toLowerCase();
  const matches: { tabIndex: number; section: string; label: string }[] = [];

  const searchItems: { tabIndex: number; section: string; items: string[] }[] = [
    { tabIndex: 0, section: 'General', items: ['launch at startup', 'hide dock icon', 'check for updates', 'max results', 'fuzzy matching'] },
    { tabIndex: 1, section: 'Appearance', items: ['theme mode', 'font size', 'border radius', 'window width', 'max window height', 'accent color'] },
    { tabIndex: 2, section: 'Shortcuts', items: ['global hotkey', 'close window', 'next result', 'previous result', 'open result'] },
    { tabIndex: 3, section: 'Plugins', items: ['installed plugins', 'enable plugin', 'disable plugin', 'plugin preferences', 'uninstall plugin'] },
    { tabIndex: 4, section: 'Data', items: ['import bookmarks', 'export settings', 'statistics', 'reset settings'] },
  ];

  searchItems.forEach(({ tabIndex, section, items }) => {
    items.forEach(item => {
      if (item.includes(query)) {
        matches.push({ tabIndex, section, label: item });
      }
    });
  });

  return matches.length > 0 ? matches : null;
});

const hasSearchResults = computed(() => filteredSettings.value !== null);

function jumpToSetting(tabIndex: number) {
  activeTab.value = tabIndex;
  searchQuery.value = "";
}

onMounted(() => {
  loadSettings();
  setupAutoSave();
});
</script>

<template>
  <div class="flex flex-col w-full h-full overflow-hidden bg-bg-primary text-text-primary" role="dialog" aria-labelledby="settings-title" aria-modal="true">
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-4 border-b border-border-default shrink-0">
      <h2 id="settings-title" class="text-lg font-semibold text-text-primary">Settings</h2>
      <button 
        class="flex items-center justify-center w-8 h-8 rounded-md text-text-tertiary hover:bg-hover-bg hover:text-text-primary transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent" 
        @click="emit('close')" 
        aria-label="Close settings panel"
      >
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
          <path d="M15 5L5 15M5 5l10 10" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <!-- Search -->
    <div class="relative px-5 py-3 border-b border-border-default shrink-0">
      <input 
        v-model="searchQuery" 
        type="search" 
        class="w-full px-3.5 py-2.5 text-sm bg-bg-secondary border border-border-default rounded-lg text-text-primary placeholder-text-tertiary focus:outline-none focus:border-accent transition-colors" 
        placeholder="Search settings..." 
        aria-label="Search settings" 
      />
      <div v-if="hasSearchResults" class="absolute top-full left-5 right-5 mt-1 bg-bg-elevated border border-border-default rounded-lg shadow-lg z-10 max-h-[200px] overflow-y-auto" role="listbox" aria-label="Search results">
        <button 
          v-for="result in filteredSettings" 
          :key="`${result.tabIndex}-${result.label}`" 
          class="flex flex-col w-full px-3.5 py-2.5 text-left hover:bg-hover-bg transition-colors" 
          role="option" 
          @click="jumpToSetting(result.tabIndex)"
        >
          <span class="text-[10px] font-semibold uppercase text-accent mb-0.5">{{ result.section }}</span>
          <span class="text-sm text-text-primary capitalize">{{ result.label }}</span>
        </button>
      </div>
    </div>

    <!-- Tabs & Content -->
    <TabGroup :selectedIndex="activeTab" @change="(index) => activeTab = index" class="flex flex-col flex-1 min-h-0">
      <TabList class="flex gap-1 px-5 py-3 border-b border-border-default shrink-0 overflow-x-auto">
        <Tab
          v-for="tab in tabs"
          :key="tab.id"
          as="template"
          v-slot="{ selected }"
        >
          <button
            class="flex items-center gap-1.5 px-3.5 py-2 text-sm font-medium rounded-md transition-colors whitespace-nowrap focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
            :class="[
              selected 
                ? 'bg-accent text-white shadow-sm' 
                : 'text-text-secondary hover:bg-hover-bg hover:text-text-primary'
            ]"
          >
            <span class="text-sm" aria-hidden="true">{{ tab.icon }}</span>
            <span>{{ tab.label }}</span>
          </button>
        </Tab>
      </TabList>

      <!-- Tab Panels -->
      <div v-if="settings" class="flex-1 overflow-y-auto p-5">
        <TabPanels class="h-full">
          <TabPanel class="h-full focus:outline-none animate-in fade-in zoom-in-95 duration-150">
            <SettingsTabGeneral :settings="settings" />
          </TabPanel>
          <TabPanel class="h-full focus:outline-none animate-in fade-in zoom-in-95 duration-150">
            <SettingsTabAppearance :settings="settings" />
          </TabPanel>
          <TabPanel class="h-full focus:outline-none animate-in fade-in zoom-in-95 duration-150">
            <SettingsTabShortcuts :settings="settings" @shortcut-update="handleShortcutUpdate" />
          </TabPanel>
          <TabPanel class="h-full focus:outline-none animate-in fade-in zoom-in-95 duration-150">
            <SettingsTabPlugins />
          </TabPanel>
          <TabPanel class="h-full focus:outline-none animate-in fade-in zoom-in-95 duration-150">
            <SettingsTabData :stats="stats" :import-status="importStatus" @import-bookmarks="importBookmarks" @export-settings="exportSettings" @import-settings="importSettingsFromFile" @reset-settings="resetSettings" />
          </TabPanel>
        </TabPanels>
      </div>

      <!-- Loading State -->
      <div v-else class="flex flex-col items-center justify-center flex-1 gap-3 text-text-secondary">
        <div class="w-6 h-6 border-2 border-border-subtle border-t-accent rounded-full animate-spin" aria-hidden="true"></div>
        <span>Loading settings...</span>
      </div>
    </TabGroup>

    <!-- Footer -->
    <div class="flex justify-end px-5 py-3 border-t border-border-default shrink-0">
      <span v-if="saving" class="flex items-center gap-1.5 text-xs text-text-secondary" aria-live="polite">
        <span class="w-3 h-3 border-2 border-border-subtle border-t-accent rounded-full animate-spin" aria-hidden="true"></span> Saving...
      </span>
      <span v-else class="flex items-center gap-1.5 text-xs text-success">Auto-saved</span>
    </div>
  </div>
</template>

<style scoped>
/* Scoped styles removed in favor of Tailwind CSS classes */
/* Animation utilities */
.animate-in {
  animation-fill-mode: both;
}
.fade-in {
  animation-name: fadeIn;
}
.zoom-in-95 {
  --tw-enter-scale: 0.95;
  --tw-enter-opacity: 0;
  animation-name: enter;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes enter {
  from {
    opacity: var(--tw-enter-opacity, 1);
    transform: scale(var(--tw-enter-scale, 1));
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
