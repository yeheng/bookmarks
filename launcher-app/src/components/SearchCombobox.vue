<script setup lang="ts">
import { ref, computed, watch, reactive, nextTick } from 'vue';
import {
  Combobox,
  ComboboxInput,
  ComboboxOptions,
  ComboboxOption,
  TransitionRoot,
} from '@headlessui/vue';
import type { SearchResult } from '../types/search';
import SearchResultItem from './SearchResultItem.vue';
import ResultGroupHeader from './ResultGroupHeader.vue';
import SkeletonLoader from './SkeletonLoader.vue';
import { useGroupedResults } from '../composables/useGroupedResults';

interface Props {
  results: SearchResult[];
  loading?: boolean;
  error?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  error: null,
});

// Ensure props.loading is accessible for computed
const isLoading = computed(() => props.loading);

interface Emits {
  (e: 'search', query: string): void;
  (e: 'select', result: SearchResult): void;
  (e: 'retry'): void;
  (e: 'escape'): void;
}

const emit = defineEmits<Emits>();

const query = ref('');
const selectedResult = ref<SearchResult | null>(null);

let debounceTimer: number | null = null;

watch(query, (newQuery) => {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  debounceTimer = window.setTimeout(() => {
    emit('search', newQuery);
  }, 50);
});

const handleSelect = (result: SearchResult | null) => {
  if (result) {
    emit('select', result);
    query.value = '';
    selectedResult.value = null;
  }
};

const displayResults = computed(() => {
  return props.results.slice(0, 10);
});

// Grouping logic
const { groupedResults } = useGroupedResults(displayResults);

// Track collapsed state for each group
const collapsedGroups = reactive<Record<string, boolean>>({});

const toggleGroup = (groupType: string) => {
  collapsedGroups[groupType] = !collapsedGroups[groupType];
};

const isGroupCollapsed = (groupType: string) => {
  return collapsedGroups[groupType] ?? false;
};

// Reset collapsed state when query changes
watch(query, () => {
  Object.keys(collapsedGroups).forEach(key => {
    collapsedGroups[key] = false;
  });
});

const hasResults = computed(() => displayResults.value.length > 0);
const hasMultipleGroups = computed(() => groupedResults.value.hasMultipleGroups);
const showEmpty = computed(() => !isLoading.value && !props.error && !hasResults.value && query.value.length > 0);
const showError = computed(() => !isLoading.value && props.error && query.value.length > 0);

// Spotlight-style: results panel only visible when there's a query
const showResultsPanel = computed(() => query.value.length > 0);

// Content height hint for dynamic window sizing (approximate content item count)
const contentItemCount = computed(() => {
  if (!showResultsPanel.value) return 0;
  if (isLoading.value) return 3; // skeleton loader shows 3 items
  if (showEmpty.value) return 3; // empty state takes ~3 item heights
  if (showError.value) return 3; // error state takes ~3 item heights
  return displayResults.value.length;
});

// Handle retry for error state
const handleRetry = () => {
  emit('retry');
  emit('search', query.value);
};

const searchInput = ref<InstanceType<typeof ComboboxInput> | null>(null);

const focusInput = () => {
  if (searchInput.value?.$el) {
    searchInput.value.$el.focus();
  }
};

const handleInputKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    e.preventDefault();
    e.stopPropagation();
    emit('escape');
  }
};

const clearQuery = () => {
    query.value = '';
};

const hasQuery = computed(() => query.value.length > 0);

// ARIA: Generate unique IDs for accessibility
const listboxId = 'search-results-listbox';

// ARIA: Keyboard navigation announcement
const navigationAnnouncement = ref('');

// Update announcement when selection changes
watch(selectedResult, (newResult) => {
  if (newResult) {
    const typeLabel = newResult.type === 'bookmark' ? 'Bookmark' : newResult.type === 'file' ? 'File' : 'Plugin Result';
    navigationAnnouncement.value = `Selected: ${newResult.title}, ${typeLabel}`;

    // Scroll selected item into view
    nextTick(() => {
      const element = document.querySelector(`[data-result-id="${newResult.id}"]`);
      element?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    });
  }
});

defineExpose({
    focusInput,
    clearQuery,
    hasQuery,
    contentItemCount,
    showResultsPanel
});
</script>

