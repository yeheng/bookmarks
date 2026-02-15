<script setup lang="ts">
/**
 * SettingToggle - Toggle switch component for boolean settings
 * Combines SettingItem layout with a toggle switch control
 */
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

const fieldId = props.id || `toggle-${Math.random().toString(36).substr(2, 9)}`;
const descriptionId = `${fieldId}-desc`;

const toggle = () => {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue);
  }
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    toggle();
  }
};
</script>

<template>
  <div
    class="setting-toggle"
    :class="{ 'setting-toggle--disabled': disabled }"
    role="group"
    :aria-labelledby="`${fieldId}-label`"
  >
    <div class="setting-toggle__header">
      <label
        :id="`${fieldId}-label`"
        :for="fieldId"
        class="setting-toggle__label"
      >
        {{ label }}
      </label>
      <p
        v-if="description"
        :id="descriptionId"
        class="setting-toggle__description"
      >
        {{ description }}
      </p>
    </div>
    <button
      :id="fieldId"
      type="button"
      role="switch"
      class="setting-toggle__switch"
      :class="{ 'setting-toggle__switch--active': modelValue }"
      :aria-checked="modelValue"
      :aria-describedby="description ? descriptionId : undefined"
      :disabled="disabled"
      @click="toggle"
      @keydown="handleKeydown"
    >
      <span class="setting-toggle__thumb" aria-hidden="true" />
    </button>
  </div>
</template>

<style scoped>
.setting-toggle {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--color-border-default, #3a3a3a);
}

.setting-toggle:last-child {
  border-bottom: none;
}

.setting-toggle--disabled {
  opacity: 0.5;
  pointer-events: none;
}

.setting-toggle__header {
  flex: 1;
  min-width: 0;
}

.setting-toggle__label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary, #e0e0e0);
  cursor: pointer;
}

.setting-toggle__description {
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
  line-height: 1.4;
}

.setting-toggle__switch {
  position: relative;
  width: 44px;
  height: 24px;
  background: var(--color-bg-secondary, #2a2a2a);
  border: 1px solid var(--color-border-default, #3a3a3a);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  padding: 0;
  flex-shrink: 0;
}

.setting-toggle__switch:hover {
  border-color: var(--color-accent, #ff6b6b);
}

.setting-toggle__switch:focus-visible {
  outline: 2px solid var(--color-accent, #ff6b6b);
  outline-offset: 2px;
}

.setting-toggle__switch--active {
  background: var(--color-accent, #ff6b6b);
  border-color: var(--color-accent, #ff6b6b);
}

.setting-toggle__thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.setting-toggle__switch--active .setting-toggle__thumb {
  transform: translateX(20px);
}

@media (prefers-color-scheme: light) {
  .setting-toggle {
    border-bottom-color: var(--color-border-default, #e0e0e0);
  }
  .setting-toggle__label {
    color: var(--color-text-primary, #1a1a1a);
  }
  .setting-toggle__description {
    color: var(--color-text-secondary, #525252);
  }
  .setting-toggle__switch {
    background: #e5e5e5;
    border-color: #d4d4d4;
  }
}

@media (prefers-reduced-motion: reduce) {
  .setting-toggle__switch,
  .setting-toggle__thumb {
    transition: none;
  }
}
</style>
