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
    :class="{ 'group-header--first': isFirst }"
    role="button"
    tabindex="0"
    :aria-expanded="!group.collapsed"
    :aria-label="ariaLabel"
    @click="handleClick"
    @keydown="handleKeydown"
  >
    <span class="group-label">{{ group.label }}</span>
    <span class="group-count">{{ group.results.length }}</span>
    <div class="group-chevron" :class="{ 'group-chevron--collapsed': group.collapsed }" aria-hidden="true">
      <svg width="10" height="10" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px 4px;
  margin-top: 4px;
  cursor: pointer;
  user-select: none;
}

.group-header--first {
  margin-top: 0;
}

.group-header:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: -2px;
  border-radius: 4px;
}

.group-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--color-text-tertiary);
  opacity: 0.7;
}

.group-count {
  font-size: 10px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  opacity: 0.5;
}

.group-chevron {
  color: var(--color-text-tertiary);
  opacity: 0.4;
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
}

.group-chevron--collapsed {
  transform: rotate(-90deg);
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .group-chevron {
    transition: none;
  }
}
</style>
