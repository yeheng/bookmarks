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
    class="flex items-center gap-1.5 px-3 pt-1.5 pb-1 mt-1 cursor-pointer select-none outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-[-2px] rounded first:mt-0 group"
    role="button"
    tabindex="0"
    :aria-expanded="!group.collapsed"
    :aria-label="ariaLabel"
    @click="handleClick"
    @keydown="handleKeydown"
  >
    <span class="text-[11px] font-semibold uppercase tracking-wider text-text-tertiary opacity-70">{{ group.label }}</span>
    <span class="text-[10px] font-medium text-text-tertiary opacity-50">{{ group.results.length }}</span>
    <div 
      class="text-text-tertiary opacity-40 transition-transform duration-200 ease-[cubic-bezier(0.4,0,0.2,1)] flex items-center motion-reduce:transition-none" 
      :class="{ '-rotate-90': group.collapsed }" 
      aria-hidden="true"
    >
      <svg width="10" height="10" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
  </div>
</template>

