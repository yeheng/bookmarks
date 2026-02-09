<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from '@tauri-apps/api/event';
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { Toaster } from 'vue-sonner';
import SearchCombobox from "./components/SearchCombobox.vue";
import SettingsPanel from "./components/settings/SettingsPanel.vue";
import ErrorBoundary from "./components/ErrorBoundary.vue";
import { ShortcutManager } from "./services/shortcuts";
import { useToast } from "./composables/useToast";
import { semanticColors } from "./design-system/tokens";
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
const { error } = useToast();

const themeStyle = computed(() => {
  if (!settings.value) return {};
  const { theme } = settings.value;
  const isLight = theme.mode === "light";
  const themeColors = isLight ? semanticColors.light : semanticColors.dark;

  return {
    "--accent-color": theme.accent_color,
    "--font-size": `${theme.font_size}px`,
    "--window-width": `${theme.window_width}px`,
    "--window-height": `${theme.window_height}px`,
    "--input-height": `${theme.input_height}px`,
    "--item-height": `${theme.item_height}px`,
    "--border-radius": `${theme.border_radius}px`,
    "--bg-color": theme.bg_color || themeColors.bg.primary,
    "--text-color": theme.text_color || themeColors.text.primary,
    "--secondary-text": theme.secondary_text_color || themeColors.text.secondary,
    "--border-color": theme.border_color || themeColors.border.default,
    "--selection-bg": theme.selection_bg_color || theme.accent_color,
    "--selection-text": theme.selection_text_color || "#ffffff",
    "--color-bg-primary": theme.bg_color || themeColors.bg.primary,
    "--color-bg-secondary": themeColors.bg.secondary,
    "--color-text-primary": theme.text_color || themeColors.text.primary,
    "--color-text-secondary": theme.secondary_text_color || themeColors.text.secondary,
    "--color-text-tertiary": themeColors.text.tertiary,
    "--color-border-default": theme.border_color || themeColors.border.default,
    "--color-border-subtle": themeColors.border.subtle,
    "--color-interactive-hover": themeColors.interactive.hover,
    "--color-interactive-active": themeColors.interactive.active,
    "--color-highlight-bg": themeColors.highlight.bg,
    "--color-highlight-border": themeColors.highlight.border,
  };
});

// Spotlight-style dynamic window height calculation
// Constants for height calculation (in px)
const SEARCH_BAR_PADDING = 12; // top/bottom visual padding around search bar
const RESULTS_PANEL_PADDING = 12; // results container padding (6px top + 6px bottom)
const BOTTOM_BAR_HEIGHT = 32; // bottom bar with result count

const searchBarHeight = computed(() => {
  const inputHeight = settings.value?.theme.input_height ?? 54;
  return inputHeight + SEARCH_BAR_PADDING;
});

const maxWindowHeight = computed(() => {
  return settings.value?.theme.window_height ?? 480;
});

const computedWindowHeight = computed(() => {
  // Settings panel uses full height
  if (showSettings.value) {
    return maxWindowHeight.value;
  }

  const itemCount = searchComboboxRef.value?.contentItemCount ?? 0;

  // No results panel → just search bar
  if (itemCount === 0) {
    return searchBarHeight.value;
  }

  // Calculate content height
  const itemHeight = settings.value?.theme.item_height ?? 44;
  const contentHeight = itemCount * itemHeight + RESULTS_PANEL_PADDING;
  const hasActualResults = searchResults.value.length > 0;
  const bottomBar = hasActualResults ? BOTTOM_BAR_HEIGHT : 0;
  const totalHeight = searchBarHeight.value + contentHeight + bottomBar;

  return Math.min(totalHeight, maxWindowHeight.value);
});

// Watch dynamic height and resize window
watch(computedWindowHeight, async (newHeight) => {
  if (!settings.value) return;
  const width = settings.value.theme.window_width;
  requestAnimationFrame(async () => {
    await appWindow.setSize(new LogicalSize(width, newHeight));
  });
});