<template>
  <Combobox v-model="selectedResult" @update:model-value="handleSelect">
    <div class="search-combobox">
      <!-- Window drag region -->
      <div class="drag-region" data-tauri-drag-region></div>

      <!-- Keyboard navigation announcements -->
      <div aria-live="polite" aria-atomic="true" class="visually-hidden">
        {{ navigationAnnouncement }}
      </div>

      <!-- Spotlight-style search bar -->
      <div class="search-bar" :class="{ 'has-panel': showResultsPanel }">
        <svg class="search-icon" :class="{ 'search-icon--loading': isLoading }" width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
          <path d="M8.5 3a5.5 5.5 0 0 1 4.383 8.823l4.147 4.147a.75.75 0 0 1-1.06 1.06l-4.147-4.147A5.5 5.5 0 1 1 8.5 3Zm0 1.5a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" fill="currentColor"/>
        </svg>
        <ComboboxInput
          ref="searchInput"
          @change="(e: Event) => query = (e.target as HTMLInputElement).value"
          @keydown="handleInputKeydown"
          :displayValue="() => query"
          class="search-input"
          placeholder="Search bookmarks and files..."
          autocomplete="off"
          autocapitalize="off"
          autocorrect="off"
          spellcheck="false"
          aria-label="Search bookmarks and files"
          aria-autocomplete="list"
          :aria-controls="listboxId"
          :aria-expanded="hasResults || loading"
        />
      </div>

      <!-- Spotlight-style: results panel only when there's a query -->
      <template v-if="showResultsPanel">
        <ComboboxOptions
          class="results-container"
          :id="listboxId"
          aria-label="Search results"
          static
        >
          <!-- Loading State -->
          <div v-if="loading" class="loading-state" role="status" aria-live="polite">
            <SkeletonLoader :count="3" />
          </div>

          <!-- Empty State -->
          <div v-else-if="showEmpty" class="state-message empty-state" role="status" aria-live="polite">
            <div class="empty-visual" aria-hidden="true">
              <svg width="40" height="40" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M8.5 3a5.5 5.5 0 0 1 4.383 8.823l4.147 4.147a.75.75 0 0 1-1.06 1.06l-4.147-4.147A5.5 5.5 0 1 1 8.5 3Zm0 1.5a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" fill="currentColor"/>
              </svg>
            </div>
            <div class="empty-text">No results for "<strong>{{ query }}</strong>"</div>
            <div class="empty-hint">Try a different search term</div>
          </div>

          <!-- Error State -->
          <div v-else-if="showError" class="state-message error-state" role="alert" aria-live="assertive">
            <div class="error-text">Search failed</div>
            <div class="error-description">{{ error }}</div>
            <button class="retry-btn" @click="handleRetry" aria-label="Retry search">
              Try Again
            </button>
          </div>

          <!-- Results -->
          <template v-else>
            <!-- Grouped Results Display -->
            <div v-if="hasMultipleGroups" class="grouped-results">
              <template v-for="(group, groupIndex) in groupedResults.groups" :key="group.type">
                <ResultGroupHeader
                  :group="{ ...group, collapsed: isGroupCollapsed(group.type) }"
                  :is-first="groupIndex === 0"
                  @toggle="toggleGroup(group.type)"
                />
                <TransitionRoot
                  :show="!isGroupCollapsed(group.type)"
                  enter="transition-all duration-200 ease-out"
                  enter-from="opacity-0 transform: translateY(-4px)"
                  enter-to="opacity-100 transform: translateY(0)"
                  leave="transition-all duration-150 ease-in"
                  leave-from="opacity-100"
                  leave-to="opacity-0"
                >
                  <div class="group-items">
                    <ComboboxOption
                      v-for="(result, rIndex) in group.results"
                      :key="result.id"
                      :value="result"
                      v-slot="{ active }"
                      class="result-option"
                      :data-result-id="result.id"
                    >
                      <SearchResultItem :result="result" :is-selected="active" :highlight-query="query" :index="rIndex" />
                    </ComboboxOption>
                  </div>
                </TransitionRoot>
              </template>
            </div>

            <!-- Flat Results Display (single type) -->
            <div v-else class="flat-results">
              <ComboboxOption
                v-for="(result, rIndex) in displayResults"
                :key="result.id"
                :value="result"
                v-slot="{ active }"
                class="result-option"
                :data-result-id="result.id"
              >
                <SearchResultItem :result="result" :is-selected="active" :highlight-query="query" :index="rIndex" />
              </ComboboxOption>
            </div>
          </template>
        </ComboboxOptions>
      </template>

      <!-- Bottom bar with shortcuts -->
      <div v-if="hasResults" class="bottom-bar">
        <div class="bottom-bar-left">
          <span class="result-count">{{ displayResults.length }} result{{ displayResults.length !== 1 ? 's' : '' }}</span>
        </div>
        <div class="bottom-bar-right">
          <button
            class="settings-icon-btn"
            @click="emit('select', { id: 'internal-settings', type: 'file', title: 'Settings', subtitle: '', path: '' })"
            aria-label="Open Settings"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z" fill="currentColor"/>
              <path fill-rule="evenodd" d="M6.858 1.212c.252-.912 1.032-.912 1.284 0l.32 1.158c.063.228.225.414.443.51l.468.204c.213.093.457.08.66-.036l1.02-.58c.805-.458 1.37.108.912.912l-.58 1.02a.754.754 0 0 0-.036.66l.205.468c.095.218.281.38.51.443l1.157.32c.912.252.912 1.032 0 1.284l-1.158.32a.754.754 0 0 0-.51.443l-.204.468a.754.754 0 0 0 .036.66l.58 1.02c.458.805-.108 1.37-.912.912l-1.02-.58a.754.754 0 0 0-.66-.036l-.468.205a.754.754 0 0 0-.443.51l-.32 1.157c-.252.912-1.032.912-1.284 0l-.32-1.158a.754.754 0 0 0-.443-.51l-.468-.204a.754.754 0 0 0-.66.036l-1.02.58c-.805.458-1.37-.108-.912-.912l.58-1.02a.754.754 0 0 0 .036-.66L3.446 9.1a.754.754 0 0 0-.51-.443l-1.157-.32c-.912-.252-.912-1.032 0-1.284l1.158-.32a.754.754 0 0 0 .51-.443l.204-.468a.754.754 0 0 0-.036-.66l-.58-1.02c-.458-.805.108-1.37.912-.912l1.02.58a.754.754 0 0 0 .66.036l.468-.205a.754.754 0 0 0 .443-.51l.32-1.157Z" fill="currentColor" opacity="0.5"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </Combobox>
