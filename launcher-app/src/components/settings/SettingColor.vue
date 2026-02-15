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
  <div class="flex items-center justify-between py-2" role="group" :aria-labelledby="`${fieldId}-label`">
    <label :id="`${fieldId}-label`" class="text-xs font-medium text-text-primary select-none">
      {{ label }}
    </label>
    <div class="flex items-center gap-2">
      <input
        v-if="optional"
        type="checkbox"
        :checked="isEnabled"
        :aria-label="`Enable custom ${label.toLowerCase()}`"
        class="w-4 h-4 rounded border-border-default text-accent focus:ring-accent cursor-pointer"
        @change="handleToggle"
      />
      <div class="relative flex items-center">
        <input
          v-if="!optional || modelValue"
          :id="fieldId"
          type="color"
          class="w-9 h-7 p-0 bg-transparent border-2 border-border-default rounded-md cursor-pointer overflow-hidden transition-colors hover:border-accent focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:border-accent"
          :value="modelValue || defaultColor"
          :aria-label="`${label} color`"
          @input="handleColorChange"
        />
        <span v-if="modelValue" class="ml-2 text-[10px] font-mono text-text-tertiary min-w-[60px]" aria-hidden="true">
          {{ modelValue }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Color input styling */
input[type="color"]::-webkit-color-swatch-wrapper {
  padding: 0;
}
input[type="color"]::-webkit-color-swatch {
  border: none;
}
</style>
