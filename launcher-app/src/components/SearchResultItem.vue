<script setup lang="ts">
import type { SearchResult } from '../types/search';

interface Props {
  result: SearchResult;
  isSelected: boolean;
}

defineProps<Props>();
</script>

<template>
  <div
    class="result-item"
    :class="{ 'result-item--selected': isSelected }"
  >
    <div class="result-icon">
      <span v-if="result.type === 'bookmark'" class="icon">🔖</span>
      <span v-else class="icon">📄</span>
    </div>
    <div class="result-content">
      <div class="result-title">{{ result.title }}</div>
      <div class="result-subtitle">{{ result.subtitle }}</div>
    </div>
    <div class="result-badge">
      {{ result.type === 'bookmark' ? 'Bookmark' : 'File' }}
    </div>
  </div>
</template>

<style scoped>
.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
  height: var(--item-height, 44px);
  border-radius: var(--border-radius, 6px);
  cursor: pointer;
  transition: none; /* Raycast is snappy */
  color: var(--text-color);
}

.result-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.result-item--selected {
  background-color: var(--accent-color);
  color: #fff !important;
}

/* Force white text on selection regardless of theme */
.result-item--selected .result-title,
.result-item--selected .result-subtitle,
.result-item--selected .result-badge {
    color: rgba(255, 255, 255, 0.9);
}

.result-icon {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
}

.result-content {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.result-title {
  font-size: var(--font-size);
  font-weight: 500;
  white-space: nowrap;
  color: var(--text-color);
}

.result-subtitle {
  font-size: calc(var(--font-size) * 0.85);
  color: var(--secondary-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-badge {
  flex-shrink: 0;
  font-size: 10px;
  color: var(--secondary-text);
  padding: 2px 6px;
  background-color: rgba(128, 128, 128, 0.1);
  border-radius: 4px;
}
</style>