</template>

<style scoped>
.search-combobox {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
  /* Vertical padding matches SEARCH_BAR_PADDING (12px = 6+6) in App.vue */
  padding: 6px 0;
}

/* Spotlight Search Bar — border only when results panel is visible */
.search-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
  height: var(--input-height, 54px);
  flex-shrink: 0;
}

.search-bar.has-panel {
  border-bottom: 0.5px solid var(--color-border-subtle, rgba(128, 128, 128, 0.15));
}

.search-icon {
  flex-shrink: 0;
  color: var(--color-text-tertiary);
  opacity: 0.6;
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.search-icon--loading {
  animation: search-pulse 1s ease-in-out infinite;
}

@keyframes search-pulse {
  0%, 100% {
    opacity: 0.6;
    transform: scale(1);
  }
  50% {
    opacity: 0.3;
    transform: scale(0.95);
  }
}

.search-input {
  flex: 1;
  min-width: 0;
  padding: 0;
  font-size: 18px;
  font-weight: 400;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text-color);
  line-height: 1.2;
}

.search-input::placeholder {
  color: var(--color-text-tertiary);
  font-weight: 300;
  opacity: 0.7;
}

/* ========== Results Container ========== */
.results-container {
  flex: 1;
  overflow-y: auto;
  padding: 6px;
  position: relative;
  /* Reset <ul> default styles from HeadlessUI ComboboxOptions */
  list-style: none;
  margin: 0;
  animation: results-slide-in 0.15s ease-out;
}

@keyframes results-slide-in {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Custom scrollbar - thin and minimal */
.results-container::-webkit-scrollbar {
  width: 4px;
}
.results-container::-webkit-scrollbar-track {
  background: transparent;
}
.results-container::-webkit-scrollbar-thumb {
  background: rgba(128, 128, 128, 0.2);
  border-radius: 2px;
}
.results-container::-webkit-scrollbar-thumb:hover {
  background: rgba(128, 128, 128, 0.35);
}

.result-option {
  outline: none;
  /* Reset <li> default styles from HeadlessUI ComboboxOption */
  list-style: none;
}

/* ========== State Messages ========== */
.state-message {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 24px;
  text-align: center;
  color: var(--color-text-secondary);
}

/* Loading State */
.loading-state {
  padding: 4px 0;
}

/* Empty State */
.empty-state {
  padding: 28px 24px;
}

.empty-visual {
  color: var(--color-text-tertiary);
  opacity: 0.3;
  margin-bottom: 12px;
}

.empty-text {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 4px;
}

.empty-text strong {
  font-weight: 600;
  color: var(--text-color);
}

.empty-hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

/* Error State */
.error-state {
  padding: 28px 24px;
}

.error-text {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 4px;
}

.error-description {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-bottom: 12px;
  max-width: 260px;
}

.retry-btn {
  display: inline-flex;
  align-items: center;
  padding: 5px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-color);
  background: rgba(128, 128, 128, 0.1);
  border: 0.5px solid rgba(128, 128, 128, 0.2);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.retry-btn:hover {
  background: rgba(128, 128, 128, 0.18);
}

.retry-btn:active {
  transform: scale(0.98);
}

/* ========== Grouped Results ========== */
.grouped-results {
  display: flex;
  flex-direction: column;
}

.group-items {
  overflow: hidden;
}

.flat-results {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

/* ========== Bottom Bar ========== */
.bottom-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 14px;
  flex-shrink: 0;
  border-top: 0.5px solid var(--color-border-subtle, rgba(128, 128, 128, 0.15));
  animation: bottom-bar-fade-in 0.2s ease-out;
}

@keyframes bottom-bar-fade-in {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.bottom-bar-left {
  display: flex;
  align-items: center;
}

.result-count {
  font-size: 11px;
  color: var(--color-text-tertiary);
  font-weight: 400;
}

.bottom-bar-right {
  display: flex;
  align-items: center;
}

.settings-icon-btn {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-tertiary);
  opacity: 0.5;
  transition: opacity 0.15s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.settings-icon-btn:hover {
  opacity: 1;
}

/* ========== Utility ========== */
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

/* Window drag region */
.drag-region {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 8px;
  z-index: 10;
  -webkit-app-region: drag;
  pointer-events: none;
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .search-icon--loading {
    animation: none;
    opacity: 0.4;
  }
  .results-container {
    animation: none;
  }
  .bottom-bar {
    animation: none;
  }
}
</style>
