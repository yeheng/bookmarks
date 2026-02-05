<script setup lang="ts">
/**
 * SettingItem - Base component for all settings rows
 * Provides consistent layout, spacing, and accessibility
 */
interface Props {
  label: string;
  description?: string;
  id?: string;
  /** Whether this is an inline field (label and control on same line) */
  inline?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  inline: true,
});

// Generate unique ID if not provided
const fieldId = props.id || `setting-${Math.random().toString(36).substr(2, 9)}`;
const descriptionId = `${fieldId}-desc`;
</script>

<template>
  <div
    class="setting-item"
    :class="{ 'setting-item--stacked': !inline }"
    role="group"
    :aria-labelledby="`${fieldId}-label`"
  >
    <div class="setting-item__header">
      <label
        :id="`${fieldId}-label`"
        :for="fieldId"
        class="setting-item__label"
      >
        {{ label }}
      </label>
      <p
        v-if="description"
        :id="descriptionId"
        class="setting-item__description"
      >
        {{ description }}
      </p>
    </div>
    <div class="setting-item__control">
      <slot :id="fieldId" :describedby="description ? descriptionId : undefined" />
    </div>
  </div>
</template>

<style scoped>
.setting-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--color-border-default, #3a3a3a);
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-item--stacked {
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
}

.setting-item__header {
  flex: 1;
  min-width: 0;
}

.setting-item__label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary, #e0e0e0);
  cursor: pointer;
}

.setting-item__description {
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
  line-height: 1.4;
}

.setting-item__control {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.setting-item--stacked .setting-item__control {
  width: 100%;
}

@media (prefers-color-scheme: light) {
  .setting-item {
    border-bottom-color: var(--color-border-default, #e0e0e0);
  }
  .setting-item__label {
    color: var(--color-text-primary, #1a1a1a);
  }
  .setting-item__description {
    color: var(--color-text-secondary, #525252);
  }
}
</style>