// Watch window width changes only (height is now dynamic)
watch(
  () => settings.value?.theme.window_width,
  async (w) => {
    if (w) {
      await appWindow.setSize(new LogicalSize(w, computedWindowHeight.value));
    }
  }
);

async function loadSettings() {
  try {
    settings.value = await invoke<AppSettings>("get_app_settings");
    // Update shortcut manager with new settings
    if (settings.value) {
      shortcutManager.value = new ShortcutManager(settings.value.hotkey);
      // Initial resize: use search bar height (Spotlight-style, not full window_height)
      const { window_width, input_height } = settings.value.theme;
      const initialHeight = input_height + SEARCH_BAR_PADDING;
      await appWindow.setSize(new LogicalSize(window_width, initialHeight));

      // Position window at 2/5 of screen height (not centered)
      const monitor = await appWindow.currentMonitor();
      if (monitor) {
        const screenScaleFactor = monitor.scaleFactor;
        const screenHeight = monitor.size.height / screenScaleFactor;
        const screenWidth = monitor.size.width / screenScaleFactor;
        const windowWidth = window_width;

        // Calculate y position: 2/5 from top (40% of screen height)
        const y = (screenHeight * 0.4) - (initialHeight / 2);
        // Center horizontally
        const x = (screenWidth - windowWidth) / 2;

        await appWindow.setPosition(new LogicalPosition(x, y));
      }
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
      // Successfully opened, no toast needed
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
  // ESC: always hide the window (direct check, independent of shortcutManager loading)
  if (e.key === 'Escape') {
    e.preventDefault();
    e.stopPropagation();
    if (showSettings.value) {
      showSettings.value = false;
      loadSettings();
    }
    searchComboboxRef.value?.clearQuery();
    searchResults.value = [];
    appWindow.hide();
    return;
  }

  // Check for settings shortcut
  if (shortcutManager.value.matches(e, 'general.settings')) {
    e.preventDefault();
    showSettings.value = !showSettings.value;
    if (!showSettings.value) loadSettings();
    return;
  }
};

onMounted(async () => {
  loadSettings();
  // Use capture phase so ESC is handled before HeadlessUI's Combobox swallows it
  window.addEventListener("keydown", handleKeydown, true);

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
  window.removeEventListener("keydown", handleKeydown, true);
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

      <Transition name="panel-fade" mode="out-in">
        <div v-if="!showSettings" key="launcher" class="launcher-container">
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
        <div v-else key="settings" class="settings-container">
          <SettingsPanel @close="handleSettingsClose" />
        </div>
      </Transition>
    </div>
  </ErrorBoundary>
</template>

<style scoped>
.app-root {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-color, rgba(30, 30, 30, 0.78));
  backdrop-filter: blur(60px) saturate(180%);
  -webkit-backdrop-filter: blur(60px) saturate(180%);
  color: var(--text-color, #e0e0e0);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  font-size: var(--font-size, 14px);
  border-radius: var(--border-radius, 12px);
  border: 0.5px solid var(--border-color, rgba(255, 255, 255, 0.12));
  box-shadow:
    0 24px 80px rgba(0, 0, 0, 0.35),
    0 0 0 0.5px rgba(255, 255, 255, 0.08) inset;
  /* Force WebKit to clip all children to border-radius (fixes macOS corner artifacts) */
  -webkit-mask-image: -webkit-radial-gradient(white, black);
  isolation: isolate;
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
  border-radius: var(--border-radius, 12px);
}

/* Panel switch transition */
.panel-fade-enter-active {
  transition: opacity 0.15s ease-out, transform 0.15s ease-out;
}

.panel-fade-leave-active {
  transition: opacity 0.1s ease-in, transform 0.1s ease-in;
}

.panel-fade-enter-from {
  opacity: 0;
  transform: scale(0.98);
}

.panel-fade-leave-to {
  opacity: 0;
  transform: scale(0.98);
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .panel-fade-enter-active,
  .panel-fade-leave-active {
    transition: none;
  }
}
</style>
