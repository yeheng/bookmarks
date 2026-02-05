<script setup lang="ts">
/**
 * SettingSelect - Select dropdown component for settings
 * Combines SettingItem layout with a styled select control
 */

interface Option {
  value: string;
  label: string;
}

interface Props {
  label: string;
  description?: string;
  modelValue: string;
  options: Option[];
  id?: string;
  disabled?: boolean;
}

interface Emits {
  (e: 'update:modelValue', value: string): void;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
});
const emit = defineEmits<Emits>();

const fieldId = props.id || `select-${Math.random().toString(36).substr(2, 9)}`;
const descriptionId = `${fieldId}-desc`;

const handleChange = (e: Event) => {
  const target = e.target as HTMLSelectElement;
  emit('update:modelValue', target.value);
};
</script>

<template>
  <div
    class="setting-select"
    :class="{ 'setting-select--disabled': disabled }"
    role="group"
    :aria-labelledby="`${fieldId}-label`"
  >
    <div class="setting-select__header">
      <label
        :id="`${fieldId}-label`"
        :for="fieldId"
        class="setting-select__label"
      >
        {{ label }}
      </label>
      <p
        v-if="description"
        :id="descriptionId"
        class="setting-select__description"
      >
        {{ description }}
      </p>
    </div>
    <div class="setting-select__control">
      <select
        :id="fieldId"
        class="setting-select__input"
        :value="modelValue"
        :aria-describedby="description ? descriptionId : undefined"
        :disabled="disabled"
        @change="handleChange"
      >
        <option
          v-for="option in options"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </select>
      <span class="setting-select__chevron" aria-hidden="true">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </span>
    </div>
  </div>
</template>

<style scoped>
.setting-select {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--color-border-default, #3a3a3a);
}

.setting-select:last-child {
  border-bottom: none;
}

.setting-select--disabled {
  opacity: 0.5;
  pointer-events: none;
}

.setting-select__header {
  flex: 1;
  min-width: 0;
}

.setting-select__label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary, #e0e0e0);
  cursor: pointer;
}

.setting-select__description {
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
  line-height: 1.4;
}

.setting-select__control {
  position: relative;
  flex-shrink: 0;
}

.setting-select__input {
  appearance: none;
  min-width: 140px;
  padding: 8px 32px 8px 12px;
  font-size: 13px;
  font-family: inherit;
  color: var(--color-text-primary, #e0e0e0);
  background: var(--color-bg-secondary, #2a2a2a);
  border: 1px solid var(--color-border-default, #3a3a3a);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.15s ease;
}

.setting-select__input:hover {
  border-color: var(--color-accent, #ff6b6b);
}

.setting-select__input:focus {
  outline: none;
  border-color: var(--color-accent, #ff6b6b);
  box-shadow: 0 0 0 2px rgba(255, 107, 107, 0.2);
}

.setting-select__chevron {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
  color: var(--color-text-secondary, #b3b3b3);
}

@media (prefers-color-scheme: light) {
  .setting-select {
    border-bottom-color: var(--color-border-default, #e0e0e0);
  }
  .setting-select__label {
    color: var(--color-text-primary, #1a1a1a);
  }
  .setting-select__description {
    color: var(--color-text-secondary, #525252);
  }
  .setting-select__input {
    color: var(--color-text-primary, #1a1a1a);
    background: var(--color-bg-secondary, #f5f5f5);
    border-color: var(--color-border-default, #e0e0e0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .setting-select__input {
    transition: none;
  }
}
</style>
