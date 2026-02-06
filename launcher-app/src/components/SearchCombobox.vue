<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue';
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

interface Emits {
  (e: 'search', query: string): void;
  (e: 'select', result: SearchResult): void;
  (e: 'retry'): void;
}

const props = defineProps<Props>();
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
const showEmpty = computed(() => !props.loading && !props.error && !hasResults.value && query.value.length > 0);
const showRecent = computed(() => !props.loading && !props.error && !hasResults.value && query.value.length === 0);
const showError = computed(() => !props.loading && props.error && query.value.length > 0);

// Handle retry for error state
const handleRetry = () => {
  emit('retry');
  emit('search', query.value);
};

const searchInput = ref<InstanceType<typeof ComboboxInput> | null>(null);

const focusInput = () => {
  // Headless UI ComboboxInput renders an input element
  // We need to access the underlying DOM element
  if (searchInput.value?.$el) {
    searchInput.value.$el.focus();
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
    navigationAnnouncement.value = `Selected: ${newResult.title}, ${newResult.type === 'bookmark' ? 'Bookmark' : 'File'}`;
  }
});

defineExpose({
    focusInput,
    clearQuery,
    hasQuery
});
</script>

<template>
  <Combobox v-model="selectedResult" @update:model-value="handleSelect">
    <div class="search-combobox">
      <!-- Keyboard navigation announcements -->
      <div aria-live="polite" aria-atomic="true" class="visually-hidden">
        {{ navigationAnnouncement }}
      </div>

      <ComboboxInput
        ref="searchInput"
        @change="(e: Event) => query = (e.target as HTMLInputElement).value"
        :displayValue="() => query"
        class="search-input"
        placeholder="Search bookmarks and files..."
        autocomplete="off"
        aria-label="Search bookmarks and files"
        aria-autocomplete="list"
        :aria-controls="listboxId"
        :aria-expanded="hasResults || loading"
      />

      <button
        class="settings-icon-btn"
        @click="emit('select', { id: 'internal-settings', type: 'file', title: 'Settings', subtitle: '', path: '' })"
        aria-label="Open Settings (Cmd+Comma or type 'settings')"
        aria-keyshortcuts="Meta+Comma"
      >
        <span role="img" aria-label="Settings icon">⚙️</span>
      </button>

      <TransitionRoot
        enter="transition-opacity duration-150"
        enter-from="opacity-0"
        enter-to="opacity-100"
        leave="transition-opacity duration-100"
        leave-from="opacity-100"
        leave-to="opacity-0"
      >
        <ComboboxOptions
          class="results-container"
          :id="listboxId"
          aria-label="Search results"
        >
          <TransitionRoot
            enter="transition-opacity duration-200 delay-200"
            enter-from="opacity-0"
            enter-to="opacity-100"
            leave="transition-opacity duration-100"
            leave-from="opacity-100"
            leave-to="opacity-0"
          >
            <div v-if="loading" class="loading-state" role="status" aria-live="polite">
              <div class="loading-header">
                <div class="loading-spinner-small" aria-hidden="true"></div>
                <span class="loading-text">Searching...</span>
              </div>
              <SkeletonLoader :count="3" />
            </div>
          </TransitionRoot>

          <div v-if="showEmpty" class="state-message empty-state" role="status" aria-live="polite">
            <div class="empty-icon-container">
              <span class="empty-icon" role="img" aria-label="No results">🔍</span>
              <span class="empty-icon-overlay">❌</span>
            </div>
            <div class="empty-text">No results for "{{ query }}"</div>
            <div class="empty-suggestions">
              <div class="empty-hint">Try these tips:</div>
              <ul class="suggestion-list">
                <li>Check your spelling</li>
                <li>Use fewer or different keywords</li>
                <li>Type <span class="highlight">settings</span> to configure search</li>
              </ul>
            </div>
          </div>

          <!-- Error State -->
          <div v-else-if="showError" class="state-message error-state" role="alert" aria-live="assertive">
            <div class="error-icon-container">
              <span class="error-icon" role="img" aria-label="Error">⚠️</span>
            </div>
            <div class="error-text">Search failed</div>
            <div class="error-description">{{ error }}</div>
            <button
              class="retry-btn"
              @click="handleRetry"
              aria-label="Retry search"
            >
              <span class="retry-icon">🔄</span>
              Try Again
            </button>
          </div>

          <div v-else-if="showRecent" class="state-message" role="status">
            <div class="section-label">Recent</div>
            <div class="empty-hint">Your recent items will appear here</div>
          </div>

          <TransitionRoot
            v-else
            enter="transition-all duration-200 ease-out"
            enter-from="opacity-0 translate-y-1"
            enter-to="opacity-100 translate-y-0"
            leave="transition-opacity duration-150"
            leave-from="opacity-100"
            leave-to="opacity-0"
          >
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
                    enter-from="opacity-0 max-h-0"
                    enter-to="opacity-100 max-h-96"
                    leave="transition-all duration-150 ease-in"
                    leave-from="opacity-100 max-h-96"
                    leave-to="opacity-0 max-h-0"
                  >
                  <div class="group-items">
                    <ComboboxOption
                      v-for="result in group.results"
                      :key="result.id"
                      :value="result"
                      v-slot="{ active }"
                      class="result-option"
                    >
                      <SearchResultItem :result="result" :is-selected="active" />
                    </ComboboxOption>
                  </div>
                </TransitionRoot>
              </template>
            </div>

            <!-- Flat Results Display (single type) -->
            <div v-else class="flat-results">
              <ComboboxOption
                v-for="result in displayResults"
                :key="result.id"
                :value="result"
                v-slot="{ active }"
                class="result-option"
              >
                <SearchResultItem :result="result" :is-selected="active" />
              </ComboboxOption>
            </div>
          </TransitionRoot>

          <div v-if="hasResults && results.length > 10" class="more-results">
            Press ↓ to see more ({{ results.length - 10 }} more)
          </div>
        </ComboboxOptions>
      </TransitionRoot>
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
}

