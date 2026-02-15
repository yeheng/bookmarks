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
    <div class="w-full h-full flex flex-col relative py-1.5">
      <!-- Window drag region -->
      <div class="absolute top-0 left-0 right-0 h-2 z-10 select-none" data-tauri-drag-region></div>

      <!-- Keyboard navigation announcements -->
      <div aria-live="polite" aria-atomic="true" class="sr-only">
        {{ navigationAnnouncement }}
      </div>

      <!-- Spotlight-style search bar -->
      <div 
        class="flex items-center gap-3 px-4 shrink-0 h-[var(--input-height,54px)]"
        :class="{ 'border-b border-border-subtle': showResultsPanel }"
      >
        <svg 
          class="shrink-0 text-text-tertiary opacity-60 transition-all duration-200"
          :class="{ 'animate-pulse opacity-30 scale-95': isLoading }"
          width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true"
        >
          <path d="M8.5 3a5.5 5.5 0 0 1 4.383 8.823l4.147 4.147a.75.75 0 0 1-1.06 1.06l-4.147-4.147A5.5 5.5 0 1 1 8.5 3Zm0 1.5a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" fill="currentColor"/>
        </svg>
        <ComboboxInput
          ref="searchInput"
          @change="(e: Event) => query = (e.target as HTMLInputElement).value"
          @keydown="handleInputKeydown"
          :displayValue="() => query"
          class="flex-1 min-w-0 p-0 text-lg font-normal border-none outline-none bg-transparent text-text-primary leading-tight placeholder:text-text-tertiary placeholder:font-light placeholder:opacity-70"
          placeholder="Search bookmarks and files..."
          autocomplete="off"
          aria-label="Search bookmarks and files"
          aria-autocomplete="list"
          :aria-controls="listboxId"
          :aria-expanded="hasResults || loading"
        />
      </div>

      <!-- Spotlight-style: results panel only when there's a query -->
      <template v-if="showResultsPanel">
        <ComboboxOptions
          class="flex-1 overflow-y-auto p-1.5 relative list-none m-0 animate-in fade-in slide-in-from-top-1 duration-150 scrollbar-thin scrollbar-thumb-gray-500/20 hover:scrollbar-thumb-gray-500/35 scrollbar-track-transparent"
          :id="listboxId"
          aria-label="Search results"
          static
        >
          <!-- Loading State -->
          <div v-if="loading" class="py-1" role="status" aria-live="polite">
            <SkeletonLoader :count="3" />
          </div>

          <!-- Empty State -->
          <div v-else-if="showEmpty" class="flex flex-col items-center justify-center py-7 px-6 text-center text-text-secondary" role="status" aria-live="polite">
            <div class="text-text-tertiary opacity-30 mb-3" aria-hidden="true">
              <svg width="40" height="40" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M8.5 3a5.5 5.5 0 0 1 4.383 8.823l4.147 4.147a.75.75 0 0 1-1.06 1.06l-4.147-4.147A5.5 5.5 0 1 1 8.5 3Zm0 1.5a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" fill="currentColor"/>
              </svg>
            </div>
            <div class="text-[13px] font-medium text-text-secondary mb-1">No results for "<strong class="font-semibold text-text-primary">{{ query }}</strong>"</div>
            <div class="text-xs text-text-tertiary">Try a different search term</div>
          </div>

          <!-- Error State -->
          <div v-else-if="showError" class="flex flex-col items-center justify-center py-7 px-6 text-center text-text-secondary" role="alert" aria-live="assertive">
            <div class="text-[13px] font-semibold text-text-primary mb-1">Search failed</div>
            <div class="text-xs text-text-secondary mb-3 max-w-[260px]">{{ error }}</div>
            <button class="inline-flex items-center px-3.5 py-1.5 text-xs font-medium text-text-primary bg-white/10 border border-white/20 rounded-md cursor-pointer hover:bg-white/20 active:scale-95 transition-all" @click="handleRetry" aria-label="Retry search">
              Try Again
            </button>
          </div>

          <!-- Results -->
          <template v-else>
            <!-- Grouped Results Display -->
            <div v-if="hasMultipleGroups" class="flex flex-col">
              <template v-for="(group, groupIndex) in groupedResults.groups" :key="group.type">
                <ResultGroupHeader
                  :group="{ ...group, collapsed: isGroupCollapsed(group.type) }"
                  :is-first="groupIndex === 0"
                  @toggle="toggleGroup(group.type)"
                />
                <TransitionRoot
                  :show="!isGroupCollapsed(group.type)"
                  enter="transition-all duration-200 ease-out"
                  enter-from="opacity-0 transform -translate-y-1"
                  enter-to="opacity-100 transform translate-y-0"
                  leave="transition-all duration-150 ease-in"
                  leave-from="opacity-100"
                  leave-to="opacity-0"
                >
                  <div class="overflow-hidden">
                    <ComboboxOption
                      v-for="(result, rIndex) in group.results"
                      :key="result.id"
                      :value="result"
                      v-slot="{ active }"
                      class="outline-none list-none"
                      :data-result-id="result.id"
                    >
                      <SearchResultItem :result="result" :is-selected="active" :highlight-query="query" :index="rIndex" />
                    </ComboboxOption>
                  </div>
                </TransitionRoot>
              </template>
            </div>

            <!-- Flat Results Display (single type) -->
            <div v-else class="flex flex-col gap-0.5">
              <ComboboxOption
                v-for="(result, rIndex) in displayResults"
                :key="result.id"
                :value="result"
                v-slot="{ active }"
                class="outline-none list-none"
                :data-result-id="result.id"
              >
                <SearchResultItem :result="result" :is-selected="active" :highlight-query="query" :index="rIndex" />
              </ComboboxOption>
            </div>
          </template>
        </ComboboxOptions>
      </template>

      <!-- Bottom bar with shortcuts -->
      <div v-if="hasResults" class="flex items-center justify-between px-3.5 py-1.5 shrink-0 border-t border-border-subtle animate-in fade-in slide-in-from-bottom-1 duration-200">
        <div class="flex items-center">
          <span class="text-[11px] text-text-tertiary font-normal">{{ displayResults.length }} result{{ displayResults.length !== 1 ? 's' : '' }}</span>
        </div>
        <div class="flex items-center">
          <button
            class="bg-none border-none p-1 cursor-pointer text-text-tertiary opacity-50 hover:opacity-100 transition-opacity flex items-center justify-center rounded"
            @click="emit('select', { id: 'internal-settings', type: 'file', title: 'Settings', subtitle: '', path: '' })"
            aria-label="Open Settings"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z" fill="currentColor"/>
              <path fill-rule="evenodd" d="M6.858 1.212c.252-.912 1.032-.912 1.284 0l.32 1.158c.063.228.225.414.443.51l.468.204c.213.093.457.08.66-.036l1.02-.58c.805-.458 1.37.108.912.912l-.58 1.02a.754.754 0 0 0-.036.66l.205.468c.095.218.281.38.51.443l1.157.32c.912.252.912 1.032 0 1.284l-1.158.32a.754.754 0 0 0-.51.443l-.204.468a.754.754 0 0 0-.66.036l-1.02.58c-.805.458-1.37-.108-.912-.912l.58-1.02a.754.754 0 0 0 .036-.66L3.446 9.1a.754.754 0 0 0-.51-.443l-1.157-.32c-.912-.252-.912-1.032 0-1.284l1.158-.32a.754.754 0 0 0 .51-.443l.204-.468a.754.754 0 0 0-.036-.66l-.58-1.02c-.458-.805.108-1.37.912-.912l1.02.58a.754.754 0 0 0 .66.036l.468-.205a.754.754 0 0 0 .443-.51l.32-1.157Z" fill="currentColor" opacity="0.5"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </Combobox>
</template>

<style scoped>
/* Scrollbar styling handled by Tailwind classes or global styles, 
   but keeping this if specific webkit targeting is needed beyond utility classes */
::-webkit-scrollbar {
  width: 4px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: rgba(128, 128, 128, 0.2);
  border-radius: 2px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(128, 128, 128, 0.35);
}
</style>