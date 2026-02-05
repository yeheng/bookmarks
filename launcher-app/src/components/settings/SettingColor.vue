<script setup lang="ts">
/**
 * SettingColor - Color picker with optional toggle
 */

interface Props {
  label: string;
  modelValue?: string;
  id?: string;
  /** If true, shows a checkbox to enable/disable custom color */
  optional?: boolean;
  defaultColor?: string;
}

interface Emits {
  (e: 'update:modelValue', value: string | undefined): void;
}

const props = withDefaults(defineProps<Props>(), {
  optional: false,
  defaultColor: '#ff6b6b',
});
const emit = defineEmits<Emits>();

const fieldId = props.id || `color-${Math.random().toString(36).substr(2, 9)}`;
const isEnabled = !!props.modelValue;

const handleToggle = (e: Event) => {
  const target = e.target as HTMLInputElement;
  if (target.checked) {
    emit('update:modelValue', props.defaultColor);
  } else {
    emit('update:modelValue', undefined);
  }
};

const handleColorChange = (e: Event) => {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', target.value);
};
</script>

<template>
  <div class="setting-color" role="group" :aria-labelledby="`${fieldId}-label`">
    <label :id="`${fieldId}-label`" class="setting-color__label">
      {{ label }}
    </label>
    <div class="setting-color__control">
      <input
        v-if="optional"
        type="checkbox"
        :checked="isEnabled"
        :aria-label="`Enable custom ${label.toLowerCase()}`"
        class="setting-color__checkbox"
        @change="handleToggle"
      />
      <input
        v-if="!optional || modelValue"
        :id="fieldId"
        type="color"
        class="setting-color__picker"
        :value="modelValue || defaultColor"
        :aria-label="`${label} color`"
        @input="handleColorChange"
      />
      <span v-if="modelValue" class="setting-color__hex" aria-hidden="true">
        {{ modelValue }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.setting-color {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
}

.setting-color__label {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary, #e0e0e0);
}

.setting-color__control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-color__checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--color-accent, #ff6b6b);
  cursor: pointer;
}

.setting-color__picker {
  width: 36px;
  height: 28px;
  border: 2px solid var(--color-border-default, #3a3a3a);
  border-radius: 6px;
  cursor: pointer;
  background: none;
  padding: 0;
  overflow: hidden;
}

.setting-color__picker:hover {
  border-color: var(--color-accent, #ff6b6b);
}

.setting-color__picker:focus-visible {
  outline: 2px solid var(--color-accent, #ff6b6b);
  outline-offset: 2px;
}

.setting-color__hex {
  font-size: 11px;
  font-family: monospace;
  color: var(--color-text-tertiary, #a3a3a3);
  min-width: 62px;
}

@media (prefers-color-scheme: light) {
  .setting-color__label {
    color: var(--color-text-primary, #1a1a1a);
  }
  .setting-color__picker {
    border-color: var(--color-border-default, #e0e0e0);
  }
  .setting-color__hex {
    color: var(--color-text-tertiary, #6b6b6b);
  }
}
</style>
