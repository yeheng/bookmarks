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
  <div class="shortcut-editor" role="group" :aria-labelledby="`shortcut-label-${actionId}`">
    <div class="shortcut-info">
      <div :id="`shortcut-label-${actionId}`" class="shortcut-label">{{ label }}</div>
      <div v-if="description" :id="`shortcut-desc-${actionId}`" class="shortcut-description">{{ description }}</div>
    </div>
    <div class="shortcut-input-container">
      <button
        v-if="!isRecording"
        class="shortcut-display"
        @click="startRecording"
        :aria-label="`Edit shortcut for ${label}. Current shortcut: ${currentShortcut}`"
        :aria-describedby="description ? `shortcut-desc-${actionId}` : undefined"
      >
        {{ currentShortcut }}
      </button>
      <input
        v-else
        class="shortcut-recording"
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
.shortcut-editor {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  border-bottom: 1px solid var(--border-color, #3a3a3a);
}

.shortcut-info {
  flex: 1;
}

.shortcut-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-color, #e0e0e0);
  margin-bottom: 4px;
}

.shortcut-description {
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
}

.shortcut-input-container {
  min-width: 140px;
}

.shortcut-display,
.shortcut-recording {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  font-family: monospace;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #e0e0e0;
  cursor: pointer;
  transition: all 0.2s;
}

.shortcut-display:hover {
  background: #3a3a3a;
  border-color: var(--accent-color, #ff6b6b);
}

.shortcut-recording {
  cursor: text;
  border-color: var(--accent-color, #ff6b6b);
  background: #1a1a1a;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(255, 107, 107, 0.4);
  }
  50% {
    box-shadow: 0 0 0 4px rgba(255, 107, 107, 0);
  }
}

@media (prefers-color-scheme: light) {
  .shortcut-display,
  .shortcut-recording {
    background: #f5f5f5;
    border-color: #e0e0e0;
    color: #1a1a1a;
  }
  
  .shortcut-display:hover {
    background: #e0e0e0;
  }
  
  .shortcut-recording {
    background: #fff;
  }
}
</style>
