<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import SearchCombobox from "./components/SearchCombobox.vue";
import type {
  SearchResult,
  BookmarkSearchResult,
  FileSearchResult,
  OpenResult,
} from "./types/search";

const appWindow = getCurrentWebviewWindow();
const searchResults = ref<SearchResult[]>([]);
const isLoading = ref(false);

function mapBookmarkToSearchResult(b: BookmarkSearchResult): SearchResult {
  return {
    id: `bookmark-${b.id}`,
    type: "bookmark",
    title: b.title,
    subtitle: b.url,
    icon: b.favicon_url ?? undefined,
    url: b.url,
  };
}

function mapFileToSearchResult(f: FileSearchResult): SearchResult {
  return {
    id: `file-${f.id}`,
    type: "file",
    title: f.name,
    subtitle: f.path,
    path: f.path,
  };
}

const handleSearch = async (query: string) => {
  if (!query.trim()) {
    searchResults.value = [];
    return;
  }

  isLoading.value = true;

  try {
    const [bookmarks, files] = await Promise.all([
      invoke<BookmarkSearchResult[]>("search_bookmarks", { query, limit: 5 }),
      invoke<FileSearchResult[]>("search_files", { query, limit: 5 }),
    ]);

    const bookmarkResults = bookmarks.map(mapBookmarkToSearchResult);
    const fileResults = files.map(mapFileToSearchResult);

    // Interleave results: bookmarks first, then files
    searchResults.value = [...bookmarkResults, ...fileResults];
  } catch (err) {
    console.error("Search failed:", err);
    searchResults.value = [];
  } finally {
    isLoading.value = false;
  }
};

const handleSelect = async (result: SearchResult) => {
  try {
    const resourceId = parseInt(result.id.split("-")[1], 10);
    const res = await invoke<OpenResult>("open_resource", {
      resourceType: result.type,
      resourceId: resourceId,
    });

    if (!res.success && res.error) {
      console.error("Failed to open resource:", res.error);
    }
  } catch (err) {
    console.error("Failed to open resource:", err);
  }

  searchResults.value = [];
  appWindow.hide();
};

const handleEscape = (e: KeyboardEvent) => {
  if (e.key === "Escape") {
    searchResults.value = [];
    appWindow.hide();
  }
};

onMounted(() => {
  window.addEventListener("keydown", handleEscape);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleEscape);
});
</script>

<template>
  <div class="launcher-container">
    <SearchCombobox
      :results="searchResults"
      :loading="isLoading"
      @search="handleSearch"
      @select="handleSelect"
    />
  </div>
</template>

<style scoped>
.launcher-container {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 16px;
  background: rgba(26, 26, 26, 0.95);
  backdrop-filter: blur(20px);
}

@media (prefers-color-scheme: light) {
  .launcher-container {
    background: rgba(255, 255, 255, 0.95);
  }
}
</style>
