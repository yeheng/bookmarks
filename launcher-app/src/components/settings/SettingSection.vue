<script setup lang="ts">
/**
 * SettingSection - Groups related settings under a titled section
 */
import { ref } from 'vue';

interface Props {
  title: string;
  id: string;
  /** Optional icon (emoji) */
  icon?: string;
  /** Whether section is collapsible */
  collapsible?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  collapsible: false,
});

const isCollapsed = ref(false);

const toggle = () => {
  if (props.collapsible) {
    isCollapsed.value = !isCollapsed.value;
  }
};
</script>

<template>
  <section
    class="mb-6 last:mb-0"
    role="group"
    :aria-labelledby="`section-${id}`"
  >
    <div
      class="flex items-center justify-between mb-2"
      :class="[collapsible ? 'cursor-pointer py-1.5 px-2 -mx-2 rounded-md hover:bg-hover-bg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent' : '']"
      :role="collapsible ? 'button' : undefined"
      :tabindex="collapsible ? 0 : undefined"
      :aria-expanded="collapsible ? !isCollapsed : undefined"
      @click="toggle"
      @keydown.enter.prevent="toggle"
      @keydown.space.prevent="toggle"
    >
      <div class="flex items-center gap-2">
        <span v-if="icon" class="text-sm" aria-hidden="true">{{ icon }}</span>
        <h3 :id="`section-${id}`" class="text-xs font-semibold uppercase tracking-wider text-text-secondary">{{ title }}</h3>
      </div>
      <svg
        v-if="collapsible"
        class="text-text-tertiary opacity-50 transition-transform duration-200"
        :class="{ '-rotate-90': isCollapsed }"
        width="14" height="14" viewBox="0 0 14 14" fill="none"
        aria-hidden="true"
      >
        <path d="M3.5 5.25L7 8.75L10.5 5.25" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
    <div v-show="!isCollapsed">
      <slot />
    </div>
  </section>
</template>

