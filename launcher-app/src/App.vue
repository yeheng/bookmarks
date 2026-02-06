<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from '@tauri-apps/api/event';
import { LogicalSize } from "@tauri-apps/api/dpi";
import { Toaster } from 'vue-sonner';
import SearchCombobox from "./components/SearchCombobox.vue";
import SettingsPanel from "./components/settings/SettingsPanel.vue";
import ErrorBoundary from "./components/ErrorBoundary.vue";
import { ShortcutManager } from "./services/shortcuts";
import { useToast } from "./composables/useToast";
import type {
  SearchResult,
  BookmarkSearchResult,
  FileSearchResult,
  OpenResult,
} from "./types/search";
import type { AppSettings } from "./types/settings";

const appWindow = getCurrentWebviewWindow();
const searchResults = ref<SearchResult[]>([]);
const searchComboboxRef = ref<InstanceType<typeof SearchCombobox> | null>(null);
const isLoading = ref(false);
const searchError = ref<string | null>(null);
const showSettings = ref(false);
const settings = ref<AppSettings | null>(null);
const shortcutManager = ref<ShortcutManager>(new ShortcutManager());
const { success, error } = useToast();

const themeStyle = computed(() => {
  if (!settings.value) return {};
  const { theme } = settings.value;
  const isLight = theme.mode === "light";

  return {
    "--accent-color": theme.accent_color,
    "--font-size": `${theme.font_size}px`,
    "--window-width": `${theme.window_width}px`,
    "--window-height": `${theme.window_height}px`,
    "--input-height": `${theme.input_height}px`,
    "--item-height": `${theme.item_height}px`,
    "--border-radius": `${theme.border_radius}px`,
    "--bg-color": theme.bg_color || (isLight ? "rgba(255,255,255, 0.95)" : "rgba(26, 26, 26, 0.95)"),
    "--text-color": theme.text_color || (isLight ? "#1a1a1a" : "#e0e0e0"),
    "--secondary-text": theme.secondary_text_color || (isLight ? "#6a6a6a" : "#9a9a9a"),
    "--border-color": theme.border_color || (isLight ? "#e0e0e0" : "#3a3a3a"),
    "--selection-bg": theme.selection_bg_color || theme.accent_color,
    "--selection-text": theme.selection_text_color || "#ffffff",
    "--color-bg-primary": theme.bg_color || (isLight ? "rgba(255,255,255, 0.95)" : "rgba(26, 26, 26, 0.95)"),
    "--color-text-primary": theme.text_color || (isLight ? "#1a1a1a" : "#e0e0e0"),
    "--color-text-secondary": theme.secondary_text_color || (isLight ? "#6a6a6a" : "#9a9a9a"),
    "--color-text-tertiary": isLight ? "#6b6b6b" : "#a3a3a3",
    "--color-border-default": theme.border_color || (isLight ? "#e0e0e0" : "#3a3a3a"),
    "--color-border-subtle": isLight ? "#f5f5f5" : "#262626",
    "--color-interactive-hover": isLight ? "rgba(0, 0, 0, 0.04)" : "rgba(255, 255, 255, 0.08)",
    "--color-interactive-active": isLight ? "rgba(0, 0, 0, 0.08)" : "rgba(255, 255, 255, 0.12)",
    "--color-highlight-bg": isLight ? "rgba(0, 0, 0, 0.05)" : "rgba(0, 0, 0, 0.05)",
    "--color-highlight-border": isLight ? "rgba(0, 0, 0, 0.08)" : "rgba(255, 255, 255, 0.08)",
  };
});

// Watch window size changes
watch(
  () => [settings.value?.theme.window_width, settings.value?.theme.window_height],
  async ([w, h]) => {
    if (w && h) {
      await appWindow.setSize(new LogicalSize(w as number, h as number));
    }
  }
);

async function loadSettings() {
  try {
    settings.value = await invoke<AppSettings>("get_app_settings");
    // Update shortcut manager with new settings
    if (settings.value) {
      shortcutManager.value = new ShortcutManager(settings.value.hotkey);
      // Initial resize
      const { window_width, window_height } = settings.value.theme;
      await appWindow.setSize(new LogicalSize(window_width, window_height));
    }
  } catch (err) {
    console.error("Failed to load settings:", err);
  }
}

function mapBookmarkToSearchResult(b: BookmarkSearchResult): SearchResult {
  const metadata: { domain?: string } = {};
  try {
    const url = new URL(b.url);
    metadata.domain = url.hostname;
  } catch {}

  return {
    id: `bookmark-${b.id}`,
    type: "bookmark",
    title: b.title,
    subtitle: b.url,
    icon: b.favicon_url ?? undefined,
    url: b.url,
    frecency_score: b.frecency_score,
    match_score: b.score,
    metadata: Object.keys(metadata).length > 0 ? metadata : undefined,
  };
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
}

