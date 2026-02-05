<script setup lang="ts">
import { ref, computed } from 'vue';
import type { SearchResult } from '../types/search';
import { useFrecency, formatFrecencyBreakdown, getFrecencyBgClass } from '../composables/useFrecency';

interface Props {
  result: SearchResult;
  isSelected: boolean;
}

const props = defineProps<Props>();

// Frecency score visualization
const frecencyScore = computed(() => {
  const score = props.result.frecency_score ?? 0;
  return useFrecency(score);
});

const frecencyTooltip = computed(() => {
  const score = frecencyScore.value.value;
  return `${score.label} (${score.normalized}/100)\n${formatFrecencyBreakdown(score.breakdown)}`;
});

const showFrecency = computed(() => {
  return props.result.frecency_score !== undefined && props.result.frecency_score > 0;
});

// Ripple effect state
const ripples = ref<Array<{ id: number; x: number; y: number; size: number }>>([]);
let rippleId = 0;

const handleMouseDown = (event: MouseEvent) => {
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  const size = Math.max(rect.width, rect.height) * 2;

  const id = rippleId++;
  ripples.value.push({ id, x, y, size });

  // Remove ripple after animation
  setTimeout(() => {
    ripples.value = ripples.value.filter(r => r.id !== id);
  }, 600);
};
</script>

<template>
  <div
    class="result-item"
    :class="{ 'result-item--selected': isSelected }"
    role="option"
    :aria-selected="isSelected"
    :aria-label="`${result.title}, ${result.type === 'bookmark' ? 'Bookmark' : 'File'}, ${result.subtitle}`"
    @mousedown="handleMouseDown"
  >
    <!-- Ripple container -->
    <div class="ripple-container" aria-hidden="true">
      <span
        v-for="ripple in ripples"
        :key="ripple.id"
        class="ripple"
        :style="{
          left: `${ripple.x}px`,
          top: `${ripple.y}px`,
          width: `${ripple.size}px`,
          height: `${ripple.size}px`,
        }"
      ></span>
    </div>

    <div class="result-icon" aria-hidden="true">
      <span v-if="result.type === 'bookmark'" class="icon" role="img" aria-label="Bookmark">🔖</span>
      <span v-else class="icon" role="img" aria-label="File">📄</span>
    </div>
    <div class="result-content">
      <div class="result-title">{{ result.title }}</div>
      <div class="result-subtitle">{{ result.subtitle }}</div>
    </div>

    <!-- Frecency Score Indicator -->
    <div
      v-if="showFrecency"
      class="frecency-indicator"
      :title="frecencyTooltip"
      :aria-label="`Frecency score: ${frecencyScore.value.label}, ${frecencyScore.value.normalized} out of 100`"
      role="meter"
      :aria-valuenow="frecencyScore.value.normalized"
      aria-valuemin="0"
      aria-valuemax="100"
    >
      <div class="frecency-bar-container">
        <div
          class="frecency-bar"
          :class="getFrecencyBgClass(frecencyScore.value.color)"
          :style="{ width: `${frecencyScore.value.percentage}%` }"
        ></div>
      </div>
      <span class="frecency-label">{{ frecencyScore.value.normalized }}</span>
    </div>

    <div class="result-badge" :aria-label="`Type: ${result.type === 'bookmark' ? 'Bookmark' : 'File'}`">
      {{ result.type === 'bookmark' ? 'Bookmark' : 'File' }}
    </div>
    <div class="result-actions" v-if="isSelected" aria-hidden="true">
      <span class="action-key">⏎</span>
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
  border-radius: var(--border-radius, 6px);
  cursor: pointer;
  color: var(--text-color);
  overflow: hidden;
  /* Enhanced hover transition */
  transition: background-color 0.15s ease, box-shadow 0.15s ease, transform 0.1s ease;
}

.result-item:hover {
  background-color: rgba(255, 255, 255, 0.08);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.result-item:active {
  transform: scale(0.995);
}

.result-item--selected {
  background-color: var(--selection-bg);
  color: var(--selection-text) !important;
  box-shadow: 0 2px 12px rgba(255, 107, 107, 0.2);
}

/* Force white text on selection regardless of theme */
.result-item--selected .result-title,
.result-item--selected .result-subtitle,
.result-item--selected .result-badge {
    color: var(--selection-text);
    opacity: 0.9;
}

/* Ripple Effect */
.ripple-container {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  overflow: hidden;
  border-radius: inherit;
}

.ripple {
  position: absolute;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.3);
  transform: translate(-50%, -50%) scale(0);
  animation: ripple-expand 0.6s ease-out forwards;
  pointer-events: none;
}

.result-item--selected .ripple {
  background: rgba(255, 255, 255, 0.2);
}

@keyframes ripple-expand {
  0% {
    transform: translate(-50%, -50%) scale(0);
    opacity: 1;
  }
  100% {
    transform: translate(-50%, -50%) scale(1);
    opacity: 0;
  }
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

/* Frecency Score Indicator */
.frecency-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-right: 8px;
  flex-shrink: 0;
}

.frecency-bar-container {
  width: 40px;
  height: 4px;
  background-color: rgba(128, 128, 128, 0.2);
  border-radius: 2px;
  overflow: hidden;
}

.frecency-bar {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}

.frecency-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--secondary-text);
  min-width: 20px;
  text-align: right;
}

.result-item--selected .frecency-label {
  color: rgba(255, 255, 255, 0.9);
}

.result-badge {
  flex-shrink: 0;
  font-size: 10px;
  color: var(--secondary-text);
  padding: 2px 6px;
  background-color: rgba(128, 128, 128, 0.1);
  border-radius: 4px;
}

.result-actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  margin-left: 8px;
}

.action-key {
  font-size: 12px;
  background: rgba(255, 255, 255, 0.2);
  padding: 2px 6px;
  border-radius: 4px;
  color: rgba(255, 255, 255, 0.9);
}
</style>
