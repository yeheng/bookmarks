<script setup lang="ts">
/**
 * SettingSelect - Select dropdown component for settings
 * Combines SettingItem layout with a Headless UI Listbox
 */
import { Listbox, ListboxButton, ListboxOptions, ListboxOption, ListboxLabel } from '@headlessui/vue';

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

const getLabel = (value: string) => {
  return props.options.find(opt => opt.value === value)?.label || value;
};
</script>

<template>
  <div class="flex items-start justify-between gap-4 py-3 border-b border-border-default last:border-b-0" :class="{ 'opacity-50 pointer-events-none': disabled }">
    <div class="flex flex-col flex-1 min-w-0">
      <Listbox :model-value="modelValue" @update:model-value="emit('update:modelValue', $event)" :disabled="disabled">
        <ListboxLabel class="text-sm font-medium text-text-primary cursor-pointer select-none">
          {{ label }}
        </ListboxLabel>
        <p v-if="description" class="mt-1 text-xs text-text-secondary leading-snug">
          {{ description }}
        </p>
        
        <div class="relative mt-2 sm:mt-0 sm:ml-auto">
          <ListboxButton class="relative w-full sm:w-auto min-w-[140px] cursor-pointer rounded-md bg-bg-secondary py-2 pl-3 pr-10 text-left text-sm border border-border-default focus:outline-none focus-visible:border-accent focus-visible:ring-1 focus-visible:ring-accent transition-colors hover:border-accent">
            <span class="block truncate text-text-primary">{{ getLabel(modelValue) }}</span>
            <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" class="text-text-secondary">
                <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </span>
          </ListboxButton>

          <transition
            leave-active-class="transition duration-100 ease-in"
            leave-from-class="opacity-100"
            leave-to-class="opacity-0"
          >
            <ListboxOptions class="absolute right-0 mt-1 max-h-60 w-full min-w-[140px] overflow-auto rounded-md bg-bg-elevated py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm z-50">
              <ListboxOption
                v-for="option in options"
                :key="option.value"
                :value="option.value"
                as="template"
                v-slot="{ active, selected }"
              >
                <li
                  :class="[
                    active ? 'bg-hover-bg text-text-primary' : 'text-text-primary',
                    'relative cursor-pointer select-none py-2 pl-3 pr-9'
                  ]"
                >
                  <span :class="[selected ? 'font-medium' : 'font-normal', 'block truncate']">{{ option.label }}</span>
                  <span v-if="selected" class="absolute inset-y-0 right-0 flex items-center pr-3 text-accent">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                      <path d="M2.5 6L4.5 8L9.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </span>
                </li>
              </ListboxOption>
            </ListboxOptions>
          </transition>
        </div>
      </Listbox>
    </div>
  </div>
</template>

