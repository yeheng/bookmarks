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
  <div class="animate-in fade-in duration-150">
    <SettingSection title="Global Hotkey" id="global-hotkey" icon="🌐">
      <div class="py-3">
        <label for="global-shortcut" class="block text-sm font-medium text-text-primary mb-1">Activation Shortcut</label>
        <p class="text-xs text-text-secondary mb-3">Press this key combination to open the launcher from anywhere</p>
        <input
          id="global-shortcut"
          v-model="settings.hotkey.global_shortcut"
          class="w-full max-w-[240px] px-3.5 py-2.5 text-sm font-mono bg-bg-secondary border border-border-default rounded-md text-text-primary outline-none focus:border-accent"
          placeholder="Cmd+F1"
          aria-describedby="hotkey-help"
        />
        <p id="hotkey-help" class="text-[11px] text-text-tertiary mt-2">Examples: Cmd+F1, Alt+Space, Ctrl+Shift+P</p>
      </div>
    </SettingSection>

    <SettingSection title="UI Shortcuts" id="ui-shortcuts" icon="⌨️">
      <div class="flex flex-col" role="list">
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

