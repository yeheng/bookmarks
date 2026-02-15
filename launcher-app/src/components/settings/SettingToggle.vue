<script setup lang="ts">
/**
 * SettingToggle - Toggle switch component for boolean settings
 * Combines SettingItem layout with a Headless UI Switch
 */
import { Switch, SwitchGroup, SwitchLabel, SwitchDescription } from '@headlessui/vue';

interface Props {
  label: string;
  description?: string;
  modelValue: boolean;
  id?: string;
  disabled?: boolean;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
});
const emit = defineEmits<Emits>();

const toggle = (value: boolean) => {
  if (!props.disabled) {
    emit('update:modelValue', value);
  }
};
</script>

<template>
  <SwitchGroup as="div" class="flex items-start justify-between gap-4 py-3 border-b border-border-default last:border-b-0" :class="{ 'opacity-50 pointer-events-none': disabled }">
    <div class="flex flex-col flex-1 min-w-0">
      <SwitchLabel as="span" class="text-sm font-medium text-text-primary cursor-pointer select-none">
        {{ label }}
      </SwitchLabel>
      <SwitchDescription v-if="description" as="span" class="mt-1 text-xs text-text-secondary leading-snug">
        {{ description }}
      </SwitchDescription>
    </div>
    <Switch
      :model-value="modelValue"
      @update:model-value="toggle"
      :disabled="disabled"
      :class="modelValue ? 'bg-accent' : 'bg-bg-secondary border border-border-default'"
      class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2"
    >
      <span class="sr-only">Use setting</span>
      <span
        aria-hidden="true"
        :class="modelValue ? 'translate-x-5' : 'translate-x-0'"
        class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
      />
    </Switch>
  </SwitchGroup>
</template>

