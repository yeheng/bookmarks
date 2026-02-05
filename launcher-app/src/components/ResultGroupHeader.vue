<script setup lang="ts">
import { computed } from 'vue';
import type { ResultGroup } from '../composables/useGroupedResults';

interface Props {
  group: ResultGroup;
  isFirst?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isFirst: false,
});

const emit = defineEmits<{
  (e: 'toggle'): void;
}>();

const handleClick = () => {
  emit('toggle');
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    emit('toggle');
  } else if (e.key === 'Escape') {
    // Collapse the group if expanded
    if (!props.group.collapsed) {
      e.preventDefault();
      emit('toggle');
    }
  }
};

const ariaLabel = computed(() => {
  const count = props.group.results.length;
  const state = props.group.collapsed ? 'collapsed' : 'expanded';
  return `${props.group.label} section, ${count} items, ${state}. Press Enter to ${props.group.collapsed ? 'expand' : 'collapse'}`;
});
</script>

<template>
  <div
    class="group-header"
    :class="{ 'group-header--first': isFirst, 'group-header--collapsed': group.collapsed }"
    role="button"
    tabindex="0"
    :aria-expanded="!group.collapsed"
    :aria-label="ariaLabel"
    @click="handleClick"
    @keydown="handleKeydown"
  >
    <div class="group-header-content">
      <span class="group-icon" aria-hidden="true">{{ group.icon }}</span>
      <span class="group-label">{{ group.label }}</span>
      <span class="group-count" aria-hidden="true">({{ group.results.length }})</span>
    </div>
    <div class="group-chevron" :class="{ 'group-chevron--collapsed': group.collapsed }" aria-hidden="true">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  margin-top: 8px;
  cursor: pointer;
  border-radius: var(--border-radius, 6px);
  transition: background-color 0.15s ease;
  user-select: none;
}

.group-header--first {
  margin-top: 0;
}

.group-header:hover {
  background: rgba(128, 128, 128, 0.08);
}

.group-header:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: -2px;
}

.group-header-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-icon {
  font-size: 14px;
  opacity: 0.8;
}

.group-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--secondary-text);
}

.group-count {
  font-size: 11px;
  font-weight: 500;
  color: var(--secondary-text);
  opacity: 0.7;
}

.group-chevron {
  color: var(--secondary-text);
  opacity: 0.5;
  transition: transform 0.2s ease, opacity 0.15s ease;
}

.group-header:hover .group-chevron {
  opacity: 0.8;
}

.group-chevron--collapsed {
  transform: rotate(-90deg);
}

/* Collapsed state styling */
.group-header--collapsed {
  background: rgba(128, 128, 128, 0.04);
}

.group-header--collapsed:hover {
  background: rgba(128, 128, 128, 0.1);
}
</style>
