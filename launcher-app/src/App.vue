<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import SearchCombobox from "./components/SearchCombobox.vue";
import type { SearchResult } from "./types/search";

const appWindow = getCurrentWebviewWindow();
const searchResults = ref<SearchResult[]>([]);
const isLoading = ref(false);

const handleSearch = async (query: string) => {
  if (!query.trim()) {
    searchResults.value = [];
    return;
  }

  isLoading.value = true;
  
  await new Promise(resolve => setTimeout(resolve, 300));
  
  searchResults.value = [
    {
      id: '1',
      type: 'bookmark',
      title: 'Example Bookmark',
      subtitle: 'https://example.com',
      url: 'https://example.com'
    },
    {
      id: '2',
      type: 'file',
      title: 'Document.pdf',
      subtitle: '/Users/username/Documents/Document.pdf',
      path: '/Users/username/Documents/Document.pdf'
    }
  ];
  
  isLoading.value = false;
};

const handleSelect = async (result: SearchResult) => {
  console.log('Selected:', result);
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