function mapFileToSearchResult(f: FileSearchResult): SearchResult {
  return {
    id: `file-${f.id}`,
    type: "file",
    title: f.name,
    subtitle: f.path,
    path: f.path,
    frecency_score: f.frecency_score,
    match_score: f.score,
    metadata: {
      size: formatFileSize(f.size),
      modified: formatDate(f.modified_at),
    },
  };
}

const handleSearch = async (query: string) => {
  // Clear previous error
  searchError.value = null;

  if (!query.trim()) {
    searchResults.value = [];
    return;
  }

  isLoading.value = true;

  try {
    // Check if query triggers settings
    if (query.toLowerCase() === "settings" || query.toLowerCase() === "config") {
       searchResults.value = [{
           id: "internal-settings",
           type: "file", // fallback type
           title: "Open Settings",
           subtitle: "Configure theme, search, and general options",
           icon: "⚙️"
       }];
       isLoading.value = false;
       return;
    }

    const [bookmarks, files] = await Promise.all([
      invoke<BookmarkSearchResult[]>("search_bookmarks", { query, limit: 5 }),
      invoke<FileSearchResult[]>("search_files", { query, limit: 5 }),
    ]);

    const bookmarkResults = bookmarks.map(mapBookmarkToSearchResult);
    const fileResults = files.map(mapFileToSearchResult);

    searchResults.value = [...bookmarkResults, ...fileResults];
  } catch (err) {
    console.error("Search failed:", err);
    searchResults.value = [];
    searchError.value = err instanceof Error ? err.message : "Unable to connect to search service. Please try again.";
  } finally {
    isLoading.value = false;
  }
};

const handleRetry = () => {
  searchError.value = null;
};

const handleSelect = async (result: SearchResult) => {
  if (result.id === "internal-settings") {
      showSettings.value = true;
      return;
  }

  try {
    const resourceId = parseInt(result.id.split("-")[1], 10);
    const res = await invoke<OpenResult>("open_resource", {
      resourceType: result.type,
      resourceId: resourceId,
    });

    if (res.success) {
      // Show success toast
      const itemType = result.type === 'bookmark' ? 'Bookmark' : 'File';
      success(`${itemType} opened`, result.title);
    } else if (res.error) {
      // Show error toast
      console.error("Failed to open resource:", res.error);
      error('Failed to open', res.error);
      return; // Don't hide window on error
    }
  } catch (err) {
    console.error("Failed to open resource:", err);
    error('Operation failed', err instanceof Error ? err.message : 'Unknown error occurred');
    return; // Don't hide window on error
  }

  searchResults.value = [];
  appWindow.hide();
};

const handleKeydown = (e: KeyboardEvent) => {
  // Check for close shortcut
  if (shortcutManager.value.matches(e, 'general.close')) {
    if (showSettings.value) {
      showSettings.value = false;
      // Reload settings in case they changed
      loadSettings(); 
    } else {
      // If there is a query, clear it first
      if (searchComboboxRef.value?.hasQuery) {
        searchComboboxRef.value.clearQuery();
      } else {
        searchResults.value = [];
        appWindow.hide();
      }
    }
    return;
  }
  
  // Check for settings shortcut
  if (shortcutManager.value.matches(e, 'general.settings')) {
    showSettings.value = !showSettings.value;
    if (!showSettings.value) loadSettings();
    return;
  }
};

onMounted(async () => {
  loadSettings();
  window.addEventListener("keydown", handleKeydown);

  // Focus input when window gets focus
  await appWindow.onFocusChanged(({ payload: focused }) => {
    if (focused && !showSettings.value) {
       nextTick(() => {
          searchComboboxRef.value?.focusInput();
       });
    }
  });

  // Listen for tauri://focus event as a backup/alternative
  await listen('tauri://focus', () => {
      if (!showSettings.value) {
          nextTick(() => {
              searchComboboxRef.value?.focusInput();
          });
      }
  });
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});

function handleSettingsClose() {
    showSettings.value = false;
    loadSettings();
}
</script>

<template>
  <ErrorBoundary
    fallback-title="App Error"
    fallback-description="The application encountered an unexpected error."
  >
    <div class="app-root" :style="themeStyle">
      <!-- Toast Notifications -->
      <Toaster
        position="top-center"
        :duration="3000"
        :close-button="true"
        rich-colors
        :theme="settings?.theme.mode || 'dark'"
      />

      <div class="launcher-container" v-show="!showSettings">
        <SearchCombobox
          ref="searchComboboxRef"
          :results="searchResults"
          :loading="isLoading"
          :error="searchError"
          @search="handleSearch"
          @select="handleSelect"
          @retry="handleRetry"
        />
      </div>
      <div class="settings-container" v-if="showSettings">
          <SettingsPanel @close="handleSettingsClose" />
      </div>
    </div>
  </ErrorBoundary>
</template>

<style scoped>
.app-root {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-color, rgba(26, 26, 26, 0.95));
  color: var(--text-color, #e0e0e0);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  font-size: var(--font-size, 14px);
  /* Use transparent background for rounded corners effect if supported */
}

.launcher-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 0;
}

.settings-container {
    width: 100%;
    height: 100%;
    background: var(--bg-color);
}
</style>
