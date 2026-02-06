<script setup lang="ts">
import { computed } from 'vue';
import type { SearchResult } from '../types/search';

interface Props {
  result: SearchResult;
  isSelected: boolean;
}

const props = defineProps<Props>();

const adaptiveMetadata = computed(() => {
  if (!props.isSelected || !props.result.metadata) return null;

  const parts: string[] = [];
  const md = props.result.metadata;

  // Add metadata parts in order of importance
  if (md?.modified) parts.push(md.modified);
  if (md?.size) parts.push(md.size);
  if (md?.domain) parts.push(md.domain);

  return parts.join(' · ');
});

const shortcutHint = computed(() => {
  return props.isSelected ? '⏎' : '';
});
</script>

<template>
  <div
    class="result-item"
    :class="{ 'result-item--selected': isSelected }"
    role="option"
    :aria-selected="isSelected"
    :aria-label="`${result.title}, ${result.type === 'bookmark' ? 'Bookmark' : 'File'}, ${result.subtitle}`"
  >

    <div class="result-icon-container" aria-hidden="true">
      <span v-if="result.type === 'bookmark'" class="icon" role="img" aria-label="Bookmark">🔖</span>
      <span v-else class="icon" role="img" aria-label="File">📄</span>
    </div>

    <div class="result-content">
      <div class="result-title">{{ result.title }}</div>
      <div class="result-subtitle">{{ result.subtitle }}</div>
    </div>

    <!-- Adaptive Metadata (visible only when selected) -->
    <div v-if="adaptiveMetadata" class="adaptive-metadata">
      {{ adaptiveMetadata }}
    </div>

    <div v-if="isSelected" class="action-hint" aria-hidden="true">
      {{ shortcutHint }}
    </div>
  </div>
</template>

<style scoped>
.result-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
  height: var(--item-height, 44px);
  border-radius: 12px;
  cursor: pointer;
  color: var(--text-color);
  overflow: hidden;
  transition: background-color 0.2s ease-out, transform 0.15s ease-out, box-shadow 0.2s ease-out, border-color 0.2s ease-out;
}

.result-item:hover {
  background-color: var(--color-interactive-hover);
}

.result-item:active {
  transform: scale(0.995);
  background-color: var(--color-interactive-active);
}

.result-item--selected {
  background-color: var(--color-highlight-bg);
  border: 1px solid var(--color-highlight-border);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

/* Icon container - uniform 32x32 box */
.result-icon-container {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.result-icon-container .icon {
  font-size: 20px;
}

/* Content area */
.result-content {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.result-title {
  font-size: var(--font-size, 14px);
  font-weight: 500;
  white-space: nowrap;
  color: var(--text-color);
}

.result-subtitle {
  font-size: calc(var(--font-size, 14px) * 0.85);
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Adaptive metadata - only when selected */
.adaptive-metadata {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--color-text-tertiary);
  font-feature-settings: 'tnum';
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-item--selected .adaptive-metadata {
  color: var(--color-text-tertiary);
  opacity: 0.8;
}

/* Action hint (keyboard shortcut) */
.action-hint {
  flex-shrink: 0;
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, SF Mono, Consolas, monospace;
  color: var(--color-text-tertiary);
  opacity: 0.4;
}

.result-item--selected .action-hint {
  opacity: 0.6;
}
</style>
