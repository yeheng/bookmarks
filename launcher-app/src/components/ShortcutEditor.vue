<script setup lang="ts">
import { ref } from 'vue';
import { ShortcutManager } from '../services/shortcuts';

interface Props {
  actionId: string;
  label: string;
  description?: string;
  currentShortcut: string;
}

interface Emits {
  (e: 'update', actionId: string, shortcut: string): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const isRecording = ref(false);
const tempShortcut = ref('');

function startRecording() {
  isRecording.value = true;
  tempShortcut.value = 'Press keys...';
}

function handleKeyDown(e: KeyboardEvent) {
  if (!isRecording.value) return;
  
  e.preventDefault();
  e.stopPropagation();
  
  // Format the event as a shortcut string
  const formatted = ShortcutManager.formatEvent(e);
  
  // Ignore lone modifier keys
  if (!formatted || formatted === 'Meta' || formatted === 'Ctrl' || 
      formatted === 'Alt' || formatted === 'Shift') {
    return;
  }
  
  tempShortcut.value = formatted;
  
  // Auto-save after a short delay
  setTimeout(() => {
    if (isRecording.value) {
      saveShortcut();
    }
  }, 300);
}

function saveShortcut() {
  if (tempShortcut.value && tempShortcut.value !== 'Press keys...') {
    emit('update', props.actionId, tempShortcut.value);
  }
  isRecording.value = false;
  tempShortcut.value = '';
}

function cancelRecording() {
  isRecording.value = false;
  tempShortcut.value = '';
}
</script>

<template>
  <div class="flex items-center justify-between py-3 border-b border-border-default last:border-b-0" role="group" :aria-labelledby="`shortcut-label-${actionId}`">
    <div class="flex-1">
      <div :id="`shortcut-label-${actionId}`" class="text-sm font-medium text-text-primary mb-1">{{ label }}</div>
      <div v-if="description" :id="`shortcut-desc-${actionId}`" class="text-xs text-text-secondary">{{ description }}</div>
    </div>
    <div class="min-w-[140px]">
      <button
        v-if="!isRecording"
        class="w-full px-3 py-2 text-xs font-mono bg-bg-secondary border border-border-default rounded-md text-text-primary cursor-pointer transition-colors hover:bg-hover-bg hover:border-accent focus:outline-none focus:ring-2 focus:ring-accent"
        @click="startRecording"
        :aria-label="`Edit shortcut for ${label}. Current shortcut: ${currentShortcut}`"
        :aria-describedby="description ? `shortcut-desc-${actionId}` : undefined"
      >
        {{ currentShortcut }}
      </button>
      <input
        v-else
        class="w-full px-3 py-2 text-xs font-mono bg-bg-secondary border border-accent rounded-md text-text-primary cursor-text outline-none animate-pulse-border"
        :value="tempShortcut"
        @keydown="handleKeyDown"
        @blur="cancelRecording"
        readonly
        autofocus
        :aria-label="`Recording shortcut for ${label}. Press a key combination.`"
        role="textbox"
        aria-live="polite"
      />
    </div>
  </div>
</template>

<style scoped>
.animate-pulse-border {
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(var(--color-accent-rgb), 0.4);
  }
  50% {
    box-shadow: 0 0 0 4px rgba(var(--color-accent-rgb), 0);
  }
}
</style>
