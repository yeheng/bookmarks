<script setup lang="ts">
/**
 * SettingSection - Groups related settings under a titled section
 */

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

import { ref } from 'vue';

const isCollapsed = ref(false);

const toggle = () => {
  if (props.collapsible) {
    isCollapsed.value = !isCollapsed.value;
  }
};
</script>

<template>
  <section
    class="setting-section"
    :class="{ 'setting-section--collapsed': isCollapsed }"
    role="group"
    :aria-labelledby="`section-${id}`"
  >
    <div
      class="setting-section__header"
      :class="{ 'setting-section__header--clickable': collapsible }"
      :role="collapsible ? 'button' : undefined"
      :tabindex="collapsible ? 0 : undefined"
      :aria-expanded="collapsible ? !isCollapsed : undefined"
      @click="toggle"
      @keydown.enter.prevent="toggle"
      @keydown.space.prevent="toggle"
    >
      <div class="setting-section__title-group">
        <span v-if="icon" class="setting-section__icon" aria-hidden="true">{{ icon }}</span>
        <h3 :id="`section-${id}`" class="setting-section__title">{{ title }}</h3>
      </div>
      <svg
        v-if="collapsible"
        class="setting-section__chevron"
        :class="{ 'setting-section__chevron--collapsed': isCollapsed }"
        width="14" height="14" viewBox="0 0 14 14" fill="none"
        aria-hidden="true"
      >
        <path d="M3.5 5.25L7 8.75L10.5 5.25" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
    <div v-show="!isCollapsed" class="setting-section__content">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.setting-section {
  margin-bottom: 24px;
}

.setting-section:last-child {
  margin-bottom: 0;
}

.setting-section__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.setting-section__header--clickable {
  cursor: pointer;
  padding: 6px 8px;
  margin: 0 -8px 8px;
  border-radius: 6px;
  transition: background-color 0.15s ease;
}

.setting-section__header--clickable:hover {
  background: rgba(128, 128, 128, 0.08);
}

.setting-section__header--clickable:focus-visible {
  outline: 2px solid var(--color-accent, #ff6b6b);
  outline-offset: -2px;
}

.setting-section__title-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-section__icon {
  font-size: 14px;
}

.setting-section__title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary, #b3b3b3);
}

.setting-section__chevron {
  color: var(--color-text-secondary, #b3b3b3);
  opacity: 0.5;
  transition: transform 0.2s ease;
}

.setting-section__chevron--collapsed {
  transform: rotate(-90deg);
}

.setting-section__content {
  padding: 0;
}

@media (prefers-color-scheme: light) {
  .setting-section__title {
    color: var(--color-text-secondary, #525252);
  }
}

@media (prefers-reduced-motion: reduce) {
  .setting-section__header--clickable,
  .setting-section__chevron {
    transition: none;
  }
}
</style>
