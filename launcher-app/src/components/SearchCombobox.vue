<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import {
  Combobox,
  ComboboxInput,
  ComboboxOptions,
  ComboboxOption,
  TransitionRoot,
} from '@headlessui/vue';
import type { SearchResult } from '../types/search';
import SearchResultItem from './SearchResultItem.vue';

interface Props {
  results: SearchResult[];
  loading?: boolean;
}

interface Emits {
  (e: 'search', query: string): void;
  (e: 'select', result: SearchResult): void;
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

const hasResults = computed(() => displayResults.value.length > 0);
const showEmpty = computed(() => !props.loading && !hasResults.value && query.value.length > 0);
const showRecent = computed(() => !props.loading && !hasResults.value && query.value.length === 0);
</script>

<template>
  <Combobox v-model="selectedResult" @update:model-value="handleSelect">
    <div class="search-combobox">
      <ComboboxInput
        v-model="query"
        class="search-input"
        placeholder="Search bookmarks and files..."
        autocomplete="off"
      />

      <TransitionRoot
        enter="transition-opacity duration-150"
        enter-from="opacity-0"
        enter-to="opacity-100"
        leave="transition-opacity duration-100"
        leave-from="opacity-100"
        leave-to="opacity-0"
      >
        <ComboboxOptions class="results-container">
          <TransitionRoot
            enter="transition-opacity duration-200 delay-50"
            enter-from="opacity-0"
            enter-to="opacity-100"
            leave="transition-opacity duration-100"
            leave-from="opacity-100"
            leave-to="opacity-0"
          >
            <div v-if="loading" class="state-message">
              <div class="loading-spinner"></div>
              <span>Searching...</span>
            </div>
          </TransitionRoot>

          <div v-if="showEmpty" class="state-message">
            <span class="empty-icon">🔍</span>
            <div class="empty-text">No results found</div>
            <div class="empty-hint">Try different keywords</div>
          </div>

          <div v-else-if="showRecent" class="state-message">
            <div class="section-label">Recent</div>
            <div class="empty-hint">Your recent items will appear here</div>
          </div>

          <TransitionRoot
            v-else
            enter="transition-all duration-100"
            enter-from="opacity-0 translate-y-1"
            enter-to="opacity-100 translate-y-0"
            leave="transition-opacity duration-75"
            leave-from="opacity-100"
            leave-to="opacity-0"
          >
            <ComboboxOption
              v-for="result in displayResults"
              :key="result.id"
              :value="result"
              v-slot="{ active }"
              class="result-option"
            >
              <SearchResultItem :result="result" :is-selected="active" />
            </ComboboxOption>
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
}

.search-input {
  width: 100%;
  padding: 12px 16px;
  font-size: 18px;
  border: none;
  outline: none;
  background: transparent;
  color: #e0e0e0;
  border-bottom: 1px solid #3a3a3a;
}

.search-input::placeholder {
  color: #9a9a9a;
}

.results-container {
  max-height: 400px;
  overflow-y: auto;
  padding: 8px 0;
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
  color: #9a9a9a;
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 107, 107, 0.2);
  border-top-color: #ff6b6b;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-text {
  font-size: 16px;
  font-weight: 500;
  color: #e0e0e0;
  margin-bottom: 8px;
}

.empty-hint {
  font-size: 14px;
  color: #9a9a9a;
}

.section-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #9a9a9a;
  margin-bottom: 12px;
}

.more-results {
  padding: 12px 16px;
  text-align: center;
  font-size: 12px;
  color: #9a9a9a;
  border-top: 1px solid #3a3a3a;
}

@media (prefers-color-scheme: light) {
  .search-input {
    color: #1a1a1a;
    border-bottom-color: #e0e0e0;
  }

  .search-input::placeholder {
    color: #6a6a6a;
  }

  .empty-text {
    color: #1a1a1a;
  }

  .empty-hint,
  .section-label,
  .more-results {
    color: #6a6a6a;
  }

  .more-results {
    border-top-color: #e0e0e0;
  }
}
</style>