.search-input {
  width: 100%;
  height: var(--input-height, 64px);
  padding: 0 50px 0 24px;
  font-size: 20px;
  font-weight: 400;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text-color);
  line-height: 1;
}

.search-input::placeholder {
  color: var(--color-text-tertiary);
  font-weight: 300;
}

.settings-icon-btn {
  position: absolute;
  top: 0;
  right: 16px;
  height: var(--input-height, 64px);
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  opacity: 0.4;
  transition: opacity 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-color);
}

.settings-icon-btn:hover {
  opacity: 1;
}

.results-container {
  flex: 1;
  overflow-y: auto;
  padding: 12px 12px 16px;
  position: relative;
}

/* Scroll mask gradient - fade out content near input */
.results-container::before {
  content: '';
  position: sticky;
  top: 0;
  left: 0;
  right: 0;
  height: 32px;
  background: linear-gradient(
    to bottom,
    var(--color-bg-primary) 0%,
    transparent 100%
  );
  pointer-events: none;
  z-index: 1;
}

/* Custom scrollbar */
.results-container::-webkit-scrollbar {
  width: 6px;
}
.results-container::-webkit-scrollbar-track {
  background: transparent;
}
.results-container::-webkit-scrollbar-thumb {
  background: var(--color-border-subtle);
  border-radius: 3px;
}
.results-container::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-default);
}

.result-option {
  outline: none;
}

.state-message {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
  color: var(--color-text-secondary);
  height: 100%;
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 107, 107, 0.2);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Enhanced Loading State */
.loading-state {
  padding: 12px 0;
}

.loading-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px 12px 16px;
}

.loading-spinner-small {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 107, 107, 0.2);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.loading-text {
  font-size: 12px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

/* Enhanced Empty State */
.empty-state {
  padding: 32px 24px;
}

.empty-icon-container {
  position: relative;
  display: inline-block;
  margin-bottom: 16px;
}

.empty-icon-overlay {
  position: absolute;
  bottom: 0;
  right: -8px;
  font-size: 20px;
  background: var(--bg-color);
  border-radius: 50%;
  padding: 2px;
}

.empty-suggestions {
  margin-top: 16px;
  text-align: left;
  max-width: 240px;
}

.suggestion-list {
  list-style: none;
  padding: 0;
  margin: 8px 0 0 0;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.suggestion-list li {
  padding: 4px 0;
  padding-left: 20px;
  position: relative;
}

.suggestion-list li::before {
  content: "•";
  position: absolute;
  left: 8px;
  color: var(--accent-color);
}

/* Error State */
.error-state {
  padding: 32px 24px;
}

.error-icon-container {
  margin-bottom: 16px;
}

.error-icon {
  font-size: 48px;
}

.error-text {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 8px;
}

.error-description {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 16px;
  max-width: 280px;
}

.retry-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-color);
  background: rgba(128, 128, 128, 0.1);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.retry-btn:hover {
  background: rgba(128, 128, 128, 0.2);
  border-color: var(--accent-color);
}

.retry-btn:active {
  transform: scale(0.98);
}

.retry-icon {
  font-size: 14px;
}

.empty-text {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-color);
  margin-bottom: 8px;
}

.empty-hint {
  font-size: 14px;
  color: var(--color-text-secondary);
}

.highlight {
  color: var(--accent-color);
  font-weight: 600;
  background: rgba(128, 128, 128, 0.1);
  padding: 2px 4px;
  border-radius: 4px;
}

.section-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
  margin-bottom: 12px;
}

.more-results {
  padding: 12px 16px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-secondary);
  border-top: 1px solid var(--color-border-subtle);
}

/* Grouped Results Styles */
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
  gap: 4px;
}

/* Visually hidden class for screen reader announcements */
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
</style>
