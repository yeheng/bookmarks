<script setup lang="ts">
/**
 * SettingsTabShortcuts - Shortcuts settings tab including global hotkey and UI shortcuts
 */
import type { AppSettings } from "../../types/settings";
import { shortcuts } from "../../composables/useAppSettings";
import SettingSection from "./SettingSection.vue";
import ShortcutEditor from "../ShortcutEditor.vue";

const props = defineProps<{
  settings: AppSettings;
}>();

const emit = defineEmits<{
  (e: 'shortcut-update', actionId: string, shortcut: string): void;
}>();

function handleShortcutUpdate(actionId: string, shortcut: string) {
  emit('shortcut-update', actionId, shortcut);
}
</script>

<template>
  <div class="settings-tab-content">
    <SettingSection title="Global Hotkey" id="global-hotkey" icon="🌐">
      <div class="global-hotkey-field">
        <label for="global-shortcut" class="hotkey-label">Activation Shortcut</label>
        <p class="hotkey-description">Press this key combination to open the launcher from anywhere</p>
        <input
          id="global-shortcut"
          v-model="settings.hotkey.global_shortcut"
          class="hotkey-input"
          placeholder="Cmd+Space"
          aria-describedby="hotkey-help"
        />
        <p id="hotkey-help" class="hotkey-help">Examples: Cmd+Space, Alt+Space, Ctrl+Shift+P</p>
      </div>
    </SettingSection>

    <SettingSection title="UI Shortcuts" id="ui-shortcuts" icon="⌨️">
      <div class="shortcuts-list" role="list">
        <ShortcutEditor
          v-for="shortcut in shortcuts"
          :key="shortcut.id"
          :action-id="shortcut.id"
          :label="shortcut.label"
          :description="shortcut.description"
          :current-shortcut="settings.hotkey.ui_shortcuts[shortcut.id] || 'Not set'"
          @update="handleShortcutUpdate"
        />
      </div>
    </SettingSection>
  </div>
</template>

<style scoped>
.settings-tab-content {
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.global-hotkey-field {
  padding: 12px 0;
}

.hotkey-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary, #e0e0e0);
  margin-bottom: 4px;
}

.hotkey-description {
  font-size: 12px;
  color: var(--color-text-secondary, #b3b3b3);
  margin-bottom: 12px;
}

.hotkey-input {
  width: 100%;
  max-width: 240px;
  padding: 10px 14px;
  font-size: 14px;
  font-family: monospace;
  background: var(--color-bg-secondary, #262626);
  border: 1px solid var(--color-border-default, #3a3a3a);
  border-radius: 6px;
  color: var(--color-text-primary, #e0e0e0);
  outline: none;
}

.hotkey-input:focus {
  border-color: var(--color-accent, #ff6b6b);
}

.hotkey-help {
  font-size: 11px;
  color: var(--color-text-tertiary, #a3a3a3);
  margin-top: 8px;
}

.shortcuts-list {
  display: flex;
  flex-direction: column;
}

@media (prefers-color-scheme: light) {
  .hotkey-input {
    background: var(--color-bg-secondary, #f5f5f5);
    border-color: var(--color-border-default, #e0e0e0);
    color: var(--color-text-primary, #1a1a1a);
  }
}

@media (prefers-reduced-motion: reduce) {
  .settings-tab-content {
    animation: none;
  }
}
</style>
