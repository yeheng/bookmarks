<script setup lang="ts">
/**
 * SettingNumber - Number input component for numeric settings
 * Combines SettingItem layout with a styled number input
 */

interface Props {
  label: string;
  description?: string;
  modelValue: number;
  id?: string;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  /** Width class: 'small' (80px), 'medium' (120px), 'large' (200px) */
  width?: 'small' | 'medium' | 'large';
}

interface Emits {
  (e: 'update:modelValue', value: number): void;
}

const props = withDefaults(defineProps<Props>(), {
  min: 0,
  max: 100,
  step: 1,
  disabled: false,
  width: 'small',
});
const emit = defineEmits<Emits>();

const fieldId = props.id || `number-${Math.random().toString(36).substr(2, 9)}`;
const descriptionId = `${fieldId}-desc`;

const handleInput = (e: Event) => {
  const target = e.target as HTMLInputElement;
  const value = parseFloat(target.value);
  if (!isNaN(value)) {
    emit('update:modelValue', value);
  }
};
</script>

<template>
  <div
    class="setting-number"
    :class="{ 'setting-number--disabled': disabled }"
    role="group"
    :aria-labelledby="`${fieldId}-label`"
  >
    <div class="setting-number__header">
      <label
        :id="`${fieldId}-label`"
        :for="fieldId"
        class="setting-number__label"
      >
        {{ label }}
      </label>
      <p
        v-if="description"
        :id="descriptionId"
        class="setting-number__description"
      >
        {{ description }}
      </p>
    </div>
    <input
      :id="fieldId"
      type="number"
      class="setting-number__input"
      :class="`setting-number__input--${width}`"
      :value="modelValue"
      :min="min"
      :max="max"
      :step="step"
      :aria-describedby="description ? descriptionId : undefined"
      :aria-valuemin="min"
      :aria-valuemax="max"
      :disabled="disabled"
      @input="handleInput"
    />
  </div>
</template>

<style scoped>
.setting-number {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--color-border-default, #3a3a3a);
}

.setting-number:last-child {
  border-bottom: none;
}

.setting-number--disabled {
  opacity: 0.5;
  pointer-events: none;
}

.setting-number__header {
  flex: 1;
  min-width: 0;
}

.setting-number__label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary, #e0e0e0);
  cursor: pointer;
}

.setting-number__description {
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
  line-height: 1.4;
}

.setting-number__input {
  padding: 8px 12px;
  font-size: 13px;
  font-family: inherit;
  color: var(--color-text-primary, #e0e0e0);
  background: var(--color-bg-secondary, #2a2a2a);
  border: 1px solid var(--color-border-default, #3a3a3a);
  border-radius: 6px;
  transition: border-color 0.15s ease;
  flex-shrink: 0;
}

.setting-number__input--small {
  width: 80px;
}

.setting-number__input--medium {
  width: 120px;
}

.setting-number__input--large {
  width: 200px;
}

.setting-number__input:hover {
  border-color: var(--color-accent, #ff6b6b);
}

.setting-number__input:focus {
  outline: none;
  border-color: var(--color-accent, #ff6b6b);
  box-shadow: 0 0 0 2px rgba(255, 107, 107, 0.2);
}

/* Remove spinner buttons for cleaner look */
.setting-number__input::-webkit-inner-spin-button,
.setting-number__input::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.setting-number__input[type=number] {
  -moz-appearance: textfield;
}

@media (prefers-color-scheme: light) {
  .setting-number {
    border-bottom-color: var(--color-border-default, #e0e0e0);
  }
  .setting-number__label {
    color: var(--color-text-primary, #1a1a1a);
  }
  .setting-number__description {
    color: var(--color-text-secondary, #525252);
  }
  .setting-number__input {
    color: var(--color-text-primary, #1a1a1a);
    background: var(--color-bg-secondary, #f5f5f5);
    border-color: var(--color-border-default, #e0e0e0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .setting-number__input {
    transition: none;
  }
}
</style>
