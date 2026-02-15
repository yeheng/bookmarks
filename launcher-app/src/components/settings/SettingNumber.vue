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

const widthClasses = {
  small: 'w-20',
  medium: 'w-32',
  large: 'w-48',
};
</script>

<template>
  <div class="flex items-start justify-between gap-4 py-3 border-b border-border-default last:border-b-0" :class="{ 'opacity-50 pointer-events-none': disabled }">
    <div class="flex flex-col flex-1 min-w-0">
      <label
        :id="`${fieldId}-label`"
        :for="fieldId"
        class="text-sm font-medium text-text-primary cursor-pointer select-none"
      >
        {{ label }}
      </label>
      <p
        v-if="description"
        :id="descriptionId"
        class="mt-1 text-xs text-text-secondary leading-snug"
      >
        {{ description }}
      </p>
    </div>
    <input
      :id="fieldId"
      type="number"
      class="px-3 py-2 text-xs font-mono text-text-primary bg-bg-secondary border border-border-default rounded-md outline-none focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
      :class="widthClasses[width]"
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
/* Remove spinner buttons for cleaner look */
input[type=number]::-webkit-inner-spin-button,
input[type=number]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

input[type=number] {
  -moz-appearance: textfield;
}
</style>
